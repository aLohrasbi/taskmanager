// crates/core/src/repo.rs
use crate::{
    habits::{streak, Streak},
    models::*,
    new_id, now, recurrence, Error, Result,
};
use chrono::{DateTime, Duration, Utc};
use rusqlite::{params, Connection, OptionalExtension, Row};
use std::collections::HashSet;

fn map_list(r: &Row) -> rusqlite::Result<List> {
    Ok(List {
        id: r.get("id")?, name: r.get("name")?, color: r.get("color")?,
        sort_order: r.get("sort_order")?, created_at: r.get("created_at")?, updated_at: r.get("updated_at")?,
    })
}
fn map_task(r: &Row) -> rusqlite::Result<Task> {
    Ok(Task {
        id: r.get("id")?, list_id: r.get("list_id")?, title: r.get("title")?, notes: r.get("notes")?,
        due_at: r.get("due_at")?, due_tz: r.get("due_tz")?, all_day: r.get("all_day")?,
        rrule: r.get("rrule")?, priority: r.get("priority")?, countdown: r.get("countdown")?,
        completed_at: r.get("completed_at")?, sort_order: r.get("sort_order")?,
        created_at: r.get("created_at")?, updated_at: r.get("updated_at")?,
    })
}
fn map_note(r: &Row) -> rusqlite::Result<Note> {
    Ok(Note {
        id: r.get("id")?, title: r.get("title")?, body: r.get("body")?, pinned: r.get("pinned")?,
        created_at: r.get("created_at")?, updated_at: r.get("updated_at")?,
    })
}
fn map_habit(r: &Row) -> rusqlite::Result<Habit> {
    Ok(Habit {
        id: r.get("id")?, name: r.get("name")?, emoji: r.get("emoji")?, color: r.get("color")?,
        weekdays_mask: r.get("weekdays_mask")?, created_at: r.get("created_at")?, updated_at: r.get("updated_at")?,
    })
}

fn query<T>(conn: &Connection, sql: &str, f: impl FnMut(&Row<'_>) -> rusqlite::Result<T>) -> Result<Vec<T>> {
    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map([], f)?;
    Ok(rows.collect::<rusqlite::Result<Vec<T>>>()?)
}

pub fn bootstrap(conn: &Connection) -> Result<Bootstrap> {
    Ok(Bootstrap {
        lists: query(conn, "SELECT * FROM lists WHERE deleted_at IS NULL ORDER BY sort_order, created_at", map_list)?,
        tasks: query(conn, "SELECT * FROM tasks WHERE deleted_at IS NULL ORDER BY sort_order, created_at", map_task)?,
        notes: query(conn, "SELECT * FROM notes WHERE deleted_at IS NULL ORDER BY pinned DESC, updated_at DESC", map_note)?,
        habits: query(conn, "SELECT * FROM habits WHERE deleted_at IS NULL ORDER BY created_at", map_habit)?,
        habit_entries: query(conn, "SELECT * FROM habit_entries WHERE deleted_at IS NULL", |r| {
            Ok(HabitEntry { habit_id: r.get("habit_id")?, local_date: r.get("local_date")?, value: r.get("value")? })
        })?,
        completions: query(conn, "SELECT * FROM task_completions WHERE deleted_at IS NULL", |r| {
            Ok(TaskCompletion { task_id: r.get("task_id")?, occurrence_key: r.get("occurrence_key")?, completed_at: r.get("completed_at")? })
        })?,
        pomodoros: query(conn, "SELECT * FROM pomodoro_sessions WHERE deleted_at IS NULL ORDER BY started_at DESC LIMIT 50", |r| {
            Ok(PomodoroSession {
                id: r.get("id")?, task_id: r.get("task_id")?, started_at: r.get("started_at")?,
                ended_at: r.get::<_, Option<String>>("ended_at")?.unwrap_or_default(),
                duration_sec: r.get("duration_sec")?, kind: r.get("kind")?,
            })
        })?,
    })
}

/* ---------- lists ---------- */

pub fn save_list(conn: &Connection, input: ListInput) -> Result<List> {
    let name = input.name.trim();
    if name.is_empty() { return Err(Error::Invalid("list name is empty".into())); }
    let ts = now();
    let id = input.id.filter(|s| !s.is_empty()).unwrap_or_else(new_id);
    let color = input.color.unwrap_or_else(|| "#7c83ff".into());
    let exists: bool = conn.query_row("SELECT EXISTS(SELECT 1 FROM lists WHERE id=?1)", [&id], |r| r.get(0))?;
    if exists {
        conn.execute("UPDATE lists SET name=?2, color=?3, updated_at=?4 WHERE id=?1", params![id, name, color, ts])?;
    } else {
        let next: i64 = conn.query_row("SELECT COALESCE(MAX(sort_order),-1)+1 FROM lists", [], |r| r.get(0))?;
        conn.execute(
            "INSERT INTO lists (id,name,color,sort_order,created_at,updated_at) VALUES (?1,?2,?3,?4,?5,?5)",
            params![id, name, color, next, ts],
        )?;
    }
    conn.query_row("SELECT * FROM lists WHERE id=?1", [&id], map_list).map_err(Into::into)
}

pub fn delete_list(conn: &Connection, id: &str) -> Result<()> {
    let ts = now();
    conn.execute("UPDATE tasks SET list_id=NULL, updated_at=?2 WHERE list_id=?1", params![id, ts])?;
    conn.execute("UPDATE lists SET deleted_at=?2, updated_at=?2 WHERE id=?1", params![id, ts])?;
    Ok(())
}

/* ---------- tasks & reminders ---------- */

pub fn get_task(conn: &Connection, id: &str) -> Result<Task> {
    conn.query_row("SELECT * FROM tasks WHERE id=?1 AND deleted_at IS NULL", [id], map_task)
        .optional()?.ok_or(Error::NotFound("task"))
}

pub fn save_task(conn: &Connection, input: TaskInput) -> Result<Task> {
    let title = input.title.trim();
    if title.is_empty() { return Err(Error::Invalid("task title is empty".into())); }
    if let Some(due) = &input.due_at {
        DateTime::parse_from_rfc3339(due).map_err(|_| Error::Invalid("dueAt must be ISO-8601".into()))?;
    }
    let ts = now();
    let id = input.id.filter(|s| !s.is_empty()).unwrap_or_else(new_id);
    let exists: bool = conn.query_row("SELECT EXISTS(SELECT 1 FROM tasks WHERE id=?1)", [&id], |r| r.get(0))?;
    let notes = input.notes.unwrap_or_default();
    let all_day = input.all_day.unwrap_or(true);
    let priority = input.priority.unwrap_or(0).clamp(0, 3);
    let countdown = input.countdown.unwrap_or(false);
    let rrule = input.rrule.filter(|s| !s.trim().is_empty());
    if exists {
        conn.execute(
            "UPDATE tasks SET list_id=?2,title=?3,notes=?4,due_at=?5,due_tz=?6,all_day=?7,rrule=?8,priority=?9,countdown=?10,updated_at=?11 WHERE id=?1",
            params![id, input.list_id, title, notes, input.due_at, input.due_tz, all_day, rrule, priority, countdown, ts],
        )?;
    } else {
        let next: i64 = conn.query_row("SELECT COALESCE(MAX(sort_order),-1)+1 FROM tasks", [], |r| r.get(0))?;
        conn.execute(
            "INSERT INTO tasks (id,list_id,title,notes,due_at,due_tz,all_day,rrule,priority,countdown,sort_order,created_at,updated_at)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?12)",
            params![id, input.list_id, title, notes, input.due_at, input.due_tz, all_day, rrule, priority, countdown, next, ts],
        )?;
    }
    if let Some(rem) = input.remind_at.filter(|s| !s.is_empty()) {
        DateTime::parse_from_rfc3339(&rem).map_err(|_| Error::Invalid("remindAt must be ISO-8601".into()))?;
        conn.execute("UPDATE reminders SET cancelled_at=?2, updated_at=?2 WHERE task_id=?1 AND fired_at IS NULL", params![id, ts])?;
        conn.execute(
            "INSERT INTO reminders (id,task_id,occurrence_key,fire_at,created_at,updated_at) VALUES (?1,?2,'base',?3,?4,?4)",
            params![new_id(), id, rem, ts],
        )?;
    }
    get_task(conn, &id)
}

pub fn pop_due_reminders(conn: &Connection, now_iso: &str) -> Result<Vec<Reminder>> {
    let mut stmt = conn.prepare(
        "SELECT r.id, r.task_id, t.title, r.fire_at
         FROM reminders r JOIN tasks t ON t.id = r.task_id
         WHERE r.fired_at IS NULL AND r.cancelled_at IS NULL AND r.deleted_at IS NULL
           AND t.deleted_at IS NULL AND t.completed_at IS NULL AND r.fire_at <= ?1"
    )?;
    let due: Vec<Reminder> = stmt.query_map([now_iso], |r| {
        Ok(Reminder { id: r.get(0)?, task_id: r.get(1)?, title: r.get(2)?, fire_at: r.get(3)? })
    })?.collect::<rusqlite::Result<Vec<_>>>()?;
    for rem in &due {
        conn.execute("UPDATE reminders SET fired_at=?2, updated_at=?2 WHERE id=?1", params![rem.id, now_iso])?;
    }
    Ok(due)
}

pub fn toggle_task(conn: &Connection, id: &str, occurrence_key: Option<String>) -> Result<()> {
    let task = get_task(conn, id)?;
    let ts = now();
    if task.rrule.is_some() {
        let key = occurrence_key.unwrap_or_else(|| {
            task.due_at.as_deref()
                .and_then(|d| DateTime::parse_from_rfc3339(d).ok())
                .map(|d| d.with_timezone(&Utc).to_rfc3339_opts(chrono::SecondsFormat::Secs, true))
                .unwrap_or_else(|| "base".into())
        });
        let existing: Option<Option<String>> = conn
            .query_row("SELECT deleted_at FROM task_completions WHERE task_id=?1 AND occurrence_key=?2",
                params![id, key], |r| r.get(0)).optional()?;
        let checking = match &existing { None => true, Some(d) => d.is_some() };
        conn.execute(
            "INSERT INTO task_completions (task_id,occurrence_key,completed_at,created_at,updated_at,deleted_at)
             VALUES (?1,?2,?3,?3,?3,?4)
             ON CONFLICT(task_id,occurrence_key) DO UPDATE SET completed_at=?3,updated_at=?3,deleted_at=?4",
            params![id, key, ts, if checking { None } else { Some(ts.clone()) }],
        )?;
    } else {
        let done = task.completed_at.is_none();
        conn.execute("UPDATE tasks SET completed_at=?2, updated_at=?3 WHERE id=?1",
            params![id, if done { Some(ts.clone()) } else { None }, ts])?;
    }
    Ok(())
}

pub fn delete_task(conn: &Connection, id: &str) -> Result<()> {
    let ts = now();
    conn.execute("UPDATE tasks SET deleted_at=?2, updated_at=?2 WHERE id=?1", params![id, ts])?;
    Ok(())
}

pub fn reorder_tasks(conn: &Connection, ids: &[String]) -> Result<()> {
    let ts = now();
    for (i, id) in ids.iter().enumerate() {
        conn.execute("UPDATE tasks SET sort_order=?2, updated_at=?3 WHERE id=?1", params![id, i as i64, ts])?;
    }
    Ok(())
}

pub fn bulk_complete(conn: &Connection, ids: &[String], done: bool) -> Result<()> {
    let ts = now();
    for id in ids {
        conn.execute("UPDATE tasks SET completed_at=?2, updated_at=?3 WHERE id=?1",
            params![id, if done { Some(ts.clone()) } else { None }, ts])?;
    }
    Ok(())
}

pub fn restore(conn: &Connection, kind: &str, ids: &[String]) -> Result<()> {
    let table = match kind {
        "tasks" => "tasks", "notes" => "notes", "habits" => "habits", "lists" => "lists",
        _ => return Err(Error::Invalid("unknown entity kind".into())),
    };
    let ts = now();
    for id in ids {
        conn.execute(&format!("UPDATE {table} SET deleted_at=NULL, updated_at=?2 WHERE id=?1"), params![id, ts])?;
    }
    Ok(())
}

/* ---------- pomodoro ---------- */

pub fn log_pomodoro(conn: &Connection, task_id: Option<String>, duration_sec: i64, kind: &str) -> Result<PomodoroSession> {
    let id = new_id();
    let ended = Utc::now();
    let started = (ended - Duration::seconds(duration_sec.max(1))).to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    let ended_str = ended.to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    conn.execute(
        "INSERT INTO pomodoro_sessions (id,task_id,started_at,ended_at,duration_sec,kind,created_at,updated_at)
         VALUES (?1,?2,?3,?4,?5,?6,?4,?4)",
        params![id, task_id, started, ended_str, duration_sec, kind],
    )?;
    Ok(PomodoroSession { id, task_id, started_at: started, ended_at: ended_str, duration_sec, kind: kind.into() })
}

/* ---------- notes ---------- */

pub fn save_note(conn: &Connection, input: NoteInput) -> Result<Note> {
    let ts = now();
    let id = input.id.filter(|s| !s.is_empty()).unwrap_or_else(new_id);
    let pinned = input.pinned.unwrap_or(false);
    let exists: bool = conn.query_row("SELECT EXISTS(SELECT 1 FROM notes WHERE id=?1)", [&id], |r| r.get(0))?;
    if exists {
        conn.execute("UPDATE notes SET title=?2,body=?3,pinned=?4,updated_at=?5 WHERE id=?1",
            params![id, input.title, input.body, pinned, ts])?;
    } else {
        conn.execute("INSERT INTO notes (id,title,body,pinned,created_at,updated_at) VALUES (?1,?2,?3,?4,?5,?5)",
            params![id, input.title, input.body, pinned, ts])?;
    }
    conn.query_row("SELECT * FROM notes WHERE id=?1", [&id], map_note).map_err(Into::into)
}

pub fn delete_note(conn: &Connection, id: &str) -> Result<()> {
    let ts = now();
    conn.execute("UPDATE notes SET deleted_at=?2, updated_at=?2 WHERE id=?1", params![id, ts])?;
    Ok(())
}

/* ---------- habits ---------- */

pub fn save_habit(conn: &Connection, input: HabitInput) -> Result<Habit> {
    let name = input.name.trim();
    if name.is_empty() { return Err(Error::Invalid("habit name is empty".into())); }
    let ts = now();
    let id = input.id.filter(|s| !s.is_empty()).unwrap_or_else(new_id);
    let emoji = input.emoji.unwrap_or_else(|| "✨".into());
    let color = input.color.unwrap_or_else(|| "#38e1ff".into());
    let exists: bool = conn.query_row("SELECT EXISTS(SELECT 1 FROM habits WHERE id=?1)", [&id], |r| r.get(0))?;
    if exists {
        conn.execute("UPDATE habits SET name=?2,emoji=?3,color=?4,updated_at=?5 WHERE id=?1",
            params![id, name, emoji, color, ts])?;
    } else {
        conn.execute("INSERT INTO habits (id,name,emoji,color,created_at,updated_at) VALUES (?1,?2,?3,?4,?5,?5)",
            params![id, name, emoji, color, ts])?;
    }
    conn.query_row("SELECT * FROM habits WHERE id=?1", [&id], map_habit).map_err(Into::into)
}

pub fn delete_habit(conn: &Connection, id: &str) -> Result<()> {
    let ts = now();
    conn.execute("UPDATE habits SET deleted_at=?2, updated_at=?2 WHERE id=?1", params![id, ts])?;
    Ok(())
}

pub fn toggle_habit(conn: &Connection, habit_id: &str, local_date: &str) -> Result<()> {
    if chrono::NaiveDate::parse_from_str(local_date, "%Y-%m-%d").is_err() {
        return Err(Error::Invalid("localDate must be YYYY-MM-DD".into()));
    }
    let ts = now();
    let existing: Option<Option<String>> = conn
        .query_row("SELECT deleted_at FROM habit_entries WHERE habit_id=?1 AND local_date=?2",
            params![habit_id, local_date], |r| r.get(0)).optional()?;
    let checking = match &existing { None => true, Some(d) => d.is_some() };
    conn.execute(
        "INSERT INTO habit_entries (habit_id,local_date,value,created_at,updated_at,deleted_at)
         VALUES (?1,?2,1,?3,?3,?4)
         ON CONFLICT(habit_id,local_date) DO UPDATE SET updated_at=?3, deleted_at=?4",
        params![habit_id, local_date, ts, if checking { None } else { Some(ts.clone()) }],
    )?;
    Ok(())
}

pub fn habit_streak(conn: &Connection, habit_id: &str, today: &str) -> Result<Streak> {
    let mut stmt = conn.prepare("SELECT local_date FROM habit_entries WHERE habit_id=?1 AND deleted_at IS NULL")?;
    let dates = stmt.query_map([habit_id], |r| r.get::<_, String>(0))?.collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(streak(&dates, today))
}

/* ---------- calendar ---------- */

pub fn calendar_range(conn: &Connection, from: &str, to: &str) -> Result<Vec<CalendarItem>> {
    let from = DateTime::parse_from_rfc3339(from)?.with_timezone(&Utc);
    let to = DateTime::parse_from_rfc3339(to)?.with_timezone(&Utc);
    let tasks = query(conn, "SELECT * FROM tasks WHERE deleted_at IS NULL AND due_at IS NOT NULL", map_task)?;
    let completions: HashSet<(String, String)> = query(
        conn, "SELECT task_id, occurrence_key FROM task_completions WHERE deleted_at IS NULL",
        |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)),
    )?.into_iter().collect();
    let raw_overrides = query(
        conn, "SELECT task_id, occurrence_key, override_json FROM task_overrides WHERE deleted_at IS NULL",
        |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?)),
    )?;
    let overrides = recurrence::parse_overrides(&raw_overrides);
    recurrence::expand(&tasks, &from, &to, &completions, &overrides)
}

/* ---------- settings ---------- */

pub fn set_local(conn: &Connection, key: &str, value: &str) -> Result<()> {
    let ts = now();
    conn.execute(
        "INSERT INTO settings_local (key,value,created_at,updated_at) VALUES (?1,?2,?3,?3)
         ON CONFLICT(key) DO UPDATE SET value=?2, updated_at=?3",
        params![key, value, ts],
    )?;
    Ok(())
}

pub fn get_local(conn: &Connection, key: &str) -> Result<Option<String>> {
    Ok(conn.query_row("SELECT value FROM settings_local WHERE key=?1", [key], |r| r.get(0)).optional()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reminder_and_pomodoro_roundtrip() {
        let conn = crate::db::open_memory().unwrap();
        let t = save_task(&conn, TaskInput {
            title: "Call client".into(),
            remind_at: Some("2026-11-14T10:00:00Z".into()),
            ..Default::default()
        }).unwrap();
        let due = pop_due_reminders(&conn, "2026-11-14T10:05:00Z").unwrap();
        assert_eq!(due.len(), 1);
        assert_eq!(due[0].task_id, t.id);
        assert!(pop_due_reminders(&conn, "2026-11-14T10:06:00Z").unwrap().is_empty());

        log_pomodoro(&conn, Some(t.id), 1500, "focus").unwrap();
        assert_eq!(bootstrap(&conn).unwrap().pomodoros.len(), 1);
    }
}