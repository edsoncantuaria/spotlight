use crate::history::HistoryDb;
use crate::search::types::{ResultKind, SearchResult};

pub fn apply_history_boost(
    mut result: SearchResult,
    fuzzy_score: i64,
    history: &HistoryDb,
) -> SearchResult {
    let frequency = history.get_count(&result.id) as f64;
    let recency = history.recency_boost(&result.id);

    let fuzzy_norm = (fuzzy_score as f64 / 100.0).clamp(0.0, 1.0);
    let frequency_norm = (frequency / 20.0).min(1.0);
    let recency_norm = recency;

    result.score = fuzzy_norm * 0.6 + recency_norm * 0.25 + frequency_norm * 0.15;
    result
}

pub fn build_result(
    id: String,
    kind: ResultKind,
    title: String,
    subtitle: Option<String>,
    icon: Option<String>,
    fuzzy_score: i64,
    query: &str,
    history: &HistoryDb,
) -> SearchResult {
    let match_ranges = super::match_highlight::compute_match_ranges(&title, query);
    apply_history_boost(
        SearchResult {
            id,
            kind,
            title,
            subtitle,
            icon,
            score: 0.0,
            match_ranges,
        },
        fuzzy_score,
        history,
    )
}

pub fn sort_results(results: &mut [SearchResult]) {
    results.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
}
