// src-tauri/src/commands.rs
use crate::{events, state::AppState};
use tauri::{AppHandle, State};
use tempo_core::{repo, Bootstrap, CalendarItem, Habit, List, Note, PomodoroSession, Task};

type R<T> = Result<T, String>;
fn e(err: impl std::fmt::Display) -> String { err.to_string() }

#[tauri::command]
pub fn get_bootstrap(state: State<'_, AppState>) -> R<Bootstrap> {
    let db = state.db.lock().map_err(e)?;
    repo::bootstrap(&db).map_err(e)
}

#[tauri::command]
pub fn get_calendar_range(state: State<'_, AppState>, from: String, to: String) -> R<Vec<CalendarItem>> {
    let db = state.db.lock().map_err(e)?;
    repo::calendar_range(&db, &from, &to).map_err(e)
}

#[tauri::command]
pub fn save_task(app: AppHandle, state: State<'_, AppState>, input: tempo_core::TaskInput) -> R<Task> {
    let out = { let db = state.db.lock().map_err(e)?; repo::save_task(&db, input).map_err(e)? };
    events::changed(&app, "tasks");
    Ok(out)
}

#[tauri::command]
pub fn toggle_task(app: AppHandle, state: State<'_, AppState>, id: String, occurrence_key: Option<String>) -> R<()> {
    { let db = state.db.lock().map_err(e)?; repo::toggle_task(&db, &id, occurrence_key).map_err(e)?; }
    events::changed(&app, "tasks");
    Ok(())
}

#[tauri::command]
pub fn delete_task(app: AppHandle, state: State<'_, AppState>, id: String) -> R<()> {
    { let db = state.db.lock().map_err(e)?; repo::delete_task(&db, &id).map_err(e)?; }
    events::changed(&app, "tasks");
    Ok(())
}

#[tauri::command]
pub fn reorder_tasks(app: AppHandle, state: State<'_, AppState>, ids: Vec<String>) -> R<()> {
    { let db = state.db.lock().map_err(e)?; repo::reorder_tasks(&db, &ids).map_err(e)?; }
    events::changed(&app, "tasks");
    Ok(())
}

#[tauri::command]
pub fn bulk_complete(app: AppHandle, state: State<'_, AppState>, ids: Vec<String>, done: bool) -> R<()> {
    { let db = state.db.lock().map_err(e)?; repo::bulk_complete(&db, &ids, done).map_err(e)?; }
    events::changed(&app, "tasks");
    Ok(())
}

#[tauri::command]
pub fn restore_entities(app: AppHandle, state: State<'_, AppState>, kind: String, ids: Vec<String>) -> R<()> {
    { let db = state.db.lock().map_err(e)?; repo::restore(&db, &kind, &ids).map_err(e)?; }
    events::changed(&app, "all");
    Ok(())
}

#[tauri::command]
pub fn log_pomodoro(app: AppHandle, state: State<'_, AppState>, task_id: Option<String>, duration_sec: i64, kind: String) -> R<PomodoroSession> {
    let out = { let db = state.db.lock().map_err(e)?; repo::log_pomodoro(&db, task_id, duration_sec, &kind).map_err(e)? };
    events::changed(&app, "pomodoro");
    Ok(out)
}

#[tauri::command]
pub fn save_list(app: AppHandle, state: State<'_, AppState>, input: tempo_core::ListInput) -> R<List> {
    let out = { let db = state.db.lock().map_err(e)?; repo::save_list(&db, input).map_err(e)? };
    events::changed(&app, "lists");
    Ok(out)
}

#[tauri::command]
pub fn delete_list(app: AppHandle, state: State<'_, AppState>, id: String) -> R<()> {
    { let db = state.db.lock().map_err(e)?; repo::delete_list(&db, &id).map_err(e)?; }
    events::changed(&app, "all");
    Ok(())
}

#[tauri::command]
pub fn save_note(app: AppHandle, state: State<'_, AppState>, input: tempo_core::NoteInput) -> R<Note> {
    let out = { let db = state.db.lock().map_err(e)?; repo::save_note(&db, input).map_err(e)? };
    events::changed(&app, "notes");
    Ok(out)
}

#[tauri::command]
pub fn delete_note(app: AppHandle, state: State<'_, AppState>, id: String) -> R<()> {
    { let db = state.db.lock().map_err(e)?; repo::delete_note(&db, &id).map_err(e)?; }
    events::changed(&app, "notes");
    Ok(())
}

#[tauri::command]
pub fn save_habit(app: AppHandle, state: State<'_, AppState>, input: tempo_core::HabitInput) -> R<Habit> {
    let out = { let db = state.db.lock().map_err(e)?; repo::save_habit(&db, input).map_err(e)? };
    events::changed(&app, "habits");
    Ok(out)
}

#[tauri::command]
pub fn delete_habit(app: AppHandle, state: State<'_, AppState>, id: String) -> R<()> {
    { let db = state.db.lock().map_err(e)?; repo::delete_habit(&db, &id).map_err(e)?; }
    events::changed(&app, "habits");
    Ok(())
}

#[tauri::command]
pub fn toggle_habit(app: AppHandle, state: State<'_, AppState>, habit_id: String, local_date: String) -> R<()> {
    { let db = state.db.lock().map_err(e)?; repo::toggle_habit(&db, &habit_id, &local_date).map_err(e)?; }
    events::changed(&app, "habits");
    Ok(())
}