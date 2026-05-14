use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, ListState},
    Frame,
};

use crate::app::{MoveDest, MoveForm};
use crate::ui::{centered_rect, draw_clear, modal_block};

pub fn render(frame: &mut Frame, form: &MoveForm) {
    let area = centered_rect(frame.area(), 55, 70);
    draw_clear(frame, area);
    let block = modal_block(&format!("Move: {}", truncate(&form.task_title, 40)));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let items: Vec<ListItem> = form
        .destinations
        .iter()
        .map(|d| match d {
            MoveDest::List(l) => ListItem::new(Line::from(vec![
                Span::raw("  → "),
                Span::raw(l.label()),
            ])),
            MoveDest::Project(_, title) => ListItem::new(Line::from(vec![
                Span::raw("  → Project: "),
                Span::styled(title.clone(), Style::default().fg(Color::Blue)),
            ])),
            MoveDest::Area(_, title) => ListItem::new(Line::from(vec![
                Span::raw("  → Area: "),
                Span::styled(title.clone(), Style::default().fg(Color::Yellow)),
            ])),
            MoveDest::Detach => ListItem::new(Line::from(Span::styled(
                "  ↺ Detach (no project / no area)",
                Style::default().fg(Color::DarkGray),
            ))),
        })
        .collect();
    let mut state = ListState::default();
    state.select(Some(form.cursor));
    let list = List::new(items).highlight_style(
        Style::default()
            .bg(Color::Cyan)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD),
    );
    frame.render_stateful_widget(list, inner, &mut state);
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
