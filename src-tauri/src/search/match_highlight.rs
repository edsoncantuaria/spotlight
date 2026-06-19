use super::types::MatchRange;

pub fn compute_match_ranges(text: &str, query: &str) -> Vec<MatchRange> {
    let query = query.trim();
    if query.is_empty() {
        return Vec::new();
    }

    let text_lower = text.to_lowercase();
    let query_lower = query.to_lowercase();

    let Some(start) = text_lower.find(&query_lower) else {
        return Vec::new();
    };

    vec![MatchRange {
        start,
        end: start + query_lower.len(),
    }]
}
