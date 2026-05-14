use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::app::App;

pub fn render(frame: &mut Frame, app: &App, area: Rect, focused: bool) {
    let border = if focused {
        Color::Cyan
    } else {
        Color::DarkGray
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border))
        .title(Span::styled(
            " Detail ",
            Style::default().fg(border).add_modifier(Modifier::BOLD),
        ));

    let Some(task) = app.current_task() else {
        let p = Paragraph::new("Nothing selected.")
            .style(Style::default().fg(Color::DarkGray))
            .block(block);
        frame.render_widget(p, area);
        return;
    };

    let mut lines: Vec<Line> = Vec::new();

    lines.push(Line::from(Span::styled(
        task.title.clone(),
        Style::default().add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));

    if task.completed {
        lines.push(Line::from(Span::styled(
            "✓ Completed",
            Style::default().fg(Color::Green),
        )));
    }
    if task.canceled {
        lines.push(Line::from(Span::styled(
            "⊘ Canceled",
            Style::default().fg(Color::Red),
        )));
    }

    if let Some(notes) = &task.notes {
        if !notes.trim().is_empty() {
            lines.push(Line::from(Span::styled(
                "Notes",
                Style::default().fg(Color::DarkGray),
            )));
            for line in notes.lines() {
                lines.push(Line::from(line.to_string()));
            }
            lines.push(Line::from(""));
        }
    }

    let mut kv = |label: &str, value: &str| {
        lines.push(Line::from(vec![
            Span::styled(
                format!("{:<10}", label),
                Style::default().fg(Color::DarkGray),
            ),
            Span::raw(value.to_string()),
        ]));
    };

    if let Some(w) = &task.activation_date {
        kv("When", w);
    }
    if let Some(d) = &task.due_date {
        kv("Deadline", d);
    }
    if let Some(p) = &task.project {
        kv("Project", p);
    }
    if let Some(a) = &task.area {
        kv("Area", a);
    }
    if let Some(c) = &task.contact {
        kv("Contact", c);
    }
    if !task.tags.is_empty() {
        kv("Tags", &task.tags.iter().map(|t| format!("#{}", t)).collect::<Vec<_>>().join(" "));
    }
    if let Some(c) = &task.creation_date {
        kv("Created", c);
    }
    if let Some(m) = &task.modification_date {
        kv("Modified", m);
    }
    if let Some(c) = &task.completion_date {
        kv("Done at", c);
    }

    if !task.checklist_items.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Checklist",
            Style::default().fg(Color::DarkGray),
        )));
        for item in &task.checklist_items {
            let mark = if item.completed { "☑" } else { "☐" };
            lines.push(Line::from(format!("  {} {}", mark, item.title)));
        }
    }

    let p = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });
    frame.render_widget(p, area);
}
