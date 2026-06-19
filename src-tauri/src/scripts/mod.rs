use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, RwLock};
use std::time::Duration;

use crate::history::HistoryDb;
use crate::paths;
use crate::search::ranking::{self, build_result};
use crate::search::types::{make_id, ResultKind, SearchResult};

#[derive(Debug, Clone, Deserialize)]
pub struct ScriptEntry {
    pub id: String,
    pub title: String,
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub keyword: Option<String>,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub needs_query: bool,
}

pub struct ScriptsIndex {
    entries: Arc<RwLock<Vec<ScriptEntry>>>,
}

impl ScriptsIndex {
    pub fn new() -> Self {
        let index = Self {
            entries: Arc::new(RwLock::new(Vec::new())),
        };
        index.reload();
        index
    }

    pub fn reload(&self) {
        if let Ok(mut guard) = self.entries.write() {
            *guard = load_entries();
        }
    }

    pub fn search(&self, query: &str, history: &HistoryDb, limit: usize) -> Vec<SearchResult> {
        let entries = match self.entries.read() {
            Ok(e) => e.clone(),
            Err(_) => return Vec::new(),
        };

        let query = query.trim();
        if query.is_empty() {
            return entries
                .iter()
                .take(limit)
                .map(|e| {
                    build_result(
                        make_id(ResultKind::Script, &e.id),
                        ResultKind::Script,
                        e.title.clone(),
                        e.keyword.clone(),
                        e.icon.clone().or(Some("utilities-terminal".to_string())),
                        500i64,
                        "",
                        history,
                    )
                })
                .collect();
        }

        let matcher = SkimMatcherV2::default();
        let mut results: Vec<SearchResult> = entries
            .iter()
            .filter_map(|entry| {
                let haystack = format!(
                    "{} {} {}",
                    entry.title,
                    entry.keyword.as_deref().unwrap_or(""),
                    entry.id
                );
                let score = matcher.fuzzy_match(&haystack, query)?;
                Some(build_result(
                    make_id(ResultKind::Script, &entry.id),
                    ResultKind::Script,
                    entry.title.clone(),
                    entry.keyword.clone(),
                    entry.icon.clone().or(Some("utilities-terminal".to_string())),
                    score,
                    query,
                    history,
                ))
            })
            .collect();

        ranking::sort_results(&mut results);
        results.into_iter().take(limit).collect()
    }

    pub fn get(&self, id: &str) -> Option<ScriptEntry> {
        self.entries.read().ok()?.iter().find(|e| e.id == id).cloned()
    }

    pub fn run(&self, id: &str, query: &str) -> Result<String, String> {
        let entry = self.get(id).ok_or_else(|| "Script não encontrado".to_string())?;
        let mut cmd = Command::new(&entry.command);
        for arg in &entry.args {
            cmd.arg(arg.replace("{query}", query));
        }
        if entry.needs_query && !query.is_empty() {
            cmd.arg(query);
        }
        cmd.stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = cmd.spawn().map_err(|e| format!("Falha ao executar: {e}"))?;

        let output = wait_timeout(&mut child, Duration::from_secs(30))
            .map_err(|e| format!("Script expirou ou falhou: {e}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Script falhou: {stderr}"));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }
}

fn wait_timeout(
    child: &mut std::process::Child,
    timeout: Duration,
) -> Result<std::process::Output, String> {
    let start = std::time::Instant::now();
    loop {
        if let Some(status) = child.try_wait().map_err(|e| e.to_string())? {
            let stdout = child.stdout.take();
            let stderr = child.stderr.take();
            let mut out = Vec::new();
            let mut err = Vec::new();
            if let Some(mut s) = stdout {
                use std::io::Read;
                let _ = s.read_to_end(&mut out);
            }
            if let Some(mut s) = stderr {
                use std::io::Read;
                let _ = s.read_to_end(&mut err);
            }
            return Ok(std::process::Output {
                status,
                stdout: out,
                stderr: err,
            });
        }
        if start.elapsed() > timeout {
            let _ = child.kill();
            return Err("timeout".to_string());
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

fn load_entries() -> Vec<ScriptEntry> {
    let Some(dir) = paths::scripts_dir() else {
        return Vec::new();
    };

    if !dir.exists() {
        let _ = fs::create_dir_all(&dir);
        let _ = write_example(&dir);
        return Vec::new();
    }

    let mut entries = Vec::new();
    if let Ok(read) = fs::read_dir(&dir) {
        for item in read.flatten() {
            let path = item.path();
            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                if let Some(entry) = parse_script_file(&path) {
                    entries.push(entry);
                }
            }
        }
    }
    entries.sort_by(|a, b| a.title.cmp(&b.title));
    entries
}

fn parse_script_file(path: &Path) -> Option<ScriptEntry> {
    let content = fs::read_to_string(path).ok()?;
    let mut entry: ScriptEntry = serde_json::from_str(&content).ok()?;
    if entry.id.is_empty() {
        entry.id = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("script")
            .to_string();
    }
    Some(entry)
}

fn write_example(dir: &Path) -> std::io::Result<()> {
    let example = dir.join("hello.spotlight.json");
    let content = r#"{
  "id": "hello",
  "title": "Dizer olá",
  "command": "echo",
  "args": ["Olá, {query}!"],
  "keyword": "hello",
  "needs_query": true
}"#;
    fs::write(example, content)
}
