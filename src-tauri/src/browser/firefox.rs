use rusqlite::Connection;
use std::path::{Path, PathBuf};

use super::{copy_db_for_read, make_entry, history_limit, BrowserEntryKind};

pub fn load_entries() -> Vec<super::BrowserEntry> {
    let mut entries = Vec::new();
    for profile in firefox_profiles() {
        if let Some(db_path) = profile.join("places.sqlite").exists().then(|| profile.join("places.sqlite")) {
            entries.extend(load_from_places(&db_path, &profile));
        }
    }
    entries
}

fn firefox_profiles() -> Vec<PathBuf> {
    let Some(home) = dirs::home_dir() else {
        return Vec::new();
    };
    let ini = home.join(".mozilla/firefox/profiles.ini");
    if !ini.exists() {
        return Vec::new();
    }

    let content = match std::fs::read_to_string(&ini) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let base = home.join(".mozilla/firefox");
    let mut profiles = Vec::new();

    for section in content.split("[Profile") {
        let mut path = None;
        for line in section.lines() {
            let line = line.trim();
            if let Some(p) = line.strip_prefix("Path=") {
                path = Some(p.trim().to_string());
            }
        }
        if let Some(rel) = path {
            let full = if rel.starts_with('/') {
                PathBuf::from(rel)
            } else {
                base.join(rel)
            };
            if full.exists() {
                profiles.push(full);
            }
        }
    }

    profiles
}

fn load_from_places(db_path: &Path, _profile: &Path) -> Vec<super::BrowserEntry> {
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

    let mut entries = Vec::new();
    entries.extend(load_bookmarks(&conn));
    entries.extend(load_history(&conn));
    let _ = std::fs::remove_file(temp);
    entries
}

fn load_bookmarks(conn: &Connection) -> Vec<super::BrowserEntry> {
    let mut stmt = match conn.prepare(
        "SELECT b.id, COALESCE(NULLIF(b.title, ''), p.title, p.url), p.url
         FROM moz_bookmarks b
         JOIN moz_places p ON b.fk = p.id
         WHERE b.type = 1",
    ) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    let rows = match stmt.query_map([], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
        ))
    }) {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };

    rows.filter_map(|r| r.ok())
        .map(|(id, title, url)| {
            make_entry(
                format!("bookmark:firefox:{id}"),
                BrowserEntryKind::Bookmark,
                title,
                url,
                "Firefox".to_string(),
                0,
            )
        })
        .collect()
}

fn load_history(conn: &Connection) -> Vec<super::BrowserEntry> {
    let sql = format!(
        "SELECT id, COALESCE(NULLIF(title, ''), url), url, visit_count
         FROM moz_places
         WHERE visit_count > 0
         ORDER BY last_visit_date DESC
         LIMIT {}",
        history_limit()
    );
    let mut stmt = match conn.prepare(&sql) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
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
        Err(_) => return Vec::new(),
    };

    rows.filter_map(|r| r.ok())
        .map(|(id, title, url, visit_count)| {
            make_entry(
                format!("browser:firefox:{id}"),
                BrowserEntryKind::History,
                title,
                url,
                "Firefox".to_string(),
                visit_count,
            )
        })
        .collect()
}
