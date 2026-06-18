use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub shortcuts: Vec<String>,
    #[serde(default = "default_clipboard_shortcut")]
    pub clipboard_shortcut: String,
}

fn default_clipboard_shortcut() -> String {
    "Ctrl+Alt+C".to_string()
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            shortcuts: vec![
                "Ctrl+Alt+Space".to_string(),
                "Super+Space".to_string(),
            ],
            clipboard_shortcut: default_clipboard_shortcut(),
        }
    }
}

pub fn config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("spotlight").join("config.toml"))
}

pub fn load() -> AppConfig {
    let Some(path) = config_path() else {
        return AppConfig::default();
    };
    fs::read_to_string(path)
        .ok()
        .and_then(|s| toml::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save(config: &AppConfig) -> Result<(), String> {
    let path = config_path().ok_or_else(|| "Config dir not found".to_string())?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let content = toml::to_string_pretty(config).map_err(|e| e.to_string())?;
    fs::write(path, content).map_err(|e| e.to_string())
}

pub fn ensure_autostart() -> Result<(), String> {
    let Some(config_dir) = dirs::config_dir() else {
        return Ok(());
    };
    let autostart_dir = config_dir.join("autostart");
    fs::create_dir_all(&autostart_dir).map_err(|e| e.to_string())?;

    let desktop = autostart_dir.join("spotlight.desktop");
    if desktop.exists() {
        return Ok(());
    }

    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let content = format!(
        "[Desktop Entry]\nType=Application\nName=Spotlight\nExec={}\nHidden=false\nNoDisplay=false\nX-GNOME-Autostart-enabled=true\n",
        exe.display()
    );
    fs::write(desktop, content).map_err(|e| e.to_string())
}
