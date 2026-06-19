use crate::extensions::SearchProvider;
use crate::history::HistoryDb;
use crate::search::ranking::build_result;
use crate::search::types::{make_id, ResultKind, SearchResult};

pub struct TranslateExtension;

impl SearchProvider for TranslateExtension {
    fn id(&self) -> &str {
        "translate"
    }

    fn title(&self) -> &str {
        "Traduzir"
    }

    fn keywords(&self) -> &[String] {
        static KW: std::sync::OnceLock<Vec<String>> = std::sync::OnceLock::new();
        KW.get_or_init(|| vec!["tr ".to_string(), "translate".to_string()])
    }

    fn search(&self, query: &str, history: &HistoryDb, limit: usize) -> Vec<SearchResult> {
        let text = query
            .strip_prefix("tr ")
            .or_else(|| query.strip_prefix("translate "))
            .unwrap_or(query)
            .trim();
        if text.is_empty() {
            return Vec::new();
        }
        vec![build_result(
            make_id(ResultKind::Extension, &format!("translate:{text}")),
            ResultKind::Extension,
            format!("Traduzir \"{text}\""),
            Some("LibreTranslate / configurável".to_string()),
            Some("preferences-desktop-locale".to_string()),
            900,
            query,
            history,
        )]
        .into_iter()
        .take(limit)
        .collect()
    }

    fn run(&self, action_id: &str, _args: &str) -> Result<String, String> {
        let text = action_id.strip_prefix("translate:").unwrap_or(action_id);
        let config = crate::config::load();
        if let Some(url) = &config.translate_api_url {
            let client = reqwest::blocking::Client::new();
            let resp = client
                .post(format!("{url}/translate"))
                .json(&serde_json::json!({
                    "q": text,
                    "source": "auto",
                    "target": config.translate_target,
                }))
                .send()
                .map_err(|e| e.to_string())?;
            let body: serde_json::Value = resp.json().map_err(|e| e.to_string())?;
            let translated = body["translatedText"].as_str().unwrap_or(text);
            crate::clipboard::write_to_clipboard(translated)?;
            return Ok(translated.to_string());
        }
        crate::browser::open_url(&format!(
            "https://translate.google.com/?sl=auto&tl={}&text={}",
            config.translate_target,
            urlencoding_simple(text)
        ))?;
        Ok("Abrindo tradutor".to_string())
    }
}

fn urlencoding_simple(s: &str) -> String {
    s.chars()
        .map(|c| if c == ' ' { '+' } else { c })
        .collect()
}
