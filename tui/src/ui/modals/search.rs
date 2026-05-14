use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::ui::{centered_rect, draw_clear, modal_block};

pub fn render(frame: &mut Frame, buffer: &str) {
    let area = centered_rect(frame.area(), 60, 20);
    draw_clear(frame, area);
    let block = modal_block("Filter visible list");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1), Constraint::Min(0)])
        .split(inner);

    let p = Paragraph::new(Line::from(vec![
        Span::raw("  > "),
        Span::raw(format!("{}_", buffer)),
    ]));
    frame.render_widget(p, rows[0]);

    let hint = Paragraph::new("Enter to apply · Esc to cancel · empty to clear")
        .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(hint, rows[1]);
}
