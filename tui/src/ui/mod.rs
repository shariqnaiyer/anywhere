use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::app::{App, Modal, Pane, ToastKind};

pub mod detail;
pub mod help;
pub mod modals;
pub mod sidebar;
pub mod task_list;

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(area);

    draw_panes(frame, app, chunks[0]);
    draw_status_line(frame, app, chunks[1]);

    if let Some(modal) = &app.modal {
        draw_modal(frame, app, modal);
    }
}

fn draw_panes(frame: &mut Frame, app: &App, area: Rect) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(28),
            Constraint::Percentage(45),
            Constraint::Min(20),
        ])
        .split(area);

    sidebar::render(frame, app, cols[0], app.focus == Pane::Sidebar);
    task_list::render(frame, app, cols[1], app.focus == Pane::List);
    detail::render(frame, app, cols[2], app.focus == Pane::Detail);
}

fn draw_status_line(frame: &mut Frame, app: &App, area: Rect) {
    let mut spans: Vec<Span> = Vec::new();
    if let Some(toast) = &app.toast {
        let color = match toast.kind {
            ToastKind::Info => Color::Green,
            ToastKind::Error => Color::Red,
        };
        spans.push(Span::styled(
            format!(" {} ", toast.text),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ));
    } else {
        let help = "j/k nav · h/l focus · Enter edit · Space done · n new · t tags · s schedule · / search · ? help · q quit";
        spans.push(Span::styled(
            format!(" {} ", help),
            Style::default().fg(Color::DarkGray),
        ));
    }
    let p = Paragraph::new(Line::from(spans));
    frame.render_widget(p, area);
}

fn draw_modal(frame: &mut Frame, app: &App, modal: &Modal) {
    match modal {
        Modal::Help => help::render(frame, app),
        Modal::NewTask(form) => modals::new_task::render(frame, form),
        Modal::EditTask(form) => modals::new_task::render_edit(frame, form),
        Modal::NewProject(form) => modals::new_project::render(frame, form),
        Modal::Schedule(form) => modals::schedule::render(frame, form),
        Modal::Tags(form) => modals::tags::render(frame, form),
        Modal::MovePicker(form) => modals::move_pick::render(frame, form),
        Modal::Search(buf) => modals::search::render(frame, buf),
        Modal::ConfirmEmptyTrash => modals::confirm::render(
            frame,
            "Empty Trash?",
            "All items in the Trash will be permanently deleted. (y/N)",
        ),
        Modal::ConfirmDelete(_, title) => modals::confirm::render(
            frame,
            "Move to Trash?",
            &format!("\"{}\" will be moved to Trash. (y/N)", title),
        ),
    }
}

pub fn centered_rect(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vert = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vert[1])[1]
}

pub fn modal_block(title: &str) -> Block<'static> {
    let owned = format!(" {} ", title);
    Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta))
        .title(Span::styled(
            owned,
            Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
        ))
}

pub fn draw_clear(frame: &mut Frame, area: Rect) {
    frame.render_widget(Clear, area);
}

