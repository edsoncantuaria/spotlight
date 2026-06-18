use tauri::{AppHandle, Emitter, Manager, State, WebviewWindow};

use crate::apps::index::AppIndex;
use crate::clipboard::{self, ClipboardDb, ClipboardItem};
use crate::history::HistoryDb;
use crate::search::router::{self, PreviewData};
use crate::search::types::SearchResponse;
use crate::settings::SettingsIndex;
use crate::window_state;

pub struct SpotlightState {
    pub apps: AppIndex,
    pub settings: SettingsIndex,
    pub history: HistoryDb,
    pub clipboard: ClipboardDb,
}

impl SpotlightState {
    pub fn new() -> Self {
        Self {
            apps: AppIndex::new(),
            settings: SettingsIndex::new(),
            history: HistoryDb::new(),
            clipboard: ClipboardDb::new(),
        }
    }
}

#[tauri::command]
pub fn search(query: String, state: State<'_, SpotlightState>) -> SearchResponse {
    router::search(&state, &query)
}

#[tauri::command]
pub fn open_result(id: String, state: State<'_, SpotlightState>) -> Result<(), String> {
    router::open_result(&state, &id)
}

#[tauri::command]
pub fn get_preview(id: String, state: State<'_, SpotlightState>) -> Option<PreviewData> {
    router::get_preview(&state, &id)
}

#[tauri::command]
pub fn run_preview_action(
    id: String,
    action: String,
    state: State<'_, SpotlightState>,
) -> Result<(), String> {
    router::run_preview_action(&state, &id, &action)
}

#[tauri::command]
pub fn get_clipboard_history(
    limit: Option<usize>,
    state: State<'_, SpotlightState>,
) -> Vec<ClipboardItem> {
    state.clipboard.list_recent(limit.unwrap_or(10))
}

#[tauri::command]
pub fn copy_clipboard_item(id: String, state: State<'_, SpotlightState>) -> Result<(), String> {
    let content = state
        .clipboard
        .get_content(&id)
        .ok_or_else(|| "Item não encontrado".to_string())?;
    clipboard::write_to_clipboard(&content)
}

#[tauri::command]
pub fn hide_window(window: WebviewWindow) -> Result<(), String> {
    hide_window_silent(&window);
    Ok(())
}

#[tauri::command]
pub fn hide_clipboard_window(window: WebviewWindow) -> Result<(), String> {
    hide_window_silent(&window);
    Ok(())
}

fn hide_window_silent(window: &WebviewWindow) {
    if window.label() == "main" {
        let _ = window_state::save_position(window);
    }
    let _ = window.hide();
}

fn hide_window_notify(window: &WebviewWindow) {
    if window.label() == "main" {
        let _ = window_state::save_position(window);
        let _ = window.emit("spotlight-hidden", ());
    } else if window.label() == "clipboard" {
        let _ = window.emit("clipboard-hidden", ());
    }
    let _ = window.hide();
}

#[tauri::command]
pub fn save_window_position(x: i32, y: i32) -> Result<(), String> {
    window_state::WindowState {
        x,
        y,
        has_position: true,
    }
    .save()
}

#[tauri::command]
pub fn resize_window(window: WebviewWindow, width: u32, height: u32) -> Result<(), String> {
    use tauri::{LogicalSize, Size};
    window
        .set_size(Size::Logical(LogicalSize::new(width as f64, height as f64)))
        .map_err(|e| e.to_string())?;
    Ok(())
}

const SPOTLIGHT_FOCUS_JS: &str = r#"(() => {
    const input = document.querySelector(".search-input");
    if (input instanceof HTMLInputElement) {
        input.focus({ preventScroll: true });
        return;
    }
    const root = document.querySelector("[data-focus-root]");
    if (root instanceof HTMLElement) root.focus({ preventScroll: true });
})();"#;

const CLIPBOARD_FOCUS_JS: &str = r#"(() => {
    const root = document.querySelector("[data-focus-root]");
    if (root instanceof HTMLElement) root.focus({ preventScroll: true });
})();"#;

fn focus_window_and_webview(window: &WebviewWindow, focus_js: &str) {
    let _ = window.set_focusable(true);
    let _ = window.set_always_on_top(true);
    linux_activate_window(window);
    let _ = window.set_focus();
    let _ = window.as_ref().set_focus();
    let _ = window.eval(focus_js);
}

fn present_window(window: &WebviewWindow, focus_js: &str) {
    let _ = window.set_focusable(true);
    let _ = window.set_always_on_top(true);

    let was_visible = window.is_visible().unwrap_or(false);
    if was_visible {
        // GNOME/Linux: hide+show só quando já visível força o WM a reativar
        let _ = window.hide();
        let _ = window.show();
    } else {
        let _ = window.show();
    }

    focus_window_and_webview(window, focus_js);
    linux_schedule_focus(window, focus_js);
}

#[cfg(target_os = "linux")]
fn linux_activate_window(window: &WebviewWindow) {
    use gtk::prelude::*;

    if let Ok(gtk_win) = window.gtk_window() {
        gtk_win.present();
    }

    let _ = window.with_webview(|wv| {
        use gtk::prelude::*;
        let webview = wv.inner();
        webview.grab_focus();
        if let Some(gdk_window) = webview.window() {
            gdk_window.focus(gtk::gdk::ffi::GDK_CURRENT_TIME as u32);
        }
    });

    // Fallback opcional se wmctrl estiver instalado
    if let Ok(title) = window.title() {
        if !title.is_empty() {
            let _ = std::process::Command::new("wmctrl")
                .args(["-a", &title])
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status();
        }
    }
}

#[cfg(not(target_os = "linux"))]
fn linux_activate_window(_window: &WebviewWindow) {}

#[cfg(target_os = "linux")]
fn linux_schedule_focus(window: &WebviewWindow, focus_js: &str) {
    use gtk::glib;

    for delay_ms in [16u32, 50, 150, 300] {
        let window = window.clone();
        let js = focus_js.to_string();
        glib::timeout_add_local_once(std::time::Duration::from_millis(delay_ms as u64), move || {
            linux_activate_window(&window);
            let _ = window.set_focus();
            let _ = window.as_ref().set_focus();
            let _ = window.eval(&js);
        });
    }
}

#[cfg(not(target_os = "linux"))]
fn linux_schedule_focus(_window: &WebviewWindow, _focus_js: &str) {}

fn schedule_focus_retries(window: &WebviewWindow, focus_js: &str) {
    let app = window.app_handle().clone();
    let window = window.clone();
    let js = focus_js.to_string();

    std::thread::spawn(move || {
        for delay in [50u64, 150, 300, 600, 1000] {
            std::thread::sleep(std::time::Duration::from_millis(delay));
            let app = app.clone();
            let window = window.clone();
            let js = js.clone();
            let _ = app.run_on_main_thread(move || {
                linux_activate_window(&window);
                let _ = window.set_focus();
                let _ = window.as_ref().set_focus();
                let _ = window.eval(&js);
            });
        }
    });
}

fn show_window_on_main(window: &WebviewWindow) {
    if let Some(clipboard) = window.app_handle().get_webview_window("clipboard") {
        hide_window_notify(&clipboard);
    }

    let _ = window.unminimize();
    window_state::restore_position(window);
    // Evento antes de show/focus para o frontend ignorar blur durante abertura
    let _ = window.emit("spotlight-shown", ());
    present_window(window, SPOTLIGHT_FOCUS_JS);
    schedule_focus_retries(window, SPOTLIGHT_FOCUS_JS);
}

fn show_clipboard_window_on_main(window: &WebviewWindow) {
    if let Some(main) = window.app_handle().get_webview_window("main") {
        hide_window_notify(&main);
    }

    let _ = window.unminimize();
    let _ = window.center();
    let _ = window.emit("clipboard-shown", ());
    present_window(window, CLIPBOARD_FOCUS_JS);
    schedule_focus_retries(window, CLIPBOARD_FOCUS_JS);
}

fn toggle_window_on_main(window: &WebviewWindow) {
    if window.is_visible().unwrap_or(false) {
        hide_window_notify(window);
    } else {
        show_window_on_main(window);
    }
}

fn toggle_clipboard_window_on_main(window: &WebviewWindow) {
    if window.is_visible().unwrap_or(false) {
        hide_window_notify(window);
    } else {
        show_clipboard_window_on_main(window);
    }
}

pub fn show_window(window: &WebviewWindow) {
    let w = window.clone();
    let w2 = w.clone();
    let _ = w.run_on_main_thread(move || show_window_on_main(&w2));
}

pub fn show_clipboard_window(window: &WebviewWindow) {
    let w = window.clone();
    let w2 = w.clone();
    let _ = w.run_on_main_thread(move || show_clipboard_window_on_main(&w2));
}

pub fn toggle_window(window: &WebviewWindow) {
    let w = window.clone();
    let w2 = w.clone();
    let _ = w.run_on_main_thread(move || toggle_window_on_main(&w2));
}

pub fn toggle_clipboard_window(window: &WebviewWindow) {
    let w = window.clone();
    let w2 = w.clone();
    let _ = w.run_on_main_thread(move || toggle_clipboard_window_on_main(&w2));
}

pub fn setup_global_shortcut(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    use std::str::FromStr;
    use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

    let config = crate::config::load();
    let mut any_registered = false;

    for key in &config.shortcuts {
        let Ok(shortcut) = Shortcut::from_str(key) else {
            eprintln!("[spotlight] Atalho inválido: {key}");
            continue;
        };

        match app.global_shortcut().on_shortcut(shortcut, {
            let app = app.clone();
            move |_app, shortcut, event| {
                if event.state == ShortcutState::Pressed {
                    eprintln!(
                        "[spotlight] Atalho acionado: {}",
                        shortcut.into_string()
                    );
                    let handle = app.clone();
                    let handle2 = handle.clone();
                    let _ = handle.run_on_main_thread(move || {
                        if let Some(window) = handle2.get_webview_window("main") {
                            toggle_window_on_main(&window);
                        }
                    });
                }
            }
        }) {
            Ok(()) => {
                eprintln!("[spotlight] Atalho registrado: {key}");
                any_registered = true;
            }
            Err(e) => eprintln!("[spotlight] Falha ao registrar {key}: {e}"),
        }
    }

    let clipboard_key = config.clipboard_shortcut.clone();
    if let Ok(shortcut) = Shortcut::from_str(&clipboard_key) {
        match app.global_shortcut().on_shortcut(shortcut, {
            let app = app.clone();
            move |_app, shortcut, event| {
                if event.state == ShortcutState::Pressed {
                    eprintln!(
                        "[spotlight] Clipboard acionado: {}",
                        shortcut.into_string()
                    );
                    let handle = app.clone();
                    let handle2 = handle.clone();
                    let _ = handle.run_on_main_thread(move || {
                        if let Some(window) = handle2.get_webview_window("clipboard") {
                            toggle_clipboard_window_on_main(&window);
                        }
                    });
                }
            }
        }) {
            Ok(()) => {
                eprintln!("[spotlight] Atalho clipboard registrado: {clipboard_key}");
                any_registered = true;
            }
            Err(e) => eprintln!("[spotlight] Falha ao registrar {clipboard_key}: {e}"),
        }
    } else {
        eprintln!("[spotlight] Atalho clipboard inválido: {clipboard_key}");
    }

    if !any_registered {
        return Err("Nenhum atalho global pôde ser registrado".into());
    }

    if let Some(window) = app.get_webview_window("main") {
        window.hide()?;
    }
    if let Some(window) = app.get_webview_window("clipboard") {
        window.hide()?;
    }

    Ok(())
}
