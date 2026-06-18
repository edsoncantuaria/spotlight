use std::path::Path;
use std::process::Command;

use super::index::AppEntry;

pub fn launch_app(entry: &AppEntry) -> Result<(), String> {
    let desktop_id = Path::new(&entry.desktop_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(&entry.id);

    if Command::new("gtk-launch")
        .arg(desktop_id)
        .spawn()
        .is_ok()
    {
        return Ok(());
    }

    Command::new("sh")
        .arg("-c")
        .arg(&entry.exec)
        .spawn()
        .map_err(|e| format!("Falha ao iniciar aplicativo: {e}"))?;

    Ok(())
}
