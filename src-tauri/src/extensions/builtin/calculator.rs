use crate::extensions::SearchProvider;
use crate::history::HistoryDb;
use crate::search::types::SearchResult;

pub struct CalculatorExtension;

impl SearchProvider for CalculatorExtension {
    fn id(&self) -> &str {
        "calculator"
    }

    fn title(&self) -> &str {
        "Calculadora"
    }

    fn keywords(&self) -> &[String] {
        static KW: std::sync::OnceLock<Vec<String>> = std::sync::OnceLock::new();
        KW.get_or_init(|| vec!["=".to_string(), "calc".to_string()])
    }

    fn search(&self, query: &str, history: &HistoryDb, _limit: usize) -> Vec<SearchResult> {
        if let Some(qa) = crate::quick_answers::try_answer(query) {
            return vec![crate::search::ranking::build_result(
                crate::search::types::make_id(
                    crate::search::types::ResultKind::Extension,
                    &format!("calc:{}", qa.value),
                ),
                crate::search::types::ResultKind::Extension,
                format!("{} = {}", qa.label, qa.value),
                qa.hint,
                Some("accessories-calculator".to_string()),
                2000,
                query,
                history,
            )];
        }
        Vec::new()
    }

    fn run(&self, action_id: &str, _args: &str) -> Result<String, String> {
        if let Some(val) = action_id.strip_prefix("calc:") {
            crate::clipboard::write_to_clipboard(val)?;
            return Ok(val.to_string());
        }
        Err("Sem resultado".to_string())
    }
}
