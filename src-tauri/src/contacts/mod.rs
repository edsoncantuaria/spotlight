use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use rusqlite::Connection;
use std::path::Path;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};

use crate::history::HistoryDb;
use crate::search::ranking::{self, build_result};
use crate::search::types::{make_id, ResultKind, SearchResult};

#[derive(Clone, Debug)]
pub struct ContactEntry {
    pub id: String,
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
}

pub struct ContactsIndex {
    entries: Arc<RwLock<Vec<ContactEntry>>>,
    refreshing: Arc<AtomicBool>,
}

impl ContactsIndex {
    pub fn new() -> Self {
        let index = Self {
            entries: Arc::new(RwLock::new(Vec::new())),
            refreshing: Arc::new(AtomicBool::new(false)),
        };
        index.spawn_refresh();
        index
    }

    pub fn search(&self, query: &str, history: &HistoryDb, limit: usize) -> Vec<SearchResult> {
        let query = query.trim();
        if query.len() < 2 {
            return Vec::new();
        }

        let entries = match self.entries.read() {
            Ok(e) => e,
            Err(_) => return Vec::new(),
        };

        let query_lower = query.to_lowercase();
        let matcher = SkimMatcherV2::default();
        let mut results: Vec<SearchResult> = entries
            .iter()
            .filter_map(|c| {
                if !c.name.to_lowercase().contains(&query_lower) {
                    let extra = format!(
                        "{} {}",
                        c.email.as_deref().unwrap_or(""),
                        c.phone.as_deref().unwrap_or("")
                    )
                    .to_lowercase();
                    if !extra.contains(&query_lower) {
                        return None;
                    }
                }
                let haystack = format!(
                    "{} {} {}",
                    c.name,
                    c.email.as_deref().unwrap_or(""),
                    c.phone.as_deref().unwrap_or("")
                );
                let score = matcher.fuzzy_match(&haystack, query)?;
                let subtitle = c
                    .email
                    .clone()
                    .or_else(|| c.phone.clone())
                    .or(Some("Contato".to_string()));
                Some(build_result(
                    c.id.clone(),
                    ResultKind::Contact,
                    c.name.clone(),
                    subtitle,
                    Some("contact-new".to_string()),
                    score,
                    query,
                    history,
                ))
            })
            .collect();

        ranking::sort_results(&mut results);
        results.into_iter().take(limit).collect()
    }

    pub fn get(&self, id: &str) -> Option<ContactEntry> {
        self.entries.read().ok()?.iter().find(|c| c.id == id).cloned()
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
        let refreshing = Arc::clone(&self.refreshing);
        std::thread::spawn(move || {
            let mut collected = Vec::new();
            collected.extend(load_vcards());
            collected.extend(load_thunderbird());
            if let Ok(mut guard) = entries.write() {
                *guard = collected;
            }
            refreshing.store(false, Ordering::Release);
        });
    }
}

pub fn open_contact(entry: &ContactEntry) -> Result<(), String> {
    if let Some(email) = &entry.email {
        let url = format!("mailto:{email}");
        Command::new("xdg-open")
            .arg(&url)
            .spawn()
            .map_err(|e| format!("Falha ao abrir contato: {e}"))?;
        return Ok(());
    }
    if let Some(phone) = &entry.phone {
        let url = format!("tel:{phone}");
        Command::new("xdg-open")
            .arg(&url)
            .spawn()
            .map_err(|e| format!("Falha ao abrir contato: {e}"))?;
        return Ok(());
    }
    Err("Contato sem e-mail ou telefone".to_string())
}

fn load_vcards() -> Vec<ContactEntry> {
    let mut entries = Vec::new();
    let Some(home) = dirs::home_dir() else {
        return entries;
    };

    let roots = [
        home.join(".local/share/contacts"),
        home.join("contacts"),
        home.join(".contacts"),
    ];

    for root in roots {
        if root.exists() {
            walk_vcards(&root, &mut entries);
        }
    }
    entries
}

fn walk_vcards(dir: &Path, out: &mut Vec<ContactEntry>) {
    let Ok(read) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in read.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk_vcards(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("vcf") {
            if let Some(contact) = parse_vcard(&path) {
                out.push(contact);
            }
        }
    }
}

fn parse_vcard(path: &Path) -> Option<ContactEntry> {
    let content = std::fs::read_to_string(path).ok()?;
    let mut name = None;
    let mut email = None;
    let mut phone = None;

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("FN:") || line.starts_with("FN;") {
            name = Some(line.split(':').nth(1)?.trim().to_string());
        } else if line.starts_with("EMAIL") {
            email = Some(line.split(':').nth(1)?.trim().to_string());
        } else if line.starts_with("TEL") {
            phone = Some(line.split(':').nth(1)?.trim().to_string());
        }
    }

    let name = name?;
    let id = make_id(ResultKind::Contact, &path.to_string_lossy());
    Some(ContactEntry {
        id,
        name,
        email,
        phone,
    })
}

fn load_thunderbird() -> Vec<ContactEntry> {
    let Some(home) = dirs::home_dir() else {
        return Vec::new();
    };
    let base = home.join(".thunderbird");
    if !base.exists() {
        return Vec::new();
    }

    let mut entries = Vec::new();
    if let Ok(read) = std::fs::read_dir(&base) {
        for profile in read.flatten() {
            let path = profile.path();
            if !path.is_dir() {
                continue;
            }
            let db = path.join("abook.sqlite");
            if db.exists() {
                entries.extend(load_thunderbird_db(&db));
            }
        }
    }
    entries
}

fn load_thunderbird_db(path: &Path) -> Vec<ContactEntry> {
    let Some(temp) = crate::browser::copy_db_for_read(path) else {
        return Vec::new();
    };
    let Ok(conn) = Connection::open(&temp) else {
        let _ = std::fs::remove_file(&temp);
        return Vec::new();
    };
    let Ok(mut stmt) = conn.prepare(
        "SELECT id, display_name, primary_email FROM properties WHERE display_name IS NOT NULL",
    ) else {
        let _ = std::fs::remove_file(&temp);
        return Vec::new();
    };
    let Ok(rows) = stmt.query_map([], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, Option<String>>(2)?,
        ))
    }) else {
        let _ = std::fs::remove_file(&temp);
        return Vec::new();
    };

    let entries: Vec<ContactEntry> = rows
        .filter_map(|r| r.ok())
        .map(|(id, name, email)| ContactEntry {
            id: format!("contact:thunderbird:{id}"),
            name,
            email,
            phone: None,
        })
        .collect();

    let _ = std::fs::remove_file(temp);
    entries
}
