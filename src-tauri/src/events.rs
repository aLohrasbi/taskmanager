// src-tauri/src/events.rs
use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tempo_core::Reminder;

#[derive(Clone, Serialize)]
pub struct Changed { pub scope: String }

pub fn changed(app: &AppHandle, scope: &str) {
    let _ = app.emit("tempo:changed", Changed { scope: scope.into() });
}

pub fn widgets(app: &AppHandle) {
    let _ = app.emit("tempo:widgets", ());
}

pub fn reminder(app: &AppHandle, rem: &Reminder) {
    let _ = app.emit("tempo:reminder", rem);
}