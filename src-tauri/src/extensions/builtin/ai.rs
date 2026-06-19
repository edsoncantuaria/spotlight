use crate::extensions::SearchProvider;
use crate::history::HistoryDb;
use crate::search::ranking::build_result;
use crate::search::types::{make_id, ResultKind, SearchResult};

pub struct AiExtension {
    enabled: bool,
}

impl AiExtension {
    pub fn new() -> Self {
        let config = crate::config::load();
        Self {
            enabled: config.ai_enabled,
        }
    }
}

impl SearchProvider for AiExtension {
    fn id(&self) -> &str {
        "ai"
    }

    fn title(&self) -> &str {
        "Ask AI"
    }

    fn keywords(&self) -> &[String] {
        static KW: std::sync::OnceLock<Vec<String>> = std::sync::OnceLock::new();
        KW.get_or_init(|| vec!["ai".to_string(), "> ask".to_string()])
    }

    fn search(&self, query: &str, history: &HistoryDb, limit: usize) -> Vec<SearchResult> {
        if !self.enabled {
            return Vec::new();
        }
        let text = query
            .strip_prefix("ai ")
            .or_else(|| query.strip_prefix("> ask "))
            .unwrap_or(query)
            .trim();
        if text.is_empty() {
            return vec![build_result(
                make_id(ResultKind::Extension, "ai:prompt"),
                ResultKind::Extension,
                "Perguntar à IA…".to_string(),
                Some("Digite sua pergunta após 'ai '".to_string()),
                Some("dialog-information".to_string()),
                500,
                query,
                history,
            )];
        }
        vec![build_result(
            make_id(ResultKind::Extension, &format!("ai:{text}")),
            ResultKind::Extension,
            format!("Perguntar: {text}"),
            Some("Ollama / API".to_string()),
            Some("dialog-information".to_string()),
            950,
            query,
            history,
        )]
        .into_iter()
        .take(limit)
        .collect()
    }

    fn run(&self, action_id: &str, _args: &str) -> Result<String, String> {
        if !self.enabled {
            return Err("IA desabilitada nas configurações".to_string());
        }
        let prompt = action_id.strip_prefix("ai:").unwrap_or(action_id);
        let config = crate::config::load();

        if let Some(url) = &config.ai_api_url {
            let client = reqwest::blocking::Client::new();
            let resp = client
                .post(url)
                .json(&serde_json::json!({
                    "model": config.ai_model,
                    "prompt": prompt,
                    "stream": false
                }))
                .send()
                .map_err(|e| e.to_string())?;
            let body: serde_json::Value = resp.json().map_err(|e| e.to_string())?;
            let answer = body["response"]
                .as_str()
                .or_else(|| body["choices"][0]["message"]["content"].as_str())
                .unwrap_or("Sem resposta");
            crate::clipboard::write_to_clipboard(answer)?;
            return Ok(answer.to_string());
        }

        let ollama = config.ai_ollama_url.as_deref().unwrap_or("http://localhost:11434/api/generate");
        let client = reqwest::blocking::Client::new();
        let resp = client
            .post(ollama)
            .json(&serde_json::json!({
                "model": config.ai_model,
                "prompt": prompt,
                "stream": false
            }))
            .send()
            .map_err(|e| format!("Ollama indisponível: {e}"))?;
        let body: serde_json::Value = resp.json().map_err(|e| e.to_string())?;
        let answer = body["response"].as_str().unwrap_or("Sem resposta");
        crate::clipboard::write_to_clipboard(answer)?;
        Ok(answer.to_string())
    }
}
