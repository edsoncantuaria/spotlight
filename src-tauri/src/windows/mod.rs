mod detect;
mod fallback;
mod gnome;
mod hyprland;
mod kde;

use serde::Serialize;

pub use detect::detect_de;

#[derive(Clone, Serialize)]
pub struct WindowInfo {
    pub id: String,
    pub title: String,
    pub app: Option<String>,
}

pub trait WindowManager: Send + Sync {
    fn list_windows(&self) -> Vec<WindowInfo>;
    fn focus(&self, id: &str) -> Result<(), String>;
    fn move_active(&self, direction: &str) -> Result<(), String>;
    fn toggle_fullscreen(&self) -> Result<(), String>;
    fn close_active(&self) -> Result<(), String>;
    fn minimize_active(&self) -> Result<(), String>;
}

pub struct WindowService {
    backend: Box<dyn WindowManager>,
}

impl WindowService {
    pub fn new() -> Self {
        let de = detect_de();
        let backend: Box<dyn WindowManager> = match de.as_str() {
            "gnome" => Box::new(gnome::GnomeWindowManager),
            "kde" => Box::new(kde::KdeWindowManager),
            "hyprland" => Box::new(hyprland::HyprlandWindowManager),
            _ => Box::new(fallback::WmctrlManager),
        };
        Self { backend }
    }

    pub fn search(&self, query: &str, history: &crate::history::HistoryDb, limit: usize) -> Vec<crate::search::types::SearchResult> {
        use crate::search::ranking::build_result;
        use crate::search::types::{make_id, ResultKind};

        let q = query.to_lowercase();
        self.backend
            .list_windows()
            .into_iter()
            .filter(|w| q.is_empty() || w.title.to_lowercase().contains(&q))
            .take(limit)
            .map(|w| {
                build_result(
                    make_id(ResultKind::Window, &w.id),
                    ResultKind::Window,
                    w.title,
                    w.app,
                    Some("window".to_string()),
                    700,
                    query,
                    history,
                )
            })
            .collect()
    }

    pub fn run_action(&self, action_id: &str, args: &str) -> Result<(), String> {
        if action_id.starts_with("window:focus:") {
            let id = action_id.strip_prefix("window:focus:").unwrap_or("");
            return self.backend.focus(id);
        }
        match action_id {
            "window:left" => self.backend.move_active("left"),
            "window:right" => self.backend.move_active("right"),
            "window:up" => self.backend.move_active("up"),
            "window:down" => self.backend.move_active("down"),
            "window:fullscreen" => self.backend.toggle_fullscreen(),
            "window:close" => self.backend.close_active(),
            "window:minimize" => self.backend.minimize_active(),
            _ => {
                if let Some(id) = action_id.strip_prefix("window:") {
                    self.backend.focus(id)
                } else if action_id.starts_with("0x") || action_id.chars().all(|c| c.is_ascii_digit()) {
                    self.backend.focus(action_id)
                } else {
                    Err(format!("Ação de janela desconhecida: {action_id}"))
                }
            }
        }
    }
}

pub fn window_commands(history: &crate::history::HistoryDb, query: &str, limit: usize) -> Vec<crate::search::types::SearchResult> {
    use crate::search::ranking::build_result;
    use crate::search::types::{make_id, ResultKind};

    let q = query.to_lowercase();
    let cmds = [
        ("window:left", "Mover janela à esquerda", "←"),
        ("window:right", "Mover janela à direita", "→"),
        ("window:fullscreen", "Tela cheia", "⛶"),
        ("window:minimize", "Minimizar janela ativa", "—"),
        ("window:close", "Fechar janela ativa", "×"),
    ];
    cmds.iter()
        .filter(|(_, title, _)| q.is_empty() || title.to_lowercase().contains(&q) || "window".contains(&q))
        .map(|(id, title, hint)| {
            build_result(
                make_id(ResultKind::Window, id),
                ResultKind::Window,
                title.to_string(),
                Some(hint.to_string()),
                Some("window".to_string()),
                850,
                query,
                history,
            )
        })
        .take(limit)
        .collect()
}
