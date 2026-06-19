use std::env;
use std::process::Command;

use tauri::{AppHandle, Manager};

use crate::config::AppConfig;
use crate::ui;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DesktopKind {
    Gnome,
    Kde,
    Other,
}

pub fn log_environment() {
    let desktop = env::var("XDG_CURRENT_DESKTOP").unwrap_or_else(|_| "?".into());
    let session = env::var("XDG_SESSION_TYPE").unwrap_or_else(|_| "?".into());
    let kind = detect_kind();
    eprintln!(
        "[spotlight] Ambiente: XDG_CURRENT_DESKTOP={desktop} XDG_SESSION_TYPE={session} ({kind:?})"
    );
}

pub fn detect_kind() -> DesktopKind {
    let desktop = env::var("XDG_CURRENT_DESKTOP")
        .unwrap_or_default()
        .to_lowercase();
    if desktop.contains("gnome") {
        DesktopKind::Gnome
    } else if desktop.contains("kde") || desktop.contains("plasma") {
        DesktopKind::Kde
    } else {
        DesktopKind::Other
    }
}

pub fn is_wayland() -> bool {
    env::var("XDG_SESSION_TYPE")
        .map(|s| s.eq_ignore_ascii_case("wayland"))
        .unwrap_or(false)
}

pub fn setup(app: &AppHandle) {
    let config = crate::config::load();
    if detect_kind() == DesktopKind::Gnome {
        if let Err(e) = sync_gnome_keybindings(app, &config) {
            eprintln!("[spotlight] GNOME keybindings: {e}");
        }
    } else if detect_kind() == DesktopKind::Kde {
        eprintln!(
            "[spotlight] KDE: use atalhos globais do Spotlight ou configure em \
             Configurações → Atalhos → Personalizado"
        );
    }
}

pub fn handle_cli_args(app: &AppHandle, args: &[String]) {
    for arg in args {
        match arg.as_str() {
            "--toggle" | "--toggle-spotlight" => {
                ui::toggle_main_window(app);
                return;
            }
            "--toggle-clipboard" => {
                if let Some(w) = app.get_webview_window("clipboard") {
                    crate::commands::toggle_clipboard_window_on_main(&w);
                }
                return;
            }
            "--show" => {
                let _ = ui::show_main_window(app);
                return;
            }
            _ => {}
        }
    }
    ui::toggle_main_window(app);
}

pub fn sync_gnome_keybindings(app: &AppHandle, config: &AppConfig) -> Result<(), String> {
    if !Command::new("gsettings")
        .arg("--version")
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
    {
        return Ok(());
    }

    let exe = env::current_exe().map_err(|e| e.to_string())?;
    let exe_str = exe.to_string_lossy();

    for (idx, key) in config.shortcuts.iter().enumerate() {
        let name = if idx == 0 {
            "spotlight-open".to_string()
        } else {
            format!("spotlight-open-{idx}")
        };
        register_gnome_custom_key(
            &name,
            &to_gnome_binding(key),
            &format!("{exe_str} --toggle"),
        )?;
    }

    register_gnome_custom_key(
        "spotlight-clipboard",
        &to_gnome_binding(&config.clipboard_shortcut),
        &format!("{exe_str} --toggle-clipboard"),
    )?;

    eprintln!(
        "[spotlight] Atalhos GNOME registrados via gsettings (fallback para Wayland/X11)"
    );
    let _ = app;
    Ok(())
}

fn register_gnome_custom_key(name: &str, binding: &str, command: &str) -> Result<(), String> {
    let base = format!(
        "/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/{name}/"
    );
    let schema = format!(
        "org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:{base}"
    );

    ensure_custom_key_path(&base)?;

    gsettings_set(&schema, "name", name)?;
    gsettings_set(&schema, "binding", binding)?;
    gsettings_set(&schema, "command", command)?;
    Ok(())
}

fn ensure_custom_key_path(path: &str) -> Result<(), String> {
    let output = Command::new("gsettings")
        .args([
            "get",
            "org.gnome.settings-daemon.plugins.media-keys",
            "custom-keybindings",
        ])
        .output()
        .map_err(|e| e.to_string())?;

    let mut paths = parse_gsettings_array(&String::from_utf8_lossy(&output.stdout));
    if paths.iter().any(|p| p == path) {
        return Ok(());
    }
    paths.push(path.to_string());

    let formatted = format!(
        "[{}]",
        paths
            .iter()
            .map(|p| format!("'{p}'"))
            .collect::<Vec<_>>()
            .join(", ")
    );

    gsettings_set(
        "org.gnome.settings-daemon.plugins.media-keys",
        "custom-keybindings",
        &formatted,
    )
}

fn parse_gsettings_array(raw: &str) -> Vec<String> {
    raw.trim()
        .trim_start_matches("@as ")
        .trim()
        .trim_start_matches('[')
        .trim_end_matches(']')
        .split(',')
        .map(|s| s.trim().trim_matches('\''))
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect()
}

fn gsettings_set(schema: &str, key: &str, value: &str) -> Result<(), String> {
    let status = Command::new("gsettings")
        .args(["set", schema, key, value])
        .status()
        .map_err(|e| e.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("gsettings set {schema} {key} falhou"))
    }
}

pub fn to_gnome_binding(shortcut: &str) -> String {
    let mut parts: Vec<String> = Vec::new();
    for token in shortcut.split('+') {
        let t = token.trim();
        match t.to_ascii_lowercase().as_str() {
            "ctrl" | "control" => parts.push("<Ctrl>".to_string()),
            "alt" => parts.push("<Alt>".to_string()),
            "shift" => parts.push("<Shift>".to_string()),
            "super" | "meta" | "win" => parts.push("<Super>".to_string()),
            "space" => parts.push("space".to_string()),
            other if other.len() == 1 => parts.push(other.to_string()),
            other => parts.push(other.to_string()),
        }
    }
    parts.join("")
}

pub fn use_wmctrl() -> bool {
    !is_wayland() && Command::new("wmctrl")
        .arg("--version")
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

pub fn check_clipboard_tools() {
    let session = env::var("XDG_SESSION_TYPE").unwrap_or_default();
    let has_xclip = Command::new("xclip")
        .arg("-version")
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    let has_wl = Command::new("wl-copy")
        .arg("--version")
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if session.eq_ignore_ascii_case("wayland") && !has_wl {
        eprintln!(
            "[spotlight] AVISO: instale wl-clipboard para clipboard no Wayland (sudo apt install wl-clipboard)"
        );
    } else if !has_xclip && !has_wl {
        eprintln!(
            "[spotlight] AVISO: instale xclip ou wl-clipboard para o histórico de clipboard"
        );
    }
}

#[allow(dead_code)]
pub fn exe_path() -> Option<std::path::PathBuf> {
    env::current_exe().ok()
}
