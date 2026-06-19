use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use crate::paths;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub shortcuts: Vec<String>,
    #[serde(default = "default_clipboard_shortcut")]
    pub clipboard_shortcut: String,
    #[serde(default = "default_web_search_engine")]
    pub web_search_engine: String,
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_clipboard_limit")]
    pub clipboard_limit: usize,
    #[serde(default)]
    pub file_roots: Vec<String>,
    #[serde(default = "default_exclude_patterns")]
    pub exclude_patterns: Vec<String>,
    #[serde(default = "default_max_index_files")]
    pub max_index_files: usize,
    #[serde(default)]
    pub extension_dirs: Vec<String>,
    #[serde(default)]
    pub translate_api_url: Option<String>,
    #[serde(default = "default_translate_target")]
    pub translate_target: String,
    #[serde(default)]
    pub ai_enabled: bool,
    #[serde(default = "default_ai_model")]
    pub ai_model: String,
    #[serde(default)]
    pub ai_ollama_url: Option<String>,
    #[serde(default)]
    pub ai_api_url: Option<String>,
    #[serde(default)]
    pub extension_store_url: Option<String>,
}

fn default_theme() -> String {
    "auto".to_string()
}

fn default_clipboard_limit() -> usize {
    50
}

pub fn clipboard_limit() -> usize {
    load().clipboard_limit.clamp(10, 500)
}

fn default_exclude_patterns() -> Vec<String> {
    vec![
        ".git".to_string(),
        "node_modules".to_string(),
        "target".to_string(),
        ".cache".to_string(),
    ]
}

fn default_max_index_files() -> usize {
    50_000
}

fn default_translate_target() -> String {
    "pt".to_string()
}

fn default_ai_model() -> String {
    "llama3".to_string()
}

fn default_web_search_engine() -> String {
    "google".to_string()
}

fn default_clipboard_shortcut() -> String {
    "Ctrl+Alt+C".to_string()
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            shortcuts: vec!["Ctrl+Alt+Space".to_string()],
            clipboard_shortcut: default_clipboard_shortcut(),
            web_search_engine: default_web_search_engine(),
            theme: default_theme(),
            clipboard_limit: default_clipboard_limit(),
            file_roots: Vec::new(),
            exclude_patterns: default_exclude_patterns(),
            max_index_files: default_max_index_files(),
            extension_dirs: Vec::new(),
            translate_api_url: None,
            translate_target: default_translate_target(),
            ai_enabled: false,
            ai_model: default_ai_model(),
            ai_ollama_url: None,
            ai_api_url: None,
            extension_store_url: None,
        }
    }
}

static CONFIG_CACHE: std::sync::OnceLock<Arc<RwLock<AppConfig>>> = std::sync::OnceLock::new();

fn cache() -> Arc<RwLock<AppConfig>> {
    CONFIG_CACHE
        .get_or_init(|| Arc::new(RwLock::new(AppConfig::load_or_default())))
        .clone()
}

pub fn config_path() -> Option<PathBuf> {
    paths::config_file()
}

pub fn load() -> AppConfig {
    cache().read().map(|c| c.clone()).unwrap_or_default()
}

pub fn reload() -> AppConfig {
    let cfg = AppConfig::load_or_default();
    if let Ok(mut guard) = cache().write() {
        *guard = cfg.clone();
    }
    cfg
}

impl AppConfig {
    pub fn load_or_default() -> Self {
        let Some(path) = config_path() else {
            return AppConfig::default();
        };
        if !path.exists() {
            let default = AppConfig::default();
            let _ = save(&default);
            return default;
        }
        fs::read_to_string(path)
            .ok()
            .and_then(|s| toml::from_str(&s).ok())
            .unwrap_or_default()
    }
}

pub fn save(config: &AppConfig) -> Result<(), String> {
    let path = config_path().ok_or_else(|| "Config dir not found".to_string())?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let content = toml::to_string_pretty(config).map_err(|e| e.to_string())?;
    fs::write(path, content).map_err(|e| e.to_string())?;
    if let Ok(mut guard) = cache().write() {
        *guard = config.clone();
    }
    Ok(())
}

pub fn ensure_autostart() -> Result<(), String> {
    let Some(config_dir) = dirs::config_dir() else {
        return Ok(());
    };
    let autostart_dir = config_dir.join("autostart");
    fs::create_dir_all(&autostart_dir).map_err(|e| e.to_string())?;

    let desktop = autostart_dir.join("spotlight.desktop");
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let content = format!(
        "[Desktop Entry]\nType=Application\nName=Spotlight\nExec={}\nHidden=false\nNoDisplay=false\nTerminal=false\nX-GNOME-Autostart-enabled=true\n",
        exe.display()
    );

    let needs_write = match fs::read_to_string(&desktop) {
        Ok(existing) => existing != content,
        Err(_) => true,
    };
    if needs_write {
        fs::write(desktop, content).map_err(|e| e.to_string())?;
    }
    Ok(())
}
