use chrono::{DateTime, Utc};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use rusqlite::{params, Connection};
use serde::Serialize;
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use crate::history::HistoryDb;
use crate::paths;
use crate::search::ranking::{self, build_result};
use crate::search::types::{make_id, ResultKind, SearchResult};

const PREVIEW_LEN: usize = 90;

fn item_limit() -> usize {
    crate::config::clipboard_limit()
}

#[derive(Clone, Serialize)]
pub struct ClipboardItem {
    pub id: String,
    pub preview: String,
    pub subtitle: String,
    pub content_type: String,
    pub preview_image: Option<String>,
    pub pinned: bool,
}

#[derive(Clone, Copy, Default)]
pub enum ClipboardFilter {
    #[default]
    All,
    Text,
    Image,
    Pinned,
}

struct ClipboardEntry {
    id: String,
    content: String,
    content_type: String,
    image_width: u32,
    image_height: u32,
    content_hash: String,
}

pub struct ClipboardDb {
    conn: Mutex<Connection>,
    last_hash: Mutex<Option<String>>,
    paste_stack: Mutex<Vec<String>>,
}

impl ClipboardDb {
    pub fn new() -> Self {
        let conn = open_connection().expect("failed to open clipboard db");
        init_schema(&conn).expect("failed to init clipboard schema");
        if let Some(dir) = paths::clipboard_images_dir() {
            let _ = fs::create_dir_all(dir);
        }
        prune_excess(&conn, item_limit());
        Self {
            conn: Mutex::new(conn),
            last_hash: Mutex::new(None),
            paste_stack: Mutex::new(Vec::new()),
        }
    }

    pub fn clone_for_watcher(&self) -> Self {
        Self::new()
    }

    pub fn insert_text(&self, text: &str) -> bool {
        let trimmed = text.trim();
        if trimmed.is_empty() || trimmed.len() > 200_000 {
            return false;
        }
        let hash = hash_str(trimmed);
        if self.is_duplicate(&hash) {
            return false;
        }
        let id = new_id();
        let Ok(conn) = self.conn.lock() else {
            return false;
        };
        if insert_row(
            &conn,
            &id,
            trimmed,
            "text",
            0,
            0,
            &hash,
            Utc::now().timestamp(),
        )
        .is_err()
        {
            return false;
        }
        prune_excess(&conn, item_limit());
        self.remember_hash(hash);
        true
    }

    pub fn insert_image(&self, width: usize, height: usize, bytes: &[u8]) -> bool {
        if bytes.is_empty() || bytes.len() > 8_000_000 {
            return false;
        }
        let hash = format!("img:{:x}", hash_bytes(bytes));
        if self.is_duplicate(&hash) {
            return false;
        }

        let id = new_id();
        let Some((image_path, img_w, img_h)) = save_image_files(&id, width, height, bytes) else {
            return false;
        };
        let path_str = image_path.to_string_lossy().to_string();

        let Ok(conn) = self.conn.lock() else {
            cleanup_image_files(&image_path);
            return false;
        };
        if insert_row(
            &conn,
            &id,
            &path_str,
            "image",
            img_w,
            img_h,
            &hash,
            Utc::now().timestamp(),
        )
        .is_err()
        {
            cleanup_image_files(&image_path);
            return false;
        }
        prune_excess(&conn, item_limit());
        self.remember_hash(hash);
        true
    }

    fn is_duplicate(&self, hash: &str) -> bool {
        self.last_hash
            .lock()
            .ok()
            .and_then(|g| g.clone())
            .as_deref()
            == Some(hash)
    }

    fn remember_hash(&self, hash: String) {
        if let Ok(mut guard) = self.last_hash.lock() {
            *guard = Some(hash);
        }
    }

    pub fn apply_limit(&self) {
        if let Ok(conn) = self.conn.lock() {
            prune_excess(&conn, item_limit());
        }
    }

    pub fn get_entry(&self, key: &str) -> Option<ClipboardEntry> {
        let conn = self.conn.lock().ok()?;
        conn.query_row(
            "SELECT id, content, content_type, image_width, image_height, content_hash
             FROM clipboard_entries WHERE id = ?1",
            params![key],
            |row| {
                Ok(ClipboardEntry {
                    id: row.get(0)?,
                    content: row.get(1)?,
                    content_type: row.get(2)?,
                    image_width: row.get::<_, i64>(3)? as u32,
                    image_height: row.get::<_, i64>(4)? as u32,
                    content_hash: row.get(5)?,
                })
            },
        )
        .ok()
    }

    pub fn get_content(&self, key: &str) -> Option<String> {
        let entry = self.get_entry(key)?;
        if entry.content_type == "image" {
            return Some(entry.content);
        }
        Some(entry.content)
    }

    pub fn get_type(&self, key: &str) -> Option<String> {
        self.get_entry(key).map(|e| e.content_type)
    }

    pub fn preview_image_path(&self, key: &str) -> Option<String> {
        let png = self.ensure_png_path(key)?;
        png_data_url(&png)
    }

    fn ensure_png_path(&self, key: &str) -> Option<PathBuf> {
        let entry = self.get_entry(key)?;
        if entry.content_type != "image" {
            return None;
        }
        let png = png_path_for(&entry.content);
        if png.exists() {
            return Some(png);
        }
        if !Path::new(&entry.content).exists() {
            return None;
        }
        let bytes = fs::read(&entry.content).ok()?;
        let rgba = normalize_rgba(entry.image_width as usize, entry.image_height as usize, &bytes)?;
        let img = image::RgbaImage::from_raw(entry.image_width, entry.image_height, rgba)?;
        img.save(&png).ok()?;
        Some(png)
    }

    pub fn search(&self, query: &str, history: &HistoryDb, limit: usize) -> Vec<SearchResult> {
        self.list_entries(item_limit(), ClipboardFilter::All)
            .into_iter()
            .filter_map(|item| {
                let q = query.trim().to_lowercase();
                if !q.is_empty() {
                    let haystack = format!("{} {}", item.preview, item.subtitle).to_lowercase();
                    if !haystack.contains(&q) {
                        let matcher = SkimMatcherV2::default();
                        if matcher
                            .fuzzy_match(&format!("{} {}", item.preview, item.subtitle), query)
                            .is_none()
                        {
                            return None;
                        }
                    }
                }
                let icon = if item.content_type == "image" {
                    "image-x-generic"
                } else {
                    "edit-copy"
                };
                Some(build_result(
                    make_id(ResultKind::Clipboard, &item.id),
                    ResultKind::Clipboard,
                    item.preview,
                    Some(item.subtitle),
                    Some(icon.to_string()),
                    800,
                    query,
                    history,
                ))
            })
            .take(limit.min(item_limit()))
            .collect()
    }

    pub fn list_recent(&self, limit: usize, filter: ClipboardFilter) -> Vec<ClipboardItem> {
        self.list_entries(limit.min(item_limit()), filter)
    }

    pub fn toggle_pin(&self, id: &str) -> Result<bool, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let pinned: i64 = conn
            .query_row(
                "SELECT pinned FROM clipboard_entries WHERE id = ?1",
                params![id],
                |row| row.get(0),
            )
            .map_err(|_| "Item não encontrado".to_string())?;
        let next = if pinned == 0 { 1 } else { 0 };
        conn.execute(
            "UPDATE clipboard_entries SET pinned = ?1 WHERE id = ?2",
            params![next, id],
        )
        .map_err(|e| e.to_string())?;
        Ok(next == 1)
    }

    pub fn add_to_paste_stack(&self, id: &str) -> Result<usize, String> {
        if self.get_entry(id).is_none() {
            return Err("Item não encontrado".into());
        }
        let mut stack = self.paste_stack.lock().map_err(|e| e.to_string())?;
        if !stack.contains(&id.to_string()) {
            stack.push(id.to_string());
        }
        Ok(stack.len())
    }

    pub fn paste_stack_count(&self) -> usize {
        self.paste_stack.lock().map(|s| s.len()).unwrap_or(0)
    }

    pub fn clear_paste_stack(&self) {
        if let Ok(mut stack) = self.paste_stack.lock() {
            stack.clear();
        }
    }

    pub fn paste_stack(&self) -> Result<(), String> {
        let ids: Vec<String> = self
            .paste_stack
            .lock()
            .map(|s| s.clone())
            .unwrap_or_default();
        if ids.is_empty() {
            return Err("Stack vazio".into());
        }

        let mut parts = Vec::new();
        for id in &ids {
            if let Some(entry) = self.get_entry(id) {
                if entry.content_type == "text" {
                    parts.push(entry.content);
                }
            }
        }
        if parts.is_empty() {
            return Err("Stack só contém imagens — copie uma por vez".into());
        }
        write_to_clipboard(&parts.join("\n\n"))?;
        std::thread::spawn(|| {
            let _ = crate::input::simulate_paste();
        });
        if let Ok(mut stack) = self.paste_stack.lock() {
            stack.clear();
        }
        Ok(())
    }

    fn list_entries(&self, limit: usize, filter: ClipboardFilter) -> Vec<ClipboardItem> {
        let Ok(conn) = self.conn.lock() else {
            return Vec::new();
        };
        let mut stmt = match conn.prepare(
            "SELECT id, content, timestamp, content_type, image_width, image_height, byte_len, pinned
             FROM clipboard_entries
             ORDER BY pinned DESC, timestamp DESC
             LIMIT ?1",
        ) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };
        let rows = match stmt.query_map(params![limit as i64], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, i64>(4)?,
                row.get::<_, i64>(5)?,
                row.get::<_, i64>(6)?,
                row.get::<_, i64>(7)?,
            ))
        }) {
            Ok(r) => r,
            Err(_) => return Vec::new(),
        };

        rows.filter_map(|row| row.ok())
            .filter_map(|(id, content, ts, content_type, w, h, byte_len, pinned)| {
                let pinned = pinned != 0;
                match filter {
                    ClipboardFilter::Text if content_type != "text" => return None,
                    ClipboardFilter::Image if content_type != "image" => return None,
                    ClipboardFilter::Pinned if !pinned => return None,
                    _ => {}
                }
                let preview_image = if content_type == "image" {
                    self.ensure_png_path(&id).and_then(|p| png_data_url(&p))
                } else {
                    None
                };
                Some(ClipboardItem {
                    preview: make_preview(&content_type, &content, byte_len),
                    subtitle: make_subtitle(ts, &content_type, byte_len, w, h),
                    id,
                    content_type,
                    preview_image,
                    pinned,
                })
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

pub fn copy_item_to_clipboard(db: &ClipboardDb, id: &str) -> Result<(), String> {
    let entry = db
        .get_entry(id)
        .ok_or_else(|| "Item não encontrado".to_string())?;

    let mut clipboard = arboard::Clipboard::new().map_err(|e| e.to_string())?;

    if entry.content_type == "image" {
        let bytes = fs::read(&entry.content).map_err(|e| e.to_string())?;
        clipboard
            .set_image(arboard::ImageData {
                width: entry.image_width as usize,
                height: entry.image_height as usize,
                bytes: bytes.into(),
            })
            .map_err(|e| e.to_string())?;
        return Ok(());
    }

    clipboard
        .set_text(entry.content)
        .map_err(|e| e.to_string())
}

pub fn start_watcher(db: ClipboardDb) {
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_millis(500));
            let Ok(mut clipboard) = arboard::Clipboard::new() else {
                continue;
            };

            if let Ok(img) = clipboard.get_image() {
                if !img.bytes.is_empty() {
                    db.insert_image(img.width, img.height, &img.bytes);
                    continue;
                }
            }

            if let Ok(text) = clipboard.get_text() {
                db.insert_text(&text);
            }
        }
    });
}

fn insert_row(
    conn: &Connection,
    id: &str,
    content: &str,
    content_type: &str,
    image_width: u32,
    image_height: u32,
    content_hash: &str,
    timestamp: i64,
) -> rusqlite::Result<()> {
    let byte_len = if content_type == "image" {
        fs::metadata(content).map(|m| m.len()).unwrap_or(0) as i64
    } else {
        content.len() as i64
    };
    conn.execute(
        "INSERT INTO clipboard_entries
         (id, content, timestamp, content_type, image_width, image_height, byte_len, content_hash)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            id,
            content,
            timestamp,
            content_type,
            image_width as i64,
            image_height as i64,
            byte_len,
            content_hash,
        ],
    )?;
    Ok(())
}

fn prune_excess(conn: &Connection, limit: usize) {
    let Ok(mut stmt) = conn.prepare(
        "SELECT id, content, content_type FROM clipboard_entries
         WHERE pinned = 0
         ORDER BY timestamp DESC
         LIMIT -1 OFFSET ?1",
    ) else {
        return;
    };
    let rows = stmt
        .query_map(params![limit as i64], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })
        .ok();

    if let Some(rows) = rows {
        for row in rows.flatten() {
            let (id, content, content_type) = row;
            if content_type == "image" {
                let _ = fs::remove_file(&content);
                let _ = fs::remove_file(png_path_for(&content));
            }
            let _ = conn.execute(
                "DELETE FROM clipboard_entries WHERE id = ?1",
                params![id],
            );
        }
    }
}

fn save_image_files(
    id: &str,
    width: usize,
    height: usize,
    bytes: &[u8],
) -> Option<(PathBuf, u32, u32)> {
    let dir = paths::clipboard_images_dir()?;
    fs::create_dir_all(&dir).ok()?;

    let (img_w, img_h, rgba) = if is_png_bytes(bytes) {
        let img = image::load_from_memory(bytes).ok()?;
        let rgba_img = img.to_rgba8();
        (rgba_img.width(), rgba_img.height(), rgba_img.into_raw())
    } else {
        let rgba = normalize_rgba(width, height, bytes)?;
        (width as u32, height as u32, rgba)
    };

    if img_w == 0 || img_h == 0 {
        return None;
    }

    let rgba_path = dir.join(format!("{id}.rgba"));
    let png_path = dir.join(format!("{id}.png"));
    fs::write(&rgba_path, &rgba).ok()?;

    let img = image::RgbaImage::from_raw(img_w, img_h, rgba)?;
    img.save(&png_path).ok()?;

    Some((rgba_path, img_w, img_h))
}

fn cleanup_image_files(rgba_path: &Path) {
    let _ = fs::remove_file(rgba_path);
    let _ = fs::remove_file(png_path_for(rgba_path.to_string_lossy().as_ref()));
}

fn is_png_bytes(bytes: &[u8]) -> bool {
    bytes.len() >= 8 && bytes.starts_with(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A])
}

fn normalize_rgba(width: usize, height: usize, bytes: &[u8]) -> Option<Vec<u8>> {
    if width == 0 || height == 0 {
        return None;
    }
    let needed = width.checked_mul(height)?.checked_mul(4)?;
    if bytes.len() == needed {
        return Some(bytes.to_vec());
    }
    if is_png_bytes(bytes) {
        return Some(image::load_from_memory(bytes).ok()?.to_rgba8().into_raw());
    }
    let stride = bytes.len() / height;
    if stride < width * 4 {
        return None;
    }
    let mut out = Vec::with_capacity(needed);
    for row in 0..height {
        let start = row * stride;
        out.extend_from_slice(&bytes[start..start + width * 4]);
    }
    Some(out)
}

fn png_data_url(path: &Path) -> Option<String> {
    let bytes = fs::read(path).ok()?;
    Some(format!("data:image/png;base64,{}", base64_encode(&bytes)))
}

fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((data.len() + 2) / 3 * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(CHARS[((n >> 18) & 63) as usize] as char);
        out.push(CHARS[((n >> 12) & 63) as usize] as char);
        out.push(if chunk.len() > 1 {
            CHARS[((n >> 6) & 63) as usize] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            CHARS[(n & 63) as usize] as char
        } else {
            '='
        });
    }
    out
}

fn png_path_for(rgba_path: &str) -> PathBuf {
    Path::new(rgba_path).with_extension("png")
}

fn new_id() -> String {
    format!("clip-{}", Utc::now().timestamp_millis())
}

fn hash_str(s: &str) -> String {
    format!("{:x}", hash_bytes(s.as_bytes()))
}

fn hash_bytes(data: &[u8]) -> u64 {
    let mut h = DefaultHasher::new();
    data.hash(&mut h);
    h.finish()
}

fn open_connection() -> rusqlite::Result<Connection> {
    let path = db_path().ok_or(rusqlite::Error::InvalidPath(PathBuf::from("")))?;
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    Connection::open(path)
}

fn db_path() -> Option<PathBuf> {
    paths::spotlight_dir().map(|d| d.join("history.db"))
}

fn init_schema(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS clipboard_entries (
            id TEXT PRIMARY KEY,
            content TEXT NOT NULL,
            timestamp INTEGER NOT NULL,
            content_type TEXT NOT NULL DEFAULT 'text',
            image_width INTEGER NOT NULL DEFAULT 0,
            image_height INTEGER NOT NULL DEFAULT 0,
            byte_len INTEGER NOT NULL DEFAULT 0,
            content_hash TEXT NOT NULL DEFAULT ''
        );
        CREATE INDEX IF NOT EXISTS idx_clipboard_ts ON clipboard_entries(timestamp DESC);",
    )?;
    migrate_schema(conn)
}

fn migrate_schema(conn: &Connection) -> rusqlite::Result<()> {
    let cols: Vec<String> = conn
        .prepare("PRAGMA table_info(clipboard_entries)")?
        .query_map([], |row| row.get::<_, String>(1))?
        .filter_map(|r| r.ok())
        .collect();

    if !cols.contains(&"pinned".to_string()) {
        let _ = conn.execute(
            "ALTER TABLE clipboard_entries ADD COLUMN pinned INTEGER NOT NULL DEFAULT 0",
            [],
        );
    }

    migrate_schema_legacy(conn)
}

fn migrate_schema_legacy(conn: &Connection) -> rusqlite::Result<()> {
    let cols: Vec<String> = conn
        .prepare("PRAGMA table_info(clipboard_entries)")?
        .query_map([], |row| row.get::<_, String>(1))?
        .filter_map(|r| r.ok())
        .collect();

    if !cols.contains(&"content_type".to_string()) {
        let _ = conn.execute(
            "ALTER TABLE clipboard_entries ADD COLUMN content_type TEXT NOT NULL DEFAULT 'text'",
            [],
        );
    }
    if !cols.contains(&"image_width".to_string()) {
        let _ = conn.execute(
            "ALTER TABLE clipboard_entries ADD COLUMN image_width INTEGER NOT NULL DEFAULT 0",
            [],
        );
    }
    if !cols.contains(&"image_height".to_string()) {
        let _ = conn.execute(
            "ALTER TABLE clipboard_entries ADD COLUMN image_height INTEGER NOT NULL DEFAULT 0",
            [],
        );
    }
    if !cols.contains(&"byte_len".to_string()) {
        let _ = conn.execute(
            "ALTER TABLE clipboard_entries ADD COLUMN byte_len INTEGER NOT NULL DEFAULT 0",
            [],
        );
    }
    if !cols.contains(&"content_hash".to_string()) {
        let _ = conn.execute(
            "ALTER TABLE clipboard_entries ADD COLUMN content_hash TEXT NOT NULL DEFAULT ''",
            [],
        );
    }
    Ok(())
}

fn make_preview(content_type: &str, content: &str, byte_len: i64) -> String {
    if content_type == "image" {
        let kb = (byte_len.max(0) as f64 / 1024.0).round() as i64;
        return format!("Imagem ({kb} KB)");
    }
    let line = content.lines().next().unwrap_or(content).trim();
    if line.chars().count() > PREVIEW_LEN {
        let truncated: String = line.chars().take(PREVIEW_LEN).collect();
        format!("{truncated}…")
    } else {
        line.to_string()
    }
}

fn make_subtitle(ts: i64, content_type: &str, byte_len: i64, w: i64, h: i64) -> String {
    let time = format_relative_time(ts);
    if content_type == "image" {
        return format!("{w}×{h} · {time}");
    }
    let chars = byte_len.max(0);
    format!("{chars} caracteres · {time}")
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
