use std::process::Command;

use crate::extensions::SearchProvider;
use crate::history::HistoryDb;
use crate::search::ranking::build_result;
use crate::search::types::{make_id, ResultKind, SearchResult};

pub struct GitExtension;

impl SearchProvider for GitExtension {
    fn id(&self) -> &str {
        "git"
    }

    fn title(&self) -> &str {
        "Git"
    }

    fn keywords(&self) -> &[String] {
        static KW: std::sync::OnceLock<Vec<String>> = std::sync::OnceLock::new();
        KW.get_or_init(|| vec!["git".to_string()])
    }

    fn search(&self, query: &str, history: &HistoryDb, limit: usize) -> Vec<SearchResult> {
        let mut results = Vec::new();
        if query.is_empty() || query.contains("status") || query.contains("git") {
            if let Ok(out) = Command::new("git").args(["status", "--short"]).output() {
                let text = String::from_utf8_lossy(&out.stdout);
                for (i, line) in text.lines().take(limit).enumerate() {
                    if line.trim().is_empty() {
                        continue;
                    }
                    results.push(build_result(
                        make_id(ResultKind::Extension, &format!("git:status:{i}")),
                        ResultKind::Extension,
                        line.trim().to_string(),
                        Some("git status".to_string()),
                        Some("git".to_string()),
                        800,
                        query,
                        history,
                    ));
                }
            }
        }
        if query.is_empty() || query.contains("branch") {
            if let Ok(out) = Command::new("git").args(["branch", "--list"]).output() {
                let text = String::from_utf8_lossy(&out.stdout);
                for line in text.lines().take(5) {
                    let name = line.trim().trim_start_matches('*').trim();
                    if name.is_empty() {
                        continue;
                    }
                    results.push(build_result(
                        make_id(ResultKind::Extension, &format!("git:branch:{name}")),
                        ResultKind::Extension,
                        name.to_string(),
                        Some("Branch".to_string()),
                        Some("git".to_string()),
                        750,
                        query,
                        history,
                    ));
                }
            }
        }
        results.into_iter().take(limit).collect()
    }

    fn run(&self, action_id: &str, _args: &str) -> Result<String, String> {
        if action_id.starts_with("git:branch:") {
            let branch = action_id.strip_prefix("git:branch:").unwrap_or("");
            Command::new("git")
                .args(["checkout", branch])
                .status()
                .map_err(|e| e.to_string())?;
            return Ok(format!("Checkout {branch}"));
        }
        Ok("Git".to_string())
    }
}
