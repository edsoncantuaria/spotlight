use rusqlite::Connection;
use serde_json::Value;
use std::path::{Path, PathBuf};

use super::{copy_db_for_read, make_entry, history_limit, BrowserEntryKind};

const BROWSERS: &[(&str, &str)] = &[
    ("Chrome", ".config/google-chrome"),
    ("Chromium", ".config/chromium"),
    ("Brave", ".config/BraveSoftware/Brave-Browser"),
    ("Edge", ".config/microsoft-edge"),
    ("Vivaldi", ".config/vivaldi"),
];

pub fn load_entries() -> Vec<super::BrowserEntry> {
    let Some(home) = dirs::home_dir() else {
        return Vec::new();
    };

    let mut entries = Vec::new();
    for (name, rel) in BROWSERS {
        let base = home.join(rel);
        if !base.exists() {
            continue;
        }
        for profile in chromium_profiles(&base) {
            let history = profile.join("History");
            if history.exists() {
                entries.extend(load_history(&history, name));
            }
            let bookmarks = profile.join("Bookmarks");
            if bookmarks.exists() {
                entries.extend(load_bookmarks_json(&bookmarks, name));
            }
        }
    }
    entries
}

fn chromium_profiles(base: &Path) -> Vec<PathBuf> {
    let mut profiles = Vec::new();
    if (base.join("History")).exists() || (base.join("Bookmarks")).exists() {
        profiles.push(base.to_path_buf());
    }
    if let Ok(read) = std::fs::read_dir(base) {
        for entry in read.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if name == "Default" || name.starts_with("Profile ") {
                    profiles.push(path);
                }
            }
        }
    }
    profiles.sort();
    profiles.dedup();
    profiles
}

fn load_history(db_path: &Path, browser: &str) -> Vec<super::BrowserEntry> {
    let Some(temp) = copy_db_for_read(db_path) else {
        return Vec::new();
    };
    let conn = match Connection::open(&temp) {
        Ok(c) => c,
        Err(_) => {
            let _ = std::fs::remove_file(&temp);
            return Vec::new();
        }
    };

    let sql = format!(
        "SELECT id, COALESCE(NULLIF(title, ''), url), url, visit_count
         FROM urls
         WHERE visit_count > 0
         ORDER BY last_visit_time DESC
         LIMIT {}",
        history_limit()
    );
    let mut stmt = match conn.prepare(&sql) {
        Ok(s) => s,
        Err(_) => {
            let _ = std::fs::remove_file(&temp);
            return Vec::new();
        }
    };

    let rows = match stmt.query_map([], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, i64>(3)?,
        ))
    }) {
        Ok(r) => r,
        Err(_) => {
            let _ = std::fs::remove_file(&temp);
            return Vec::new();
        }
    };

    let entries: Vec<_> = rows
        .filter_map(|r| r.ok())
        .map(|(id, title, url, visit_count)| {
            make_entry(
                format!("browser:{browser}:{id}"),
                BrowserEntryKind::History,
                title,
                url,
                browser.to_string(),
                visit_count,
            )
        })
        .collect();

    let _ = std::fs::remove_file(temp);
    entries
}

fn load_bookmarks_json(path: &Path, browser: &str) -> Vec<super::BrowserEntry> {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    let json: Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };

    let mut entries = Vec::new();
    if let Some(roots) = json.get("roots") {
        for key in ["bookmark_bar", "other", "synced"] {
            if let Some(node) = roots.get(key) {
                walk_bookmark_node(node, browser, &mut entries);
            }
        }
    }
    entries
}

fn walk_bookmark_node(node: &Value, browser: &str, out: &mut Vec<super::BrowserEntry>) {
    if node.get("type").and_then(|t| t.as_str()) == Some("url") {
        let title = node
            .get("name")
            .and_then(|n| n.as_str())
            .unwrap_or("")
            .to_string();
        let url = node
            .get("url")
            .and_then(|u| u.as_str())
            .unwrap_or("")
            .to_string();
        if !url.is_empty() {
            out.push(make_entry(
                format!("bookmark:{browser}:{}", hash_str(&url)),
                BrowserEntryKind::Bookmark,
                if title.is_empty() { url.clone() } else { title },
                url,
                browser.to_string(),
                0,
            ));
        }
    }

    if let Some(children) = node.get("children").and_then(|c| c.as_array()) {
        for child in children {
            walk_bookmark_node(child, browser, out);
        }
    }
}

fn hash_str(s: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    s.hash(&mut h);
    h.finish()
}
