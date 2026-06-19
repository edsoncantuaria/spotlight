use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use rusqlite::{params, Connection};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;
use std::time::Duration;
use walkdir::WalkDir;

use crate::config::load as load_config;
use crate::history::HistoryDb;
use crate::paths;
use crate::search::ranking::{self, build_result};
use crate::search::types::{make_id, ResultKind, SearchResult};

static INDEX_READY: AtomicBool = AtomicBool::new(false);
static DB: OnceLock<Arc<Mutex<Connection>>> = OnceLock::new();

pub fn db() -> Arc<Mutex<Connection>> {
    DB.get_or_init(|| {
        let conn = open_connection().expect("file index db");
        init_schema(&conn).expect("file index schema");
        Arc::new(Mutex::new(conn))
    })
    .clone()
}

pub fn start_file_watcher() {
    thread::spawn(|| {
        start_initial_index();
        watch_loop();
    });
}

pub fn search_index(query: &str, history: &HistoryDb, limit: usize) -> Vec<SearchResult> {
    if !INDEX_READY.load(Ordering::Relaxed) {
        return Vec::new();
    }

    let query = query.trim();
    if query.len() < 2 {
        return Vec::new();
    }

    let db = db();
    let Ok(conn) = db.lock() else {
        return Vec::new();
    };

    let pattern = format!("%{query}%");
    let mut stmt = match conn.prepare(
        "SELECT path, name, parent FROM file_cache WHERE name LIKE ?1 COLLATE NOCASE LIMIT ?2",
    ) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    let rows = match stmt.query_map(params![pattern, (limit * 4) as i64], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
        ))
    }) {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };

    let matcher = SkimMatcherV2::default();
    let mut results: Vec<SearchResult> = rows
        .filter_map(|r| r.ok())
        .filter_map(|(path, name, parent)| {
            let score = matcher.fuzzy_match(&name, query)?;
            Some(build_result(
                make_id(ResultKind::File, &path),
                ResultKind::File,
                name,
                Some(parent),
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

fn start_initial_index() {
    let config = load_config();
    let roots = file_roots(&config.file_roots);
    let max = config.max_index_files;
    let excludes = config.exclude_patterns.clone();

    thread::spawn(move || {
        let mut count = 0usize;
        for root in roots {
            if count >= max {
                break;
            }
            for entry in WalkDir::new(&root)
                .follow_links(false)
                .into_iter()
                .filter_entry(|e| !should_skip(e.path(), &excludes))
            {
                if count >= max {
                    break;
                }
                let Ok(entry) = entry else { continue };
                if !entry.file_type().is_file() {
                    continue;
                }
                let path = entry.path();
                if upsert_file(path).is_ok() {
                    count += 1;
                }
            }
        }
        INDEX_READY.store(true, Ordering::Relaxed);
    });
}

fn watch_loop() {
    let config = load_config();
    let roots = file_roots(&config.file_roots);
    if roots.is_empty() {
        return;
    }

    let (tx, rx) = std::sync::mpsc::channel();
    let Ok(mut watcher) = RecommendedWatcher::new(tx, Config::default()) else {
        return;
    };

    for dir in &roots {
        let _ = watcher.watch(dir, RecursiveMode::Recursive);
    }

    loop {
        match rx.recv_timeout(Duration::from_secs(30)) {
            Ok(Ok(event)) => {
                for path in event.paths {
                    if path.is_file() {
                        let _ = upsert_file(&path);
                    } else if !path.exists() {
                        let _ = remove_file(&path);
                    }
                }
            }
            Ok(Err(e)) => eprintln!("[spotlight] watcher error: {e}"),
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }
}

pub fn upsert_file(path: &Path) -> rusqlite::Result<()> {
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();
    if name.is_empty() {
        return Ok(());
    }
    let parent = path
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();
    let mtime = path
        .metadata()
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let path_str = path.to_string_lossy().to_string();

    let db = db();
    let conn = db.lock().map_err(|_| rusqlite::Error::InvalidQuery)?;
    conn.execute(
        "INSERT INTO file_cache (path, name, parent, mtime) VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(path) DO UPDATE SET name=?2, parent=?3, mtime=?4",
        params![path_str, name, parent, mtime],
    )?;
    Ok(())
}

fn remove_file(path: &Path) -> rusqlite::Result<()> {
    let path_str = path.to_string_lossy().to_string();
    let db = db();
    let conn = db.lock().map_err(|_| rusqlite::Error::InvalidQuery)?;
    conn.execute("DELETE FROM file_cache WHERE path = ?1", params![path_str])?;
    Ok(())
}

fn should_skip(path: &Path, excludes: &[String]) -> bool {
    let s = path.to_string_lossy();
    for pat in excludes {
        if s.contains(pat) {
            return true;
        }
    }
    false
}

fn file_roots(configured: &[String]) -> Vec<PathBuf> {
    if !configured.is_empty() {
        return configured.iter().map(PathBuf::from).filter(|p| p.exists()).collect();
    }
    let Some(home) = dirs::home_dir() else {
        return Vec::new();
    };
    ["Documents", "Downloads", "Desktop"]
        .into_iter()
        .map(|d| home.join(d))
        .filter(|p| p.exists())
        .collect()
}

fn open_connection() -> rusqlite::Result<Connection> {
    let path = paths::spotlight_dir()
        .map(|d| d.join("history.db"))
        .ok_or(rusqlite::Error::InvalidQuery)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    Connection::open(path)
}

fn init_schema(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS file_cache (
            path TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            parent TEXT NOT NULL,
            mtime INTEGER NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_file_name ON file_cache(name);",
    )
}
