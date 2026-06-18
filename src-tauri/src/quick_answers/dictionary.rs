use regex::Regex;
use std::sync::LazyLock;

use super::QuickAnswer;

static DEFINE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^(?:define|definir|o que é|what is)\s+(.+)$").unwrap()
});

pub fn try_define(query: &str) -> Option<QuickAnswer> {
    let word = DEFINE_RE.captures(query.trim())?.get(1)?.as_str().trim();
    if word.is_empty() || word.contains(' ') && word.split_whitespace().count() > 3 {
        return None;
    }

    let url = format!(
        "https://api.dictionaryapi.dev/api/v2/entries/en/{}",
        urlencoding_simple(word)
    );
    let response = reqwest::blocking::get(&url).ok()?;
    if !response.status().is_success() {
        return None;
    }
    let json: serde_json::Value = response.json().ok()?;
    let entry = json.as_array()?.first()?;
    let meaning = entry
        .get("meanings")?
        .as_array()?
        .first()?
        .get("definitions")?
        .as_array()?
        .first()?
        .get("definition")?
        .as_str()?;

    Some(QuickAnswer {
        kind: "definition".to_string(),
        label: word.to_string(),
        value: meaning.to_string(),
        hint: Some("Enter para copiar".to_string()),
    })
}

fn urlencoding_simple(s: &str) -> String {
    s.replace(' ', "%20")
}
