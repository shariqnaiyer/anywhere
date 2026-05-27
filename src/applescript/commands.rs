use crate::applescript::run_applescript;
use crate::models::{
    AppInfo, Area, Contact, CreateArea, CreateContact, CreateProject, CreateTag, CreateTask,
    ListInfo, ParseInput, Project, QuickEntry, Tag, Task, UpdateArea, UpdateProject, UpdateTag,
    UpdateTask, UpdateWindow, WindowInfo,
};

fn things_auth_token() -> String {
    std::env::var("THINGS_AUTH_TOKEN").unwrap_or_default()
}

/// Escape a string for safe embedding in AppleScript double-quoted strings.
fn esc(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Parse `missing value` returns from AppleScript into Option<String>.
fn parse_optional(s: &str) -> Option<String> {
    let s = s.trim();
    if s.is_empty() || s == "missing value" {
        None
    } else {
        Some(s.to_string())
    }
}

fn list_name(list_filter: Option<&str>) -> String {
    match list_filter {
        None | Some("inbox") | Some("") => "Inbox".to_string(),
        Some("today") => "Today".to_string(),
        Some("upcoming") => "Upcoming".to_string(),
        Some("anytime") => "Anytime".to_string(),
        Some("someday") => "Someday".to_string(),
        Some("logbook") => "Logbook".to_string(),
        Some("trash") => "Trash".to_string(),
        Some(other) => other.to_string(),
    }
}

// ---------------------------------------------------------------------------
// Tasks
// ---------------------------------------------------------------------------

/// AppleScript fragment that serialises the current `t` (a to do reference) to
/// a 15-field record. Returned string assigns to `taskOutput`.
const TASK_RECORD_FRAGMENT: &str = r#"
        set tid to id of t
        set ttitle to name of t
        set tnotes to notes of t
        if tnotes is missing value then set tnotes to ""
        set tdue to due date of t
        if tdue is missing value then
            set tdue to ""
        else
            set tdue to (tdue as string)
        end if
        set tactivation to ""
        try
            set ad to activation date of t
            if ad is not missing value then set tactivation to (ad as string)
        end try
        set tproject to ""
        if project of t is not missing value then set tproject to name of project of t
        set tarea to ""
        if area of t is not missing value then set tarea to name of area of t
        set tcontact to ""
        if contact of t is not missing value then set tcontact to name of contact of t
        set ttags to ""
        set tagList to tags of t
        repeat with tg in tagList
            if ttags is "" then
                set ttags to name of tg
            else
                set ttags to ttags & "," & name of tg
            end if
        end repeat
        set tcompleted to (status of t is completed)
        set tcanceled to (status of t is canceled)
        set tcreation to creation date of t
        if tcreation is missing value then
            set tcreation to ""
        else
            set tcreation to (tcreation as string)
        end if
        set tmodification to ""
        try
            set md to modification date of t
            if md is not missing value then set tmodification to (md as string)
        end try
        set tcompletion to completion date of t
        if tcompletion is missing value then
            set tcompletion to ""
        else
            set tcompletion to (tcompletion as string)
        end if
        set tcancellation to ""
        try
            set cd to cancellation date of t
            if cd is not missing value then set tcancellation to (cd as string)
        end try
        set taskOutput to tid & "␞" & ttitle & "␞" & tnotes & "␞" & tdue & "␞" & tactivation & "␞" & tproject & "␞" & tarea & "␞" & tcontact & "␞" & ttags & "␞" & tcompleted & "␞" & tcanceled & "␞" & tcreation & "␞" & tmodification & "␞" & tcompletion & "␞" & tcancellation"#;

pub fn get_tasks(
    list_filter: Option<&str>,
    limit: Option<usize>,
    offset: Option<usize>,
) -> Result<Vec<Task>, String> {
    let list_spec = format!("list \"{}\"", esc(&list_name(list_filter)));

    let offset_val = offset.unwrap_or(0);
    let limit_val = limit.unwrap_or(50);
    let start = offset_val + 1;
    let end = offset_val + limit_val;

    let script = format!(
        r#"tell application "Things3"
    set output to ""
    set theTasks to to dos of {list_spec}
    set taskCount to count of theTasks
    set startIdx to {start}
    set endIdx to {end}
    if endIdx > taskCount then set endIdx to taskCount
    if startIdx > taskCount then return ""
    repeat with i from startIdx to endIdx
        set t to item i of theTasks
{record}
        set output to output & taskOutput & "␟"
    end repeat
    return output
end tell"#,
        record = TASK_RECORD_FRAGMENT,
    );

    let raw = run_applescript(&script)?;
    raw.split('␟')
        .filter(|l| !l.trim().is_empty())
        .map(|l| parse_task_line(l.trim()))
        .collect()
}

pub fn get_task_by_id(task_id: &str) -> Result<Task, String> {
    let script = format!(
        r#"tell application "Things3"
    set t to to do id "{id}"
{record}
    return taskOutput
end tell"#,
        id = esc(task_id),
        record = TASK_RECORD_FRAGMENT,
    );

    let raw = run_applescript(&script)?;
    parse_task_line(raw.trim())
}

fn parse_task_line(line: &str) -> Result<Task, String> {
    let parts: Vec<&str> = line.splitn(15, '␞').collect();
    if parts.len() < 15 {
        return Err(format!("Unexpected task format: {}", line));
    }

    let tags: Vec<String> = if parts[8].is_empty() {
        vec![]
    } else {
        parts[8].split(',').map(|s| s.trim().to_string()).collect()
    };

    Ok(Task {
        id: parts[0].to_string(),
        title: parts[1].to_string(),
        notes: parse_optional(parts[2]),
        due_date: parse_optional(parts[3]),
        activation_date: parse_optional(parts[4]),
        list: None,
        project: parse_optional(parts[5]),
        area: parse_optional(parts[6]),
        contact: parse_optional(parts[7]),
        tags,
        checklist_items: vec![],
        completed: parts[9].trim() == "true",
        canceled: parts[10].trim() == "true",
        creation_date: parse_optional(parts[11]),
        modification_date: parse_optional(parts[12]),
        completion_date: parse_optional(parts[13]),
        cancellation_date: parse_optional(parts[14]),
    })
}

pub fn create_task(payload: &CreateTask) -> Result<Task, String> {
    let title = esc(&payload.title);
    let notes = payload.notes.as_deref().map(esc).unwrap_or_default();
    let due_date = payload.due_date.as_deref().unwrap_or("").to_string();

    let mut props = format!("name:\"{title}\"");
    if !notes.is_empty() {
        props.push_str(&format!(", notes:\"{notes}\""));
    }
    if !due_date.is_empty() {
        props.push_str(&format!(", due date:(date \"{due_date}\")"));
    }
    if let Some(tags) = &payload.tags {
        if !tags.is_empty() {
            let joined = tags.iter().map(|t| esc(t)).collect::<Vec<_>>().join(", ");
            props.push_str(&format!(", tag names:\"{joined}\""));
        }
    }

    let mut lines = vec![
        "tell application \"Things3\"".to_string(),
        format!("    set newTask to make new to do with properties {{{props}}}"),
    ];

    // NB: `move newTask to project "X"` reliably fails with Things 3 error 301
    // ("Cannot move to-do"). `set project of newTask to project "X"` is the
    // documented assignment form that actually works. Same story for area.
    if let Some(project) = &payload.project {
        lines.push(format!(
            "    set project of newTask to project \"{}\"",
            esc(project)
        ));
    } else if let Some(area) = &payload.area {
        lines.push(format!(
            "    set area of newTask to area \"{}\"",
            esc(area)
        ));
    }

    if let Some(activation) = &payload.activation_date {
        if !activation.is_empty() {
            lines.push(format!(
                "    schedule newTask for (date \"{}\")",
                esc(activation)
            ));
        }
    } else if let Some(list) = &payload.list {
        let l = list_name(Some(list));
        lines.push(format!("    move newTask to list \"{}\"", esc(&l)));
    }

    if let Some(contact) = &payload.contact {
        if !contact.is_empty() {
            lines.push(format!(
                "    set contact of newTask to contact \"{}\"",
                esc(contact)
            ));
        }
    }

    lines.push("    return id of newTask".to_string());
    lines.push("end tell".to_string());
    let script = lines.join("\n");

    let id = run_applescript(&script)?.trim().to_string();

    // Checklist items are the one piece AppleScript can't manipulate; use URL scheme.
    if let Some(items) = &payload.checklist_items {
        if !items.is_empty() {
            let token = things_auth_token();
            let auth = if token.is_empty() {
                String::new()
            } else {
                format!("&auth-token={token}")
            };
            let checklist_json: Vec<String> = items
                .iter()
                .map(|item| {
                    format!(
                        "{{\"type\":\"checklist-item\",\"attributes\":{{\"title\":\"{}\"}}}}",
                        esc(item)
                    )
                })
                .collect();
            let json_str = format!("[{}]", checklist_json.join(","));
            let encoded = urlencoding::encode(&json_str);
            let url = format!("things:///update?id={id}&checklist-items={encoded}{auth}");
            let open_script = format!(
                "do shell script \"open -g '{}'\"",
                url.replace('\'', "'\\''")
            );
            let _ = run_applescript(&open_script);
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
    }

    get_task_by_id(&id)
}

pub fn update_task(task_id: &str, payload: &UpdateTask) -> Result<Task, String> {
    let mut updates: Vec<String> = vec![];

    if let Some(title) = &payload.title {
        updates.push(format!("set name of t to \"{}\"", esc(title)));
    }
    if let Some(notes) = &payload.notes {
        updates.push(format!("set notes of t to \"{}\"", esc(notes)));
    }
    if let Some(due_date) = &payload.due_date {
        if due_date.is_empty() {
            updates.push("set due date of t to missing value".to_string());
        } else {
            updates.push(format!(
                "set due date of t to (date \"{}\")",
                esc(due_date)
            ));
        }
    }
    if let Some(tags) = &payload.tags {
        let tag_list = tags
            .iter()
            .map(|t| format!("\"{}\"", esc(t)))
            .collect::<Vec<_>>()
            .join(", ");
        updates.push(format!("set tag names of t to {{{tag_list}}}"));
    }
    if let Some(project) = &payload.project {
        if project.is_empty() {
            updates.push("set project of t to missing value".to_string());
        } else {
            // `move t to project "X"` fails with Things 3 error 301; use assignment instead.
            updates.push(format!("set project of t to project \"{}\"", esc(project)));
        }
    }
    if let Some(area) = &payload.area {
        if area.is_empty() {
            updates.push("set area of t to missing value".to_string());
        } else {
            updates.push(format!("set area of t to area \"{}\"", esc(area)));
        }
    }
    if let Some(contact) = &payload.contact {
        if contact.is_empty() {
            updates.push("set contact of t to missing value".to_string());
        } else {
            updates.push(format!(
                "set contact of t to contact \"{}\"",
                esc(contact)
            ));
        }
    }
    if let Some(list) = &payload.list {
        let l = list_name(Some(list));
        updates.push(format!("move t to list \"{}\"", esc(&l)));
    }
    if let Some(activation) = &payload.activation_date {
        if activation.is_empty() {
            // AppleScript cannot directly clear activation date; move back to Anytime.
            updates.push("move t to list \"Anytime\"".to_string());
        } else {
            updates.push(format!(
                "schedule t for (date \"{}\")",
                esc(activation)
            ));
        }
    }
    if let Some(completed) = payload.completed {
        if completed {
            updates.push("set status of t to completed".to_string());
        } else {
            updates.push("set status of t to open".to_string());
        }
    }
    if let Some(canceled) = payload.canceled {
        if canceled {
            updates.push("set status of t to canceled".to_string());
        } else {
            updates.push("set status of t to open".to_string());
        }
    }

    if updates.is_empty() {
        return get_task_by_id(task_id);
    }

    let body = updates.join("\n    ");
    let script = format!(
        r#"tell application "Things3"
    set t to to do id "{id}"
    {body}
end tell"#,
        id = esc(task_id),
    );

    run_applescript(&script)?;
    get_task_by_id(task_id)
}

pub fn complete_task(task_id: &str) -> Result<Task, String> {
    let script = format!(
        r#"tell application "Things3"
    set t to to do id "{id}"
    set status of t to completed
end tell"#,
        id = esc(task_id)
    );
    run_applescript(&script)?;
    get_task_by_id(task_id)
}

pub fn cancel_task(task_id: &str) -> Result<Task, String> {
    let script = format!(
        r#"tell application "Things3"
    set t to to do id "{id}"
    set status of t to canceled
end tell"#,
        id = esc(task_id)
    );
    run_applescript(&script)?;
    get_task_by_id(task_id)
}

pub fn delete_task(task_id: &str) -> Result<(), String> {
    let script = format!(
        r#"tell application "Things3"
    delete (to do id "{id}")
end tell"#,
        id = esc(task_id)
    );
    run_applescript(&script)?;
    Ok(())
}

pub fn show_task(task_id: &str) -> Result<(), String> {
    let script = format!(
        r#"tell application "Things3"
    show to do id "{id}"
end tell"#,
        id = esc(task_id)
    );
    run_applescript(&script)?;
    Ok(())
}

pub fn edit_task(task_id: &str) -> Result<(), String> {
    let script = format!(
        r#"tell application "Things3"
    edit to do id "{id}"
end tell"#,
        id = esc(task_id)
    );
    run_applescript(&script)?;
    Ok(())
}

pub fn edit_project(project_id: &str) -> Result<(), String> {
    let script = format!(
        r#"tell application "Things3"
    edit (project id "{id}")
end tell"#,
        id = esc(project_id)
    );
    run_applescript(&script)?;
    Ok(())
}

pub fn get_selected_tasks() -> Result<Vec<Task>, String> {
    let script = format!(
        r#"tell application "Things3"
    set output to ""
    set theTasks to selected to dos
    repeat with t in theTasks
{record}
        set output to output & taskOutput & "␟"
    end repeat
    return output
end tell"#,
        record = TASK_RECORD_FRAGMENT,
    );

    let raw = run_applescript(&script)?;
    raw.split('␟')
        .filter(|l| !l.trim().is_empty())
        .map(|l| parse_task_line(l.trim()))
        .collect()
}

// ---------------------------------------------------------------------------
// Trash / system
// ---------------------------------------------------------------------------

pub fn empty_trash() -> Result<(), String> {
    let script = r#"tell application "Things3"
    empty trash
end tell"#;
    run_applescript(script)?;
    Ok(())
}

pub fn log_completed_now() -> Result<(), String> {
    let script = r#"tell application "Things3"
    log completed now
end tell"#;
    run_applescript(script)?;
    Ok(())
}

pub fn show_quick_entry_panel(payload: &QuickEntry) -> Result<(), String> {
    let mut props: Vec<String> = Vec::new();
    if let Some(title) = &payload.title {
        props.push(format!("name:\"{}\"", esc(title)));
    }
    if let Some(notes) = &payload.notes {
        props.push(format!("notes:\"{}\"", esc(notes)));
    }
    if let Some(due) = &payload.due_date {
        if !due.is_empty() {
            props.push(format!("due date:(date \"{}\")", esc(due)));
        }
    }
    if let Some(tags) = &payload.tags {
        if !tags.is_empty() {
            let joined = tags.iter().map(|t| esc(t)).collect::<Vec<_>>().join(", ");
            props.push(format!("tag names:\"{}\"", joined));
        }
    }

    let autofill = payload.autofill.unwrap_or(false);
    let mut parts: Vec<String> = Vec::new();
    if autofill {
        parts.push("with autofill true".to_string());
    }
    if !props.is_empty() {
        parts.push(format!("with properties {{{}}}", props.join(", ")));
    }
    let tail = if parts.is_empty() {
        String::new()
    } else {
        format!(" {}", parts.join(" "))
    };

    let script = format!(
        r#"tell application "Things3"
    show quick entry panel{tail}
end tell"#
    );
    run_applescript(&script)?;
    Ok(())
}

pub fn parse_quicksilver(payload: &ParseInput) -> Result<Task, String> {
    let script = format!(
        r#"tell application "Things3"
    set newTask to parse quicksilver input "{text}"
    return id of newTask
end tell"#,
        text = esc(&payload.text),
    );
    let id = run_applescript(&script)?.trim().to_string();
    get_task_by_id(&id)
}

// ---------------------------------------------------------------------------
// Lists (the seven special lists)
// ---------------------------------------------------------------------------

pub fn get_lists() -> Result<Vec<ListInfo>, String> {
    let script = r#"tell application "Things3"
    set output to ""
    set theLists to lists
    repeat with l in theLists
        set lid to id of l
        set lname to name of l
        set output to output & lid & "␞" & lname & "␟"
    end repeat
    return output
end tell"#;
    let raw = run_applescript(script)?;
    raw.split('␟')
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            let parts: Vec<&str> = l.trim().splitn(2, '␞').collect();
            if parts.len() < 2 {
                Err(format!("Unexpected list format: {}", l))
            } else {
                Ok(ListInfo {
                    id: parts[0].to_string(),
                    name: parts[1].to_string(),
                })
            }
        })
        .collect()
}

pub fn show_list(name: &str) -> Result<(), String> {
    let script = format!(
        r#"tell application "Things3"
    show list "{name}"
end tell"#,
        name = esc(name)
    );
    run_applescript(&script)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Projects
// ---------------------------------------------------------------------------

const PROJECT_RECORD_FRAGMENT: &str = r#"
        set pid to id of p
        set ptitle to name of p
        set pnotes to notes of p
        if pnotes is missing value then set pnotes to ""
        set pdue to due date of p
        if pdue is missing value then
            set pdue to ""
        else
            set pdue to (pdue as string)
        end if
        set pactivation to ""
        try
            set ad to activation date of p
            if ad is not missing value then set pactivation to (ad as string)
        end try
        set parea to ""
        if area of p is not missing value then set parea to name of area of p
        set ptags to ""
        set tagList to tags of p
        repeat with tg in tagList
            if ptags is "" then
                set ptags to name of tg
            else
                set ptags to ptags & "," & name of tg
            end if
        end repeat
        set pcompleted to (status of p is completed)
        set pcanceled to (status of p is canceled)
        set pcreation to creation date of p
        if pcreation is missing value then
            set pcreation to ""
        else
            set pcreation to (pcreation as string)
        end if
        set pmodification to ""
        try
            set md to modification date of p
            if md is not missing value then set pmodification to (md as string)
        end try
        set pcompletion to completion date of p
        if pcompletion is missing value then
            set pcompletion to ""
        else
            set pcompletion to (pcompletion as string)
        end if
        set pcancellation to ""
        try
            set cd to cancellation date of p
            if cd is not missing value then set pcancellation to (cd as string)
        end try
        set projOutput to pid & "␞" & ptitle & "␞" & pnotes & "␞" & pdue & "␞" & pactivation & "␞" & parea & "␞" & ptags & "␞" & pcompleted & "␞" & pcanceled & "␞" & pcreation & "␞" & pmodification & "␞" & pcompletion & "␞" & pcancellation"#;

pub fn get_projects() -> Result<Vec<Project>, String> {
    let script = format!(
        r#"tell application "Things3"
    set output to ""
    set theProjects to projects
    repeat with p in theProjects
{record}
        set output to output & projOutput & "␟"
    end repeat
    return output
end tell"#,
        record = PROJECT_RECORD_FRAGMENT,
    );

    let raw = run_applescript(&script)?;
    raw.split('␟')
        .filter(|l| !l.trim().is_empty())
        .map(|l| parse_project_line(l.trim()))
        .collect()
}

pub fn get_project_by_id(project_id: &str) -> Result<Project, String> {
    let script = format!(
        r#"tell application "Things3"
    set p to project id "{id}"
{record}
    return projOutput
end tell"#,
        id = esc(project_id),
        record = PROJECT_RECORD_FRAGMENT,
    );
    let raw = run_applescript(&script)?;
    parse_project_line(raw.trim())
}

fn parse_project_line(line: &str) -> Result<Project, String> {
    let parts: Vec<&str> = line.splitn(13, '␞').collect();
    if parts.len() < 13 {
        return Err(format!("Unexpected project format: {}", line));
    }

    let tags: Vec<String> = if parts[6].is_empty() {
        vec![]
    } else {
        parts[6].split(',').map(|s| s.trim().to_string()).collect()
    };

    Ok(Project {
        id: parts[0].to_string(),
        title: parts[1].to_string(),
        notes: parse_optional(parts[2]),
        due_date: parse_optional(parts[3]),
        activation_date: parse_optional(parts[4]),
        area: parse_optional(parts[5]),
        tags,
        completed: parts[7].trim() == "true",
        canceled: parts[8].trim() == "true",
        creation_date: parse_optional(parts[9]),
        modification_date: parse_optional(parts[10]),
        completion_date: parse_optional(parts[11]),
        cancellation_date: parse_optional(parts[12]),
    })
}

pub fn create_project(payload: &CreateProject) -> Result<Project, String> {
    let title = esc(&payload.title);
    let notes = payload.notes.as_deref().map(esc).unwrap_or_default();
    let due_date = payload.due_date.as_deref().unwrap_or("").to_string();

    let mut props = format!("name:\"{title}\"");
    if !notes.is_empty() {
        props.push_str(&format!(", notes:\"{notes}\""));
    }
    if !due_date.is_empty() {
        props.push_str(&format!(", due date:(date \"{due_date}\")"));
    }
    if let Some(tags) = &payload.tags {
        if !tags.is_empty() {
            let joined = tags.iter().map(|t| esc(t)).collect::<Vec<_>>().join(", ");
            props.push_str(&format!(", tag names:\"{joined}\""));
        }
    }

    let mut lines = vec![
        "tell application \"Things3\"".to_string(),
        format!("    set newProj to make new project with properties {{{props}}}"),
    ];
    if let Some(area) = &payload.area {
        if !area.is_empty() {
            lines.push(format!("    set area of newProj to area \"{}\"", esc(area)));
        }
    }
    if let Some(activation) = &payload.activation_date {
        if !activation.is_empty() {
            lines.push(format!(
                "    schedule newProj for (date \"{}\")",
                esc(activation)
            ));
        }
    }
    lines.push("    return id of newProj".to_string());
    lines.push("end tell".to_string());
    let script = lines.join("\n");

    let id = run_applescript(&script)?.trim().to_string();
    get_project_by_id(&id)
}

pub fn update_project(project_id: &str, payload: &UpdateProject) -> Result<Project, String> {
    let mut updates: Vec<String> = vec![];

    if let Some(title) = &payload.title {
        updates.push(format!("set name of p to \"{}\"", esc(title)));
    }
    if let Some(notes) = &payload.notes {
        updates.push(format!("set notes of p to \"{}\"", esc(notes)));
    }
    if let Some(due_date) = &payload.due_date {
        if due_date.is_empty() {
            updates.push("set due date of p to missing value".to_string());
        } else {
            updates.push(format!(
                "set due date of p to (date \"{}\")",
                esc(due_date)
            ));
        }
    }
    if let Some(tags) = &payload.tags {
        let tag_list = tags
            .iter()
            .map(|t| format!("\"{}\"", esc(t)))
            .collect::<Vec<_>>()
            .join(", ");
        updates.push(format!("set tag names of p to {{{tag_list}}}"));
    }
    if let Some(area) = &payload.area {
        if area.is_empty() {
            updates.push("set area of p to missing value".to_string());
        } else {
            updates.push(format!("set area of p to area \"{}\"", esc(area)));
        }
    }
    if let Some(activation) = &payload.activation_date {
        if !activation.is_empty() {
            updates.push(format!(
                "schedule p for (date \"{}\")",
                esc(activation)
            ));
        }
    }
    if let Some(completed) = payload.completed {
        if completed {
            updates.push("set status of p to completed".to_string());
        } else {
            updates.push("set status of p to open".to_string());
        }
    }
    if let Some(canceled) = payload.canceled {
        if canceled {
            updates.push("set status of p to canceled".to_string());
        } else {
            updates.push("set status of p to open".to_string());
        }
    }

    if updates.is_empty() {
        return get_project_by_id(project_id);
    }

    let body = updates.join("\n    ");
    let script = format!(
        r#"tell application "Things3"
    set p to project id "{id}"
    {body}
end tell"#,
        id = esc(project_id),
    );
    run_applescript(&script)?;
    get_project_by_id(project_id)
}

pub fn delete_project(project_id: &str) -> Result<(), String> {
    let script = format!(
        r#"tell application "Things3"
    delete (project id "{id}")
end tell"#,
        id = esc(project_id)
    );
    run_applescript(&script)?;
    Ok(())
}

pub fn show_project(project_id: &str) -> Result<(), String> {
    let script = format!(
        r#"tell application "Things3"
    show project id "{id}"
end tell"#,
        id = esc(project_id)
    );
    run_applescript(&script)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Tags
// ---------------------------------------------------------------------------

pub fn get_tags() -> Result<Vec<Tag>, String> {
    let script = r#"tell application "Things3"
    set output to ""
    set theTags to tags
    repeat with t in theTags
        set tid to id of t
        set tname to name of t
        set tshort to ""
        try
            set ks to keyboard shortcut of t
            if ks is not missing value then set tshort to ks
        end try
        set tparent to ""
        try
            if parent tag of t is not missing value then set tparent to name of parent tag of t
        end try
        set output to output & tid & "␞" & tname & "␞" & tshort & "␞" & tparent & "␟"
    end repeat
    return output
end tell"#;

    let raw = run_applescript(script)?;
    raw.split('␟')
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            let parts: Vec<&str> = l.trim().splitn(4, '␞').collect();
            if parts.len() < 4 {
                return Err(format!("Unexpected tag format: {}", l));
            }
            Ok(Tag {
                id: parts[0].to_string(),
                name: parts[1].to_string(),
                keyboard_shortcut: parse_optional(parts[2]),
                parent_tag: parse_optional(parts[3]),
            })
        })
        .collect()
}

pub fn create_tag(payload: &CreateTag) -> Result<Tag, String> {
    let name = esc(&payload.name);
    let mut props = format!("name:\"{name}\"");
    if let Some(ks) = &payload.keyboard_shortcut {
        if !ks.is_empty() {
            props.push_str(&format!(", keyboard shortcut:\"{}\"", esc(ks)));
        }
    }

    let mut lines = vec![
        "tell application \"Things3\"".to_string(),
        format!("    set newTag to make new tag with properties {{{props}}}"),
    ];
    if let Some(parent) = &payload.parent_tag {
        if !parent.is_empty() {
            lines.push(format!(
                "    set parent tag of newTag to tag \"{}\"",
                esc(parent)
            ));
        }
    }
    lines.push("    return id of newTag".to_string());
    lines.push("end tell".to_string());
    let script = lines.join("\n");
    let id = run_applescript(&script)?.trim().to_string();

    // Look up by id to return the full record (the id-by-id case isn't a separate
    // sdef command for tags, but `tag id "..."` works).
    get_tag_by_id(&id)
}

pub fn get_tag_by_id(tag_id: &str) -> Result<Tag, String> {
    let script = format!(
        r#"tell application "Things3"
    set t to tag id "{id}"
    set tid to id of t
    set tname to name of t
    set tshort to ""
    try
        set ks to keyboard shortcut of t
        if ks is not missing value then set tshort to ks
    end try
    set tparent to ""
    try
        if parent tag of t is not missing value then set tparent to name of parent tag of t
    end try
    return tid & "␞" & tname & "␞" & tshort & "␞" & tparent
end tell"#,
        id = esc(tag_id)
    );
    let raw = run_applescript(&script)?;
    let parts: Vec<&str> = raw.trim().splitn(4, '␞').collect();
    if parts.len() < 4 {
        return Err(format!("Unexpected tag format: {}", raw));
    }
    Ok(Tag {
        id: parts[0].to_string(),
        name: parts[1].to_string(),
        keyboard_shortcut: parse_optional(parts[2]),
        parent_tag: parse_optional(parts[3]),
    })
}

pub fn update_tag(tag_id: &str, payload: &UpdateTag) -> Result<Tag, String> {
    let mut updates: Vec<String> = vec![];
    if let Some(name) = &payload.name {
        updates.push(format!("set name of t to \"{}\"", esc(name)));
    }
    if let Some(ks) = &payload.keyboard_shortcut {
        if ks.is_empty() {
            updates.push("set keyboard shortcut of t to missing value".to_string());
        } else {
            updates.push(format!(
                "set keyboard shortcut of t to \"{}\"",
                esc(ks)
            ));
        }
    }
    if let Some(parent) = &payload.parent_tag {
        if parent.is_empty() {
            updates.push("set parent tag of t to missing value".to_string());
        } else {
            updates.push(format!(
                "set parent tag of t to tag \"{}\"",
                esc(parent)
            ));
        }
    }

    if updates.is_empty() {
        return get_tag_by_id(tag_id);
    }

    let body = updates.join("\n    ");
    let script = format!(
        r#"tell application "Things3"
    set t to tag id "{id}"
    {body}
end tell"#,
        id = esc(tag_id),
    );
    run_applescript(&script)?;
    get_tag_by_id(tag_id)
}

pub fn delete_tag(tag_id: &str) -> Result<(), String> {
    let script = format!(
        r#"tell application "Things3"
    delete (tag id "{id}")
end tell"#,
        id = esc(tag_id)
    );
    run_applescript(&script)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Areas
// ---------------------------------------------------------------------------

pub fn get_areas() -> Result<Vec<Area>, String> {
    let script = r#"tell application "Things3"
    set output to ""
    set theAreas to areas
    repeat with a in theAreas
        set aid to id of a
        set atitle to name of a
        set acollapsed to (collapsed of a)
        set atags to ""
        set tagList to tags of a
        repeat with tg in tagList
            if atags is "" then
                set atags to name of tg
            else
                set atags to atags & "," & name of tg
            end if
        end repeat
        set output to output & aid & "␞" & atitle & "␞" & acollapsed & "␞" & atags & "␟"
    end repeat
    return output
end tell"#;

    let raw = run_applescript(script)?;
    raw.split('␟')
        .filter(|l| !l.trim().is_empty())
        .map(|l| parse_area_line(l.trim()))
        .collect()
}

pub fn get_area_by_id(area_id: &str) -> Result<Area, String> {
    let script = format!(
        r#"tell application "Things3"
    set a to area id "{id}"
    set aid to id of a
    set atitle to name of a
    set acollapsed to (collapsed of a)
    set atags to ""
    set tagList to tags of a
    repeat with tg in tagList
        if atags is "" then
            set atags to name of tg
        else
            set atags to atags & "," & name of tg
        end if
    end repeat
    return aid & "␞" & atitle & "␞" & acollapsed & "␞" & atags
end tell"#,
        id = esc(area_id)
    );
    let raw = run_applescript(&script)?;
    parse_area_line(raw.trim())
}

fn parse_area_line(line: &str) -> Result<Area, String> {
    let parts: Vec<&str> = line.splitn(4, '␞').collect();
    if parts.len() < 4 {
        return Err(format!("Unexpected area format: {}", line));
    }

    let tags: Vec<String> = if parts[3].is_empty() {
        vec![]
    } else {
        parts[3].split(',').map(|s| s.trim().to_string()).collect()
    };

    Ok(Area {
        id: parts[0].to_string(),
        title: parts[1].to_string(),
        collapsed: parts[2].trim() == "true",
        tags,
    })
}

pub fn create_area(payload: &CreateArea) -> Result<Area, String> {
    let title = esc(&payload.title);
    let mut props = format!("name:\"{title}\"");
    if let Some(tags) = &payload.tags {
        if !tags.is_empty() {
            let joined = tags.iter().map(|t| esc(t)).collect::<Vec<_>>().join(", ");
            props.push_str(&format!(", tag names:\"{joined}\""));
        }
    }
    if let Some(collapsed) = payload.collapsed {
        props.push_str(&format!(", collapsed:{}", collapsed));
    }
    let script = format!(
        r#"tell application "Things3"
    set newArea to make new area with properties {{{props}}}
    return id of newArea
end tell"#
    );
    let id = run_applescript(&script)?.trim().to_string();
    get_area_by_id(&id)
}

pub fn update_area(area_id: &str, payload: &UpdateArea) -> Result<Area, String> {
    let mut updates: Vec<String> = vec![];
    if let Some(title) = &payload.title {
        updates.push(format!("set name of a to \"{}\"", esc(title)));
    }
    if let Some(tags) = &payload.tags {
        let tag_list = tags
            .iter()
            .map(|t| format!("\"{}\"", esc(t)))
            .collect::<Vec<_>>()
            .join(", ");
        updates.push(format!("set tag names of a to {{{tag_list}}}"));
    }
    if let Some(collapsed) = payload.collapsed {
        updates.push(format!("set collapsed of a to {}", collapsed));
    }

    if updates.is_empty() {
        return get_area_by_id(area_id);
    }

    let body = updates.join("\n    ");
    let script = format!(
        r#"tell application "Things3"
    set a to area id "{id}"
    {body}
end tell"#,
        id = esc(area_id),
    );
    run_applescript(&script)?;
    get_area_by_id(area_id)
}

pub fn delete_area(area_id: &str) -> Result<(), String> {
    let script = format!(
        r#"tell application "Things3"
    delete (area id "{id}")
end tell"#,
        id = esc(area_id)
    );
    run_applescript(&script)?;
    Ok(())
}

pub fn show_area(area_id: &str) -> Result<(), String> {
    let script = format!(
        r#"tell application "Things3"
    show area id "{id}"
end tell"#,
        id = esc(area_id)
    );
    run_applescript(&script)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Contacts
// ---------------------------------------------------------------------------

pub fn get_contacts() -> Result<Vec<Contact>, String> {
    let script = r#"tell application "Things3"
    set output to ""
    set theContacts to contacts
    repeat with c in theContacts
        set cid to id of c
        set cname to name of c
        set output to output & cid & "␞" & cname & "␟"
    end repeat
    return output
end tell"#;
    let raw = run_applescript(script)?;
    raw.split('␟')
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            let parts: Vec<&str> = l.trim().splitn(2, '␞').collect();
            if parts.len() < 2 {
                Err(format!("Unexpected contact format: {}", l))
            } else {
                Ok(Contact {
                    id: parts[0].to_string(),
                    name: parts[1].to_string(),
                })
            }
        })
        .collect()
}

pub fn create_contact(payload: &CreateContact) -> Result<Contact, String> {
    let script = format!(
        r#"tell application "Things3"
    set newContact to add contact named "{name}"
    return (id of newContact) & "␞" & (name of newContact)
end tell"#,
        name = esc(&payload.name)
    );
    let raw = run_applescript(&script)?;
    let parts: Vec<&str> = raw.trim().splitn(2, '␞').collect();
    if parts.len() < 2 {
        return Err(format!("Unexpected contact format: {}", raw));
    }
    Ok(Contact {
        id: parts[0].to_string(),
        name: parts[1].to_string(),
    })
}

pub fn delete_contact(contact_id: &str) -> Result<(), String> {
    let script = format!(
        r#"tell application "Things3"
    delete (contact id "{id}")
end tell"#,
        id = esc(contact_id)
    );
    run_applescript(&script)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Count (standard suite `count` command)
// ---------------------------------------------------------------------------

pub fn count_tasks(list_filter: Option<&str>) -> Result<usize, String> {
    let list_spec = format!("list \"{}\"", esc(&list_name(list_filter)));
    let script = format!(
        r#"tell application "Things3"
    return count of to dos of {list_spec}
end tell"#
    );
    let raw = run_applescript(&script)?;
    raw.trim()
        .parse::<usize>()
        .map_err(|e| format!("Failed to parse count: {} ({})", raw, e))
}

pub fn count_projects() -> Result<usize, String> {
    let script = r#"tell application "Things3"
    return count of projects
end tell"#;
    let raw = run_applescript(script)?;
    raw.trim()
        .parse::<usize>()
        .map_err(|e| format!("Failed to parse count: {} ({})", raw, e))
}

pub fn count_areas() -> Result<usize, String> {
    let script = r#"tell application "Things3"
    return count of areas
end tell"#;
    let raw = run_applescript(script)?;
    raw.trim()
        .parse::<usize>()
        .map_err(|e| format!("Failed to parse count: {} ({})", raw, e))
}

pub fn count_tags() -> Result<usize, String> {
    let script = r#"tell application "Things3"
    return count of tags
end tell"#;
    let raw = run_applescript(script)?;
    raw.trim()
        .parse::<usize>()
        .map_err(|e| format!("Failed to parse count: {} ({})", raw, e))
}

pub fn count_contacts() -> Result<usize, String> {
    let script = r#"tell application "Things3"
    return count of contacts
end tell"#;
    let raw = run_applescript(script)?;
    raw.trim()
        .parse::<usize>()
        .map_err(|e| format!("Failed to parse count: {} ({})", raw, e))
}

// ---------------------------------------------------------------------------
// Sub-resource listings (element collections on project/area/tag/contact)
// ---------------------------------------------------------------------------

fn tasks_of(scope_expr: &str) -> Result<Vec<Task>, String> {
    let script = format!(
        r#"tell application "Things3"
    set output to ""
    set theTasks to to dos of {scope_expr}
    repeat with t in theTasks
{record}
        set output to output & taskOutput & "␟"
    end repeat
    return output
end tell"#,
        record = TASK_RECORD_FRAGMENT,
    );
    let raw = run_applescript(&script)?;
    raw.split('␟')
        .filter(|l| !l.trim().is_empty())
        .map(|l| parse_task_line(l.trim()))
        .collect()
}

pub fn get_project_tasks(project_id: &str) -> Result<Vec<Task>, String> {
    tasks_of(&format!("project id \"{}\"", esc(project_id)))
}

pub fn get_area_tasks(area_id: &str) -> Result<Vec<Task>, String> {
    tasks_of(&format!("area id \"{}\"", esc(area_id)))
}

pub fn get_tag_tasks(tag_id: &str) -> Result<Vec<Task>, String> {
    tasks_of(&format!("tag id \"{}\"", esc(tag_id)))
}

pub fn get_contact_tasks(contact_id: &str) -> Result<Vec<Task>, String> {
    tasks_of(&format!("contact id \"{}\"", esc(contact_id)))
}

pub fn get_tag_children(tag_id: &str) -> Result<Vec<Tag>, String> {
    let script = format!(
        r#"tell application "Things3"
    set output to ""
    set theTags to tags of tag id "{id}"
    repeat with t in theTags
        set tid to id of t
        set tname to name of t
        set tshort to ""
        try
            set ks to keyboard shortcut of t
            if ks is not missing value then set tshort to ks
        end try
        set tparent to ""
        try
            if parent tag of t is not missing value then set tparent to name of parent tag of t
        end try
        set output to output & tid & "␞" & tname & "␞" & tshort & "␞" & tparent & "␟"
    end repeat
    return output
end tell"#,
        id = esc(tag_id),
    );
    let raw = run_applescript(&script)?;
    raw.split('␟')
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            let parts: Vec<&str> = l.trim().splitn(4, '␞').collect();
            if parts.len() < 4 {
                return Err(format!("Unexpected tag format: {}", l));
            }
            Ok(Tag {
                id: parts[0].to_string(),
                name: parts[1].to_string(),
                keyboard_shortcut: parse_optional(parts[2]),
                parent_tag: parse_optional(parts[3]),
            })
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Application info & windows
// ---------------------------------------------------------------------------

pub fn get_app_info() -> Result<AppInfo, String> {
    let script = r#"tell application "Things3"
    set aname to name
    set aversion to version
    set afront to frontmost
    set acurname to ""
    try
        set cn to current list name
        if cn is not missing value then set acurname to cn
    end try
    set acururl to ""
    try
        set cu to current list url
        if cu is not missing value then set acururl to cu
    end try
    return aname & "␞" & aversion & "␞" & afront & "␞" & acurname & "␞" & acururl
end tell"#;
    let raw = run_applescript(script)?;
    let parts: Vec<&str> = raw.trim().splitn(5, '␞').collect();
    if parts.len() < 5 {
        return Err(format!("Unexpected app info format: {}", raw));
    }
    Ok(AppInfo {
        name: parts[0].to_string(),
        version: parts[1].to_string(),
        frontmost: parts[2].trim() == "true",
        current_list_name: parse_optional(parts[3]),
        current_list_url: parse_optional(parts[4]),
    })
}

pub fn get_windows() -> Result<Vec<WindowInfo>, String> {
    let script = r#"tell application "Things3"
    set output to ""
    set theWindows to windows
    repeat with w in theWindows
        set wid to id of w
        set wname to name of w
        set widx to index of w
        set wb to bounds of w
        set wx to item 1 of wb
        set wy to item 2 of wb
        set ww to item 3 of wb
        set wh to item 4 of wb
        set wvis to visible of w
        set wmin to minimized of w
        set wzoom to zoomed of w
        set wcl to closeable of w
        set wmnb to minimizable of w
        set wrsz to resizable of w
        set wzmb to zoomable of w
        set output to output & wid & "␞" & wname & "␞" & widx & "␞" & wx & "," & wy & "," & ww & "," & wh & "␞" & wvis & "␞" & wmin & "␞" & wzoom & "␞" & wcl & "␞" & wmnb & "␞" & wrsz & "␞" & wzmb & "␟"
    end repeat
    return output
end tell"#;
    let raw = run_applescript(script)?;
    raw.split('␟')
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            let parts: Vec<&str> = l.trim().splitn(11, '␞').collect();
            if parts.len() < 11 {
                return Err(format!("Unexpected window format: {}", l));
            }
            let bounds_parts: Vec<&str> = parts[3].split(',').collect();
            if bounds_parts.len() != 4 {
                return Err(format!("Unexpected bounds format: {}", parts[3]));
            }
            let parse_int = |s: &str| {
                s.trim()
                    .parse::<i64>()
                    .map_err(|e| format!("int parse error: {} ({})", s, e))
            };
            Ok(WindowInfo {
                id: parse_int(parts[0])?,
                name: parts[1].to_string(),
                index: parse_int(parts[2])?,
                bounds: [
                    parse_int(bounds_parts[0])?,
                    parse_int(bounds_parts[1])?,
                    parse_int(bounds_parts[2])?,
                    parse_int(bounds_parts[3])?,
                ],
                visible: parts[4].trim() == "true",
                minimized: parts[5].trim() == "true",
                zoomed: parts[6].trim() == "true",
                closeable: parts[7].trim() == "true",
                minimizable: parts[8].trim() == "true",
                resizable: parts[9].trim() == "true",
                zoomable: parts[10].trim() == "true",
            })
        })
        .collect()
}

pub fn update_window(window_id: i64, payload: &UpdateWindow) -> Result<(), String> {
    let mut updates: Vec<String> = vec![];
    if let Some(idx) = payload.index {
        updates.push(format!("set index of w to {}", idx));
    }
    if let Some(b) = payload.bounds {
        updates.push(format!("set bounds of w to {{{}, {}, {}, {}}}", b[0], b[1], b[2], b[3]));
    }
    if let Some(v) = payload.visible {
        updates.push(format!("set visible of w to {}", v));
    }
    if let Some(m) = payload.minimized {
        updates.push(format!("set minimized of w to {}", m));
    }
    if let Some(z) = payload.zoomed {
        updates.push(format!("set zoomed of w to {}", z));
    }
    if updates.is_empty() {
        return Ok(());
    }
    let body = updates.join("\n    ");
    let script = format!(
        r#"tell application "Things3"
    set w to window id {id}
    {body}
end tell"#,
        id = window_id,
    );
    run_applescript(&script)?;
    Ok(())
}

pub fn close_window(window_id: i64) -> Result<(), String> {
    let script = format!(
        r#"tell application "Things3"
    close window id {id}
end tell"#,
        id = window_id,
    );
    run_applescript(&script)?;
    Ok(())
}

pub fn quit_app() -> Result<(), String> {
    let script = r#"tell application "Things3"
    quit
end tell"#;
    run_applescript(script)?;
    Ok(())
}
