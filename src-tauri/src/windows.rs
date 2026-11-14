// src-tauri/src/windows.rs
use crate::{events, state::AppState};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use tempo_core::repo;

#[derive(Serialize)]
pub struct WidgetState { pub label: String, pub visible: bool, pub pin: String }

#[derive(Serialize, Deserialize)]
struct SavedGeom { x: i32, y: i32, w: u32, h: u32, visible: bool, pin: String }

pub const WIDGETS: [&str; 2] = ["week", "next7"];

pub fn setup(app: &AppHandle) {
    let (sw, sh) = app
        .get_webview_window("main")
        .and_then(|w| w.primary_monitor().ok().flatten())
        .map(|m| {
            let s = m.size().to_logical::<f64>(m.scale_factor());
            (s.width, s.height)
        })
        .unwrap_or((1600.0, 900.0));

    for (label, x, y) in [("week", 48.0, (sh - 430.0) / 2.0), ("next7", sw - 360.0 - 48.0, (sh - 640.0) / 2.0)] {
        if let Some(w) = app.get_webview_window(label) {
            frost(&w);
            let _ = w.set_visible_on_all_workspaces(true);
            let saved = {
                let state = app.state::<AppState>();
                let db = state.db.lock().unwrap();
                repo::get_local(&db, &format!("geom:{label}")).ok().flatten()
            }.and_then(|s| serde_json::from_str::<SavedGeom>(&s).ok());

            if let Some(g) = saved {
                let _ = w.set_position(tauri::PhysicalPosition::new(g.x, g.y));
                let _ = w.set_size(tauri::PhysicalSize::new(g.w, g.h));
                apply_pin(&w, &g.pin);
                app.state::<AppState>().pins.lock().unwrap().insert(label.into(), g.pin);
                if g.visible { let _ = w.show(); }
            } else {
                let _ = w.set_position(tauri::LogicalPosition::new(x.max(8.0), y.max(8.0)));
                let _ = w.set_always_on_bottom(true);
                let _ = w.show();
            }
        }
    }
}

fn apply_pin(w: &tauri::WebviewWindow, mode: &str) {
    if mode == "top" {
        let _ = w.set_always_on_bottom(false);
        let _ = w.set_always_on_top(true);
    } else {
        let _ = w.set_always_on_top(false);
        let _ = w.set_always_on_bottom(true);
    }
}

pub fn save_geom(app: &AppHandle, label: &str) {
    if !WIDGETS.contains(&label) { return; }
    let Some(w) = app.get_webview_window(label) else { return };
    let (Ok(pos), Ok(size), Ok(visible)) = (w.outer_position(), w.outer_size(), w.is_visible()) else { return };
    let state = app.state::<AppState>();
    let pin = state.pins.lock().unwrap().get(label).cloned().unwrap_or_else(|| "bottom".into());
    if let Ok(json) = serde_json::to_string(&SavedGeom { x: pos.x, y: pos.y, w: size.width, h: size.height, visible, pin }) {
        if let Ok(db) = state.db.lock() {
            let _ = repo::set_local(&db, &format!("geom:{label}"), &json);
        }
    }
}

fn frost(w: &tauri::WebviewWindow) {
    #[cfg(target_os = "windows")]
    {
        if window_vibrancy::apply_acrylic(w, Some((14, 16, 28, 150))).is_err() {
            let _ = window_vibrancy::apply_blur(w, Some((14, 16, 28, 150)));
        }
    }
    #[cfg(target_os = "macos")]
    {
        let _ = window_vibrancy::apply_vibrancy(w, window_vibrancy::NSVisualEffectMaterial::HudWindow, None, None);
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    { let _ = w; }
}

#[tauri::command]
pub fn widget_states(app: AppHandle) -> Vec<WidgetState> {
    let state = app.state::<AppState>();
    let pins = state.pins.lock().unwrap();
    WIDGETS.iter().map(|label| {
        let visible = app.get_webview_window(label).and_then(|w| w.is_visible().ok()).unwrap_or(false);
        WidgetState { label: label.to_string(), visible, pin: pins.get(*label).cloned().unwrap_or("bottom".into()) }
    }).collect()
}

#[tauri::command]
pub fn toggle_widget(app: AppHandle, label: String) -> Result<bool, String> {
    let w = app.get_webview_window(&label).ok_or("unknown window")?;
    let visible = w.is_visible().map_err(|e| e.to_string())?;
    if visible { w.hide().map_err(|e| e.to_string())?; }
    else { w.show().map_err(|e| e.to_string())?; let _ = w.set_focus(); }
    save_geom(&app, &label);
    events::widgets(&app);
    Ok(!visible)
}

#[tauri::command]
pub fn hide_widget(app: AppHandle, label: String) -> Result<(), String> {
    if let Some(w) = app.get_webview_window(&label) { w.hide().map_err(|e| e.to_string())?; }
    save_geom(&app, &label);
    events::widgets(&app);
    Ok(())
}

#[tauri::command]
pub fn set_widget_pin(app: AppHandle, label: String, mode: String) -> Result<(), String> {
    let w = app.get_webview_window(&label).ok_or("unknown window")?;
    apply_pin(&w, &mode);
    app.state::<AppState>().pins.lock().unwrap().insert(label.clone(), mode);
    save_geom(&app, &label);
    events::widgets(&app);
    Ok(())
}

#[tauri::command]
pub fn focus_main(app: AppHandle) -> Result<(), String> {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show(); let _ = w.unminimize(); let _ = w.set_focus();
    }
    Ok(())
}