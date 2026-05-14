//! Application state + the main message-passing loop.
//!
//! `App` owns everything mutable. UI rendering is a pure function of `App`.
//! All network calls run in spawned tasks; they send `Message`s back through
//! an mpsc channel which the loop drains every tick.

use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::event::{Event, EventStream, KeyCode, KeyEventKind, KeyModifiers};
use futures::StreamExt;
use ratatui::Terminal;
use ratatui::backend::Backend;
use tokio::sync::mpsc;

use crate::api::models::{
    Area, CreateProject, CreateTask, Project, Tag, Task, UpdateTask,
};
use crate::api::ApiClient;
use crate::config::{Probe, TuiConfig};
use crate::keys::{browse_action, Action};
use crate::ui;

/// Top-level navigation node in the sidebar.
#[derive(Debug, Clone, PartialEq)]
pub enum Selection {
    List(SpecialList),
    Area(String),    // id
    Project(String), // id
    Tag(String),     // id
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecialList {
    Inbox,
    Today,
    Upcoming,
    Anytime,
    Someday,
    Logbook,
    Trash,
}

impl SpecialList {
    pub fn label(self) -> &'static str {
        match self {
            Self::Inbox => "Inbox",
            Self::Today => "Today",
            Self::Upcoming => "Upcoming",
            Self::Anytime => "Anytime",
            Self::Someday => "Someday",
            Self::Logbook => "Logbook",
            Self::Trash => "Trash",
        }
    }

    pub fn api_value(self) -> &'static str {
        match self {
            Self::Inbox => "inbox",
            Self::Today => "today",
            Self::Upcoming => "upcoming",
            Self::Anytime => "anytime",
            Self::Someday => "someday",
            Self::Logbook => "logbook",
            Self::Trash => "trash",
        }
    }

    pub const ALL: [SpecialList; 7] = [
        Self::Inbox,
        Self::Today,
        Self::Upcoming,
        Self::Anytime,
        Self::Someday,
        Self::Logbook,
        Self::Trash,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pane {
    Sidebar,
    List,
    Detail,
}

impl Pane {
    pub fn next(self) -> Self {
        match self {
            Self::Sidebar => Self::List,
            Self::List => Self::Detail,
            Self::Detail => Self::Sidebar,
        }
    }
    pub fn prev(self) -> Self {
        match self {
            Self::Sidebar => Self::Detail,
            Self::List => Self::Sidebar,
            Self::Detail => Self::List,
        }
    }
}

#[derive(Debug, Clone)]
pub struct StatusToast {
    pub text: String,
    pub kind: ToastKind,
    pub at: Instant,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ToastKind {
    Info,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateChoice {
    Today,
    Tomorrow,
    ThisWeekend,
    NextWeek,
    Someday,
    Anytime,
    Clear,
    Specific,
}

#[derive(Debug, Clone)]
pub enum Modal {
    Help,
    NewTask(NewTaskForm),
    EditTask(EditTaskForm),
    NewProject(NewProjectForm),
    Schedule(ScheduleForm),
    Tags(TagsForm),
    MovePicker(MoveForm),
    Search(String),
    ConfirmEmptyTrash,
    ConfirmDelete(String /* task id */, String /* title */),
}

#[derive(Debug, Clone, Default)]
pub struct NewTaskForm {
    pub title: String,
    pub notes: String,
    pub when: DateField,
    pub deadline: DateField,
    pub tags: String,
    pub focused: NewTaskField,
}

#[derive(Debug, Clone, Default)]
pub struct EditTaskForm {
    pub task_id: String,
    pub title: String,
    pub notes: String,
    pub when: DateField,
    pub deadline: DateField,
    pub tags: String,
    pub focused: NewTaskField,
}

#[derive(Debug, Clone, Default)]
pub struct NewProjectForm {
    pub title: String,
    pub notes: String,
    pub area: String,
    pub focused: NewProjectField,
}

#[derive(Debug, Clone)]
pub struct ScheduleForm {
    pub task_id: String,
    pub task_title: String,
    pub choices: Vec<DateChoice>,
    pub selected: usize,
    pub specific_buffer: String,
    pub editing_specific: bool,
}

#[derive(Debug, Clone)]
pub struct TagsForm {
    pub task_id: String,
    pub task_title: String,
    pub available: Vec<String>,
    pub selected_indices: Vec<usize>, // indices into available
    pub cursor: usize,
}

#[derive(Debug, Clone)]
pub struct MoveForm {
    pub task_id: String,
    pub task_title: String,
    pub destinations: Vec<MoveDest>,
    pub cursor: usize,
}

#[derive(Debug, Clone)]
#[allow(dead_code)] // Project/Area id is unused today but kept for future "show in Things" wiring.
pub enum MoveDest {
    List(SpecialList),
    Project(String /* id */, String /* title */),
    Area(String /* id */, String /* title */),
    Detach,
}

#[derive(Debug, Clone, Default)]
pub struct DateField {
    pub choice: Option<DateChoice>,
    pub specific: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum NewTaskField {
    #[default]
    Title,
    Notes,
    When,
    Deadline,
    Tags,
}

impl NewTaskField {
    pub const ORDER: [Self; 5] = [
        Self::Title,
        Self::Notes,
        Self::When,
        Self::Deadline,
        Self::Tags,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum NewProjectField {
    #[default]
    Title,
    Notes,
    Area,
}

impl NewProjectField {
    pub const ORDER: [Self; 3] = [Self::Title, Self::Notes, Self::Area];
}

/// Network responses delivered back to the event loop.
#[derive(Debug)]
#[allow(dead_code)] // `Tick` and `Toast` are reserved for future producers; their variants compile-check the loop.
pub enum Message {
    Tick,
    TasksLoaded {
        for_selection: Selection,
        tasks: Result<Vec<Task>, String>,
    },
    SidebarLoaded {
        areas: Result<Vec<Area>, String>,
        projects: Result<Vec<Project>, String>,
        tags: Result<Vec<Tag>, String>,
    },
    Toast(StatusToast),
    /// A mutation finished; refresh the current list.
    MutationDone(Result<String, String>),
}

pub struct App {
    pub client: ApiClient,
    #[allow(dead_code)]
    pub config: TuiConfig,
    #[allow(dead_code)]
    pub probe: Probe,

    pub focus: Pane,
    pub sidebar_cursor: usize,
    pub sidebar_nodes: Vec<SidebarNode>,
    pub selection: Selection,

    pub tasks: Vec<Task>,
    pub task_cursor: usize,
    pub tasks_loading: bool,

    pub areas: Vec<Area>,
    pub projects: Vec<Project>,
    pub tags: Vec<Tag>,

    pub modal: Option<Modal>,
    pub toast: Option<StatusToast>,
    pub last_refresh: Instant,
    pub should_quit: bool,
    pub filter: Option<String>,

    pub tx: mpsc::UnboundedSender<Message>,
}

#[derive(Debug, Clone)]
pub enum SidebarNode {
    SectionHeader(&'static str),
    List(SpecialList),
    Area { id: String, title: String },
    /// `nested = true` when the project sits under its area in the tree, so we
    /// render it with extra indent. Projects with no area are flat.
    Project { id: String, title: String, nested: bool },
    Tag { id: String, name: String },
}

impl SidebarNode {
    pub fn is_selectable(&self) -> bool {
        !matches!(self, SidebarNode::SectionHeader(_))
    }
}

impl App {
    pub fn new(client: ApiClient, config: TuiConfig, probe: Probe) -> (Self, mpsc::UnboundedReceiver<Message>) {
        let (tx, rx) = mpsc::unbounded_channel();
        let sidebar_nodes = Self::build_sidebar(&[], &[], &[]);
        let app = Self {
            client,
            config,
            probe,
            focus: Pane::List,
            sidebar_cursor: 1, // skip first header
            sidebar_nodes,
            selection: Selection::List(SpecialList::Today),
            tasks: vec![],
            task_cursor: 0,
            tasks_loading: false,
            areas: vec![],
            projects: vec![],
            tags: vec![],
            modal: None,
            toast: None,
            last_refresh: Instant::now(),
            should_quit: false,
            filter: None,
            tx,
        };
        (app, rx)
    }

    pub fn build_sidebar(
        areas: &[Area],
        projects: &[Project],
        tags: &[Tag],
    ) -> Vec<SidebarNode> {
        let mut out = vec![SidebarNode::SectionHeader("Lists")];
        for l in SpecialList::ALL {
            out.push(SidebarNode::List(l));
        }

        // Partition projects: those with a parent area go under their area;
        // the rest land in a flat "Projects" group.
        let active_projects: Vec<&Project> =
            projects.iter().filter(|p| !p.completed && !p.canceled).collect();
        let mut orphans: Vec<&Project> = Vec::new();
        let mut by_area: std::collections::HashMap<String, Vec<&Project>> =
            std::collections::HashMap::new();
        for p in active_projects {
            match p.area.as_deref() {
                Some(a) if !a.is_empty() => by_area.entry(a.to_string()).or_default().push(p),
                _ => orphans.push(p),
            }
        }

        if !areas.is_empty() {
            out.push(SidebarNode::SectionHeader("Areas"));
            for a in areas {
                out.push(SidebarNode::Area {
                    id: a.id.clone(),
                    title: a.title.clone(),
                });
                if let Some(ps) = by_area.get(&a.title) {
                    for p in ps {
                        out.push(SidebarNode::Project {
                            id: p.id.clone(),
                            title: p.title.clone(),
                            nested: true,
                        });
                    }
                }
            }
        }

        if !orphans.is_empty() {
            out.push(SidebarNode::SectionHeader("Projects"));
            for p in orphans {
                out.push(SidebarNode::Project {
                    id: p.id.clone(),
                    title: p.title.clone(),
                    nested: false,
                });
            }
        }

        if !tags.is_empty() {
            out.push(SidebarNode::SectionHeader("Tags"));
            for t in tags {
                out.push(SidebarNode::Tag {
                    id: t.id.clone(),
                    name: t.name.clone(),
                });
            }
        }
        out
    }

    pub fn current_task(&self) -> Option<&Task> {
        self.visible_tasks().get(self.task_cursor).copied()
    }

    pub fn visible_tasks(&self) -> Vec<&Task> {
        match &self.filter {
            None => self.tasks.iter().collect(),
            Some(f) if f.is_empty() => self.tasks.iter().collect(),
            Some(f) => {
                let needle = f.to_lowercase();
                self.tasks
                    .iter()
                    .filter(|t| {
                        t.title.to_lowercase().contains(&needle)
                            || t.notes
                                .as_deref()
                                .map(|n| n.to_lowercase().contains(&needle))
                                .unwrap_or(false)
                            || t.tags.iter().any(|tg| tg.to_lowercase().contains(&needle))
                    })
                    .collect()
            }
        }
    }

    pub fn schedule_initial_loads(&self) {
        self.spawn_sidebar_load();
        self.spawn_task_load(self.selection.clone());
    }

    pub fn spawn_sidebar_load(&self) {
        let client = self.client.clone();
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let (areas, projects, tags) = tokio::join!(
                client.list_areas(),
                client.list_projects(),
                client.list_tags(),
            );
            let _ = tx.send(Message::SidebarLoaded { areas, projects, tags });
        });
    }

    pub fn spawn_task_load(&self, selection: Selection) {
        let client = self.client.clone();
        let tx = self.tx.clone();
        let sel = selection.clone();
        tokio::spawn(async move {
            let result = match &sel {
                Selection::List(l) => client.list_tasks(Some(l.api_value())).await,
                Selection::Area(id) => client.list_area_tasks(id).await,
                Selection::Project(id) => client.list_project_tasks(id).await,
                Selection::Tag(id) => client.list_tag_tasks(id).await,
            };
            let _ = tx.send(Message::TasksLoaded {
                for_selection: sel,
                tasks: result,
            });
        });
    }

    /// Handle a keypress in browse mode (no modal open).
    fn handle_browse_key(&mut self, action: Action) {
        match action {
            Action::Quit => self.should_quit = true,
            Action::Help => self.modal = Some(Modal::Help),
            Action::Refresh => {
                self.spawn_sidebar_load();
                self.spawn_task_load(self.selection.clone());
                self.toast = Some(StatusToast {
                    text: "Refreshing…".into(),
                    kind: ToastKind::Info,
                    at: Instant::now(),
                });
            }
            Action::Up => self.move_cursor(-1),
            Action::Down => self.move_cursor(1),
            Action::PageUp => self.move_cursor(-10),
            Action::PageDown => self.move_cursor(10),
            Action::Top => self.set_cursor(0),
            Action::Bottom => self.set_cursor(i32::MAX),
            Action::FocusLeft => self.focus = Pane::Sidebar,
            Action::FocusRight => self.focus = self.focus.next(),
            Action::FocusNext => self.focus = self.focus.next(),
            Action::FocusPrev => self.focus = self.focus.prev(),
            Action::OpenEdit => match self.focus {
                Pane::Sidebar => self.activate_sidebar_selection(),
                Pane::List | Pane::Detail => self.open_edit_modal(),
            },
            Action::ToggleComplete => self.toggle_complete(),
            Action::ToggleCancel => self.toggle_cancel(),
            Action::NewTask => self.modal = Some(Modal::NewTask(NewTaskForm::default())),
            Action::NewProject => {
                self.modal = Some(Modal::NewProject(NewProjectForm::default()))
            }
            Action::Delete => {
                if let Some(t) = self.current_task() {
                    self.modal = Some(Modal::ConfirmDelete(t.id.clone(), t.title.clone()));
                }
            }
            Action::EmptyTrash => self.modal = Some(Modal::ConfirmEmptyTrash),
            Action::Schedule => self.open_schedule_modal(),
            Action::Tags => self.open_tags_modal(),
            Action::Move => self.open_move_modal(),
            Action::Search => self.modal = Some(Modal::Search(String::new())),
            Action::ShowQuickEntry => self.show_quick_entry(),
        }
    }

    fn move_cursor(&mut self, delta: i32) {
        match self.focus {
            Pane::Sidebar => {
                let len = self.sidebar_nodes.len() as i32;
                if len == 0 {
                    return;
                }
                let mut idx = self.sidebar_cursor as i32 + delta;
                // Skip non-selectable section headers
                let step = if delta >= 0 { 1 } else { -1 };
                idx = idx.clamp(0, len - 1);
                while idx >= 0 && idx < len && !self.sidebar_nodes[idx as usize].is_selectable() {
                    idx += step;
                    if idx < 0 || idx >= len {
                        // try the other direction from the original
                        idx = (self.sidebar_cursor as i32).clamp(0, len - 1);
                        break;
                    }
                }
                self.sidebar_cursor = idx.clamp(0, len - 1) as usize;
                self.activate_sidebar_selection();
            }
            Pane::List | Pane::Detail => {
                let len = self.visible_tasks().len() as i32;
                if len == 0 {
                    self.task_cursor = 0;
                    return;
                }
                let mut idx = self.task_cursor as i32 + delta;
                idx = idx.clamp(0, len - 1);
                self.task_cursor = idx as usize;
            }
        }
    }

    fn set_cursor(&mut self, idx: i32) {
        match self.focus {
            Pane::Sidebar => {
                let last = self.sidebar_nodes.len().saturating_sub(1) as i32;
                self.sidebar_cursor = idx.clamp(0, last) as usize;
                if !self.sidebar_nodes[self.sidebar_cursor].is_selectable() {
                    self.move_cursor(if idx == 0 { 1 } else { -1 });
                }
                self.activate_sidebar_selection();
            }
            Pane::List | Pane::Detail => {
                let last = self.visible_tasks().len().saturating_sub(1) as i32;
                self.task_cursor = idx.clamp(0, last.max(0)) as usize;
            }
        }
    }

    fn activate_sidebar_selection(&mut self) {
        let Some(node) = self.sidebar_nodes.get(self.sidebar_cursor) else {
            return;
        };
        let new_sel = match node {
            SidebarNode::SectionHeader(_) => return,
            SidebarNode::List(l) => Selection::List(*l),
            SidebarNode::Area { id, .. } => Selection::Area(id.clone()),
            SidebarNode::Project { id, .. } => Selection::Project(id.clone()),
            SidebarNode::Tag { id, .. } => Selection::Tag(id.clone()),
        };
        if new_sel != self.selection {
            self.selection = new_sel.clone();
            self.tasks.clear();
            self.task_cursor = 0;
            self.tasks_loading = true;
            self.spawn_task_load(new_sel);
        }
    }

    fn toggle_complete(&mut self) {
        let Some(t) = self.current_task().cloned() else {
            return;
        };
        let already_done = t.completed;
        let client = self.client.clone();
        let tx = self.tx.clone();
        let id = t.id.clone();
        if !already_done {
            // optimistic
            if let Some(local) = self.tasks.iter_mut().find(|x| x.id == id) {
                local.completed = true;
            }
            tokio::spawn(async move {
                let r = client.complete_task(&id).await.map(|()| "Completed".to_string());
                let _ = tx.send(Message::MutationDone(r));
            });
        } else {
            // un-complete by patching `list` back to anytime (the only way our server allows)
            let body = UpdateTask {
                list: Some("anytime".into()),
                ..Default::default()
            };
            tokio::spawn(async move {
                let r = client.update_task(&id, &body).await.map(|_| "Reopened".to_string());
                let _ = tx.send(Message::MutationDone(r));
            });
        }
    }

    fn toggle_cancel(&mut self) {
        let Some(t) = self.current_task().cloned() else {
            return;
        };
        let client = self.client.clone();
        let tx = self.tx.clone();
        let id = t.id.clone();
        tokio::spawn(async move {
            let r = client.cancel_task(&id).await.map(|_| "Canceled".to_string());
            let _ = tx.send(Message::MutationDone(r));
        });
    }

    fn open_edit_modal(&mut self) {
        let Some(t) = self.current_task().cloned() else {
            return;
        };
        let form = EditTaskForm {
            task_id: t.id,
            title: t.title,
            notes: t.notes.unwrap_or_default(),
            when: DateField {
                choice: None,
                specific: t.activation_date.unwrap_or_default(),
            },
            deadline: DateField {
                choice: None,
                specific: t.due_date.unwrap_or_default(),
            },
            tags: t.tags.join(", "),
            focused: NewTaskField::Title,
        };
        self.modal = Some(Modal::EditTask(form));
    }

    fn open_schedule_modal(&mut self) {
        let Some(t) = self.current_task().cloned() else {
            return;
        };
        self.modal = Some(Modal::Schedule(ScheduleForm {
            task_id: t.id,
            task_title: t.title,
            choices: vec![
                DateChoice::Today,
                DateChoice::Tomorrow,
                DateChoice::ThisWeekend,
                DateChoice::NextWeek,
                DateChoice::Anytime,
                DateChoice::Someday,
                DateChoice::Specific,
                DateChoice::Clear,
            ],
            selected: 0,
            specific_buffer: String::new(),
            editing_specific: false,
        }));
    }

    fn open_tags_modal(&mut self) {
        let Some(t) = self.current_task().cloned() else {
            return;
        };
        let available: Vec<String> = self.tags.iter().map(|x| x.name.clone()).collect();
        let selected_indices: Vec<usize> = available
            .iter()
            .enumerate()
            .filter_map(|(i, name)| if t.tags.iter().any(|x| x == name) { Some(i) } else { None })
            .collect();
        self.modal = Some(Modal::Tags(TagsForm {
            task_id: t.id,
            task_title: t.title,
            available,
            selected_indices,
            cursor: 0,
        }));
    }

    fn open_move_modal(&mut self) {
        let Some(t) = self.current_task().cloned() else {
            return;
        };
        let mut destinations: Vec<MoveDest> = SpecialList::ALL
            .iter()
            .copied()
            .filter(|l| !matches!(l, SpecialList::Logbook | SpecialList::Trash | SpecialList::Upcoming))
            .map(MoveDest::List)
            .collect();
        destinations.push(MoveDest::Detach);
        for p in &self.projects {
            if p.completed || p.canceled {
                continue;
            }
            destinations.push(MoveDest::Project(p.id.clone(), p.title.clone()));
        }
        for a in &self.areas {
            destinations.push(MoveDest::Area(a.id.clone(), a.title.clone()));
        }
        self.modal = Some(Modal::MovePicker(MoveForm {
            task_id: t.id,
            task_title: t.title,
            destinations,
            cursor: 0,
        }));
    }

    fn show_quick_entry(&self) {
        let client = self.client.clone();
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let r = client.show_quick_entry().await.map(|_| "Quick Entry opened".to_string());
            let _ = tx.send(Message::MutationDone(r));
        });
    }

    pub fn finish_new_task(&mut self, form: &NewTaskForm) {
        if form.title.trim().is_empty() {
            self.toast = Some(StatusToast {
                text: "Title is required".into(),
                kind: ToastKind::Error,
                at: Instant::now(),
            });
            return;
        }
        let mut body = CreateTask {
            title: form.title.trim().to_string(),
            notes: opt_string(&form.notes),
            activation_date: when_to_date(&form.when),
            due_date: when_to_date(&form.deadline),
            tags: parse_tags(&form.tags),
            ..Default::default()
        };
        // Add context based on current selection.
        match &self.selection {
            Selection::List(l) => match l {
                SpecialList::Inbox => body.list = Some("inbox".into()),
                SpecialList::Today => body.activation_date = body.activation_date.or(Some("today".into())),
                SpecialList::Upcoming | SpecialList::Anytime => body.list = Some("anytime".into()),
                SpecialList::Someday => body.list = Some("someday".into()),
                SpecialList::Logbook | SpecialList::Trash => body.list = Some("inbox".into()),
            },
            Selection::Project(id) => {
                if let Some(p) = self.projects.iter().find(|p| &p.id == id) {
                    body.project = Some(p.title.clone());
                }
            }
            Selection::Area(id) => {
                if let Some(a) = self.areas.iter().find(|a| &a.id == id) {
                    body.area = Some(a.title.clone());
                }
            }
            Selection::Tag(_) => {} // tags already added
        }

        let client = self.client.clone();
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let r = client.create_task(&body).await.map(|_| "Task created".to_string());
            let _ = tx.send(Message::MutationDone(r));
        });
    }

    pub fn finish_edit_task(&mut self, form: &EditTaskForm) {
        let body = UpdateTask {
            title: Some(form.title.clone()),
            notes: Some(form.notes.clone()),
            activation_date: when_to_date(&form.when),
            due_date: when_to_date(&form.deadline),
            tags: Some(parse_tags(&form.tags).unwrap_or_default()),
            ..Default::default()
        };
        let client = self.client.clone();
        let tx = self.tx.clone();
        let id = form.task_id.clone();
        tokio::spawn(async move {
            let r = client.update_task(&id, &body).await.map(|_| "Saved".to_string());
            let _ = tx.send(Message::MutationDone(r));
        });
    }

    pub fn finish_new_project(&mut self, form: &NewProjectForm) {
        if form.title.trim().is_empty() {
            self.toast = Some(StatusToast {
                text: "Title is required".into(),
                kind: ToastKind::Error,
                at: Instant::now(),
            });
            return;
        }
        let body = CreateProject {
            title: form.title.trim().to_string(),
            notes: opt_string(&form.notes),
            area: opt_string(&form.area),
            ..Default::default()
        };
        let client = self.client.clone();
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let r = client.create_project(&body).await.map(|_| "Project created".to_string());
            let _ = tx.send(Message::MutationDone(r));
        });
    }

    pub fn finish_schedule(&mut self, form: &ScheduleForm) {
        let Some(choice) = form.choices.get(form.selected).copied() else {
            return;
        };
        let date: Option<String> = match choice {
            DateChoice::Today => Some("today".into()),
            DateChoice::Tomorrow => Some(tomorrow_string()),
            DateChoice::ThisWeekend => Some(next_saturday_string()),
            DateChoice::NextWeek => Some(next_monday_string()),
            DateChoice::Anytime => Some("anytime".into()),
            DateChoice::Someday => Some("someday".into()),
            DateChoice::Clear => Some(String::new()),
            DateChoice::Specific => Some(form.specific_buffer.clone()),
        };
        let date = date.unwrap_or_default();
        let id = form.task_id.clone();
        // 'anytime' and 'someday' are list moves, not date assignments
        let body = match date.as_str() {
            "today" | "tomorrow" => UpdateTask {
                activation_date: Some(date),
                ..Default::default()
            },
            "anytime" | "someday" | "inbox" => UpdateTask {
                list: Some(date),
                ..Default::default()
            },
            _ => UpdateTask {
                activation_date: Some(date),
                ..Default::default()
            },
        };
        let client = self.client.clone();
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let r = client.update_task(&id, &body).await.map(|_| "Rescheduled".to_string());
            let _ = tx.send(Message::MutationDone(r));
        });
    }

    pub fn finish_tags(&mut self, form: &TagsForm) {
        let tags: Vec<String> = form
            .selected_indices
            .iter()
            .filter_map(|i| form.available.get(*i).cloned())
            .collect();
        let body = UpdateTask {
            tags: Some(tags),
            ..Default::default()
        };
        let id = form.task_id.clone();
        let client = self.client.clone();
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let r = client.update_task(&id, &body).await.map(|_| "Tags updated".to_string());
            let _ = tx.send(Message::MutationDone(r));
        });
    }

    pub fn finish_move(&mut self, form: &MoveForm) {
        let Some(dest) = form.destinations.get(form.cursor) else {
            return;
        };
        let body = match dest {
            MoveDest::List(l) => UpdateTask {
                list: Some(l.api_value().into()),
                ..Default::default()
            },
            MoveDest::Project(_, title) => UpdateTask {
                project: Some(title.clone()),
                ..Default::default()
            },
            MoveDest::Area(_, title) => UpdateTask {
                area: Some(title.clone()),
                ..Default::default()
            },
            MoveDest::Detach => UpdateTask {
                project: Some(String::new()),
                area: Some(String::new()),
                ..Default::default()
            },
        };
        let id = form.task_id.clone();
        let client = self.client.clone();
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let r = client.update_task(&id, &body).await.map(|_| "Moved".to_string());
            let _ = tx.send(Message::MutationDone(r));
        });
    }

    pub fn finish_delete(&mut self, id: &str) {
        let client = self.client.clone();
        let tx = self.tx.clone();
        let id = id.to_string();
        tokio::spawn(async move {
            let r = client.delete_task(&id).await.map(|_| "Moved to Trash".to_string());
            let _ = tx.send(Message::MutationDone(r));
        });
    }

    pub fn finish_empty_trash(&mut self) {
        let client = self.client.clone();
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let r = client.empty_trash().await.map(|_| "Trash emptied".to_string());
            let _ = tx.send(Message::MutationDone(r));
        });
    }
}

fn opt_string(s: &str) -> Option<String> {
    let t = s.trim();
    if t.is_empty() {
        None
    } else {
        Some(t.to_string())
    }
}

fn parse_tags(s: &str) -> Option<Vec<String>> {
    let parts: Vec<String> = s
        .split(',')
        .map(|p| p.trim().trim_start_matches('#').to_string())
        .filter(|p| !p.is_empty())
        .collect();
    if parts.is_empty() {
        None
    } else {
        Some(parts)
    }
}

fn when_to_date(f: &DateField) -> Option<String> {
    if let Some(c) = f.choice {
        return match c {
            DateChoice::Today => Some("today".into()),
            DateChoice::Tomorrow => Some(tomorrow_string()),
            DateChoice::ThisWeekend => Some(next_saturday_string()),
            DateChoice::NextWeek => Some(next_monday_string()),
            DateChoice::Anytime | DateChoice::Someday => None,
            DateChoice::Clear => Some(String::new()),
            DateChoice::Specific => Some(f.specific.clone()),
        };
    }
    if !f.specific.trim().is_empty() {
        return Some(f.specific.trim().to_string());
    }
    None
}

// ---- date helpers: produce "April 14, 2026" strings AppleScript accepts ----

fn days_from_civil(year: i64, month: u32, day: u32) -> i64 {
    let y = if month <= 2 { year - 1 } else { year };
    let era = y.div_euclid(400);
    let yoe = (y - era * 400) as u64;
    let doy = ((153 * (if month > 2 { month - 3 } else { month + 9 } as u32) as i64 + 2) / 5
        + day as i64
        - 1) as u64;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe as i64 - 719468
}

fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 }.div_euclid(146097);
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

fn now_civil() -> (i64, u32, u32) {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    // Use local-ish offset by reading TZ-adjusted wall clock seconds. We don't have chrono;
    // approximate by using the system's epoch directly — for date-only purposes, UTC is fine
    // within a few hours. AppleScript will interpret day boundaries against the user's TZ.
    civil_from_days(secs / 86400)
}

fn day_of_week(z: i64) -> u32 {
    ((z + 4).rem_euclid(7)) as u32 // 0 = Sunday
}

fn month_name(m: u32) -> &'static str {
    [
        "January", "February", "March", "April", "May", "June", "July", "August", "September",
        "October", "November", "December",
    ][(m - 1) as usize]
}

fn format_civil(y: i64, m: u32, d: u32) -> String {
    format!("{} {}, {}", month_name(m), d, y)
}

fn tomorrow_string() -> String {
    let (y, m, d) = now_civil();
    let z = days_from_civil(y, m, d) + 1;
    let (y, m, d) = civil_from_days(z);
    format_civil(y, m, d)
}

fn next_saturday_string() -> String {
    let (y, m, d) = now_civil();
    let z = days_from_civil(y, m, d);
    let dow = day_of_week(z); // 0=Sun, 6=Sat
    let delta = ((6 + 7 - dow as i64) % 7).max(1);
    let z2 = z + delta;
    let (y, m, d) = civil_from_days(z2);
    format_civil(y, m, d)
}

fn next_monday_string() -> String {
    let (y, m, d) = now_civil();
    let z = days_from_civil(y, m, d);
    let dow = day_of_week(z);
    let delta = ((1 + 7 - dow as i64) % 7).max(1);
    let z2 = z + delta;
    let (y, m, d) = civil_from_days(z2);
    format_civil(y, m, d)
}

// ---- the main run loop ----

pub async fn run<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    mut rx: mpsc::UnboundedReceiver<Message>,
) -> Result<()> {
    let mut events = EventStream::new();
    app.schedule_initial_loads();

    loop {
        terminal.draw(|f| ui::render(f, &app))?;
        if app.should_quit {
            break;
        }

        tokio::select! {
            maybe_ev = events.next() => {
                if let Some(Ok(Event::Key(key))) = maybe_ev {
                    if key.kind != KeyEventKind::Press {
                        continue;
                    }
                    handle_key(&mut app, key);
                }
            }
            msg = rx.recv() => {
                if let Some(m) = msg {
                    handle_message(&mut app, m);
                }
            }
            _ = tokio::time::sleep(Duration::from_millis(500)) => {
                // Tick: expire toasts.
                if let Some(t) = &app.toast {
                    if t.at.elapsed() > Duration::from_secs(4) {
                        app.toast = None;
                    }
                }
            }
        }
    }
    Ok(())
}

fn handle_key(app: &mut App, key: crossterm::event::KeyEvent) {
    // Esc closes any modal in any state.
    if key.code == KeyCode::Esc {
        app.modal = None;
        return;
    }

    // Ctrl-C always quits.
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        app.should_quit = true;
        return;
    }

    if app.modal.is_some() {
        handle_modal_key(app, key);
        return;
    }

    if let Some(action) = browse_action(&key) {
        app.handle_browse_key(action);
    }
}

fn handle_modal_key(app: &mut App, key: crossterm::event::KeyEvent) {
    let modal = app.modal.take();
    let Some(modal) = modal else {
        return;
    };
    match modal {
        Modal::Help => {
            app.modal = None; // any key dismisses
        }
        Modal::Search(mut buf) => match key.code {
            KeyCode::Enter => {
                app.filter = if buf.is_empty() { None } else { Some(buf) };
                app.task_cursor = 0;
                app.modal = None;
            }
            KeyCode::Backspace => {
                buf.pop();
                app.modal = Some(Modal::Search(buf));
            }
            KeyCode::Char(c) => {
                buf.push(c);
                app.modal = Some(Modal::Search(buf));
            }
            _ => app.modal = Some(Modal::Search(buf)),
        },
        Modal::NewTask(mut form) => {
            handle_new_task_key(&mut form, key);
            if key.code == KeyCode::Char('s') && key.modifiers.contains(KeyModifiers::CONTROL) {
                app.finish_new_task(&form);
            } else {
                app.modal = Some(Modal::NewTask(form));
            }
        }
        Modal::EditTask(mut form) => {
            handle_edit_task_key(&mut form, key);
            if key.code == KeyCode::Char('s') && key.modifiers.contains(KeyModifiers::CONTROL) {
                app.finish_edit_task(&form);
            } else {
                app.modal = Some(Modal::EditTask(form));
            }
        }
        Modal::NewProject(mut form) => {
            handle_new_project_key(&mut form, key);
            if key.code == KeyCode::Char('s') && key.modifiers.contains(KeyModifiers::CONTROL) {
                app.finish_new_project(&form);
            } else {
                app.modal = Some(Modal::NewProject(form));
            }
        }
        Modal::Schedule(mut form) => {
            let done = handle_schedule_key(&mut form, key);
            if done {
                app.finish_schedule(&form);
            } else {
                app.modal = Some(Modal::Schedule(form));
            }
        }
        Modal::Tags(mut form) => {
            let done = handle_tags_key(&mut form, key);
            if done {
                app.finish_tags(&form);
            } else {
                app.modal = Some(Modal::Tags(form));
            }
        }
        Modal::MovePicker(mut form) => {
            let done = handle_move_key(&mut form, key);
            if done {
                app.finish_move(&form);
            } else {
                app.modal = Some(Modal::MovePicker(form));
            }
        }
        Modal::ConfirmEmptyTrash => match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                app.finish_empty_trash();
            }
            _ => {}
        },
        Modal::ConfirmDelete(id, title) => match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                app.finish_delete(&id);
            }
            _ => {
                app.modal = Some(Modal::ConfirmDelete(id, title));
            }
        },
    }
}

fn handle_new_task_key(form: &mut NewTaskForm, key: crossterm::event::KeyEvent) {
    if key.code == KeyCode::Tab {
        let pos = NewTaskField::ORDER.iter().position(|f| *f == form.focused).unwrap_or(0);
        form.focused = NewTaskField::ORDER[(pos + 1) % NewTaskField::ORDER.len()];
        return;
    }
    if key.code == KeyCode::BackTab {
        let pos = NewTaskField::ORDER.iter().position(|f| *f == form.focused).unwrap_or(0);
        let len = NewTaskField::ORDER.len();
        form.focused = NewTaskField::ORDER[(pos + len - 1) % len];
        return;
    }
    let target = match form.focused {
        NewTaskField::Title => &mut form.title,
        NewTaskField::Notes => &mut form.notes,
        NewTaskField::When => &mut form.when.specific,
        NewTaskField::Deadline => &mut form.deadline.specific,
        NewTaskField::Tags => &mut form.tags,
    };
    edit_string(target, key);
}

fn handle_edit_task_key(form: &mut EditTaskForm, key: crossterm::event::KeyEvent) {
    if key.code == KeyCode::Tab {
        let pos = NewTaskField::ORDER.iter().position(|f| *f == form.focused).unwrap_or(0);
        form.focused = NewTaskField::ORDER[(pos + 1) % NewTaskField::ORDER.len()];
        return;
    }
    if key.code == KeyCode::BackTab {
        let pos = NewTaskField::ORDER.iter().position(|f| *f == form.focused).unwrap_or(0);
        let len = NewTaskField::ORDER.len();
        form.focused = NewTaskField::ORDER[(pos + len - 1) % len];
        return;
    }
    let target = match form.focused {
        NewTaskField::Title => &mut form.title,
        NewTaskField::Notes => &mut form.notes,
        NewTaskField::When => &mut form.when.specific,
        NewTaskField::Deadline => &mut form.deadline.specific,
        NewTaskField::Tags => &mut form.tags,
    };
    edit_string(target, key);
}

fn handle_new_project_key(form: &mut NewProjectForm, key: crossterm::event::KeyEvent) {
    if key.code == KeyCode::Tab {
        let pos = NewProjectField::ORDER.iter().position(|f| *f == form.focused).unwrap_or(0);
        form.focused = NewProjectField::ORDER[(pos + 1) % NewProjectField::ORDER.len()];
        return;
    }
    if key.code == KeyCode::BackTab {
        let pos = NewProjectField::ORDER.iter().position(|f| *f == form.focused).unwrap_or(0);
        let len = NewProjectField::ORDER.len();
        form.focused = NewProjectField::ORDER[(pos + len - 1) % len];
        return;
    }
    let target = match form.focused {
        NewProjectField::Title => &mut form.title,
        NewProjectField::Notes => &mut form.notes,
        NewProjectField::Area => &mut form.area,
    };
    edit_string(target, key);
}

fn handle_schedule_key(form: &mut ScheduleForm, key: crossterm::event::KeyEvent) -> bool {
    if form.editing_specific {
        match key.code {
            KeyCode::Enter => {
                form.editing_specific = false;
                return true;
            }
            KeyCode::Esc => {
                form.editing_specific = false;
                return false;
            }
            _ => edit_string(&mut form.specific_buffer, key),
        }
        return false;
    }
    match key.code {
        KeyCode::Down | KeyCode::Char('j') => {
            if !form.choices.is_empty() {
                form.selected = (form.selected + 1) % form.choices.len();
            }
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if !form.choices.is_empty() {
                form.selected = (form.selected + form.choices.len() - 1) % form.choices.len();
            }
        }
        KeyCode::Enter => {
            if form.choices.get(form.selected) == Some(&DateChoice::Specific) {
                form.editing_specific = true;
                return false;
            }
            return true;
        }
        _ => {}
    }
    false
}

fn handle_tags_key(form: &mut TagsForm, key: crossterm::event::KeyEvent) -> bool {
    match key.code {
        KeyCode::Down | KeyCode::Char('j') => {
            if !form.available.is_empty() {
                form.cursor = (form.cursor + 1) % form.available.len();
            }
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if !form.available.is_empty() {
                form.cursor = (form.cursor + form.available.len() - 1) % form.available.len();
            }
        }
        KeyCode::Char(' ') => {
            if form.available.is_empty() {
                return false;
            }
            if let Some(pos) = form.selected_indices.iter().position(|i| *i == form.cursor) {
                form.selected_indices.remove(pos);
            } else {
                form.selected_indices.push(form.cursor);
            }
        }
        KeyCode::Enter => return true,
        _ => {}
    }
    false
}

fn handle_move_key(form: &mut MoveForm, key: crossterm::event::KeyEvent) -> bool {
    match key.code {
        KeyCode::Down | KeyCode::Char('j') => {
            if !form.destinations.is_empty() {
                form.cursor = (form.cursor + 1) % form.destinations.len();
            }
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if !form.destinations.is_empty() {
                form.cursor = (form.cursor + form.destinations.len() - 1) % form.destinations.len();
            }
        }
        KeyCode::Enter => return true,
        _ => {}
    }
    false
}

fn edit_string(buf: &mut String, key: crossterm::event::KeyEvent) {
    match key.code {
        KeyCode::Char(c) => buf.push(c),
        KeyCode::Backspace => {
            buf.pop();
        }
        _ => {}
    }
}

fn handle_message(app: &mut App, msg: Message) {
    match msg {
        Message::Tick => {}
        Message::TasksLoaded { for_selection, tasks } => {
            if for_selection != app.selection {
                return; // user moved on
            }
            app.tasks_loading = false;
            match tasks {
                Ok(t) => {
                    app.tasks = t;
                    app.task_cursor = app.task_cursor.min(app.tasks.len().saturating_sub(1));
                    if app.tasks.is_empty() {
                        app.task_cursor = 0;
                    }
                }
                Err(e) => {
                    app.toast = Some(StatusToast {
                        text: format!("Load failed: {e}"),
                        kind: ToastKind::Error,
                        at: Instant::now(),
                    });
                }
            }
            app.last_refresh = Instant::now();
        }
        Message::SidebarLoaded { areas, projects, tags } => {
            app.areas = areas.unwrap_or_default();
            app.projects = projects.unwrap_or_default();
            app.tags = tags.unwrap_or_default();
            app.sidebar_nodes = App::build_sidebar(&app.areas, &app.projects, &app.tags);
        }
        Message::Toast(t) => app.toast = Some(t),
        Message::MutationDone(result) => {
            match result {
                Ok(text) => {
                    app.toast = Some(StatusToast {
                        text,
                        kind: ToastKind::Info,
                        at: Instant::now(),
                    });
                    app.modal = None;
                    app.spawn_task_load(app.selection.clone());
                }
                Err(e) => {
                    app.toast = Some(StatusToast {
                        text: format!("✗ {e}"),
                        kind: ToastKind::Error,
                        at: Instant::now(),
                    });
                    app.spawn_task_load(app.selection.clone());
                }
            }
        }
    }
}

