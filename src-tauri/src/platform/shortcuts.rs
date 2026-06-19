use std::process::Command;
use std::str::FromStr;

use serde::Serialize;
use tauri::AppHandle;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

use super::desktop::{detect_kind, to_gnome_binding, DesktopKind};

#[derive(Debug, Clone, Serialize)]
pub struct ShortcutValidation {
    pub shortcut: String,
    pub valid_format: bool,
    pub registrable: bool,
    pub available: bool,
    pub gnome_conflict: Option<String>,
    pub message: String,
}

pub fn validate_shortcut(app: &AppHandle, shortcut: &str) -> ShortcutValidation {
    let shortcut = shortcut.trim();
    let mut result = ShortcutValidation {
        shortcut: shortcut.to_string(),
        valid_format: false,
        registrable: false,
        available: false,
        gnome_conflict: None,
        message: String::new(),
    };

    if shortcut.is_empty() {
        result.message = "Informe um atalho.".into();
        return result;
    }

    let Ok(parsed) = Shortcut::from_str(shortcut) else {
        result.message =
            "Formato inválido. Use algo como Ctrl+Alt+Space ou Super+Shift+S.".into();
        return result;
    };
    result.valid_format = true;

    if let Some(conflict) = gnome_binding_conflict(shortcut) {
        result.gnome_conflict = Some(conflict.clone());
        result.message = format!("Conflito no GNOME: {conflict}");
    }

    match app.global_shortcut().register(parsed) {
        Ok(()) => {
            let _ = app.global_shortcut().unregister(parsed);
            result.registrable = true;
            if result.message.is_empty() {
                result.message = "Atalho disponível.".into();
            } else {
                result.message = format!(
                    "{} O plugin global conseguiu registrar (fallback GNOME pode ajudar).",
                    result.message
                );
            }
        }
        Err(e) => {
            result.registrable = false;
            if result.gnome_conflict.is_some() {
                result.message = format!(
                    "Não registrado pelo plugin global ({e}). \
                     Remova o conflito em Configurações → Teclado ou use outra combinação."
                );
            } else {
                result.message = format!(
                    "Atalho em uso ou indisponível neste ambiente: {e}. \
                     Tente outra combinação."
                );
            }
        }
    }

    result.available = result.registrable
        || (detect_kind() == DesktopKind::Gnome
            && result.gnome_conflict.is_none()
            && result.valid_format);

    if result.available && result.message.is_empty() {
        result.message = if result.registrable {
            "Atalho disponível.".into()
        } else {
            "Será registrado via GNOME (gsettings).".into()
        };
    } else if !result.available && result.registrable == false && result.gnome_conflict.is_some() {
        // message already set
    }

    result
}

pub fn validate_shortcut_list(app: &AppHandle, shortcuts: &[String]) -> Vec<ShortcutValidation> {
    shortcuts.iter().map(|s| validate_shortcut(app, s)).collect()
}

pub fn shortcuts_overlap(a: &str, b: &str) -> bool {
    normalize_shortcut(a) == normalize_shortcut(b)
}

fn normalize_shortcut(raw: &str) -> String {
    raw.split('+')
        .map(|p| p.trim().to_ascii_lowercase())
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join("+")
}

fn gnome_binding_conflict(shortcut: &str) -> Option<String> {
    if detect_kind() != DesktopKind::Gnome {
        return None;
    }
    if !Command::new("gsettings")
        .arg("--version")
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
    {
        return None;
    }

    let target = to_gnome_binding(shortcut);
    let output = Command::new("gsettings")
        .args([
            "get",
            "org.gnome.settings-daemon.plugins.media-keys",
            "custom-keybindings",
        ])
        .output()
        .ok()?;

    let paths = parse_gsettings_array(&String::from_utf8_lossy(&output.stdout));
    for path in paths {
        let schema = format!(
            "org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:{path}"
        );
        let binding = gsettings_get(&schema, "binding")?;
        if binding.eq_ignore_ascii_case(&target) {
            let name = gsettings_get(&schema, "name").unwrap_or_else(|| "outro atalho".into());
            let cmd = gsettings_get(&schema, "command").unwrap_or_default();
            if cmd.contains("spotlight") {
                continue;
            }
            return Some(format!("'{name}' ({binding})"));
        }
    }
    None
}

fn gsettings_get(schema: &str, key: &str) -> Option<String> {
    let output = Command::new("gsettings")
        .args(["get", schema, key])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let raw = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Some(raw.trim_matches('\'').to_string())
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
