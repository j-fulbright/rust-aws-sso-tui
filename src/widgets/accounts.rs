use std::rc::Rc;

use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    symbols::border,
    text::{Line, Span, Text},
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table, Wrap},
    Frame,
};

use crate::app::App;

pub fn get_layout(f: &mut Frame) -> Rc<[Rect]> {
    Layout::horizontal([Constraint::Min(5)]).split(f.area())
}

pub fn handle_key_events(app: &mut App, key: KeyEvent) -> Result<(), ()> {
    match key.code {
        KeyCode::Down => {
            app.next();
        }
        KeyCode::Up => {
            app.previous();
        }
        KeyCode::Right => {
            app.select_account();
            app.current_page = crate::app::CurrentPage::Roles;
        }
        KeyCode::Char('c') => {
            app.currently_editing = true;
            app.current_page = crate::app::CurrentPage::Config;
        }
        KeyCode::Char('q') => {
            app.exit = true;
        }
        _ => {}
    }

    Ok(())
}

pub fn render_accounts(f: &mut Frame, app: &mut App, area: Rect) {
    if app.authenticating {
        render_authentication_ui(f, app, area);
        return;
    }

    if app.aws_config_provider.account_info_provider.is_none() {
        render_loading_ui(f, app, area);
        return;
    }

    let style = if app.is_selected {
        Style::new().white()
    } else {
        Style::new().blue()
    };

    let instructions = Line::from(vec![
        " Scroll Up ".into(),
        "<Up>".blue().bold(),
        " Scroll Down ".into(),
        "<Down>".blue().bold(),
        " Select Account ".into(),
        "<Right>".blue().bold(),
        " Config ".into(),
        "<C>".yellow().bold(),
        " Quit ".into(),
        "<Q> ".red().bold(),
    ]);

    let start_url = app
        .config_options
        .options
        .iter()
        .find(|o| o.name == "start_url")
        .unwrap()
        .value
        .clone();

    let account_list_block = Block::bordered()
        .title_top(Line::from(format!(" Accounts ({}) ", app.rows.len())).bold().left_aligned())
        .title_top(Line::from(format!(" Start URL: {} ", start_url)).bold().right_aligned())
        .title_bottom(instructions.centered())
        .title_bottom(Line::from(format!(" v{} ", env!("CARGO_PKG_VERSION"))).dark_gray().right_aligned())
        .border_set(border::THICK);

    let widths = [Constraint::Min(10), Constraint::Min(20)];

    let rows = app.rows.iter().map(|row| {
        Row::new(vec![
            Cell::from(row.account_name.clone()),
            Cell::from(row.account_id.clone()),
        ])
    });

    let footer_row = Row::new(vec![
        Cell::from("Selected Account:").style(Style::new().bold()),
        Cell::from(app.selected_account.account_id.clone())
            .style(Style::new().bold().yellow()),
    ]);

    let table = Table::new(rows, widths)
        .column_spacing(1)
        .style(style)
        .header(Row::new(vec!["Account Name", "Account ID"]).style(Style::new().bold()))
        .footer(footer_row)
        .block(account_list_block)
        .row_highlight_style(Style::new().reversed())
        .highlight_symbol(">>");

    f.render_stateful_widget(table, area, &mut app.table_state);
}

pub fn render_authentication_ui(f: &mut Frame, app: &mut App, area: Rect) {
    let auth_block = Block::bordered()
        .title_top(Line::from(" Authenticating with AWS SSO ").bold().centered())
        .title_bottom(Line::from(vec![" Cancel ".into(), "<ESC> ".red().bold()]).centered())
        .title_bottom(Line::from(format!(" v{} ", env!("CARGO_PKG_VERSION"))).dark_gray().right_aligned())
        .border_set(border::THICK)
        .borders(Borders::ALL);

    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(50),
            Constraint::Percentage(25),
        ])
        .split(area);

    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(10),
            Constraint::Percentage(80),
            Constraint::Percentage(10),
        ])
        .split(vertical[1]);

    let auth_area = horizontal[1];

    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let spinner_chars = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    let spinner = spinner_chars[(now / 100) as usize % spinner_chars.len()];

    let auth_text = if app.token_prompt.is_empty() {
        Text::from(vec![
            Line::from(vec![Span::styled(
                format!("{} Authenticating with AWS SSO...", spinner),
                Style::default().fg(Color::Yellow),
            )]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Please wait while we authenticate with AWS.",
                Style::default().fg(Color::White),
            )]),
            Line::from(vec![Span::styled(
                "Your browser should open automatically for authentication.",
                Style::default().fg(Color::Gray),
            )]),
        ])
    } else {
        Text::from(vec![
            Line::from(vec![Span::styled(
                format!("{} AWS SSO Authentication", spinner),
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![Span::styled(
                &app.token_prompt,
                Style::default().fg(Color::White),
            )]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Complete authentication in your browser, then return here.",
                Style::default().fg(Color::Gray),
            )]),
        ])
    };

    let auth_paragraph = Paragraph::new(auth_text)
        .block(auth_block)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });

    f.render_widget(Clear, area);
    f.render_widget(auth_paragraph, auth_area);
}

pub fn render_loading_ui(f: &mut Frame, app: &mut App, area: Rect) {
    use std::time::{SystemTime, UNIX_EPOCH};

    let instructions = Line::from(vec![
        " Config ".into(),
        "<C> ".yellow().bold(),
        " Quit ".into(),
        "<Q> ".red().bold(),
    ]);

    let start_url = app
        .config_options
        .options
        .iter()
        .find(|o| o.name == "start_url")
        .unwrap()
        .value
        .clone();

    let loading_block = Block::bordered()
        .title_top(Line::from(" Loading AWS Accounts ").bold().centered())
        .title_top(Line::from(format!(" Start URL: {} ", start_url)).bold().right_aligned())
        .title_bottom(instructions.centered())
        .title_bottom(Line::from(format!(" v{} ", env!("CARGO_PKG_VERSION"))).dark_gray().right_aligned())
        .border_set(border::THICK)
        .borders(Borders::ALL);

    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(50),
            Constraint::Percentage(25),
        ])
        .split(area);

    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(10),
            Constraint::Percentage(80),
            Constraint::Percentage(10),
        ])
        .split(vertical[1]);

    let loading_area = horizontal[1];

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let spinner_chars = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    let spinner = spinner_chars[(now / 100) as usize % spinner_chars.len()];

    let loading_text = if start_url.is_empty() {
        Text::from(vec![
            Line::from(vec![Span::styled(
                "Configuration Required",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Please configure your AWS SSO Start URL first.",
                Style::default().fg(Color::White),
            )]),
            Line::from(vec![Span::styled(
                "Press 'C' to open configuration.",
                Style::default().fg(Color::Gray),
            )]),
        ])
    } else {
        Text::from(vec![
            Line::from(vec![Span::styled(
                format!("{} Initializing AWS SSO...", spinner),
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Connecting to AWS and loading your accounts.",
                Style::default().fg(Color::White),
            )]),
            Line::from(vec![Span::styled(
                "This may take a moment on first run.",
                Style::default().fg(Color::Gray),
            )]),
        ])
    };

    let loading_paragraph = Paragraph::new(loading_text)
        .block(loading_block)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });

    f.render_widget(loading_paragraph, loading_area);
}
