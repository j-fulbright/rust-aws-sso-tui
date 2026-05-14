use std::rc::Rc;

use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    symbols::border,
    text::Line,
    widgets::{Block, Cell, Row, Table},
    Frame,
};

use crate::app::App;

pub fn get_layout(f: &mut Frame) -> Rc<[Rect]> {
    Layout::horizontal([Constraint::Min(5), Constraint::Min(5)]).split(f.area())
}

pub fn handle_key_events(app: &mut App, key: KeyEvent) -> Result<(), ()> {
    match key.code {
        KeyCode::Down => {
            app.next_role();
        }
        KeyCode::Up => {
            app.previous_role();
        }
        KeyCode::Left => {
            app.is_selected = false;
            app.current_page = crate::app::CurrentPage::AccountList;
        }
        KeyCode::Right => {
            app.select_role();
            app.current_page = crate::app::CurrentPage::Credentials;
        }
        KeyCode::Char('q') => {
            app.exit = true;
        }
        _ => {}
    }

    Ok(())
}

pub fn render_roles(f: &mut Frame, app: &mut App, area: Rect) {
    let instructions = Line::from(vec![
        " Scroll Up ".into(),
        "<Up>".blue().bold(),
        " Scroll Down ".into(),
        "<Down>".blue().bold(),
        " Select Role ".into(),
        "<Enter>".blue().bold(),
        " Back ".into(),
        "<Left>".blue().bold(),
        " Quit ".into(),
        "<Q> ".blue().bold(),
    ]);

    let role_list_block = Block::bordered()
        .title_top(Line::from(format!(" {} - Roles ", app.selected_account.account_name)).bold().left_aligned())
        .title_bottom(instructions.centered())
        .border_set(border::THICK);

    let widths = [Constraint::Min(10)];

    let rows = app.selected_account.roles.iter().map(|row| {
        Row::new(vec![Cell::from(row.clone())])
    });

    let table = Table::new(rows, widths)
        .column_spacing(1)
        .style(Style::new().blue())
        .header(Row::new(vec!["Role"]).style(Style::new().bold()))
        .block(role_list_block)
        .row_highlight_style(Style::new().reversed())
        .highlight_symbol(">>");

    f.render_stateful_widget(table, area, &mut app.role_table_state);
}
