use super::types::MatchRange;

pub fn compute_match_ranges(text: &str, query: &str) -> Vec<MatchRange> {
    if query.is_empty() {
        return Vec::new();
    }

    let text_chars: Vec<(usize, char)> = text.char_indices().collect();
    let query_lower: Vec<char> = query.to_lowercase().chars().collect();
    let mut ranges = Vec::new();
    let mut qi = 0;
    let mut match_start: Option<usize> = None;
    let mut match_end: Option<usize> = None;

    for (byte_idx, ch) in &text_chars {
        if qi < query_lower.len() && ch.to_lowercase().next() == Some(query_lower[qi]) {
            if match_start.is_none() {
                match_start = Some(*byte_idx);
            }
            let ch_len = ch.len_utf8();
            match_end = Some(byte_idx + ch_len);
            qi += 1;
            if qi == query_lower.len() {
                if let (Some(start), Some(end)) = (match_start, match_end) {
                    ranges.push(MatchRange { start, end });
                }
                break;
            }
        }
    }

    ranges
}
