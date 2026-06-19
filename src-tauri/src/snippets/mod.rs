use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use serde::Deserialize;
use std::fs;
use std::sync::{Arc, RwLock};

use crate::clipboard;
use crate::history::HistoryDb;
use crate::paths;
use crate::search::ranking::{self, build_result};
use crate::search::types::{make_id, ResultKind, SearchResult};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SnippetMode {
    Paste,
    Copy,
}

impl Default for SnippetMode {
    fn default() -> Self {
        Self::Paste
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct SnippetEntry {
    pub keyword: String,
    pub name: String,
    pub text: String,
    #[serde(default)]
    pub mode: SnippetMode,
}

#[derive(Debug, Deserialize)]
struct SnippetsFile {
    #[serde(default)]
    snippet: Vec<SnippetEntry>,
}

pub struct SnippetsIndex {
    entries: Arc<RwLock<Vec<SnippetEntry>>>,
}

impl SnippetsIndex {
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
            return Vec::new();
        }

        let matcher = SkimMatcherV2::default();
        let mut results: Vec<SearchResult> = entries
            .iter()
            .filter_map(|entry| {
                if query.starts_with(&entry.keyword) || query == entry.keyword {
                    return Some(build_result(
                        make_id(ResultKind::Snippet, &entry.keyword),
                        ResultKind::Snippet,
                        entry.name.clone(),
                        Some(format!("Snippet {}", entry.keyword)),
                        Some("text-x-generic".to_string()),
                        2000,
                        query,
                        history,
                    ));
                }
                let haystack = format!("{} {} {}", entry.name, entry.keyword, entry.text);
                let score = matcher.fuzzy_match(&haystack, query)?;
                Some(build_result(
                    make_id(ResultKind::Snippet, &entry.keyword),
                    ResultKind::Snippet,
                    entry.name.clone(),
                    Some(entry.keyword.clone()),
                    Some("text-x-generic".to_string()),
                    score,
                    query,
                    history,
                ))
            })
            .collect();

        ranking::sort_results(&mut results);
        results.into_iter().take(limit).collect()
    }

    pub fn get(&self, keyword: &str) -> Option<SnippetEntry> {
        self.entries
            .read()
            .ok()?
            .iter()
            .find(|e| e.keyword == keyword)
            .cloned()
    }

    pub fn apply(&self, keyword: &str) -> Result<(), String> {
        let entry = self.get(keyword).ok_or_else(|| "Snippet não encontrado".to_string())?;
        match entry.mode {
            SnippetMode::Copy => clipboard::write_to_clipboard(&entry.text),
            SnippetMode::Paste => {
                clipboard::write_to_clipboard(&entry.text)?;
                std::thread::spawn(|| {
                    let _ = crate::input::simulate_paste();
                });
                Ok(())
            }
        }
    }
}

fn load_entries() -> Vec<SnippetEntry> {
    let Some(path) = paths::snippets_file() else {
        return Vec::new();
    };

    if !path.exists() {
        let _ = write_defaults(&path);
    }

    fs::read_to_string(path)
        .ok()
        .and_then(|s| toml::from_str::<SnippetsFile>(&s).ok())
        .map(|f| f.snippet)
        .unwrap_or_default()
}

fn write_defaults(path: &std::path::Path) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let content = r#"[[snippet]]
keyword = ";email"
name = "Assinatura de e-mail"
text = "Atenciosamente,\nSeu Nome"
mode = "paste"
"#;
    fs::write(path, content)
}
