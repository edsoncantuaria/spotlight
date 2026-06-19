mod chromium;
mod firefox;

use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock, RwLock};
use std::time::{Duration, Instant};

use crate::history::HistoryDb;
use crate::search::ranking::build_result;
use crate::search::types::{ResultKind, SearchResult};

const HISTORY_PER_BROWSER: usize = 500;
const MIN_QUERY_LEN: usize = 3;
const MAX_FUZZY_EVALS: usize = 200;

#[derive(Clone, Debug)]
pub struct BrowserEntry {
    pub id: String,
    pub kind: BrowserEntryKind,
    pub title: String,
    pub url: String,
    pub source: String,
    pub visit_count: i64,
    domain: String,
    title_lower: String,
    haystack: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BrowserEntryKind {
    Bookmark,
    History,
}

pub struct BrowserIndex {
    entries: Arc<RwLock<Arc<Vec<BrowserEntry>>>>,
    last_refresh: Arc<RwLock<Option<Instant>>>,
    refreshing: Arc<AtomicBool>,
}

impl BrowserIndex {
    pub fn new() -> Self {
        let index = Self {
            entries: Arc::new(RwLock::new(Arc::new(Vec::new()))),
            last_refresh: Arc::new(RwLock::new(None)),
            refreshing: Arc::new(AtomicBool::new(false)),
        };
        index.spawn_refresh();
        index.spawn_periodic_refresh();
        index
    }

    pub fn search(
        &self,
        query: &str,
        history: &HistoryDb,
        bookmark_limit: usize,
        history_limit: usize,
    ) -> (Vec<SearchResult>, Vec<SearchResult>) {
        let query = query.trim();
        if query.len() < MIN_QUERY_LEN {
            return (Vec::new(), Vec::new());
        }

        let entries = match self.entries.read() {
            Ok(guard) => Arc::clone(&guard),
            Err(_) => return (Vec::new(), Vec::new()),
        };

        if entries.is_empty() {
            return (Vec::new(), Vec::new());
        }

        let query_lower = query.to_lowercase();
        let bookmarks = search_kind(
            &entries,
            query,
            &query_lower,
            BrowserEntryKind::Bookmark,
            bookmark_limit,
            history,
        );
        let history_items = search_kind(
            &entries,
            query,
            &query_lower,
            BrowserEntryKind::History,
            history_limit,
            history,
        );
        (bookmarks, history_items)
    }

    pub fn get_url(&self, id: &str) -> Option<String> {
        self.get_entry(id).map(|e| e.url)
    }

    pub fn get_entry(&self, id: &str) -> Option<BrowserEntry> {
        let entries = self.entries.read().ok()?.clone();
        entries.iter().find(|e| e.id == id).cloned()
    }

    fn spawn_periodic_refresh(&self) {
        let this = self.clone_for_refresh();
        std::thread::spawn(move || loop {
            std::thread::sleep(Duration::from_secs(600));
            this.spawn_refresh();
        });
    }

    fn spawn_refresh(&self) {
        if self
            .refreshing
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            return;
        }

        let entries = Arc::clone(&self.entries);
        let last_refresh = Arc::clone(&self.last_refresh);
        let refreshing = Arc::clone(&self.refreshing);

        std::thread::spawn(move || {
            let mut collected = Vec::new();
            collected.extend(firefox::load_entries());
            collected.extend(chromium::load_entries());
            collected.sort_by(|a, b| b.visit_count.cmp(&a.visit_count));
            collected.dedup_by(|a, b| a.url == b.url && a.kind == b.kind);
            collected.truncate(3000);

            if let Ok(mut guard) = entries.write() {
                *guard = Arc::new(collected);
            }
            if let Ok(mut guard) = last_refresh.write() {
                *guard = Some(Instant::now());
            }
            refreshing.store(false, Ordering::Release);
        });
    }

    fn clone_for_refresh(&self) -> Self {
        Self {
            entries: Arc::clone(&self.entries),
            last_refresh: Arc::clone(&self.last_refresh),
            refreshing: Arc::clone(&self.refreshing),
        }
    }
}

fn search_kind(
    entries: &[BrowserEntry],
    query: &str,
    query_lower: &str,
    kind: BrowserEntryKind,
    limit: usize,
    history: &HistoryDb,
) -> Vec<SearchResult> {
    if limit == 0 {
        return Vec::new();
    }

    let matcher = SkimMatcherV2::default();
    let mut candidates: Vec<(usize, i64)> = Vec::with_capacity(limit * 4);
    let mut evals = 0usize;

    for (i, entry) in entries.iter().enumerate() {
        if entry.kind != kind {
            continue;
        }
        if !entry_matches(entry, query_lower) {
            continue;
        }
        if evals >= MAX_FUZZY_EVALS {
            break;
        }
        evals += 1;

        let score = matcher
            .fuzzy_match(&entry.title, query)
            .or_else(|| matcher.fuzzy_match(&entry.domain, query))
            .unwrap_or(0)
            + entry.visit_count.min(50);

        if score <= 0 {
            continue;
        }

        candidates.push((i, score));
        if candidates.len() > limit * 8 {
            candidates.sort_by(|a, b| b.1.cmp(&a.1));
            candidates.truncate(limit * 4);
        }
    }

    candidates.sort_by(|a, b| b.1.cmp(&a.1));
    candidates.truncate(limit);

    candidates
        .into_iter()
        .map(|(i, score)| {
            let entry = &entries[i];
            build_result(
                entry.id.clone(),
                match kind {
                    BrowserEntryKind::Bookmark => ResultKind::Bookmark,
                    BrowserEntryKind::History => ResultKind::Browser,
                },
                entry.title.clone(),
                Some(format!("{} · {}", entry.source, entry.domain)),
                cached_browser_icon(&entry.source),
                score,
                query,
                history,
            )
        })
        .collect()
}

fn entry_matches(entry: &BrowserEntry, query_lower: &str) -> bool {
    if query_lower.len() < 4 {
        entry.title_lower.contains(query_lower) || entry.domain.contains(query_lower)
    } else {
        entry.haystack.contains(query_lower)
    }
}

pub(crate) fn make_entry(
    id: String,
    kind: BrowserEntryKind,
    title: String,
    url: String,
    source: String,
    visit_count: i64,
) -> BrowserEntry {
    let domain = url_domain(&url);
    let title_lower = title.to_lowercase();
    let haystack = format!("{title_lower} {domain} {}", url.to_lowercase());
    BrowserEntry {
        id,
        kind,
        title,
        url,
        source,
        visit_count,
        domain,
        title_lower,
        haystack,
    }
}

pub fn open_url(url: &str) -> Result<(), String> {
    Command::new("xdg-open")
        .arg(url)
        .spawn()
        .map_err(|e| format!("Falha ao abrir URL: {e}"))?;
    Ok(())
}

pub(crate) fn copy_db_for_read(src: &Path) -> Option<PathBuf> {
    let name = src.file_name()?.to_string_lossy();
    let dest = std::env::temp_dir().join(format!("spotlight-{name}-{}.sqlite", std::process::id()));
    std::fs::copy(src, &dest).ok()?;
    Some(dest)
}

fn url_domain(url: &str) -> String {
    url.split("//")
        .nth(1)
        .unwrap_or(url)
        .split('/')
        .next()
        .unwrap_or(url)
        .to_string()
}

fn cached_browser_icon(source: &str) -> Option<String> {
    static CACHE: OnceLock<HashMap<String, Option<String>>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| {
        [
            "Firefox",
            "Chrome",
            "Chromium",
            "Brave",
            "Edge",
            "Vivaldi",
        ]
        .into_iter()
        .map(|s| (s.to_string(), resolve_browser_icon(s)))
        .collect()
    });
    cache.get(source).cloned().flatten()
}

fn resolve_browser_icon(source: &str) -> Option<String> {
    let icon = match source {
        "Firefox" => "firefox",
        "Chrome" => "google-chrome",
        "Chromium" => "chromium",
        "Brave" => "brave-browser",
        "Edge" => "microsoft-edge",
        "Vivaldi" => "vivaldi",
        _ => "web-browser",
    };
    let candidates = [
        format!("/usr/share/icons/hicolor/scalable/apps/{icon}.svg"),
        format!("/usr/share/pixmaps/{icon}.png"),
    ];
    candidates.into_iter().find(|p| Path::new(p).exists())
}

pub(crate) fn history_limit() -> usize {
    HISTORY_PER_BROWSER
}
