use rusqlite::{params, Connection};
use std::sync::Mutex;

use crate::extensions::SearchProvider;
use crate::history::HistoryDb;
use crate::paths;
use crate::search::ranking::build_result;
use crate::search::types::{make_id, ResultKind, SearchResult};

pub struct NotesExtension {
    conn: Mutex<Connection>,
}

impl NotesExtension {
    pub fn new() -> Self {
        let path = paths::spotlight_dir()
            .map(|d| d.join("notes.db"))
            .unwrap_or_default();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let conn = Connection::open(path).expect("notes db");
        let _ = conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS notes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                body TEXT NOT NULL,
                updated INTEGER NOT NULL
            );",
        );
        Self {
            conn: Mutex::new(conn),
        }
    }
}

impl SearchProvider for NotesExtension {
    fn id(&self) -> &str {
        "notes"
    }

    fn title(&self) -> &str {
        "Notas"
    }

    fn keywords(&self) -> &[String] {
        static KW: std::sync::OnceLock<Vec<String>> = std::sync::OnceLock::new();
        KW.get_or_init(|| vec!["notes".to_string(), "nota".to_string()])
    }

    fn search(&self, query: &str, history: &HistoryDb, limit: usize) -> Vec<SearchResult> {
        let Ok(conn) = self.conn.lock() else {
            return Vec::new();
        };
        let q = format!("%{query}%");
        let mut stmt = match conn.prepare(
            "SELECT id, title, body FROM notes WHERE title LIKE ?1 OR body LIKE ?1
             ORDER BY updated DESC LIMIT ?2",
        ) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };
        let rows = match stmt.query_map(params![q, limit as i64], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?))
        }) {
            Ok(r) => r,
            Err(_) => return Vec::new(),
        };
        rows.filter_map(|r| r.ok())
            .map(|(id, title, body)| {
                build_result(
                    make_id(ResultKind::Extension, &format!("notes:{id}")),
                    ResultKind::Extension,
                    title,
                    Some(body.chars().take(80).collect()),
                    Some("text-x-generic".to_string()),
                    850,
                    query,
                    history,
                )
            })
            .collect()
    }

    fn run(&self, action_id: &str, args: &str) -> Result<String, String> {
        if action_id == "new" {
            let conn = self.conn.lock().map_err(|e| e.to_string())?;
            let now = chrono::Utc::now().timestamp();
            conn.execute(
                "INSERT INTO notes (title, body, updated) VALUES (?1, ?2, ?3)",
                params![args, "", now],
            )
            .map_err(|e| e.to_string())?;
            return Ok("Nota criada".to_string());
        }
        if let Some(id) = action_id.strip_prefix("notes:") {
            let conn = self.conn.lock().map_err(|e| e.to_string())?;
            let body: String = conn
                .query_row("SELECT body FROM notes WHERE id = ?1", params![id], |row| row.get(0))
                .map_err(|e| e.to_string())?;
            crate::clipboard::write_to_clipboard(&body)?;
            return Ok("Nota copiada".to_string());
        }
        Err("Ação desconhecida".to_string())
    }
}
