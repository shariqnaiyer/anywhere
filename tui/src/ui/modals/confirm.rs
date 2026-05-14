use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::ui::{centered_rect, draw_clear, modal_block};

pub fn render(frame: &mut Frame, title: &str, body: &str) {
    let area = centered_rect(frame.area(), 50, 30);
    draw_clear(frame, area);
    let block = modal_block(title);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let lines: Vec<Line> = vec![
        Line::from(""),
        Line::from(body.to_string()),
        Line::from(""),
        Line::from(Span::styled(
            "  y/Enter confirm · any other key cancels",
            Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
        )),
    ];
    let p = Paragraph::new(lines);
    frame.render_widget(p, inner);
}
