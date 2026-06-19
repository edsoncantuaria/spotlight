use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager,
};

pub fn setup_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let open_i = MenuItem::with_id(app, "tray-open", "Abrir Spotlight", true, None::<&str>)?;
    let clip_i = MenuItem::with_id(app, "tray-clipboard", "Área de transferência", true, None::<&str>)?;
    let sep1 = PredefinedMenuItem::separator(app)?;
    let settings_i = MenuItem::with_id(app, "tray-settings", "Configurações", true, None::<&str>)?;
    let extensions_i = MenuItem::with_id(app, "tray-extensions", "Extensões instaladas", true, None::<&str>)?;
    let store_i = MenuItem::with_id(app, "tray-store", "Loja de extensões", true, None::<&str>)?;
    let guide_i = MenuItem::with_id(app, "tray-guide", "Como criar extensões", true, None::<&str>)?;
    let sep2 = PredefinedMenuItem::separator(app)?;
    let quit_i = MenuItem::with_id(app, "tray-quit", "Sair", true, None::<&str>)?;

    let menu = Menu::with_items(
        app,
        &[
            &open_i,
            &clip_i,
            &sep1,
            &settings_i,
            &extensions_i,
            &store_i,
            &guide_i,
            &sep2,
            &quit_i,
        ],
    )?;

    let icon = app
        .default_window_icon()
        .cloned()
        .ok_or("Ícone padrão não encontrado")?;

    let _tray = TrayIconBuilder::with_id("spotlight-tray")
        .icon(icon)
        .tooltip("Spotlight")
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| {
            handle_tray_menu(app, event.id.as_ref());
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                crate::ui::toggle_main_window(app);
            }
        })
        .build(app)?;

    Ok(())
}

fn handle_tray_menu(app: &AppHandle, id: &str) {
    match id {
        "tray-open" => {
            let _ = crate::ui::show_main_window(app);
        }
        "tray-clipboard" => {
            if let Some(w) = app.get_webview_window("clipboard") {
                crate::commands::toggle_clipboard_window_on_main(&w);
            }
        }
        "tray-settings" => {
            let _ = crate::ui::show_settings_window(app);
        }
        "tray-extensions" => {
            let _ = crate::ui::show_extensions_window(app);
        }
        "tray-store" => {
            let _ = crate::ui::show_store_window(app);
        }
        "tray-guide" => {
            let _ = crate::ui::show_guide_window(app);
        }
        "tray-quit" => {
            app.exit(0);
        }
        _ => {}
    }
}
