// src-tauri/src/lib.rs
mod commands;
mod events;
mod state;
mod windows;

use state::AppState;
use std::time::Duration;
use tauri::Manager;

#[cfg(desktop)]
fn setup_tray(app: &tauri::AppHandle) -> tauri::Result<()> {
    use tauri::menu::{Menu, MenuItem};
    use tauri::tray::TrayIconBuilder;

    let open = MenuItem::with_id(app, "open", "Open Tempo", true, None::<&str>)?;
    let week = MenuItem::with_id(app, "week", "Toggle Week Widget", true, None::<&str>)?;
    let next7 = MenuItem::with_id(app, "next7", "Toggle Next 7 Days Widget", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit Tempo", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open, &week, &next7, &quit])?;

    let mut builder = TrayIconBuilder::new().menu(&menu).on_menu_event(|app, e| match e.id.as_ref() {
        "open" => { let _ = windows::focus_main(app.clone()); }
        "week" => { let _ = windows::toggle_widget(app.clone(), "week".into()); }
        "next7" => { let _ = windows::toggle_widget(app.clone(), "next7".into()); }
        "quit" => app.exit(0),
        _ => {}
    });
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    builder.build(app)?;
    Ok(())
}

fn spawn_scheduler(app: tauri::AppHandle) {
    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_secs(15));
        let due = {
            let state = app.state::<AppState>();
            let Ok(db) = state.db.lock() else { continue };
            tempo_core::repo::pop_due_reminders(&db, &tempo_core::now()).unwrap_or_default()
        };
        for rem in due {
            events::reminder(&app, &rem);
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&dir)?;
            let conn = tempo_core::db::open(&dir.join("tempo.db"))?;
            app.manage(AppState::new(conn));
            let handle = app.handle().clone();
            windows::setup(&handle);
            #[cfg(desktop)]
            let _ = setup_tray(&handle);
            spawn_scheduler(handle);
            Ok(())
        })
        .on_window_event(|window, event| match event {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                api.prevent_close();
                let _ = window.hide();
                windows::save_geom(window.app_handle(), window.label());
                events::widgets(window.app_handle());
            }
            tauri::WindowEvent::Moved(_) | tauri::WindowEvent::Resized(_) => {
                windows::save_geom(window.app_handle(), window.label());
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_bootstrap,
            commands::get_calendar_range,
            commands::save_task,
            commands::toggle_task,
            commands::delete_task,
            commands::reorder_tasks,
            commands::bulk_complete,
            commands::restore_entities,
            commands::log_pomodoro,
            commands::save_list,
            commands::delete_list,
            commands::save_note,
            commands::delete_note,
            commands::save_habit,
            commands::delete_habit,
            commands::toggle_habit,
            windows::widget_states,
            windows::toggle_widget,
            windows::hide_widget,
            windows::set_widget_pin,
            windows::focus_main,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Tempo");
}