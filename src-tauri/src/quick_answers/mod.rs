mod calculator;
mod conversion;
mod currency;
mod dictionary;
mod timezone;

use serde::Serialize;

#[derive(Clone, Serialize)]
pub struct QuickAnswer {
    pub kind: String,
    pub label: String,
    pub value: String,
    pub hint: Option<String>,
}

pub fn try_answer(query: &str) -> Option<QuickAnswer> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return None;
    }

    currency::try_convert(trimmed)
        .or_else(|| conversion::try_convert(trimmed))
        .or_else(|| calculator::try_evaluate(trimmed))
        .or_else(|| dictionary::try_define(trimmed))
        .or_else(|| timezone::try_timezone(trimmed))
}
