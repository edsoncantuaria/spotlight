use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct WindowState {
    pub x: i32,
    pub y: i32,
    pub has_position: bool,
}

impl WindowState {
    fn path() -> Option<PathBuf> {
        dirs::config_dir().map(|d| d.join("spotlight").join("window.json"))
    }

    pub fn load() -> Self {
        Self::path()
            .and_then(|p| fs::read_to_string(p).ok())
            .and_then(|c| serde_json::from_str(&c).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::path().ok_or_else(|| "Config dir not found".to_string())?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        fs::write(path, serde_json::to_string(self).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())
    }
}

pub fn save_position(window: &tauri::WebviewWindow) -> Result<(), String> {
    let pos = window.outer_position().map_err(|e| e.to_string())?;
    WindowState {
        x: pos.x,
        y: pos.y,
        has_position: true,
    }
    .save()
}

pub fn restore_position(window: &tauri::WebviewWindow) {
    let state = WindowState::load();
    if state.has_position {
        let _ = window.set_position(tauri::PhysicalPosition::new(state.x, state.y));
    } else {
        let _ = window.center();
    }
}
