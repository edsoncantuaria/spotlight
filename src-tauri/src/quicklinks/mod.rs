use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use serde::Deserialize;
use std::fs;
use std::sync::{Arc, RwLock};

use crate::history::HistoryDb;
use crate::paths;
use crate::search::ranking::{self, build_result};
use crate::search::types::{make_id, ResultKind, SearchResult};

#[derive(Debug, Clone, Deserialize)]
pub struct QuicklinkEntry {
    pub keyword: String,
    pub title: String,
    pub url: String,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub alias: Option<String>,
}

#[derive(Debug, Deserialize)]
struct QuicklinksFile {
    #[serde(default)]
    quicklink: Vec<QuicklinkEntry>,
}

pub struct QuicklinksIndex {
    entries: Arc<RwLock<Vec<QuicklinkEntry>>>,
}

impl QuicklinksIndex {
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
                let trigger = entry.keyword.trim();
                let alias = entry.alias.as_deref().unwrap_or(trigger);

                if query.starts_with(trigger) || query.starts_with(alias) {
                    let rest = query
                        .strip_prefix(trigger)
                        .or_else(|| query.strip_prefix(alias))
                        .unwrap_or("")
                        .trim();
                    let title = if rest.is_empty() {
                        entry.title.clone()
                    } else {
                        format!("{} — {}", entry.title, rest)
                    };
                    return Some(build_result(
                        make_id(ResultKind::Quicklink, &entry.keyword),
                        ResultKind::Quicklink,
                        title,
                        Some(entry.url.clone()),
                        entry.icon.clone().or(Some("web-browser".to_string())),
                        2000,
                        query,
                        history,
                    ));
                }

                let haystack = format!("{} {} {}", entry.title, trigger, alias);
                let score = matcher.fuzzy_match(&haystack, query)?;
                Some(build_result(
                    make_id(ResultKind::Quicklink, &entry.keyword),
                    ResultKind::Quicklink,
                    entry.title.clone(),
                    Some(format!("{trigger} → URL")),
                    entry.icon.clone().or(Some("web-browser".to_string())),
                    score,
                    query,
                    history,
                ))
            })
            .collect();

        ranking::sort_results(&mut results);
        results.into_iter().take(limit).collect()
    }

    pub fn resolve_url(&self, keyword: &str, query: &str) -> Option<String> {
        let entries = self.entries.read().ok()?;
        let entry = entries.iter().find(|e| e.keyword == keyword)?;
        let rest = query
            .strip_prefix(&entry.keyword)
            .or_else(|| entry.alias.as_ref().and_then(|a| query.strip_prefix(a)))
            .unwrap_or(query)
            .trim();
        let encoded: String = rest
            .chars()
            .map(|c| if c == ' ' { '+' } else { c })
            .collect();
        Some(entry.url.replace("{query}", &encoded))
    }

    pub fn get(&self, keyword: &str) -> Option<QuicklinkEntry> {
        self.entries.read().ok()?.iter().find(|e| e.keyword == keyword).cloned()
    }
}

fn load_entries() -> Vec<QuicklinkEntry> {
    let Some(path) = paths::quicklinks_file() else {
        return default_entries();
    };

    if !path.exists() {
        let _ = write_defaults(&path);
        return default_entries();
    }

    fs::read_to_string(&path)
        .ok()
        .and_then(|s| toml::from_str::<QuicklinksFile>(&s).ok())
        .map(|f| f.quicklink)
        .unwrap_or_else(default_entries)
}

fn default_entries() -> Vec<QuicklinkEntry> {
    vec![
        QuicklinkEntry {
            keyword: "!g".to_string(),
            title: "Pesquisar no Google".to_string(),
            url: "https://www.google.com/search?q={query}".to_string(),
            icon: Some("web-browser".to_string()),
            alias: None,
        },
        QuicklinkEntry {
            keyword: "!yt".to_string(),
            title: "Pesquisar no YouTube".to_string(),
            url: "https://www.youtube.com/results?search_query={query}".to_string(),
            icon: Some("youtube".to_string()),
            alias: None,
        },
    ]
}

fn write_defaults(path: &std::path::Path) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let content = r#"[[quicklink]]
keyword = "!g"
title = "Pesquisar no Google"
url = "https://www.google.com/search?q={query}"
icon = "web-browser"

[[quicklink]]
keyword = "!yt"
title = "Pesquisar no YouTube"
url = "https://www.youtube.com/results?search_query={query}"
icon = "youtube"
"#;
    fs::write(path, content)
}
