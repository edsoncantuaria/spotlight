use tauri::{AppHandle, Emitter, Manager, State, WebviewWindow};

use crate::apps::index::AppIndex;
use crate::browser::BrowserIndex;
use crate::clipboard::{self, ClipboardDb, ClipboardItem};
use crate::contacts::ContactsIndex;
use crate::history::HistoryDb;
use crate::extensions::ExtensionRegistry;
use crate::quicklinks::QuicklinksIndex;
use crate::scripts::ScriptsIndex;
use crate::search::router::{self, PreviewData};
use crate::search::types::SearchResponse;
use crate::settings::SettingsIndex;
use crate::snippets::SnippetsIndex;
use crate::windows::WindowService;
use crate::window_state;
use std::sync::Mutex;

pub struct SpotlightState {
    pub apps: AppIndex,
    pub settings: SettingsIndex,
    pub history: HistoryDb,
    pub clipboard: ClipboardDb,
    pub browser: BrowserIndex,
    pub contacts: ContactsIndex,
    pub quicklinks: QuicklinksIndex,
    pub snippets: SnippetsIndex,
    pub scripts: ScriptsIndex,
    pub extensions: ExtensionRegistry,
    pub windows: WindowService,
    app_handle: Mutex<Option<AppHandle>>,
}

impl SpotlightState {
    pub fn new() -> Self {
        Self {
            apps: AppIndex::new(),
            settings: SettingsIndex::new(),
            history: HistoryDb::new(),
            clipboard: ClipboardDb::new(),
            browser: BrowserIndex::new(),
            contacts: ContactsIndex::new(),
            quicklinks: QuicklinksIndex::new(),
            snippets: SnippetsIndex::new(),
            scripts: ScriptsIndex::new(),
            extensions: ExtensionRegistry::new(),
            windows: WindowService::new(),
            app_handle: Mutex::new(None),
        }
    }

    pub fn set_app_handle(&self, app: AppHandle) {
        if let Ok(mut guard) = self.app_handle.lock() {
            *guard = Some(app);
        }
    }

    pub fn app_handle(&self) -> Option<AppHandle> {
        self.app_handle.lock().ok().and_then(|g| g.clone())
    }

    pub fn reload_config(&self) {
        let _ = crate::config::reload();
        self.quicklinks.reload();
        self.snippets.reload();
        self.scripts.reload();
        self.extensions.reload();
        self.clipboard.apply_limit();
    }
}

#[tauri::command]
pub fn search(query: String, state: State<'_, SpotlightState>) -> SearchResponse {
    router::search(&state, &query)
}

#[tauri::command]
pub fn open_result(
    id: String,
    query: Option<String>,
    state: State<'_, SpotlightState>,
) -> Result<(), String> {
    router::open_result(&state, &id, query.as_deref())
}

#[tauri::command]
pub fn get_preview(id: String, state: State<'_, SpotlightState>) -> Option<PreviewData> {
    router::get_preview(&state, &id)
}

#[tauri::command]
pub fn run_preview_action(
    id: String,
    action: String,
    query: Option<String>,
    state: State<'_, SpotlightState>,
) -> Result<(), String> {
    router::run_preview_action(&state, &id, &action, query.as_deref())
}

#[tauri::command]
pub fn get_config() -> crate::config::AppConfig {
    crate::config::load()
}

#[tauri::command]
pub fn save_config(
    config: crate::config::AppConfig,
    app: tauri::AppHandle,
    state: State<'_, SpotlightState>,
) -> Result<(), String> {
    crate::config::save(&config)?;
    state.reload_config();
    setup_global_shortcut(&app).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn reload_config(state: State<'_, SpotlightState>) -> Result<crate::config::AppConfig, String> {
    state.reload_config();
    Ok(crate::config::load())
}

#[tauri::command]
pub fn list_extensions(state: State<'_, SpotlightState>) -> Vec<crate::extensions::ExtensionInfo> {
    state.extensions.list()
}

#[tauri::command]
pub fn run_extension(
    extension_id: String,
    action_id: String,
    args: Option<String>,
    state: State<'_, SpotlightState>,
) -> Result<String, String> {
    state.extensions.run_extension(&extension_id, &action_id, args.as_deref().unwrap_or(""))
}
#[tauri::command]
pub fn backup_spotlight() -> Result<String, String> {
    let path = crate::extensions::backup_config()?;
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
pub fn open_settings(state: State<'_, SpotlightState>) -> Result<(), String> {
    let app = state.app_handle().ok_or_else(|| "App não inicializado".to_string())?;
    crate::ui::show_settings_window(&app)
}

#[tauri::command]
pub fn get_clipboard_history(
    limit: Option<usize>,
    filter: Option<String>,
    state: State<'_, SpotlightState>,
) -> Vec<ClipboardItem> {
    let f = parse_clipboard_filter(filter.as_deref());
    state
        .clipboard
        .list_recent(limit.unwrap_or_else(crate::config::clipboard_limit), f)
}

#[tauri::command]
pub fn toggle_clipboard_pin(id: String, state: State<'_, SpotlightState>) -> Result<bool, String> {
    state.clipboard.toggle_pin(&id)
}

#[tauri::command]
pub fn add_clipboard_to_stack(id: String, state: State<'_, SpotlightState>) -> Result<usize, String> {
    state.clipboard.add_to_paste_stack(&id)
}

#[tauri::command]
pub fn paste_clipboard_stack(state: State<'_, SpotlightState>) -> Result<(), String> {
    state.clipboard.paste_stack()
}

#[tauri::command]
pub fn get_clipboard_stack_count(state: State<'_, SpotlightState>) -> usize {
    state.clipboard.paste_stack_count()
}

#[tauri::command]
pub fn clear_clipboard_stack(state: State<'_, SpotlightState>) {
    state.clipboard.clear_paste_stack();
}

#[tauri::command]
pub fn list_store_extensions() -> Result<Vec<crate::extension_store::StoreExtension>, String> {
    crate::extension_store::list_store()
}

#[tauri::command]
pub fn install_store_extension(
    id: String,
    state: State<'_, SpotlightState>,
) -> Result<String, String> {
    let items = crate::extension_store::list_store()?;
    let ext = items
        .into_iter()
        .find(|e| e.id == id)
        .ok_or_else(|| "Extensão não encontrada na loja".to_string())?;
    let path = crate::extension_store::install_from_store(&ext)?;
    state.extensions.reload();
    Ok(path)
}

#[tauri::command]
pub fn get_extensions_guide() -> String {
    crate::extension_store::read_guide()
}

fn parse_clipboard_filter(raw: Option<&str>) -> clipboard::ClipboardFilter {
    match raw.unwrap_or("all").to_lowercase().as_str() {
        "text" | "texto" => clipboard::ClipboardFilter::Text,
        "image" | "imagem" | "imagens" => clipboard::ClipboardFilter::Image,
        "pinned" | "fixados" | "fixado" => clipboard::ClipboardFilter::Pinned,
        _ => clipboard::ClipboardFilter::All,
    }
}

#[tauri::command]
pub fn copy_clipboard_item(id: String, state: State<'_, SpotlightState>) -> Result<(), String> {
    clipboard::copy_item_to_clipboard(&state.clipboard, &id)
}

#[tauri::command]
pub fn paste_clipboard_item(id: String, state: State<'_, SpotlightState>) -> Result<(), String> {
    clipboard::paste_item(&state.clipboard, &id)
}

#[tauri::command]
pub fn hide_window(window: WebviewWindow) -> Result<(), String> {
    hide_window_notify(&window);
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

pub fn hide_window_notify(window: &WebviewWindow) {
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
    linux_activate_window(window, true);
    let _ = window.set_focus();
    let _ = window.as_ref().set_focus();
    let _ = window.eval(focus_js);
}

fn present_window(window: &WebviewWindow, focus_js: &str) {
    let _ = window.set_focusable(true);
    let _ = window.set_always_on_top(true);

    #[cfg(target_os = "linux")]
    {
        if !window.is_visible().unwrap_or(false) {
            let _ = window.show();
        }
    }
    #[cfg(not(target_os = "linux"))]
    {
        let was_visible = window.is_visible().unwrap_or(false);
        if was_visible {
            let _ = window.hide();
            let _ = window.show();
        } else {
            let _ = window.show();
        }
    }

    focus_window_and_webview(window, focus_js);
    linux_schedule_focus(window, focus_js);
}

#[cfg(target_os = "linux")]
fn linux_activate_window(window: &WebviewWindow, use_wmctrl: bool) {
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

    if use_wmctrl {
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
}

#[cfg(not(target_os = "linux"))]
fn linux_activate_window(_window: &WebviewWindow, _use_wmctrl: bool) {}

#[cfg(target_os = "linux")]
fn linux_schedule_focus(window: &WebviewWindow, focus_js: &str) {
    use gtk::glib;

    for delay_ms in [50u32, 200, 450] {
        let window = window.clone();
        let js = focus_js.to_string();
        let use_wmctrl = delay_ms >= 450;
        glib::timeout_add_local_once(std::time::Duration::from_millis(delay_ms as u64), move || {
            linux_activate_window(&window, use_wmctrl);
            let _ = window.set_focus();
            let _ = window.as_ref().set_focus();
            let _ = window.eval(&js);
        });
    }
}

#[cfg(not(target_os = "linux"))]
fn linux_schedule_focus(_window: &WebviewWindow, _focus_js: &str) {}

fn schedule_focus_retries(window: &WebviewWindow, focus_js: &str) {
    let w = window.clone();
    let js = focus_js.to_string();

    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(400));
        let w2 = w.clone();
        let _ = w.run_on_main_thread(move || {
            linux_activate_window(&w2, true);
            let _ = w2.set_focus();
            let _ = w2.as_ref().set_focus();
            let _ = w2.eval(&js);
        });
    });
}

fn show_window_on_main(window: &WebviewWindow) {
    if let Some(clipboard) = window.app_handle().get_webview_window("clipboard") {
        hide_window_notify(&clipboard);
    }

    let _ = window.unminimize();
    window_state::restore_position(window);
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

pub fn toggle_clipboard_window_on_main(window: &WebviewWindow) {
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

    // Limpa atalhos desta instância antes de registrar (útil ao reiniciar em dev).
    let _ = app.global_shortcut().unregister_all();

    let config = crate::config::load();
    let mut any_registered = false;

    for key in &config.shortcuts {
        let Ok(shortcut) = Shortcut::from_str(key) else {
            eprintln!("[spotlight] Atalho inválido: {key}");
            continue;
        };

        let _ = app.global_shortcut().unregister(shortcut);
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
                if key.eq_ignore_ascii_case("Super+Space") {
                    eprintln!(
                        "[spotlight] AVISO: Super+Space conflita com troca de idioma no GNOME \
                         (<Super>space). Use Ctrl+Alt+Space ou altere em Configurações."
                    );
                }
                any_registered = true;
            }
            Err(e) => eprintln!(
                "[spotlight] Falha ao registrar {key}: {e} \
                 (pode haver outra instância do Spotlight em execução)"
            ),
        }
    }

    let clipboard_key = config.clipboard_shortcut.clone();
    if let Ok(shortcut) = Shortcut::from_str(&clipboard_key) {
        let _ = app.global_shortcut().unregister(shortcut);
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
            Err(e) => eprintln!(
                "[spotlight] Falha ao registrar {clipboard_key}: {e} \
                 (pode haver outra instância do Spotlight em execução)"
            ),
        }
    } else {
        eprintln!("[spotlight] Atalho clipboard inválido: {clipboard_key}");
    }

    if !any_registered {
        eprintln!(
            "[spotlight] AVISO: nenhum atalho global foi registrado. \
             Encerre outras instâncias com `pkill -f spotlight` e reinicie, \
             ou use o app sem atalhos por enquanto."
        );
    }

    if let Some(window) = app.get_webview_window("main") {
        window.hide()?;
    }
    if let Some(window) = app.get_webview_window("clipboard") {
        window.hide()?;
    }

    Ok(())
}
