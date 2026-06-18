use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use serde::Serialize;
use std::path::PathBuf;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

const MAX_ENTRIES: usize = 100;
const PREVIEW_LEN: usize = 90;

#[derive(Clone, Serialize)]
pub struct ClipboardItem {
    pub id: String,
    pub preview: String,
    pub subtitle: String,
}

pub struct ClipboardDb {
    conn: Mutex<Connection>,
}

impl ClipboardDb {
    pub fn new() -> Self {
        let conn = open_connection().expect("failed to open clipboard db");
        init_schema(&conn).expect("failed to init clipboard schema");
        Self {
            conn: Mutex::new(conn),
        }
    }

    pub fn clone_for_watcher(&self) -> Self {
        Self::new()
    }

    pub fn insert(&self, content: &str) -> bool {
        let trimmed = content.trim();
        if trimmed.is_empty() || trimmed.len() > 200_000 {
            return false;
        }

        let Ok(conn) = self.conn.lock() else {
            return false;
        };

        let last: Option<String> = conn
            .query_row(
                "SELECT content FROM clipboard_entries ORDER BY timestamp DESC LIMIT 1",
                [],
                |row| row.get(0),
            )
            .ok();

        if last.as_deref() == Some(trimmed) {
            return false;
        }

        let id = format!("{}-{}", Utc::now().timestamp_millis(), trimmed.len());
        let now = Utc::now().timestamp();

        if conn
            .execute(
                "INSERT INTO clipboard_entries (id, content, timestamp) VALUES (?1, ?2, ?3)",
                params![id, trimmed, now],
            )
            .is_err()
        {
            return false;
        }

        let _ = conn.execute(
            "DELETE FROM clipboard_entries WHERE id NOT IN (
                SELECT id FROM clipboard_entries ORDER BY timestamp DESC LIMIT ?1
            )",
            params![MAX_ENTRIES as i64],
        );

        true
    }

    pub fn get_content(&self, key: &str) -> Option<String> {
        let conn = self.conn.lock().ok()?;
        conn.query_row(
            "SELECT content FROM clipboard_entries WHERE id = ?1",
            params![key],
            |row| row.get(0),
        )
        .ok()
    }

    pub fn list_recent(&self, limit: usize) -> Vec<ClipboardItem> {
        let Ok(conn) = self.conn.lock() else {
            return Vec::new();
        };

        let mut stmt = match conn.prepare(
            "SELECT id, content, timestamp FROM clipboard_entries ORDER BY timestamp DESC LIMIT ?1",
        ) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        let rows = match stmt.query_map(params![limit as i64], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
            ))
        }) {
            Ok(r) => r,
            Err(_) => return Vec::new(),
        };

        rows.filter_map(|row| row.ok())
            .map(|(id, content, ts)| ClipboardItem {
                preview: make_preview(&content),
                subtitle: make_subtitle(&content, ts),
                id,
            })
            .collect()
    }
}

pub fn write_to_clipboard(text: &str) -> Result<(), String> {
    arboard::Clipboard::new()
        .map_err(|e| e.to_string())?
        .set_text(text.to_string())
        .map_err(|e| e.to_string())
}

pub fn start_watcher(db: ClipboardDb) {
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_millis(600));
            if let Ok(mut clipboard) = arboard::Clipboard::new() {
                if let Ok(text) = clipboard.get_text() {
                    db.insert(&text);
                }
            }
        }
    });
}

fn open_connection() -> rusqlite::Result<Connection> {
    let path = db_path().ok_or(rusqlite::Error::InvalidPath(PathBuf::from("")))?;
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    Connection::open(path)
}

fn db_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("spotlight").join("history.db"))
}

fn init_schema(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS clipboard_entries (
            id TEXT PRIMARY KEY,
            content TEXT NOT NULL,
            timestamp INTEGER NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_clipboard_ts ON clipboard_entries(timestamp DESC);",
    )
}

fn make_preview(content: &str) -> String {
    let line = content.lines().next().unwrap_or(content).trim();
    if line.chars().count() > PREVIEW_LEN {
        let truncated: String = line.chars().take(PREVIEW_LEN).collect();
        format!("{truncated}…")
    } else {
        line.to_string()
    }
}

fn make_subtitle(content: &str, ts: i64) -> String {
    let lines = content.lines().count();
    let chars = content.chars().count();
    let time = format_relative_time(ts);
    if lines > 1 {
        format!("{lines} linhas · {chars} caracteres · {time}")
    } else {
        format!("{chars} caracteres · {time}")
    }
}

fn format_relative_time(ts: i64) -> String {
    let dt = DateTime::from_timestamp(ts, 0).unwrap_or_else(Utc::now);
    let mins = (Utc::now() - dt).num_minutes();
    if mins < 1 {
        "agora".to_string()
    } else if mins < 60 {
        format!("há {mins} min")
    } else {
        let hours = mins / 60;
        if hours < 24 {
            format!("há {hours} h")
        } else {
            format!("há {} dias", hours / 24)
        }
    }
}
