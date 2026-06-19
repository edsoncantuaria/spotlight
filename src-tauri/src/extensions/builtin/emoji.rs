use crate::extensions::SearchProvider;
use crate::history::HistoryDb;
use crate::search::ranking::build_result;
use crate::search::types::{make_id, ResultKind, SearchResult};

pub struct EmojiExtension;

const EMOJIS: &[(&str, &str)] = &[
    ("😀", "grinning face smile feliz"),
    ("😂", "joy laugh rir"),
    ("❤️", "heart love amor coração"),
    ("👍", "thumbs up ok positivo"),
    ("🔥", "fire fogo quente"),
    ("✅", "check done feito"),
    ("🚀", "rocket foguete launch"),
    ("☕", "coffee café"),
    ("🐧", "penguin linux tux"),
    ("🎉", "party festa celebração"),
];

impl SearchProvider for EmojiExtension {
    fn id(&self) -> &str {
        "emoji"
    }

    fn title(&self) -> &str {
        "Emoji"
    }

    fn keywords(&self) -> &[String] {
        static KW: std::sync::OnceLock<Vec<String>> = std::sync::OnceLock::new();
        KW.get_or_init(|| vec!["emoji".to_string(), "e ".to_string()])
    }

    fn search(&self, query: &str, history: &HistoryDb, limit: usize) -> Vec<SearchResult> {
        let q = query.trim().to_lowercase();
        EMOJIS
            .iter()
            .filter(|(_, tags)| q.is_empty() || tags.to_lowercase().contains(&q) || q.contains(tags))
            .take(limit)
            .map(|(emoji, tags)| {
                build_result(
                    make_id(ResultKind::Extension, &format!("emoji:{emoji}")),
                    ResultKind::Extension,
                    format!("{emoji} {tags}"),
                    Some("Emoji".to_string()),
                    Some("face-smile".to_string()),
                    900,
                    query,
                    history,
                )
            })
            .collect()
    }

    fn run(&self, action_id: &str, _args: &str) -> Result<String, String> {
        let emoji = action_id.strip_prefix("emoji:").unwrap_or(action_id);
        crate::clipboard::write_to_clipboard(emoji)?;
        Ok(format!("Copiado {emoji}"))
    }
}
