use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use crate::paths;

const EMBEDDED_CATALOG: &str =
    include_str!("../../docs/extension-store/catalog.json");
const EMBEDDED_GUIDE: &str = include_str!("../../docs/extension-store/GUIA-EXTENSOES.md");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreExtension {
    pub id: String,
    pub title: String,
    pub description: String,
    pub repo: String,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub builtin: bool,
}

#[derive(Debug, Deserialize)]
struct StoreCatalog {
    #[serde(default)]
    extensions: Vec<StoreExtension>,
}

pub fn list_store() -> Result<Vec<StoreExtension>, String> {
    let url = store_catalog_url();
    if !url.starts_with("http") {
        return parse_catalog_file(url.trim_start_matches("file://"));
    }

    match fetch_remote_catalog(&url) {
        Ok(items) if !items.is_empty() => Ok(items),
        _ => parse_catalog_json(EMBEDDED_CATALOG),
    }
}

fn fetch_remote_catalog(url: &str) -> Result<Vec<StoreExtension>, String> {
    let body = reqwest::blocking::Client::builder()
        .user_agent("Spotlight/1.0")
        .timeout(std::time::Duration::from_secs(8))
        .build()
        .map_err(|e| e.to_string())?
        .get(url)
        .send()
        .map_err(|e| format!("Falha ao buscar loja: {e}"))?
        .text()
        .map_err(|e| e.to_string())?;
    parse_catalog_json(&body)
}

fn store_catalog_url() -> String {
    crate::config::load()
        .extension_store_url
        .unwrap_or_else(default_store_url)
}

fn default_store_url() -> String {
    "https://raw.githubusercontent.com/edsoncantuaria/spotlight/main/docs/extension-store/catalog.json"
        .to_string()
}

fn parse_catalog_file(path: &str) -> Result<Vec<StoreExtension>, String> {
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    parse_catalog_json(&content)
}

fn parse_catalog_json(content: &str) -> Result<Vec<StoreExtension>, String> {
    let catalog: StoreCatalog =
        serde_json::from_str(content).map_err(|e| format!("Catálogo inválido: {e}"))?;
    Ok(catalog.extensions)
}

pub fn install_from_store(ext: &StoreExtension) -> Result<String, String> {
    if ext.builtin {
        return install_builtin_example(ext);
    }

    let dest_name = ext.id.clone();
    let Some(extensions_dir) = paths::extensions_dir() else {
        return Err("Diretório de extensões indisponível".into());
    };
    fs::create_dir_all(&extensions_dir).map_err(|e| e.to_string())?;
    let dest = extensions_dir.join(&dest_name);

    if dest.exists() {
        return Err(format!("Extensão '{dest_name}' já instalada"));
    }

    let repo_url = format!("https://github.com/{}.git", ext.repo);

    if Command::new("git")
        .args([
            "clone",
            "--depth",
            "1",
            &repo_url,
            dest.to_string_lossy().as_ref(),
        ])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
    {
        return Ok(dest.to_string_lossy().to_string());
    }

    install_via_archive(&ext.repo, &dest)
}

fn install_builtin_example(ext: &StoreExtension) -> Result<String, String> {
    let Some(extensions_dir) = paths::extensions_dir() else {
        return Err("Diretório de extensões indisponível".into());
    };
    fs::create_dir_all(&extensions_dir).map_err(|e| e.to_string())?;
    let dest = extensions_dir.join(&ext.id);

    if dest.exists() {
        return Ok(dest.to_string_lossy().to_string());
    }

    crate::extensions::user::write_example_extension();
    let example = extensions_dir.join("example-hello");
    if example.exists() && ext.id != "example-hello" {
        copy_dir_recursive(&example, &dest)?;
    } else if example.exists() {
        return Ok(example.to_string_lossy().to_string());
    }

    Err("Exemplo builtin não encontrado".into())
}

fn install_via_archive(repo: &str, dest: &PathBuf) -> Result<String, String> {
    let url = format!("https://github.com/{repo}/archive/refs/heads/main.tar.gz");
    let tmp = std::env::temp_dir().join(format!("spotlight-ext-{repo}.tar.gz"));
    let bytes = reqwest::blocking::get(&url)
        .map_err(|e| format!("Download falhou: {e}"))?
        .bytes()
        .map_err(|e| e.to_string())?;
    fs::write(&tmp, &bytes).map_err(|e| e.to_string())?;

    fs::create_dir_all(dest).map_err(|e| e.to_string())?;
    let status = Command::new("tar")
        .args([
            "-xzf",
            tmp.to_string_lossy().as_ref(),
            "-C",
            dest.to_string_lossy().as_ref(),
            "--strip-components=1",
        ])
        .status()
        .map_err(|e| format!("tar não disponível: {e}"))?;

    if !status.success() {
        let _ = fs::remove_dir_all(dest);
        return Err("Falha ao extrair extensão".into());
    }

    make_scripts_executable(dest);
    let _ = fs::remove_file(tmp);
    Ok(dest.to_string_lossy().to_string())
}

fn copy_dir_recursive(src: &PathBuf, dest: &PathBuf) -> Result<(), String> {
    fs::create_dir_all(dest).map_err(|e| e.to_string())?;
    for entry in fs::read_dir(src).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let ty = entry.file_type().map_err(|e| e.to_string())?;
        let to = dest.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_recursive(&entry.path(), &to)?;
        } else {
            fs::copy(entry.path(), to).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn make_scripts_executable(dir: &PathBuf) {
    let Ok(read) = fs::read_dir(dir) else {
        return;
    };
    for entry in read.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("sh") {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Ok(meta) = fs::metadata(&path) {
                    let mut perms = meta.permissions();
                    perms.set_mode(0o755);
                    let _ = fs::set_permissions(&path, perms);
                }
            }
        }
    }
}

pub fn guide_path() -> Option<PathBuf> {
    let candidates = [
        PathBuf::from("docs/extension-store/GUIA-EXTENSOES.md"),
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../docs/extension-store/GUIA-EXTENSOES.md"),
    ];
    candidates.into_iter().find(|p| p.exists())
}

pub fn read_guide() -> String {
    guide_path()
        .and_then(|p| fs::read_to_string(p).ok())
        .unwrap_or_else(|| EMBEDDED_GUIDE.to_string())
}
