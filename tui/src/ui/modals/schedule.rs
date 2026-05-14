use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::app::{DateChoice, ScheduleForm};
use crate::ui::{centered_rect, draw_clear, modal_block};

pub fn render(frame: &mut Frame, form: &ScheduleForm) {
    let area = centered_rect(frame.area(), 50, 60);
    draw_clear(frame, area);
    let block = modal_block(&format!("Schedule: {}", truncate(&form.task_title, 40)));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if form.editing_specific {
        let p = Paragraph::new(vec![
            Line::from(""),
            Line::from("Enter date (e.g. \"May 20, 2026\")"),
            Line::from(""),
            Line::from(format!("  > {}_", form.specific_buffer)),
            Line::from(""),
            Line::from(Span::styled(
                "Enter to confirm · Esc to back out",
                Style::default().fg(Color::DarkGray),
            )),
        ]);
        frame.render_widget(p, inner);
        return;
    }

    let items: Vec<ListItem> = form
        .choices
        .iter()
        .map(|c| ListItem::new(label(*c)))
        .collect();
    let mut state = ListState::default();
    state.select(Some(form.selected));
    let list = List::new(items).highlight_style(
        Style::default()
            .bg(Color::Cyan)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD),
    );
    frame.render_stateful_widget(list, inner, &mut state);
}

fn label(c: DateChoice) -> &'static str {
    match c {
        DateChoice::Today => "  Today",
        DateChoice::Tomorrow => "  Tomorrow",
        DateChoice::ThisWeekend => "  This Weekend (Saturday)",
        DateChoice::NextWeek => "  Next Week (Monday)",
        DateChoice::Anytime => "  Anytime",
        DateChoice::Someday => "  Someday",
        DateChoice::Specific => "  Specific date…",
        DateChoice::Clear => "  Clear schedule",
    }
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
