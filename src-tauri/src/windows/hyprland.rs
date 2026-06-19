use std::process::Command;

use super::{WindowInfo, WindowManager};

pub struct HyprlandWindowManager;

impl WindowManager for HyprlandWindowManager {
    fn list_windows(&self) -> Vec<WindowInfo> {
        let output = Command::new("hyprctl")
            .args(["clients", "-j"])
            .output()
            .ok();
        let Some(output) = output else {
            return Vec::new();
        };
        if !output.status.success() {
            return Vec::new();
        }
        let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap_or_default();
        let Some(arr) = json.as_array() else {
            return Vec::new();
        };
        arr.iter()
            .filter_map(|c| {
                let addr = c["address"].as_str()?.to_string();
                let title = c["title"].as_str().unwrap_or("").to_string();
                let app = c["class"].as_str().map(|s| s.to_string());
                Some(WindowInfo { id: addr, title, app })
            })
            .collect()
    }

    fn focus(&self, id: &str) -> Result<(), String> {
        Command::new("hyprctl")
            .args(["dispatch", "focuswindow", &format!("address:{id}")])
            .status()
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn move_active(&self, direction: &str) -> Result<(), String> {
        let cmd = match direction {
            "left" => "movewindow l",
            "right" => "movewindow r",
            "up" => "movewindow u",
            "down" => "movewindow d",
            _ => return Err("Direção inválida".to_string()),
        };
        Command::new("hyprctl")
            .args(["dispatch", cmd])
            .status()
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn toggle_fullscreen(&self) -> Result<(), String> {
        Command::new("hyprctl")
            .args(["dispatch", "fullscreen", "1"])
            .status()
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn close_active(&self) -> Result<(), String> {
        Command::new("hyprctl")
            .args(["dispatch", "closewindow", "active"])
            .status()
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn minimize_active(&self) -> Result<(), String> {
        Command::new("hyprctl")
            .args(["dispatch", "movetoworkspacesilent", "special"])
            .status()
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}
