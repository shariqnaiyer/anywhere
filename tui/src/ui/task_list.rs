use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::app::{App, Selection, SpecialList};

pub fn render(frame: &mut Frame, app: &App, area: Rect, focused: bool) {
    let border = if focused {
        Color::Cyan
    } else {
        Color::DarkGray
    };
    let title = list_title(app);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border))
        .title(Span::styled(
            format!(" {} ", title),
            Style::default().fg(border).add_modifier(Modifier::BOLD),
        ));

    let visible = app.visible_tasks();
    if visible.is_empty() {
        let msg = if app.tasks_loading {
            "Loading…"
        } else {
            "No tasks here."
        };
        let p = Paragraph::new(msg)
            .style(Style::default().fg(Color::DarkGray))
            .block(block);
        frame.render_widget(p, area);
        return;
    }

    let items: Vec<ListItem> = visible
        .iter()
        .map(|t| {
            let mut spans: Vec<Span> = Vec::new();
            let checkbox = if t.completed {
                "☑ "
            } else if t.canceled {
                "⊘ "
            } else {
                "☐ "
            };
            spans.push(Span::styled(
                checkbox,
                Style::default().fg(if t.completed {
                    Color::Green
                } else if t.canceled {
                    Color::Red
                } else {
                    Color::Gray
                }),
            ));
            let title_style = if t.completed || t.canceled {
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::CROSSED_OUT)
            } else {
                Style::default()
            };
            spans.push(Span::styled(t.title.clone(), title_style));
            if let Some(p) = &t.project {
                spans.push(Span::styled(
                    format!("  · {}", p),
                    Style::default().fg(Color::Blue),
                ));
            } else if let Some(a) = &t.area {
                spans.push(Span::styled(
                    format!("  · {}", a),
                    Style::default().fg(Color::Yellow),
                ));
            }
            if !t.tags.is_empty() {
                spans.push(Span::raw("  "));
                for tag in &t.tags {
                    spans.push(Span::styled(
                        format!("#{} ", tag),
                        Style::default().fg(Color::Magenta),
                    ));
                }
            }
            if let Some(d) = &t.due_date {
                spans.push(Span::styled(
                    format!("  ⚑ {}", short_date(d)),
                    Style::default().fg(Color::Red),
                ));
            }
            ListItem::new(Line::from(spans))
        })
        .collect();

    let mut state = ListState::default();
    state.select(Some(app.task_cursor.min(visible.len().saturating_sub(1))));

    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .bg(Color::Cyan)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▎");
    frame.render_stateful_widget(list, area, &mut state);
}

fn list_title(app: &App) -> String {
    match &app.selection {
        Selection::List(SpecialList::Inbox) => "Inbox".into(),
        Selection::List(SpecialList::Today) => "Today".into(),
        Selection::List(SpecialList::Upcoming) => "Upcoming".into(),
        Selection::List(SpecialList::Anytime) => "Anytime".into(),
        Selection::List(SpecialList::Someday) => "Someday".into(),
        Selection::List(SpecialList::Logbook) => "Logbook".into(),
        Selection::List(SpecialList::Trash) => "Trash".into(),
        Selection::Area(id) => app
            .areas
            .iter()
            .find(|a| &a.id == id)
            .map(|a| a.title.clone())
            .unwrap_or_else(|| "Area".into()),
        Selection::Project(id) => app
            .projects
            .iter()
            .find(|p| &p.id == id)
            .map(|p| p.title.clone())
            .unwrap_or_else(|| "Project".into()),
        Selection::Tag(id) => app
            .tags
            .iter()
            .find(|t| &t.id == id)
            .map(|t| format!("#{}", t.name))
            .unwrap_or_else(|| "Tag".into()),
    }
}

fn short_date(d: &str) -> String {
    // Things returns RFC-ish strings; just take the first 10 chars if it looks like a date.
    if d.len() >= 10 && d.chars().nth(4) == Some('-') {
        d[..10].to_string()
    } else {
        d.to_string()
    }
}
