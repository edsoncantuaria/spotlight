use std::path::PathBuf;

pub fn spotlight_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("spotlight"))
}

pub fn config_file() -> Option<PathBuf> {
    spotlight_dir().map(|d| d.join("config.toml"))
}

pub fn quicklinks_file() -> Option<PathBuf> {
    spotlight_dir().map(|d| d.join("quicklinks.toml"))
}

pub fn snippets_file() -> Option<PathBuf> {
    spotlight_dir().map(|d| d.join("snippets.toml"))
}

pub fn scripts_dir() -> Option<PathBuf> {
    spotlight_dir().map(|d| d.join("scripts"))
}

pub fn extensions_dir() -> Option<PathBuf> {
    spotlight_dir().map(|d| d.join("extensions"))
}

pub fn clipboard_images_dir() -> Option<PathBuf> {
    spotlight_dir().map(|d| d.join("clipboard-images"))
}

pub fn rates_cache_file() -> Option<PathBuf> {
    spotlight_dir().map(|d| d.join("rates.json"))
}
