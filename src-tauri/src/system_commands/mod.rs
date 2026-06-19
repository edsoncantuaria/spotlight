use crate::history::HistoryDb;
use crate::search::ranking::build_result;
use crate::search::types::{make_id, ResultKind, SearchResult};
use std::process::Command;

pub struct SystemCommand {
    pub id: &'static str,
    pub title: &'static str,
    pub subtitle: &'static str,
    pub icon: &'static str,
}

const COMMANDS: &[SystemCommand] = &[
    SystemCommand {
        id: "lock",
        title: "Bloquear tela",
        subtitle: "Trava a sessão",
        icon: "system-lock-screen",
    },
    SystemCommand {
        id: "suspend",
        title: "Suspender",
        subtitle: "Modo de suspensão",
        icon: "system-suspend",
    },
    SystemCommand {
        id: "logout",
        title: "Encerrar sessão",
        subtitle: "Logout do usuário",
        icon: "system-log-out",
    },
    SystemCommand {
        id: "reboot",
        title: "Reiniciar",
        subtitle: "Reinicia o sistema",
        icon: "system-reboot",
    },
    SystemCommand {
        id: "shutdown",
        title: "Desligar",
        subtitle: "Desliga o computador",
        icon: "system-shutdown",
    },
    SystemCommand {
        id: "settings",
        title: "Configurações do Spotlight",
        subtitle: "Abrir painel de configuração",
        icon: "preferences-system",
    },
    SystemCommand {
        id: "screenshot",
        title: "Captura de tela",
        subtitle: "Flameshot ou grim",
        icon: "camera-photo",
    },
    SystemCommand {
        id: "empty_trash",
        title: "Esvaziar lixeira",
        subtitle: "Remove arquivos da lixeira",
        icon: "user-trash",
    },
];

pub fn search(query: &str, history: &HistoryDb, limit: usize) -> Vec<SearchResult> {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return Vec::new();
    }

    COMMANDS
        .iter()
        .filter(|c| {
            c.title.to_lowercase().contains(&q)
                || c.subtitle.to_lowercase().contains(&q)
                || c.id.contains(&q)
        })
        .take(limit)
        .map(|c| {
            build_result(
                make_id(ResultKind::Setting, &format!("syscmd:{}", c.id)),
                ResultKind::Setting,
                c.title.to_string(),
                Some(c.subtitle.to_string()),
                Some(c.icon.to_string()),
                950,
                query,
                history,
            )
        })
        .collect()
}

pub fn run(command_id: &str) -> Result<(), String> {
    match command_id {
        "lock" => run_lock(),
        "suspend" => run_systemctl("suspend"),
        "logout" => run_logout(),
        "reboot" => run_systemctl("reboot"),
        "shutdown" => run_systemctl("poweroff"),
        "settings" => Ok(()),
        "screenshot" => run_screenshot(),
        "empty_trash" => run_empty_trash(),
        other => Err(format!("Comando desconhecido: {other}")),
    }
}

fn run_lock() -> Result<(), String> {
    let attempts = [
        Command::new("loginctl").args(["lock-session"]).status(),
        Command::new("gnome-screensaver-command").arg("-l").status(),
        Command::new("xdg-screensaver").args(["lock"]).status(),
        Command::new("i3lock").status(),
    ];
    if attempts.iter().any(|r| r.as_ref().map(|s| s.success()).unwrap_or(false)) {
        return Ok(());
    }
    Err("Não foi possível bloquear a tela".into())
}

fn run_systemctl(action: &str) -> Result<(), String> {
    let status = Command::new("systemctl")
        .arg(action)
        .status()
        .map_err(|e| e.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("systemctl {action} falhou"))
    }
}

fn run_logout() -> Result<(), String> {
    let status = Command::new("loginctl")
        .args(["terminate-user", &whoami()])
        .status()
        .map_err(|e| e.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err("Não foi possível encerrar a sessão".into())
    }
}

fn whoami() -> String {
    std::env::var("USER").unwrap_or_else(|_| "unknown".into())
}

fn run_screenshot() -> Result<(), String> {
    if Command::new("flameshot")
        .args(["gui"])
        .spawn()
        .map(|_| ())
        .is_ok()
    {
        return Ok(());
    }
    if Command::new("grim")
        .args(["-g", "root", "/tmp/spotlight-screenshot.png"])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
    {
        return Ok(());
    }
    Err("Instale flameshot ou grim para capturas".into())
}

fn run_empty_trash() -> Result<(), String> {
    let home = dirs::home_dir().ok_or_else(|| "HOME não definido".to_string())?;
    for name in [".local/share/Trash/files", ".Trash/files"] {
        let path = home.join(name);
        if path.exists() {
            let _ = std::fs::remove_dir_all(&path);
            let _ = std::fs::create_dir_all(&path);
        }
    }
    Ok(())
}
