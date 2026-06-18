use chrono::Utc;
use chrono_tz::Tz;
use regex::Regex;
use std::sync::LazyLock;

use super::QuickAnswer;

static TIME_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^(?:time in|hora em|horário em)\s+(.+)$").unwrap()
});

static CITIES: &[(&str, &str)] = &[
    ("tokyo", "Asia/Tokyo"),
    ("toquio", "Asia/Tokyo"),
    ("london", "Europe/London"),
    ("londres", "Europe/London"),
    ("new york", "America/New_York"),
    ("nova york", "America/New_York"),
    ("paris", "Europe/Paris"),
    ("sao paulo", "America/Sao_Paulo"),
    ("são paulo", "America/Sao_Paulo"),
    ("los angeles", "America/Los_Angeles"),
    ("berlin", "Europe/Berlin"),
    ("sydney", "Australia/Sydney"),
    ("utc", "UTC"),
];

pub fn try_timezone(query: &str) -> Option<QuickAnswer> {
    let city = TIME_RE.captures(query.trim())?.get(1)?.as_str().trim().to_lowercase();
    let tz_name = CITIES
        .iter()
        .find(|(name, _)| *name == city)
        .map(|(_, tz)| *tz)?;

    let tz: Tz = tz_name.parse().ok()?;
    let now = Utc::now().with_timezone(&tz);
    let formatted = now.format("%H:%M").to_string();

    Some(QuickAnswer {
        kind: "timezone".to_string(),
        label: city.to_string(),
        value: formatted,
        hint: Some("Enter para copiar".to_string()),
    })
}
