use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::Span,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::{NewProjectField, NewProjectForm};
use crate::ui::{centered_rect, draw_clear, modal_block};

pub fn render(frame: &mut Frame, form: &NewProjectForm) {
    let area = centered_rect(frame.area(), 60, 50);
    draw_clear(frame, area);
    let block = modal_block("New Project");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(4),
            Constraint::Length(3),
            Constraint::Length(2),
        ])
        .split(inner);

    field(frame, rows[0], "Title", &form.title, form.focused == NewProjectField::Title);
    field_multi(frame, rows[1], "Notes", &form.notes, form.focused == NewProjectField::Notes);
    field(frame, rows[2], "Area (name)", &form.area, form.focused == NewProjectField::Area);

    let footer = Paragraph::new("Tab next · Ctrl-S save · Esc cancel")
        .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(footer, rows[3]);
}

fn field(frame: &mut Frame, area: ratatui::layout::Rect, label: &str, value: &str, focused: bool) {
    let border = if focused { Color::Cyan } else { Color::DarkGray };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border))
        .title(Span::styled(format!(" {} ", label), Style::default().fg(border)));
    let display = if focused { format!("{}_", value) } else { value.to_string() };
    frame.render_widget(Paragraph::new(display).block(block), area);
}

fn field_multi(frame: &mut Frame, area: ratatui::layout::Rect, label: &str, value: &str, focused: bool) {
    let border = if focused { Color::Cyan } else { Color::DarkGray };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border))
        .title(Span::styled(format!(" {} ", label), Style::default().fg(border)));
    let display = if focused { format!("{}_", value) } else { value.to_string() };
    frame.render_widget(
        Paragraph::new(display)
            .block(block)
            .wrap(ratatui::widgets::Wrap { trim: false }),
        area,
    );
}
