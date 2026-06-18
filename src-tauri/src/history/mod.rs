use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use std::path::PathBuf;
use std::sync::Mutex;

use crate::search::types::{ResultKind, SearchResult};

pub struct HistoryDb {
    conn: Mutex<Connection>,
}

impl HistoryDb {
    pub fn new() -> Self {
        let conn = open_connection().expect("failed to open history db");
        init_schema(&conn).expect("failed to init history schema");
        Self {
            conn: Mutex::new(conn),
        }
    }

    pub fn record_launch(&self, id: &str, kind: ResultKind) {
        let Ok(conn) = self.conn.lock() else {
            return;
        };
        let now = Utc::now().timestamp();
        let _ = conn.execute(
            "INSERT INTO launches (id, kind, timestamp) VALUES (?1, ?2, ?3)",
            params![id, kind.as_str(), now],
        );
        let _ = conn.execute(
            "INSERT INTO access_counts (id, count) VALUES (?1, 1)
             ON CONFLICT(id) DO UPDATE SET count = count + 1",
            params![id],
        );
    }

    pub fn get_count(&self, id: &str) -> i64 {
        let Ok(conn) = self.conn.lock() else {
            return 0;
        };
        conn.query_row(
            "SELECT count FROM access_counts WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )
        .unwrap_or(0)
    }

    pub fn recency_boost(&self, id: &str) -> f64 {
        let Ok(conn) = self.conn.lock() else {
            return 0.0;
        };
        let ts: Option<i64> = conn
            .query_row(
                "SELECT timestamp FROM launches WHERE id = ?1 ORDER BY timestamp DESC LIMIT 1",
                params![id],
                |row| row.get(0),
            )
            .ok();

        let Some(ts) = ts else {
            return 0.0;
        };

        let launched: DateTime<Utc> = DateTime::from_timestamp(ts, 0).unwrap_or_else(Utc::now);
        let hours = (Utc::now() - launched).num_hours();
        if hours < 24 {
            1.0
        } else if hours < 168 {
            0.6
        } else if hours < 720 {
            0.3
        } else {
            0.1
        }
    }

    pub fn recent_results(&self, limit: usize) -> Vec<SearchResult> {
        let Ok(conn) = self.conn.lock() else {
            return Vec::new();
        };

        let mut stmt = match conn.prepare(
            "SELECT l.id, l.kind, COALESCE(m.title, l.id), COALESCE(m.subtitle, ''), COALESCE(m.icon, '')
             FROM launches l
             LEFT JOIN result_meta m ON m.id = l.id
             GROUP BY l.id
             ORDER BY MAX(l.timestamp) DESC
             LIMIT ?1",
        ) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        let rows = stmt.query_map(params![limit as i64], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
            ))
        });

        let Ok(rows) = rows else {
            return Vec::new();
        };

        rows.filter_map(|r| r.ok())
            .map(|(id, kind, title, subtitle, icon)| {
                let kind = match kind.as_str() {
                    "file" => ResultKind::File,
                    "setting" => ResultKind::Setting,
                    _ => ResultKind::App,
                };
                SearchResult {
                    id: id.clone(),
                    kind,
                    title,
                    subtitle: if subtitle.is_empty() {
                        None
                    } else {
                        Some(subtitle)
                    },
                    icon: if icon.is_empty() { None } else { Some(icon) },
                    score: 1.0,
                    match_ranges: Vec::new(),
                }
            })
            .collect()
    }

    pub fn save_meta(&self, id: &str, title: &str, subtitle: Option<&str>, icon: Option<&str>) {
        let Ok(conn) = self.conn.lock() else {
            return;
        };
        let _ = conn.execute(
            "INSERT INTO result_meta (id, title, subtitle, icon) VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(id) DO UPDATE SET title=?2, subtitle=?3, icon=?4",
            params![id, title, subtitle.unwrap_or(""), icon.unwrap_or("")],
        );
    }
}

fn db_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("spotlight").join("history.db"))
}

fn open_connection() -> rusqlite::Result<Connection> {
    let path = db_path().ok_or(rusqlite::Error::InvalidQuery)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    Connection::open(path)
}

fn init_schema(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS launches (
            id TEXT NOT NULL,
            kind TEXT NOT NULL,
            timestamp INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS access_counts (
            id TEXT PRIMARY KEY,
            count INTEGER NOT NULL DEFAULT 0
        );
        CREATE TABLE IF NOT EXISTS result_meta (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            subtitle TEXT,
            icon TEXT
        );
        CREATE TABLE IF NOT EXISTS file_cache (
            path TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            parent TEXT NOT NULL,
            mtime INTEGER NOT NULL
        );",
    )
}
