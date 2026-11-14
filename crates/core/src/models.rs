// crates/core/src/models.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct List {
    pub id: String,
    pub name: String,
    pub color: String,
    pub sort_order: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub id: String,
    pub list_id: Option<String>,
    pub title: String,
    pub notes: String,
    pub due_at: Option<String>,
    pub due_tz: Option<String>,
    pub all_day: bool,
    pub rrule: Option<String>,
    pub priority: i64,
    pub countdown: bool,
    pub completed_at: Option<String>,
    pub sort_order: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Note {
    pub id: String,
    pub title: String,
    pub body: String,
    pub pinned: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Habit {
    pub id: String,
    pub name: String,
    pub emoji: String,
    pub color: String,
    pub weekdays_mask: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HabitEntry {
    pub habit_id: String,
    pub local_date: String,
    pub value: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskCompletion {
    pub task_id: String,
    pub occurrence_key: String,
    pub completed_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarItem {
    pub task_id: String,
    pub occurrence_key: String,
    pub title: String,
    pub start_at: String,
    pub all_day: bool,
    pub priority: i64,
    pub countdown: bool,
    pub list_id: Option<String>,
    pub completed: bool,
    pub recurring: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Reminder {
    pub id: String,
    pub task_id: String,
    pub title: String,
    pub fire_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PomodoroSession {
    pub id: String,
    pub task_id: Option<String>,
    pub started_at: String,
    pub ended_at: String,
    pub duration_sec: i64,
    pub kind: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskInput {
    pub id: Option<String>,
    pub list_id: Option<String>,
    pub title: String,
    pub notes: Option<String>,
    pub due_at: Option<String>,
    pub due_tz: Option<String>,
    pub all_day: Option<bool>,
    pub rrule: Option<String>,
    pub priority: Option<i64>,
    pub countdown: Option<bool>,
    pub remind_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListInput {
    pub id: Option<String>,
    pub name: String,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteInput {
    pub id: Option<String>,
    pub title: String,
    pub body: String,
    pub pinned: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HabitInput {
    pub id: Option<String>,
    pub name: String,
    pub emoji: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Bootstrap {
    pub lists: Vec<List>,
    pub tasks: Vec<Task>,
    pub notes: Vec<Note>,
    pub habits: Vec<Habit>,
    pub habit_entries: Vec<HabitEntry>,
    pub completions: Vec<TaskCompletion>,
    pub pomodoros: Vec<PomodoroSession>,
}