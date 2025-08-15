use std::rc::Rc;

use ratatui::{    
    crossterm::event::{KeyCode, KeyEvent}, layout::{Alignment, Constraint, Direction, Layout, Rect}, style::{Color, Modifier, Style, Stylize}, symbols::border, text::{Line, Span, Text}, widgets::{
        block::{Position, Title}, Block, Borders, Cell, Clear, Paragraph, Row, Table, Wrap
    }, Frame
};

use crate::app::App;

pub fn get_layout(f: &mut Frame) -> Rc<[Rect]> {
    Layout::horizontal([Constraint::Min(5)]).split(f.size())
}

pub fn handle_key_events(app: &mut App, key: KeyEvent) -> Result<(), ()>{
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
    // If authenticating, show authentication UI instead of account list
    if app.authenticating {
        render_authentication_ui(f, app, area);
        return;
    }

    let style = {
        if app.is_selected {
            Style::new().white()
        } else {
            Style::new().blue()
        }
    };
    let instructions = Title::from(Line::from(vec![
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
    ]));
    
    let start_url = app.config_options.options.iter().find(|option| option.name == "start_url").unwrap().value.clone();
    let url_title = Title::from(format!(" Start URL: {} ", start_url).bold());

    let account_list_title = Title::from(format!(" Accounts ({}) ", app.rows.len()).bold());        
    let account_list_block = Block::bordered()
        .title(account_list_title.alignment(Alignment::Left))   
        .title(instructions
            .alignment(Alignment::Center)
            .position(Position::Bottom)
        )   
        .title (url_title.alignment(Alignment::Right))     
        .border_set(border::THICK);

    let widths = [
        Constraint::Min(10),
        Constraint::Min(20)
    ];

    let rows = app.rows.iter().map(|row| {
        Row::new(vec![
            Cell::from(row.account_name.clone()),
            Cell::from(row.account_id.clone())
        ])
    });    

    let footer_row = Row::new(vec![
        Cell::from("Selected Account:").style(Style::new().bold()),
        Cell::from(app.selected_account.account_id.clone()).style(Style::new().bold().yellow())
    ]);    

    let table = Table::new(rows, widths)
        .column_spacing(1)
        .style(style)
        .header(
            Row::new(vec!["Account Name", "Account ID"])
                .style(Style::new().bold())                            
        )                                
        .footer(footer_row)
        .block(account_list_block)
        .highlight_style(Style::new().reversed())
        .highlight_symbol(">>");

    f.render_stateful_widget(table, area, &mut app.table_state);
}

pub fn render_authentication_ui(f: &mut Frame, app: &mut App, area: Rect) {

    let instructions = Title::from(Line::from(vec![
        " Cancel ".into(),
        "<ESC> ".red().bold(),
    ]));

    let auth_title = Title::from(" Authenticating with AWS SSO ".bold());
    let auth_block = Block::bordered()
        .title(auth_title.alignment(Alignment::Center))
        .title(instructions
            .alignment(Alignment::Center)
            .position(Position::Bottom)
        )
        .border_set(border::THICK)
        .borders(Borders::ALL);

    // Create centered layout for auth message
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

    // Authentication message text
    let auth_text = if app.token_prompt.is_empty() {
        Text::from(vec![
            Line::from(vec![
                Span::styled("Authenticating with AWS SSO...", Style::default().fg(Color::Yellow)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Please wait while we authenticate with AWS.", Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled("Your browser should open automatically for authentication.", Style::default().fg(Color::Gray)),
            ]),
        ])
    } else {
        Text::from(vec![
            Line::from(vec![
                Span::styled("AWS SSO Authentication", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(&app.token_prompt, Style::default().fg(Color::White)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Complete authentication in your browser, then return here.", Style::default().fg(Color::Gray)),
            ]),
        ])
    };

    let auth_paragraph = Paragraph::new(auth_text)
        .block(auth_block)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });

    // Clear the area first to ensure clean rendering
    f.render_widget(Clear, area);
    f.render_widget(auth_paragraph, auth_area);
}