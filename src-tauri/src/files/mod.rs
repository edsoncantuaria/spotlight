use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::history::HistoryDb;
use crate::search::ranking::{self, build_result};
use crate::search::types::{make_id, ResultKind, SearchResult};

const SKIP_DIRS: &[&str] = &[
    ".git",
    "node_modules",
    "target",
    ".cache",
    ".local/share/Trash",
];

pub fn search_files(query: &str, history: &HistoryDb, limit: usize) -> Vec<SearchResult> {
    let query = query.trim();
    if query.is_empty() {
        return Vec::new();
    }

    let paths = search_with_fd(query, limit * 3).unwrap_or_else(|| search_with_walk(query, limit * 3));

    let matcher = SkimMatcherV2::default();
    let mut results: Vec<SearchResult> = paths
        .par_iter()
        .filter_map(|path| {
            let name = path.file_name()?.to_str()?;
            let score = matcher.fuzzy_match(name, query)?;
            let parent = path.parent()?.to_string_lossy().to_string();
            let id = make_id(ResultKind::File, &path.to_string_lossy());
            Some(build_result(
                id,
                ResultKind::File,
                name.to_string(),
                Some(parent),
                file_icon(path),
                score,
                query,
                history,
            ))
        })
        .collect();

    ranking::sort_results(&mut results);
    results.into_iter().take(limit).collect()
}

fn search_with_fd(query: &str, limit: usize) -> Option<Vec<PathBuf>> {
    let home = dirs::home_dir()?;
    let output = Command::new("fd")
        .args([
            "-i",
            query,
            "--type",
            "f",
            "--max-results",
            &limit.to_string(),
            "--exclude",
            ".git",
            "--exclude",
            "node_modules",
            "--exclude",
            "target",
        ])
        .arg(&home)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    Some(
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter_map(|line| {
                let p = PathBuf::from(line.trim());
                if p.exists() { Some(p) } else { None }
            })
            .collect(),
    )
}

fn search_with_walk(query: &str, limit: usize) -> Vec<PathBuf> {
    let query_lower = query.to_lowercase();
    let mut roots = Vec::new();
    if let Some(home) = dirs::home_dir() {
        for sub in ["Documents", "Downloads", "Desktop"] {
            let p = home.join(sub);
            if p.exists() {
                roots.push(p);
            }
        }
    }

    let mut results = Vec::new();
    for root in roots {
        if results.len() >= limit {
            break;
        }
        walkdir::WalkDir::new(root)
            .max_depth(6)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter(|e| {
                let path = e.path();
                !should_skip(path)
            })
            .filter_map(|e| {
                let name = e.file_name().to_str()?;
                if name.to_lowercase().contains(&query_lower) {
                    Some(e.path().to_path_buf())
                } else {
                    None
                }
            })
            .take(limit - results.len())
            .for_each(|p| results.push(p));
    }
    results
}

fn should_skip(path: &Path) -> bool {
    path.components().any(|c| {
        c.as_os_str()
            .to_str()
            .map(|s| SKIP_DIRS.iter().any(|skip| s == *skip || s.starts_with('.')))
            .unwrap_or(false)
    })
}

fn file_icon(path: &Path) -> Option<String> {
    mime_guess::from_path(path)
        .first()
        .map(|_| path.to_string_lossy().to_string())
}

pub fn open_file(path: &str) -> Result<(), String> {
    Command::new("xdg-open")
        .arg(path)
        .spawn()
        .map_err(|e| format!("Falha ao abrir arquivo: {e}"))?;
    Ok(())
}

pub fn reveal_in_folder(path: &str) -> Result<(), String> {
    let parent = Path::new(path)
        .parent()
        .ok_or_else(|| "Caminho inválido".to_string())?;
    Command::new("xdg-open")
        .arg(parent)
        .spawn()
        .map_err(|e| format!("Falha ao abrir pasta: {e}"))?;
    Ok(())
}

pub fn read_preview_text(path: &str, max_lines: usize) -> Option<String> {
    let meta = std::fs::metadata(path).ok()?;
    if meta.len() > 512_000 {
        return None;
    }
    let content = std::fs::read_to_string(path).ok()?;
    Some(
        content
            .lines()
            .take(max_lines)
            .collect::<Vec<_>>()
            .join("\n"),
    )
}

pub fn is_image(path: &str) -> bool {
    mime_guess::from_path(path)
        .first()
        .map(|m| m.type_() == mime_guess::mime::IMAGE)
        .unwrap_or(false)
}
