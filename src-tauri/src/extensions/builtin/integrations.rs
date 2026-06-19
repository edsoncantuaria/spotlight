use std::process::Command;

use crate::extensions::SearchProvider;
use crate::history::HistoryDb;
use crate::search::ranking::build_result;
use crate::search::types::{make_id, ResultKind, SearchResult};

pub struct IntegrationsExtension;

impl SearchProvider for IntegrationsExtension {
    fn id(&self) -> &str {
        "integrations"
    }

    fn title(&self) -> &str {
        "Integrações"
    }

    fn keywords(&self) -> &[String] {
        static KW: std::sync::OnceLock<Vec<String>> = std::sync::OnceLock::new();
        KW.get_or_init(|| {
            vec![
                "screenshot".to_string(),
                "print".to_string(),
                "calendar".to_string(),
            ]
        })
    }

    fn search(&self, query: &str, history: &HistoryDb, limit: usize) -> Vec<SearchResult> {
        let q = query.to_lowercase();
        let mut results = Vec::new();

        if q.is_empty() || q.contains("screenshot") || q.contains("print") {
            results.push(build_result(
                make_id(ResultKind::Extension, "integrations:flameshot"),
                ResultKind::Extension,
                "Captura de tela (Flameshot)".to_string(),
                Some("flameshot gui".to_string()),
                Some("camera-photo".to_string()),
                700,
                query,
                history,
            ));
            results.push(build_result(
                make_id(ResultKind::Extension, "integrations:grim"),
                ResultKind::Extension,
                "Captura de tela (grim)".to_string(),
                Some("grim -g \"$(slurp)\"".to_string()),
                Some("camera-photo".to_string()),
                680,
                query,
                history,
            ));
        }

        if q.is_empty() || q.contains("calendar") || q.contains("calend") {
            results.push(build_result(
                make_id(ResultKind::Extension, "integrations:calendar"),
                ResultKind::Extension,
                "Abrir calendário (khal)".to_string(),
                Some("khal calendar".to_string()),
                Some("office-calendar".to_string()),
                650,
                query,
                history,
            ));
        }

        results.into_iter().take(limit).collect()
    }

    fn run(&self, action_id: &str, _args: &str) -> Result<String, String> {
        match action_id {
            "integrations:flameshot" => {
                Command::new("flameshot")
                    .arg("gui")
                    .spawn()
                    .map_err(|e| e.to_string())?;
                Ok("Flameshot iniciado".to_string())
            }
            "integrations:grim" => {
                Command::new("sh")
                    .args(["-c", "grim -g \"$(slurp)\""])
                    .spawn()
                    .map_err(|e| e.to_string())?;
                Ok("grim iniciado".to_string())
            }
            "integrations:calendar" => {
                Command::new("khal")
                    .arg("calendar")
                    .spawn()
                    .map_err(|e| e.to_string())?;
                Ok("khal calendar".to_string())
            }
            _ => Err("Integração desconhecida".to_string()),
        }
    }
}
