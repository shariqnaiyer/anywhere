use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

use crate::app::{App, SidebarNode, SpecialList};

pub fn render(frame: &mut Frame, app: &App, area: Rect, focused: bool) {
    let items: Vec<ListItem> = app
        .sidebar_nodes
        .iter()
        .map(|node| match node {
            SidebarNode::SectionHeader(name) => ListItem::new(Line::from(Span::styled(
                format!(" {}", name.to_uppercase()),
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            ))),
            SidebarNode::List(l) => ListItem::new(Line::from(vec![
                Span::raw(" "),
                Span::raw(list_icon(*l)),
                Span::raw(" "),
                Span::raw(l.label()),
            ])),
            SidebarNode::Area { title, .. } => ListItem::new(Line::from(vec![
                Span::raw("  "),
                Span::styled("◆ ", Style::default().fg(Color::Yellow)),
                Span::raw(title.clone()),
            ])),
            SidebarNode::Project { title, nested, .. } => {
                let indent = if *nested { "    " } else { "  " };
                ListItem::new(Line::from(vec![
                    Span::raw(indent),
                    Span::styled("◐ ", Style::default().fg(Color::Blue)),
                    Span::raw(title.clone()),
                ]))
            }
            SidebarNode::Tag { name, .. } => ListItem::new(Line::from(vec![
                Span::raw("  "),
                Span::styled("#", Style::default().fg(Color::Magenta)),
                Span::raw(name.clone()),
            ])),
        })
        .collect();

    let border = if focused {
        Color::Cyan
    } else {
        Color::DarkGray
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border))
        .title(Span::styled(" Lists ", Style::default().fg(border).add_modifier(Modifier::BOLD)));

    let mut state = ListState::default();
    state.select(Some(app.sidebar_cursor));

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

fn list_icon(l: SpecialList) -> &'static str {
    match l {
        SpecialList::Inbox => "📥",
        SpecialList::Today => "⭐",
        SpecialList::Upcoming => "📅",
        SpecialList::Anytime => "🗂",
        SpecialList::Someday => "💭",
        SpecialList::Logbook => "📚",
        SpecialList::Trash => "🗑",
    }
}
