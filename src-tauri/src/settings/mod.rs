use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use regex::Regex;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::LazyLock;

use crate::history::HistoryDb;
use crate::search::ranking::{self, build_result};
use crate::search::types::{make_id, ResultKind, SearchResult};

#[derive(Clone, Serialize)]
pub struct SettingEntry {
    pub id: String,
    pub name: String,
    pub panel: String,
    pub icon: Option<String>,
}

pub struct SettingsIndex {
    entries: Vec<SettingEntry>,
}

static EXEC_FIELD_CODES: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"%[fFuUick]").unwrap());

impl SettingsIndex {
    pub fn new() -> Self {
        let mut entries = scan_settings_desktops();
        entries.extend(static_gnome_panels());
        entries.sort_by(|a, b| a.name.cmp(&b.name));
        entries.dedup_by(|a, b| a.panel == b.panel);
        Self { entries }
    }

    pub fn get(&self, panel: &str) -> Option<&SettingEntry> {
        self.entries.iter().find(|e| e.panel == panel || e.id == panel)
    }

    pub fn search(&self, query: &str, history: &HistoryDb, limit: usize) -> Vec<SearchResult> {
        let query = query.trim();
        if query.is_empty() {
            return Vec::new();
        }

        let matcher = SkimMatcherV2::default();
        let mut results: Vec<SearchResult> = self
            .entries
            .iter()
            .filter_map(|entry| {
                let score = matcher.fuzzy_match(&entry.name, query)?;
                let id = make_id(ResultKind::Setting, &entry.panel);
                Some(build_result(
                    id,
                    ResultKind::Setting,
                    entry.name.clone(),
                    Some("Configurações do sistema".to_string()),
                    entry.icon.clone(),
                    score,
                    query,
                    history,
                ))
            })
            .collect();

        ranking::sort_results(&mut results);
        results.into_iter().take(limit).collect()
    }
}

pub fn open_setting(panel: &str) -> Result<(), String> {
    if Command::new("gnome-control-center")
        .arg(panel)
        .spawn()
        .is_ok()
    {
        return Ok(());
    }
    if Command::new("systemsettings")
        .arg(panel)
        .spawn()
        .is_ok()
    {
        return Ok(());
    }
    if Command::new("kcmshell6")
        .arg(panel)
        .spawn()
        .is_ok()
    {
        return Ok(());
    }
    Command::new("xdg-open")
        .arg(format!("gnome-control-center-{panel}.desktop"))
        .spawn()
        .map_err(|e| format!("Falha ao abrir configuração: {e}"))?;
    Ok(())
}

fn scan_settings_desktops() -> Vec<SettingEntry> {
    let mut entries = Vec::new();
    let dirs = [
        PathBuf::from("/usr/share/applications"),
        PathBuf::from("/usr/local/share/applications"),
    ];

    for dir in dirs {
        if !dir.exists() {
            continue;
        }
        if let Ok(read) = fs::read_dir(&dir) {
            for entry in read.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) != Some("desktop") {
                    continue;
                }
                if let Some(item) = parse_settings_desktop(&path) {
                    entries.push(item);
                }
            }
        }
    }
    entries
}

fn parse_settings_desktop(path: &Path) -> Option<SettingEntry> {
    let content = fs::read_to_string(path).ok()?;
    let mut name = None;
    let mut exec = None;
    let mut icon = None;
    let mut categories = String::new();
    let mut is_settings = false;

    for line in content.lines() {
        let line = line.trim();
        if let Some((key, value)) = line.split_once('=') {
            match key.trim() {
                "Name" => name = Some(value.trim().to_string()),
                "Exec" => exec = Some(value.trim().to_string()),
                "Icon" => icon = Some(value.trim().to_string()),
                "Categories" => categories = value.trim().to_string(),
                "X-GNOME-Settings-Panel" => {
                    is_settings = true;
                    exec = Some(value.trim().to_string());
                }
                _ => {}
            }
        }
    }

    if !is_settings && !categories.to_lowercase().contains("settings") {
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        if !stem.starts_with("kcm_") && !stem.starts_with("kcmshell") {
            return None;
        }
        is_settings = true;
    }

    let name = name?;
    let panel = exec
        .as_ref()
        .map(|e| EXEC_FIELD_CODES.replace_all(e, "").trim().to_string())
        .filter(|s| !s.is_empty())
        .or_else(|| {
            path.file_stem()
                .and_then(|s| s.to_str())
                .map(|s| s.replace("gnome-control-center-", ""))
        })?;

    let id = panel.clone();
    Some(SettingEntry {
        id,
        name,
        panel,
        icon: icon.and_then(|i| resolve_icon(&i)),
    })
}

fn static_gnome_panels() -> Vec<SettingEntry> {
    [
        ("wifi", "Wi-Fi"),
        ("network", "Rede"),
        ("bluetooth", "Bluetooth"),
        ("background", "Plano de fundo"),
        ("notifications", "Notificações"),
        ("display", "Tela"),
        ("sound", "Som"),
        ("power", "Energia"),
        ("privacy", "Privacidade e segurança"),
        ("info", "Sobre"),
        ("default-apps", "Aplicativos padrão"),
        ("datetime", "Data e hora"),
        ("keyboard", "Teclado"),
        ("mouse", "Mouse"),
        ("region", "Região e idioma"),
        ("ubuntu", "Ubuntu"),
    ]
    .into_iter()
    .map(|(panel, name)| SettingEntry {
        id: panel.to_string(),
        name: name.to_string(),
        panel: panel.to_string(),
        icon: Some("preferences-system".to_string()),
    })
    .collect()
}

fn resolve_icon(icon: &str) -> Option<String> {
    let candidates = [
        format!("/usr/share/icons/hicolor/scalable/apps/{icon}.svg"),
        format!("/usr/share/pixmaps/{icon}.png"),
    ];
    candidates.into_iter().find(|p| Path::new(p).exists())
}
