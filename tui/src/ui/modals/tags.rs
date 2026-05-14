use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::app::TagsForm;
use crate::ui::{centered_rect, draw_clear, modal_block};

pub fn render(frame: &mut Frame, form: &TagsForm) {
    let area = centered_rect(frame.area(), 50, 60);
    draw_clear(frame, area);
    let block = modal_block(&format!("Tags: {}", truncate(&form.task_title, 36)));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if form.available.is_empty() {
        let p = Paragraph::new("No tags defined yet. Create them in Things 3 first.")
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(p, inner);
        return;
    }

    let items: Vec<ListItem> = form
        .available
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let checked = form.selected_indices.contains(&i);
            let mark = if checked { "[x]" } else { "[ ]" };
            ListItem::new(Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    mark,
                    Style::default().fg(if checked { Color::Green } else { Color::DarkGray }),
                ),
                Span::raw(" "),
                Span::styled(format!("#{}", name), Style::default().fg(Color::Magenta)),
            ]))
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
