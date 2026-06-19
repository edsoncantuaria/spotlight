use tauri::{AppHandle, Manager, WebviewWindow};

pub fn show_main_window(app: &AppHandle) -> Result<(), String> {
    let main = app
        .get_webview_window("main")
        .ok_or_else(|| "Janela principal não encontrada".to_string())?;
    crate::commands::show_window(&main);
    Ok(())
}

pub fn toggle_main_window(app: &AppHandle) {
    if let Some(main) = app.get_webview_window("main") {
        if main.is_visible().unwrap_or(false) {
            crate::commands::hide_window_notify(&main);
        } else {
            let _ = show_main_window(app);
        }
    }
}

pub fn show_settings_window(app: &AppHandle) -> Result<(), String> {
    open_or_focus(app, "settings", "Spotlight — Configurações", 780.0, 640.0)
}

pub fn show_store_window(app: &AppHandle) -> Result<(), String> {
    open_or_focus(app, "store", "Spotlight — Loja de Extensões", 760.0, 560.0)
}

pub fn show_extensions_window(app: &AppHandle) -> Result<(), String> {
    open_or_focus(app, "extensions", "Spotlight — Extensões", 820.0, 580.0)
}

pub fn show_guide_window(app: &AppHandle) -> Result<(), String> {
    open_or_focus(app, "guide", "Spotlight — Guia de Extensões", 720.0, 620.0)
}

fn open_or_focus(
    app: &AppHandle,
    label: &str,
    title: &str,
    width: f64,
    height: f64,
) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(label) {
        let _ = window.show();
        let _ = window.set_focus();
        let _ = window.unminimize();
        return Ok(());
    }

    let window = tauri::WebviewWindowBuilder::new(
        app,
        label,
        tauri::WebviewUrl::App("index.html".into()),
    )
    .title(title)
    .inner_size(width, height)
    .center()
    .build()
    .map_err(|e| e.to_string())?;

    let _ = window.show();
    let _ = window.set_focus();
    Ok(())
}

pub fn show_main_with_query(app: &AppHandle, query: &str) -> Result<(), String> {
    let main = app
        .get_webview_window("main")
        .ok_or_else(|| "Janela principal não encontrada".to_string())?;
    crate::commands::show_window(&main);
    let js = format!(
        r#"window.dispatchEvent(new CustomEvent("spotlight-set-query", {{ detail: {:?} }}));"#,
        query
    );
    let _ = main.eval(&js);
    Ok(())
}

pub fn hide_window_notify(window: &WebviewWindow) {
    crate::commands::hide_window_notify(window);
}
