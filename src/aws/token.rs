use std::{path::Path, fs};
use crate::{utils::serde::json_date_format, App};
use anyhow::{Result, anyhow};
use aws_config::SdkConfig;
use aws_sdk_ssooidc::Client;
use chrono::{DateTime, Duration, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use tokio::time::{sleep, timeout, Duration as TokioDuration};


#[derive(Serialize, Deserialize, PartialEq, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AccessToken {
    pub start_url: String,
    pub region: String,
    pub access_token: String,
    #[serde(with = "json_date_format")]
    pub expires_at: DateTime<Utc>,
    #[serde(flatten)]
    pub device_client: DeviceClient,
    pub refresh_token: String,
}

impl AccessToken {
    pub fn is_expired(&self) -> bool {
        self.expires_at < Utc::now()
    }    
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DeviceClient {
    pub client_id: String,
    pub client_secret: String,
    #[serde(with = "json_date_format")]
    pub registration_expires_at: DateTime<Utc>,
}

impl DeviceClient {
    
}

#[derive(Clone)]
pub struct SsoAccessTokenProvider {
    sso_session_name: String,
    client: Client,
    cache: super::AccessTokenCache,
}

impl SsoAccessTokenProvider {
    const CLIENT_NAME: &'static str = "assumer";
    const DEVICE_GRANT_TYPE: &'static str = "urn:ietf:params:oauth:grant-type:device_code";
    const REFRESH_GRANT_TYPE: &'static str = "refresh_token";

    pub fn new(config: &SdkConfig, sso_session_name: &str, config_dir: &Path) -> anyhow::Result<Self> {
        let sso_cache_dir = config_dir.join("sso").join("cache");
        if !sso_cache_dir.exists() {
            fs::create_dir_all(&sso_cache_dir)?;
        }
        Ok(Self {
            sso_session_name: String::from(sso_session_name),
            client: Client::new(config),
            cache: super::AccessTokenCache::new(
                sso_session_name,
                sso_cache_dir.as_path(),
            ),
        })
    }

    pub async fn get_access_token(&self, start_url: &str, new_token: bool, app: &mut App) -> Result<AccessToken> {        
        let cached_token_option = self.cache.get_cached_token();        

        match cached_token_option {
            Ok(cached_token) => {
                if cached_token.is_expired() || new_token {
                    self.get_new_token(start_url, app).await
                } else {
                    self.refresh_token(cached_token).await
                }
            }
            Err(_) => self.get_new_token(start_url, app).await,
        }
    }

    async fn get_new_token(&self, start_url: &str, app: &mut App) -> Result<AccessToken> {
        let device_client = self.register_device_client().await?;
        self.authenticate(start_url, device_client, app).await
    }

    async fn register_device_client(&self) -> Result<DeviceClient, anyhow::Error> {
        let response = self
            .client
            .register_client()
            .client_name(format!("{}-{}", Self::CLIENT_NAME, self.sso_session_name))
            .client_type("public")
            .scopes("sso:account:access")
            .send()
            .await?;
        let client_id = response.client_id().unwrap();
        let client_secret = response.client_secret().unwrap();
        let registration_expires_at = Utc
            .timestamp_opt(response.client_secret_expires_at(), 0)
            .unwrap();
        let device_client = DeviceClient {
            client_id: String::from(client_id),
            client_secret: String::from(client_secret),
            registration_expires_at
        };
        Ok(device_client)
    }

    async fn authenticate(&self, start_url: &str, device_client: DeviceClient, app: &mut App) -> Result<AccessToken> {
        let auth_response = self
            .client
            .start_device_authorization()
            .client_id(device_client.client_id.as_str())
            .client_secret(device_client.client_secret.as_str())
            .start_url(start_url)
            .send()
            .await?;

        open::that(auth_response.verification_uri_complete().unwrap())?;

        app.token_prompt = format!("Verify authorization code: {} (Press ESC to cancel)", auth_response.user_code().unwrap());

        let interval = auth_response.interval();
        let max_retries = 120; // 120 retries * 5 seconds = 10 minutes max
        let total_timeout = TokioDuration::from_secs(600); // 10 minutes total timeout
        let mut retry_count = 0;

        // Wrap the entire polling loop in a timeout
        let result = timeout(total_timeout, async {
            loop {
                // Check for cancellation flag
                if app.exit && app.authenticating {
                    return Err(anyhow!("Authentication cancelled by user"));
                }

                let token_response = self
                    .client
                    .create_token()
                    .client_id(device_client.client_id.as_str())
                    .client_secret(device_client.client_secret.as_str())
                    .grant_type(Self::DEVICE_GRANT_TYPE)
                    .device_code(auth_response.device_code().unwrap())
                    .send()
                    .await;

                match token_response {
                    Ok(out) => {
                        let access_token = out.access_token().unwrap();
                        let refresh_token = out.refresh_token().unwrap();
                        let expires_at = Utc::now() + Duration::seconds(out.expires_in() as i64);

                        let access_token = AccessToken {
                            region: String::from(self.client.config().region().unwrap().to_string()),
                            start_url: String::from(start_url),
                            access_token: String::from(access_token),
                            expires_at,
                            device_client,
                            refresh_token: String::from(refresh_token),
                        };

                        app.token_prompt = String::new();

                        return Ok(self.cache.cache_token(access_token)?);
                    }
                    Err(err) => {
                        let service_error = err.into_service_error();
                        
                        // Handle specific AWS SSO errors
                        if service_error.is_access_denied_exception() {
                            return Err(anyhow!("Access request rejected"));
                        }
                        
                        if service_error.is_expired_token_exception() {
                            return Err(anyhow!("Authentication token expired. Please try again."));
                        }

                        // Check retry limit
                        retry_count += 1;
                        if retry_count >= max_retries {
                            return Err(anyhow!("Authentication timed out after {} retries. Please try again.", max_retries));
                        }

                        // Update prompt with retry info
                        app.token_prompt = format!("Verify authorization code: {} (Attempt {}/{}) (Press ESC to cancel)", 
                            auth_response.user_code().unwrap(), retry_count, max_retries);

                        // Use async sleep instead of blocking sleep
                        let sleep_duration = TokioDuration::from_secs(interval as u64);
                        sleep(sleep_duration).await;
                    }
                }
            }
        }).await;

        // Handle timeout and clear prompt
        app.token_prompt = String::new();
        
        match result {
            Ok(auth_result) => auth_result,
            Err(_) => Err(anyhow!("Authentication timed out after 10 minutes. Please try again.")),
        }
    }

    async fn refresh_token(&self, cached_token: AccessToken) -> Result<AccessToken> {
        let device_client = &cached_token.device_client;
        let response = self
            .client
            .create_token()
            .client_id(device_client.client_id.as_str())
            .client_secret(device_client.client_secret.as_str())
            .grant_type(Self::REFRESH_GRANT_TYPE)
            .refresh_token(cached_token.refresh_token.as_str())
            .send()
            .await?;

        let access_token = response.access_token().unwrap();
        let refresh_token = response.refresh_token().unwrap();
        let expires_at = Utc::now() + Duration::seconds(response.expires_in() as i64);

        let new_access_token = AccessToken {
            region: String::from(self.client.config().region().unwrap().to_string()),
            start_url: cached_token.start_url.clone(),
            access_token: String::from(access_token),
            expires_at,
            device_client: cached_token.device_client,
            refresh_token: String::from(refresh_token),
        };

        Ok(self.cache.cache_token(new_access_token)?)
    }
}