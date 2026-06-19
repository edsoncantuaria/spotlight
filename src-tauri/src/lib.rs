mod apps;
mod browser;
mod clipboard;
mod commands;
mod config;
mod contacts;
mod file_index;
mod files;
mod history;
mod paths;
mod quick_answers;
mod quicklinks;
mod scripts;
mod search;
mod settings;
mod snippets;
mod ui;
mod web_search;
mod extensions;
mod window_state;
mod windows;
mod input;
mod system_commands;
mod extension_store;
mod tray;
mod platform;

use tauri::Manager;

pub use commands::SpotlightState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state = SpotlightState::new();
    let clipboard_db = state.clipboard.clone_for_watcher();

    platform::log_startup_environment();
    platform::desktop::check_clipboard_tools();

    let _ = config::sync_autostart();
    file_index::start_file_watcher();
    clipboard::start_watcher(clipboard_db);
    std::thread::spawn(|| crate::quick_answers::warm_cache());

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            platform::handle_second_instance(app, &args);
        }))
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .manage(state)
        .setup(|app| {
            let state = app.state::<SpotlightState>();
            state.set_app_handle(app.handle().clone());
            #[cfg(desktop)]
            {
                commands::setup_global_shortcut(app.handle())?;
                platform::setup_desktop_integration(app.handle());
                tray::setup_tray(app.handle())?;
                for label in ["main", "clipboard"] {
                    if let Some(w) = app.get_webview_window(label) {
                        let _ = w.hide();
                    }
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::search,
            commands::open_result,
            commands::get_preview,
            commands::run_preview_action,
            commands::get_clipboard_history,
            commands::toggle_clipboard_pin,
            commands::add_clipboard_to_stack,
            commands::paste_clipboard_stack,
            commands::get_clipboard_stack_count,
            commands::clear_clipboard_stack,
            commands::copy_clipboard_item,
            commands::paste_clipboard_item,
            commands::list_store_extensions,
            commands::install_store_extension,
            commands::get_extensions_guide,
            commands::get_config,
            commands::save_config,
            commands::reload_config,
            commands::list_extensions,
            commands::run_extension,
            commands::backup_spotlight,
            commands::validate_shortcut,
            commands::validate_shortcuts,
            commands::open_settings,
            commands::open_store,
            commands::open_extensions,
            commands::set_extension_enabled,
            commands::hide_window,
            commands::hide_clipboard_window,
            commands::present_main,
            commands::present_clipboard,
            commands::resize_window,
            commands::save_window_position,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
