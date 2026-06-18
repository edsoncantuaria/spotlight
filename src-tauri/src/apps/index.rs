use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use crate::history::HistoryDb;
use crate::search::ranking::build_result;
use crate::search::types::{make_id, ResultKind, SearchResult};

#[derive(Clone)]
pub struct AppEntry {
    pub id: String,
    pub name: String,
    pub exec: String,
    pub icon: Option<String>,
    pub desktop_path: String,
    pub comment: Option<String>,
}

pub struct AppIndex {
    apps: Vec<AppEntry>,
}

static EXEC_FIELD_CODES: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"%[fFuUick]").unwrap());

impl AppIndex {
    pub fn new() -> Self {
        let apps = scan_applications();
        Self { apps }
    }

    pub fn get_by_id(&self, id: &str) -> Option<&AppEntry> {
        self.apps.iter().find(|app| app.id == id)
    }

    pub fn search_results(
        &self,
        query: &str,
        history: &HistoryDb,
        limit: usize,
    ) -> Vec<SearchResult> {
        let query = query.trim();
        let matcher = SkimMatcherV2::default();

        let mut results: Vec<SearchResult> = if query.is_empty() {
            self.apps
                .iter()
                .take(limit)
                .map(|app| {
                    build_result(
                        make_id(ResultKind::App, &app.id),
                        ResultKind::App,
                        app.name.clone(),
                        app.comment.clone(),
                        app.icon.clone(),
                        50,
                        query,
                        history,
                    )
                })
                .collect()
        } else {
            self.apps
                .iter()
                .filter_map(|app| {
                    let score = matcher.fuzzy_match(&app.name, query)?;
                    Some(build_result(
                        make_id(ResultKind::App, &app.id),
                        ResultKind::App,
                        app.name.clone(),
                        app.comment.clone(),
                        app.icon.clone(),
                        score,
                        query,
                        history,
                    ))
                })
                .collect()
        };

        crate::search::ranking::sort_results(&mut results);
        results.into_iter().take(limit).collect()
    }
}

fn scan_applications() -> Vec<AppEntry> {
    let mut apps = Vec::new();
    let mut dirs = vec![
        PathBuf::from("/usr/share/applications"),
        PathBuf::from("/usr/local/share/applications"),
    ];

    if let Some(home) = dirs::home_dir() {
        dirs.push(home.join(".local/share/applications"));
    }

    for dir in dirs {
        if !dir.exists() {
            continue;
        }
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("desktop") {
                    if let Some(app) = parse_desktop_file(&path) {
                        apps.push(app);
                    }
                }
            }
        }
    }

    apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    apps.dedup_by(|a, b| a.exec == b.exec && a.name == b.name);
    apps
}

fn parse_desktop_file(path: &Path) -> Option<AppEntry> {
    let content = fs::read_to_string(path).ok()?;
    let mut name = None;
    let mut exec = None;
    let mut icon = None;
    let mut comment = None;
    let mut categories = String::new();
    let mut no_display = false;
    let mut hidden = false;
    let mut in_desktop_entry = false;
    let mut is_application = false;

    for line in content.lines() {
        let line = line.trim();
        if line == "[Desktop Entry]" {
            in_desktop_entry = true;
            continue;
        }
        if line.starts_with('[') && line != "[Desktop Entry]" {
            in_desktop_entry = false;
        }
        if !in_desktop_entry {
            continue;
        }

        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim();
            match key {
                "Type" => is_application = value == "Application",
                "Name" => name = Some(value.to_string()),
                "Exec" => exec = Some(normalize_exec(value)),
                "Icon" => icon = Some(value.to_string()),
                "Comment" => comment = Some(value.to_string()),
                "Categories" => categories = value.to_string(),
                "NoDisplay" => no_display = value == "true",
                "Hidden" => hidden = value == "true",
                "X-GNOME-Settings-Panel" => return None,
                _ => {}
            }
        }
    }

    if !is_application || no_display || hidden {
        return None;
    }

    if categories.to_lowercase().contains("settings") {
        return None;
    }

    let name = name?;
    let exec = exec?;
    if exec.is_empty() {
        return None;
    }

    let id = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_string();

    let icon_path = icon.as_deref().and_then(resolve_icon_path);

    Some(AppEntry {
        id,
        name,
        exec,
        icon: icon_path,
        desktop_path: path.to_string_lossy().to_string(),
        comment,
    })
}

fn normalize_exec(exec: &str) -> String {
    EXEC_FIELD_CODES
        .replace_all(exec, "")
        .trim()
        .to_string()
}

fn resolve_icon_path(icon: &str) -> Option<String> {
    let path = Path::new(icon);
    if path.is_absolute() && path.exists() {
        return Some(icon.to_string());
    }

    let search_dirs = [
        "/usr/share/pixmaps",
        "/usr/share/icons/hicolor/scalable/apps",
        "/usr/share/icons/hicolor/48x48/apps",
        "/usr/share/icons/hicolor/32x32/apps",
        "/usr/share/icons/Adwaita/scalable/legacy",
        "/usr/share/icons/Adwaita/48x48/legacy",
    ];

    for dir in search_dirs {
        for ext in ["", ".png", ".svg", ".xpm"] {
            let candidate = format!("{dir}/{icon}{ext}");
            if Path::new(&candidate).exists() {
                return Some(candidate);
            }
        }
    }

    None
}
