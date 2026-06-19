use std::process::Command;

use super::{WindowInfo, WindowManager};

pub struct WmctrlManager;

impl WindowManager for WmctrlManager {
    fn list_windows(&self) -> Vec<WindowInfo> {
        let output = Command::new("wmctrl").args(["-l"]).output().ok();
        let Some(output) = output else {
            return Vec::new();
        };
        if !output.status.success() {
            return Vec::new();
        }
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter_map(parse_wmctrl_line)
            .collect()
    }

    fn focus(&self, id: &str) -> Result<(), String> {
        Command::new("wmctrl")
            .args(["-ia", id])
            .status()
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn move_active(&self, direction: &str) -> Result<(), String> {
        let geo = match direction {
            "left" => "0,0,50%,100%",
            "right" => "50%,0,50%,100%",
            "up" => "0,0,100%,50%",
            "down" => "0,50%,100%,50%",
            _ => return Err("Direção inválida".to_string()),
        };
        Command::new("wmctrl")
            .args(["-r", ":ACTIVE:", "-e", &format!("0,{geo}")])
            .status()
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn toggle_fullscreen(&self) -> Result<(), String> {
        Command::new("wmctrl")
            .args(["-r", ":ACTIVE:", "-b", "toggle,fullscreen"])
            .status()
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn close_active(&self) -> Result<(), String> {
        Command::new("wmctrl")
            .args(["-c", ":ACTIVE:"])
            .status()
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn minimize_active(&self) -> Result<(), String> {
        Command::new("wmctrl")
            .args(["-r", ":ACTIVE:", "-b", "add,hidden"])
            .status()
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}

fn parse_wmctrl_line(line: &str) -> Option<WindowInfo> {
    let mut parts = line.split_whitespace();
    let id = parts.next()?.to_string();
    let _desktop = parts.next()?;
    let _host = parts.next()?;
    let title = parts.collect::<Vec<_>>().join(" ");
    Some(WindowInfo {
        id,
        title,
        app: None,
    })
}
