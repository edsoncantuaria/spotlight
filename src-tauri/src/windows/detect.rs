use std::env;
use std::process::Command;

pub fn detect_de() -> String {
    if env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok() {
        return "hyprland".to_string();
    }
    let desktop = env::var("XDG_CURRENT_DESKTOP").unwrap_or_default().to_lowercase();
    if desktop.contains("gnome") {
        return "gnome".to_string();
    }
    if desktop.contains("kde") || desktop.contains("plasma") {
        return "kde".to_string();
    }
    if Command::new("hyprctl").arg("--version").output().map(|o| o.status.success()).unwrap_or(false) {
        return "hyprland".to_string();
    }
    "fallback".to_string()
}
