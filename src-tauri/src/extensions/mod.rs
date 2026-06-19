use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use crate::config::AppConfig;
use crate::history::HistoryDb;
use crate::paths;
use crate::search::types::{ResultSection, SearchResponse};

pub mod builtin;
pub mod host;
pub mod user;

pub use host::ExtensionHost;

pub trait SearchProvider: Send + Sync {
    fn id(&self) -> &str;
    fn title(&self) -> &str;
    fn keywords(&self) -> &[String];
    fn search(&self, query: &str, history: &HistoryDb, limit: usize) -> Vec<crate::search::types::SearchResult>;
    fn run(&self, action_id: &str, args: &str) -> Result<String, String>;
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExtensionManifest {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub keywords: Vec<String>,
    #[serde(default)]
    pub permissions: Vec<String>,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub search_command: Option<String>,
    #[serde(default)]
    pub run_command: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExtensionInfo {
    pub id: String,
    pub title: String,
    pub icon: Option<String>,
    pub keywords: Vec<String>,
    pub enabled: bool,
    pub builtin: bool,
}

pub fn load_user_manifests() -> Vec<ExtensionManifest> {
    user::load_user_providers()
        .into_iter()
        .map(|(m, _)| m)
        .collect()
}

pub struct ExtensionRegistry {
    providers: Arc<RwLock<Vec<Box<dyn SearchProvider>>>>,
    manifests: Arc<RwLock<Vec<ExtensionInfo>>>,
}

impl ExtensionRegistry {
    pub fn new() -> Self {
        let registry = Self {
            providers: Arc::new(RwLock::new(Vec::new())),
            manifests: Arc::new(RwLock::new(Vec::new())),
        };
        registry.reload();
        registry
    }

    pub fn reload(&self) {
        user::write_example_extension();
        let mut providers: Vec<Box<dyn SearchProvider>> = Vec::new();
        let mut infos = Vec::new();
        let disabled = crate::config::load().disabled_extensions;

        for p in builtin::all_builtin() {
            let id = p.id().to_string();
            let enabled = !disabled.iter().any(|d| d == &id);
            infos.push(ExtensionInfo {
                id: id.clone(),
                title: p.title().to_string(),
                icon: Some("applications-other".to_string()),
                keywords: p.keywords().to_vec(),
                enabled,
                builtin: true,
            });
            if enabled {
                providers.push(p);
            }
        }

        for (m, dir) in user::load_all_user_extensions() {
            infos.push(ExtensionInfo {
                id: m.id.clone(),
                title: m.title.clone(),
                icon: m.icon.clone(),
                keywords: m.keywords.clone(),
                enabled: m.enabled,
                builtin: false,
            });
            if m.enabled {
                providers.push(Box::new(user::UserScriptExtension::new(m, dir)));
            }
        }

        if let Ok(mut p) = self.providers.write() {
            *p = providers;
        }
        if let Ok(mut i) = self.manifests.write() {
            *i = infos;
        }
    }

    pub fn list(&self) -> Vec<ExtensionInfo> {
        self.manifests.read().map(|m| m.clone()).unwrap_or_default()
    }

    pub fn set_enabled(&self, id: &str, enabled: bool) -> Result<(), String> {
        let is_builtin = self
            .manifests
            .read()
            .map(|m| m.iter().any(|e| e.builtin && e.id == id))
            .unwrap_or(false);

        if is_builtin {
            let mut config = crate::config::load();
            if enabled {
                config.disabled_extensions.retain(|x| x != id);
            } else if !config.disabled_extensions.iter().any(|x| x == id) {
                config.disabled_extensions.push(id.to_string());
            }
            crate::config::save(&config)?;
            self.reload();
            return Ok(());
        }

        user::set_user_extension_enabled(id, enabled)?;
        self.reload();
        Ok(())
    }

    fn should_search_provider(
        provider: &dyn SearchProvider,
        query: &str,
        root_keyword: Option<&str>,
    ) -> bool {
        if let Some(kw) = root_keyword {
            return provider.keywords().iter().any(|k| k == kw) || provider.id() == kw;
        }
        if query.len() < 2 {
            return false;
        }
        let q = query.to_lowercase();
        if provider.id().eq_ignore_ascii_case(&q) {
            return true;
        }
        provider.keywords().iter().any(|k| {
            let kl = k.to_lowercase();
            q.contains(&kl) || kl.starts_with(&q)
        })
    }

    pub fn search_all(
        &self,
        query: &str,
        history: &HistoryDb,
        limit: usize,
        root_keyword: Option<&str>,
    ) -> Vec<crate::search::types::SearchResult> {
        let providers = self.providers.read().ok();
        let Some(providers) = providers else {
            return Vec::new();
        };

        let mut all = Vec::new();
        for provider in providers.iter() {
            if !Self::should_search_provider(provider.as_ref(), query, root_keyword) {
                continue;
            }
            all.extend(provider.search(query, history, limit));
        }
        all.truncate(limit);
        all
    }

    pub fn run_extension(&self, extension_id: &str, action_id: &str, args: &str) -> Result<String, String> {
        let providers = self.providers.read().map_err(|e| e.to_string())?;
        let provider = providers
            .iter()
            .find(|p| p.id() == extension_id)
            .ok_or_else(|| "Extensão não encontrada".to_string())?;
        provider.run(action_id, args)
    }

    pub fn parse_root_search(query: &str) -> Option<(&str, &str)> {
        let q = query.trim();
        if let Some((kw, rest)) = q.split_once(' ') {
            if !kw.is_empty() && !rest.is_empty() {
                return Some((kw, rest.trim()));
            }
        }
        None
    }
}

pub fn backup_config() -> Result<PathBuf, String> {
    let dir = paths::spotlight_dir().ok_or_else(|| "Config dir not found".to_string())?;
    let backup = dir.join(format!("backup-{}.tar", chrono::Utc::now().format("%Y%m%d-%H%M%S")));
    let _ = backup;
    // Minimal backup: copy config files
    let backup_dir = dir.join("backups");
    fs::create_dir_all(&backup_dir).map_err(|e| e.to_string())?;
    let stamp = chrono::Utc::now().format("%Y%m%d-%H%M%S").to_string();
    let dest = backup_dir.join(&format!("backup-{stamp}"));
    fs::create_dir_all(&dest).map_err(|e| e.to_string())?;
    for name in ["config.toml", "quicklinks.toml", "snippets.toml"] {
        let src = dir.join(name);
        if src.exists() {
            fs::copy(&src, dest.join(name)).map_err(|e| e.to_string())?;
        }
    }
    Ok(dest)
}

pub fn export_config_snapshot() -> Result<AppConfig, String> {
    Ok(crate::config::load())
}
