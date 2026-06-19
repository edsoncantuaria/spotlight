use std::process::Command;
use std::thread;
use std::time::Duration;

/// Cola o conteúdo do clipboard no app focado (Linux).
pub fn simulate_paste() -> Result<(), String> {
    thread::sleep(Duration::from_millis(120));

    #[cfg(target_os = "linux")]
    {
        if try_wtype_paste() || try_xdotool_paste() || try_ydotool_paste() {
            return Ok(());
        }
        return Err(
            "Instale wtype (Wayland) ou xdotool (X11) para colar snippets automaticamente".into(),
        );
    }

    #[cfg(not(target_os = "linux"))]
    {
        let _ = Command::new("true").status();
        Err("Colar automático só disponível no Linux".into())
    }
}

#[cfg(target_os = "linux")]
fn try_wtype_paste() -> bool {
    Command::new("wtype")
        .args(["-M", "ctrl", "-k", "v"])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

#[cfg(not(target_os = "linux"))]
fn try_wtype_paste() -> bool {
    false
}

#[cfg(target_os = "linux")]
fn try_xdotool_paste() -> bool {
    Command::new("xdotool")
        .args(["key", "--clearmodifiers", "ctrl+v"])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

#[cfg(not(target_os = "linux"))]
fn try_xdotool_paste() -> bool {
    false
}

#[cfg(target_os = "linux")]
fn try_ydotool_paste() -> bool {
    Command::new("ydotool")
        .args(["key", "29:1", "47:1", "47:0", "29:0"])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

#[cfg(not(target_os = "linux"))]
fn try_ydotool_paste() -> bool {
    false
}
