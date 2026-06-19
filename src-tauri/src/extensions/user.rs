use std::fs;
use std::path::PathBuf;
use std::process::Command;

use serde::Deserialize;

use crate::extensions::{ExtensionManifest, SearchProvider};
use crate::history::HistoryDb;
use crate::search::ranking::build_result;
use crate::search::types::{make_id, ResultKind, SearchResult};

#[derive(Debug, Deserialize)]
struct ScriptSearchHit {
    title: String,
    #[serde(default)]
    subtitle: Option<String>,
    #[serde(default)]
    action: Option<String>,
    #[serde(default)]
    icon: Option<String>,
}

pub struct UserScriptExtension {
    manifest: ExtensionManifest,
    dir: PathBuf,
}

impl UserScriptExtension {
    pub fn new(manifest: ExtensionManifest, dir: PathBuf) -> Self {
        Self { manifest, dir }
    }

    fn search_command(&self) -> Option<PathBuf> {
        self.manifest
            .search_command
            .as_ref()
            .map(|c| self.dir.join(c))
    }

    fn run_command(&self) -> Option<PathBuf> {
        self.manifest
            .run_command
            .as_ref()
            .map(|c| self.dir.join(c))
    }
}

impl SearchProvider for UserScriptExtension {
    fn id(&self) -> &str {
        &self.manifest.id
    }

    fn title(&self) -> &str {
        &self.manifest.title
    }

    fn keywords(&self) -> &[String] {
        &self.manifest.keywords
    }

    fn search(&self, query: &str, history: &HistoryDb, limit: usize) -> Vec<SearchResult> {
        let Some(cmd_path) = self.search_command() else {
            if query.is_empty() {
                return vec![build_result(
                    make_id(ResultKind::Extension, &format!("{}:root", self.manifest.id)),
                    ResultKind::Extension,
                    self.manifest.title.clone(),
                    Some("Extensão de usuário".into()),
                    self.manifest.icon.clone(),
                    500,
                    query,
                    history,
                )];
            }
            return Vec::new();
        };

        if !cmd_path.exists() {
            return Vec::new();
        }

        let output = match Command::new(&cmd_path)
            .arg(query)
            .current_dir(&self.dir)
            .env("SPOTLIGHT_QUERY", query)
            .env("SPOTLIGHT_EXTENSION_ID", &self.manifest.id)
            .output()
        {
            Ok(o) if o.status.success() => o,
            _ => return Vec::new(),
        };

        let stdout = String::from_utf8_lossy(&output.stdout);
        let hits: Vec<ScriptSearchHit> = serde_json::from_str(&stdout).unwrap_or_default();

        hits.into_iter()
            .take(limit)
            .map(|hit| {
                let action = hit.action.unwrap_or_else(|| "default".into());
                build_result(
                    make_id(
                        ResultKind::Extension,
                        &format!("{}:{}", self.manifest.id, action),
                    ),
                    ResultKind::Extension,
                    hit.title,
                    hit.subtitle,
                    hit.icon.or_else(|| self.manifest.icon.clone()),
                    750,
                    query,
                    history,
                )
            })
            .collect()
    }

    fn run(&self, action_id: &str, args: &str) -> Result<String, String> {
        let Some(cmd_path) = self.run_command() else {
            return Ok(String::new());
        };
        if !cmd_path.exists() {
            return Err("Script run_command não encontrado".into());
        }

        let output = Command::new(&cmd_path)
            .args([action_id, args])
            .current_dir(&self.dir)
            .env("SPOTLIGHT_ACTION", action_id)
            .env("SPOTLIGHT_ARGS", args)
            .env("SPOTLIGHT_EXTENSION_ID", &self.manifest.id)
            .output()
            .map_err(|e| e.to_string())?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
        }
    }
}

pub fn load_user_providers() -> Vec<(ExtensionManifest, PathBuf)> {
    load_user_extensions(false)
}

pub fn load_all_user_extensions() -> Vec<(ExtensionManifest, PathBuf)> {
    load_user_extensions(true)
}

fn load_user_extensions(include_disabled: bool) -> Vec<(ExtensionManifest, PathBuf)> {
    let dirs = extension_search_dirs();
    let mut out = Vec::new();

    for dir in dirs {
        if !dir.exists() {
            let _ = std::fs::create_dir_all(&dir);
            continue;
        }
        let Ok(read) = std::fs::read_dir(&dir) else {
            continue;
        };
        for item in read.flatten() {
            if !item.path().is_dir() {
                continue;
            }
            let manifest_path = item.path().join("manifest.json");
            if !manifest_path.exists() {
                continue;
            }
            let Ok(content) = std::fs::read_to_string(&manifest_path) else {
                continue;
            };
            if let Ok(m) = serde_json::from_str::<ExtensionManifest>(&content) {
                if include_disabled || m.enabled {
                    out.push((m, item.path()));
                }
            }
        }
    }
    out
}

pub fn set_user_extension_enabled(id: &str, enabled: bool) -> Result<(), String> {
    for dir in extension_search_dirs() {
        if !dir.exists() {
            continue;
        }
        let ext_dir = dir.join(id);
        let manifest_path = ext_dir.join("manifest.json");
        if !manifest_path.exists() {
            continue;
        }
        let content = fs::read_to_string(&manifest_path).map_err(|e| e.to_string())?;
        let mut manifest: ExtensionManifest =
            serde_json::from_str(&content).map_err(|e| e.to_string())?;
        manifest.enabled = enabled;
        let updated = serde_json::to_string_pretty(&manifest).map_err(|e| e.to_string())?;
        fs::write(&manifest_path, updated).map_err(|e| e.to_string())?;
        return Ok(());
    }
    Err(format!("Extensão de usuário '{id}' não encontrada"))
}

fn extension_search_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(d) = crate::paths::extensions_dir() {
        dirs.push(d);
    }
    for d in crate::config::load().extension_dirs {
        if !d.is_empty() {
            dirs.push(PathBuf::from(d));
        }
    }
    dirs
}

pub fn write_example_extension() {
    let Some(base) = crate::paths::extensions_dir() else {
        return;
    };
    let example = base.join("example-hello");
    if example.exists() {
        return;
    }
    let _ = std::fs::create_dir_all(&example);
    let _ = std::fs::write(
        example.join("manifest.json"),
        r#"{
  "id": "example-hello",
  "title": "Hello Extension",
  "icon": "face-smile",
  "keywords": ["hello", "oi"],
  "enabled": true,
  "search_command": "search.sh",
  "run_command": "run.sh"
}"#,
    );
    let _ = std::fs::write(
        example.join("search.sh"),
        r#"#!/bin/sh
echo '[{"title":"Olá do Spotlight!","subtitle":"Extensão de exemplo","action":"greet","icon":"face-smile"}]'
"#,
    );
    let _ = std::fs::write(
        example.join("run.sh"),
        r#"#!/bin/sh
notify-send "Spotlight" "Extensão example-hello: $1"
"#,
    );
    let _ = Command::new("chmod")
        .args(["+x", "search.sh", "run.sh"])
        .current_dir(&example)
        .status();
}
