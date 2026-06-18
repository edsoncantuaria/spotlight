mod apps;
mod clipboard;
mod commands;
mod config;
mod file_index;
mod files;
mod history;
mod quick_answers;
mod search;
mod settings;
mod window_state;

pub use commands::SpotlightState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state = SpotlightState::new();
    let clipboard_db = state.clipboard.clone_for_watcher();

    let _ = config::ensure_autostart();
    file_index::start_file_watcher();
    clipboard::start_watcher(clipboard_db);

    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .manage(state)
        .setup(|app| {
            #[cfg(desktop)]
            commands::setup_global_shortcut(app.handle())?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::search,
            commands::open_result,
            commands::get_preview,
            commands::run_preview_action,
            commands::get_clipboard_history,
            commands::copy_clipboard_item,
            commands::hide_window,
            commands::hide_clipboard_window,
            commands::resize_window,
            commands::save_window_position,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
