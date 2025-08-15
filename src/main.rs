mod errors;
mod tui;
mod sso;
mod aws;
mod utils;
mod widgets;
mod app;

use app::*;
use color_eyre::Result;

fn main() -> Result<()> {
    // Initialize logging - set RUST_LOG=debug for detailed output, default to warn level
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();
    
    log::debug!("Starting assumer TUI application");
    
    errors::install_hooks()?;  
    let mut terminal = tui::init()?;
    App::default().run(&mut terminal)?;
    tui::restore()?;
    Ok(())
}