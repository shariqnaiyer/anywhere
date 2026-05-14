use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::{EditTaskForm, NewTaskField, NewTaskForm};
use crate::ui::{centered_rect, draw_clear, modal_block};

pub fn render(frame: &mut Frame, form: &NewTaskForm) {
    let area = centered_rect(frame.area(), 70, 60);
    draw_clear(frame, area);
    let block = modal_block("New Task");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(4),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(2),
        ])
        .split(inner);

    draw_field(frame, rows[0], "Title", &form.title, form.focused == NewTaskField::Title);
    draw_field_multi(frame, rows[1], "Notes", &form.notes, form.focused == NewTaskField::Notes);
    draw_field(
        frame,
        rows[2],
        "When (today, tomorrow, May 20 2026…)",
        &form.when.specific,
        form.focused == NewTaskField::When,
    );
    draw_field(
        frame,
        rows[3],
        "Deadline (date)",
        &form.deadline.specific,
        form.focused == NewTaskField::Deadline,
    );
    draw_field(
        frame,
        rows[4],
        "Tags (comma-separated)",
        &form.tags,
        form.focused == NewTaskField::Tags,
    );

    let footer = Paragraph::new(Line::from(vec![
        Span::styled("Tab", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(" next  "),
        Span::styled("Ctrl-S", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(" save  "),
        Span::styled("Esc", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(" cancel"),
    ]))
    .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(footer, rows[5]);
}

pub fn render_edit(frame: &mut Frame, form: &EditTaskForm) {
    let area = centered_rect(frame.area(), 70, 60);
    draw_clear(frame, area);
    let block = modal_block(&format!("Edit Task: {}", truncate(&form.title, 40)));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(4),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(2),
        ])
        .split(inner);

    draw_field(frame, rows[0], "Title", &form.title, form.focused == NewTaskField::Title);
    draw_field_multi(frame, rows[1], "Notes", &form.notes, form.focused == NewTaskField::Notes);
    draw_field(
        frame,
        rows[2],
        "When",
        &form.when.specific,
        form.focused == NewTaskField::When,
    );
    draw_field(
        frame,
        rows[3],
        "Deadline",
        &form.deadline.specific,
        form.focused == NewTaskField::Deadline,
    );
    draw_field(
        frame,
        rows[4],
        "Tags",
        &form.tags,
        form.focused == NewTaskField::Tags,
    );

    let footer = Paragraph::new("Tab next · Ctrl-S save · Esc cancel")
        .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(footer, rows[5]);
}

fn draw_field(frame: &mut Frame, area: ratatui::layout::Rect, label: &str, value: &str, focused: bool) {
    let border = if focused { Color::Cyan } else { Color::DarkGray };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border))
        .title(Span::styled(
            format!(" {} ", label),
            Style::default().fg(border),
        ));
    let display = if focused {
        format!("{}_", value)
    } else {
        value.to_string()
    };
    frame.render_widget(Paragraph::new(display).block(block), area);
}

fn draw_field_multi(frame: &mut Frame, area: ratatui::layout::Rect, label: &str, value: &str, focused: bool) {
    let border = if focused { Color::Cyan } else { Color::DarkGray };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border))
        .title(Span::styled(
            format!(" {} ", label),
            Style::default().fg(border),
        ));
    let display = if focused {
        format!("{}_", value)
    } else {
        value.to_string()
    };
    frame.render_widget(
        Paragraph::new(display)
            .block(block)
            .wrap(ratatui::widgets::Wrap { trim: false }),
        area,
    );
}

fn truncate(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        s.to_string()
    } else {
        let mut out: String = s.chars().take(n).collect();
        out.push('…');
        out
    }
}
