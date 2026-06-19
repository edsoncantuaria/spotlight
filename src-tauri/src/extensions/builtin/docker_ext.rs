use std::process::Command;

use crate::extensions::SearchProvider;
use crate::history::HistoryDb;
use crate::search::ranking::build_result;
use crate::search::types::{make_id, ResultKind, SearchResult};

pub struct DockerExtension;

impl SearchProvider for DockerExtension {
    fn id(&self) -> &str {
        "docker"
    }

    fn title(&self) -> &str {
        "Docker"
    }

    fn keywords(&self) -> &[String] {
        static KW: std::sync::OnceLock<Vec<String>> = std::sync::OnceLock::new();
        KW.get_or_init(|| vec!["docker".to_string()])
    }

    fn search(&self, query: &str, history: &HistoryDb, limit: usize) -> Vec<SearchResult> {
        let output = Command::new("docker")
            .args(["ps", "--format", "{{.ID}}\t{{.Names}}\t{{.Status}}"])
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
                let parts: Vec<&str> = line.split('\t').collect();
                let id = parts.first().copied().unwrap_or("");
                let name = parts.get(1).copied().unwrap_or(id);
                build_result(
                    make_id(ResultKind::Extension, &format!("docker:{id}")),
                    ResultKind::Extension,
                    name.to_string(),
                    parts.get(2).map(|s| s.to_string()),
                    Some("docker".to_string()),
                    800,
                    query,
                    history,
                )
            })
            .collect()
    }

    fn run(&self, action_id: &str, _args: &str) -> Result<String, String> {
        if let Some(id) = action_id.strip_prefix("docker:") {
            Command::new("docker")
                .args(["logs", "--tail", "50", id])
                .status()
                .map_err(|e| e.to_string())?;
            return Ok("Logs exibidos no terminal".to_string());
        }
        Err("Container não encontrado".to_string())
    }
}
