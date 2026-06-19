use std::process::Command;

use crate::extensions::SearchProvider;
use crate::history::HistoryDb;
use crate::search::ranking::build_result;
use crate::search::types::{make_id, ResultKind, SearchResult};

pub struct SystemdExtension;

impl SearchProvider for SystemdExtension {
    fn id(&self) -> &str {
        "systemd"
    }

    fn title(&self) -> &str {
        "Systemd"
    }

    fn keywords(&self) -> &[String] {
        static KW: std::sync::OnceLock<Vec<String>> = std::sync::OnceLock::new();
        KW.get_or_init(|| vec!["systemd".to_string(), "service".to_string()])
    }

    fn search(&self, query: &str, history: &HistoryDb, limit: usize) -> Vec<SearchResult> {
        let output = Command::new("systemctl")
            .args(["--user", "list-units", "--type=service", "--no-pager", "--no-legend"])
            .output()
            .ok();
        let Some(output) = output else {
            return Vec::new();
        };
        if !output.status.success() {
            return Vec::new();
        }

        let text = String::from_utf8_lossy(&output.stdout);
        let q = query.to_lowercase();
        text.lines()
            .filter(|l| !l.trim().is_empty())
            .filter(|l| q.is_empty() || l.to_lowercase().contains(&q))
            .take(limit)
            .map(|line| {
                let name = line.split_whitespace().next().unwrap_or(line).to_string();
                build_result(
                    make_id(ResultKind::Extension, &format!("systemd:{name}")),
                    ResultKind::Extension,
                    name.clone(),
                    Some("Serviço systemd".to_string()),
                    Some("system-run".to_string()),
                    750,
                    query,
                    history,
                )
            })
            .collect()
    }

    fn run(&self, action_id: &str, args: &str) -> Result<String, String> {
        if let Some(name) = action_id.strip_prefix("systemd:") {
            let action = if args.is_empty() { "restart" } else { args };
            Command::new("systemctl")
                .args(["--user", action, name])
                .status()
                .map_err(|e| e.to_string())?;
            return Ok(format!("{action} {name}"));
        }
        Err("Serviço não encontrado".to_string())
    }
}
