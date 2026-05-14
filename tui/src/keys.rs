//! Central keymap. Keep all bindings here so swapping a key is a one-line change.
//!
//! Modal states (NewTask, Schedule, Tags, Search) handle their own keys directly —
//! this enum is for the always-active "browser" mode.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Quit,
    Help,
    Refresh,

    Up,
    Down,
    PageUp,
    PageDown,
    Top,
    Bottom,

    FocusLeft,
    FocusRight,
    FocusNext,
    FocusPrev,

    OpenEdit,
    ToggleComplete,
    ToggleCancel,
    NewTask,
    NewProject,
    Delete,
    EmptyTrash,
    Schedule,
    Tags,
    Move,
    Search,
    ShowQuickEntry,
}

pub fn browse_action(ev: &KeyEvent) -> Option<Action> {
    let ctrl = ev.modifiers.contains(KeyModifiers::CONTROL);
    let shift = ev.modifiers.contains(KeyModifiers::SHIFT);

    Some(match ev.code {
        KeyCode::Char('q') => Action::Quit,
        KeyCode::Char('?') => Action::Help,
        KeyCode::Char('r') if !ctrl => Action::Refresh,

        KeyCode::Char('j') | KeyCode::Down => Action::Down,
        KeyCode::Char('k') | KeyCode::Up => Action::Up,
        KeyCode::Char('h') | KeyCode::Left => Action::FocusLeft,
        KeyCode::Char('l') | KeyCode::Right => Action::FocusRight,
        KeyCode::Tab => Action::FocusNext,
        KeyCode::BackTab => Action::FocusPrev,

        KeyCode::PageUp => Action::PageUp,
        KeyCode::PageDown => Action::PageDown,
        KeyCode::Char('g') => Action::Top,
        KeyCode::Char('G') => Action::Bottom,

        KeyCode::Enter => Action::OpenEdit,
        KeyCode::Char(' ') => Action::ToggleComplete,
        KeyCode::Char('x') => Action::ToggleCancel,

        KeyCode::Char('n') if !shift => Action::NewTask,
        KeyCode::Char('N') => Action::NewProject,
        KeyCode::Char('d') => Action::Delete,
        KeyCode::Char('D') => Action::EmptyTrash,

        KeyCode::Char('s') => Action::Schedule,
        KeyCode::Char('t') => Action::Tags,
        KeyCode::Char('m') => Action::Move,
        KeyCode::Char('/') => Action::Search,

        KeyCode::Char('c') if ctrl => Action::ShowQuickEntry,

        _ => return None,
    })
}
