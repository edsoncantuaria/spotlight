use std::process::Command;
use std::sync::OnceLock;

use crate::search::match_highlight;
use crate::search::types::{make_id, ResultKind, SearchResult};

static ENGINE_LABEL: OnceLock<String> = OnceLock::new();

pub fn search_web(query: &str) -> Option<SearchResult> {
    let query = query.trim();
    if query.len() < 2 {
        return None;
    }

    let label = ENGINE_LABEL.get_or_init(engine_label).clone();
    Some(SearchResult {
        id: make_id(ResultKind::Web, query),
        kind: ResultKind::Web,
        title: format!("Pesquisar \"{query}\" na web"),
        subtitle: Some(label),
        icon: Some("web-browser".to_string()),
        score: 1.0,
        match_ranges: match_highlight::compute_match_ranges(&format!("Pesquisar \"{query}\" na web"), query),
    })
}

pub fn open_web_search(query: &str) -> Result<(), String> {
    let engine = crate::config::load().web_search_engine;
    let url = search_url(query, &engine);
    Command::new("xdg-open")
        .arg(&url)
        .spawn()
        .map_err(|e| format!("Falha ao abrir busca: {e}"))?;
    Ok(())
}

pub fn search_url(query: &str, engine: &str) -> String {
    let encoded: String = query
        .chars()
        .flat_map(|c| match c {
            ' ' => "+".chars().collect::<Vec<_>>(),
            c if c.is_ascii_alphanumeric() || "-._~".contains(c) => vec![c],
            c => format!("%{:02X}", c as u8).chars().collect(),
        })
        .collect();

    match engine {
        "duckduckgo" => format!("https://duckduckgo.com/?q={encoded}"),
        "bing" => format!("https://www.bing.com/search?q={encoded}"),
        _ => format!("https://www.google.com/search?q={encoded}"),
    }
}

fn engine_label() -> String {
    match crate::config::load().web_search_engine.as_str() {
        "duckduckgo" => "DuckDuckGo".to_string(),
        "bing" => "Bing".to_string(),
        _ => "Google".to_string(),
    }
}

pub fn engine_label_cached() -> &'static str {
    ENGINE_LABEL.get_or_init(engine_label)
}
