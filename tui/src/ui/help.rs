use ratatui::{
    layout::Alignment,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::App;
use crate::ui::{centered_rect, draw_clear, modal_block};

pub fn render(frame: &mut Frame, _app: &App) {
    let area = centered_rect(frame.area(), 60, 70);
    draw_clear(frame, area);
    let block = modal_block("Help");

    let mut lines: Vec<Line> = Vec::new();

    push_header(&mut lines, "Navigation");
    push_row(&mut lines, "j / k", "move down / up");
    push_row(&mut lines, "h / l", "focus left / right");
    push_row(&mut lines, "Tab / Shift-Tab", "cycle pane focus");
    push_row(&mut lines, "PgUp / PgDn", "jump 10 rows");
    push_row(&mut lines, "g / G", "top / bottom");
    push_row(&mut lines, "r", "refresh");

    lines.push(Line::from(""));
    push_header(&mut lines, "Tasks");
    push_row(&mut lines, "Enter", "edit selected (or activate sidebar)");
    push_row(&mut lines, "Space", "toggle complete");
    push_row(&mut lines, "x", "cancel task");
    push_row(&mut lines, "n", "new task");
    push_row(&mut lines, "N", "new project");
    push_row(&mut lines, "d", "move to trash");
    push_row(&mut lines, "D", "empty trash");
    push_row(&mut lines, "s", "schedule (when)");
    push_row(&mut lines, "t", "edit tags");
    push_row(&mut lines, "m", "move to list/project/area");
    push_row(&mut lines, "/", "filter visible list");
    push_row(&mut lines, "Ctrl-C", "quick capture (Things popup)");

    lines.push(Line::from(""));
    push_header(&mut lines, "Modals");
    push_row(&mut lines, "Tab", "next form field");
    push_row(&mut lines, "Ctrl-S", "save form");
    push_row(&mut lines, "Esc", "close modal");

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "  Press any key to dismiss.",
        Style::default().fg(Color::DarkGray),
    )));

    let p = Paragraph::new(lines).block(block).alignment(Alignment::Left);
    frame.render_widget(p, area);
}

fn push_header(lines: &mut Vec<Line<'static>>, name: &str) {
    lines.push(Line::from(Span::styled(
        name.to_string(),
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    )));
}

fn push_row(lines: &mut Vec<Line<'static>>, key: &str, desc: &str) {
    lines.push(Line::from(vec![
        Span::styled(
            format!("  {:<14}", key),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(desc.to_string()),
    ]));
}
