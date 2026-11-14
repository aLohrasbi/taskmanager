// src-tauri/src/state.rs
use rusqlite::Connection;
use std::collections::HashMap;
use std::sync::Mutex;

pub struct AppState {
    pub db: Mutex<Connection>,
    pub pins: Mutex<HashMap<String, String>>,
}

impl AppState {
    pub fn new(db: Connection) -> Self {
        let mut pins = HashMap::new();
        pins.insert("week".into(), "bottom".into());
        pins.insert("next7".into(), "bottom".into());
        Self { db: Mutex::new(db), pins: Mutex::new(pins) }
    }
}