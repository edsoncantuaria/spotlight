use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

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

fn fd_binary() -> &'static str {
    static BIN: OnceLock<&'static str> = OnceLock::new();
    BIN.get_or_init(|| {
        if Command::new("fdfind")
            .arg("--version")
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
        {
            "fdfind"
        } else {
            "fd"
        }
    })
}

pub fn search_files(query: &str, history: &HistoryDb, limit: usize) -> Vec<SearchResult> {
    let query = query.trim();
    if query.len() < 3 {
        return Vec::new();
    }

    let indexed = crate::file_index::search_index(query, history, limit);
    if indexed.len() >= limit {
        return indexed;
    }

    // Só consulta o disco quando o índice não preenche e a query tem 4+ caracteres.
    if query.len() < 4 {
        return indexed;
    }

    let remaining = limit - indexed.len();
    let mut paths = search_with_fd(query, remaining).unwrap_or_default();
    let existing: std::collections::HashSet<String> = indexed.iter().map(|r| r.id.clone()).collect();
    paths.retain(|p| !existing.contains(&make_id(ResultKind::File, &p.to_string_lossy())));

    let matcher = SkimMatcherV2::default();
    let mut results: Vec<SearchResult> = paths
        .into_iter()
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
                file_icon(&path),
                score,
                query,
                history,
            ))
        })
        .collect();

    ranking::sort_results(&mut results);
    let mut combined = indexed;
    combined.extend(results.into_iter().take(remaining));
    combined
}

fn search_with_fd(query: &str, limit: usize) -> Option<Vec<PathBuf>> {
    let home = dirs::home_dir()?;
    let mut results = Vec::new();
    let bin = fd_binary();

    for dir in file_search_roots(&home) {
        if results.len() >= limit {
            break;
        }
        let remaining = limit - results.len();
        let output = Command::new(bin)
            .args([
                "-i",
                query,
                "--type",
                "f",
                "--max-results",
                &remaining.to_string(),
                "--exclude",
                ".git",
                "--exclude",
                "node_modules",
                "--exclude",
                "target",
                "--exclude",
                ".cache",
            ])
            .arg(&dir)
            .output()
            .ok()?;

        if output.status.success() {
            for line in String::from_utf8_lossy(&output.stdout).lines() {
                let p = PathBuf::from(line.trim());
                if p.exists() {
                    results.push(p);
                }
            }
        }
    }

    if results.is_empty() {
        None
    } else {
        Some(results)
    }
}

fn file_search_roots(home: &Path) -> Vec<PathBuf> {
    [
        "Documents",
        "Downloads",
        "Desktop",
        "Pictures",
        "Music",
        "Videos",
        "Projects",
        "Projetos",
    ]
    .into_iter()
    .map(|sub| home.join(sub))
    .filter(|p| p.exists())
    .collect()
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
