use regex::Regex;
use std::sync::LazyLock;

use super::QuickAnswer;

static CONVERSION_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^([\d.]+)\s*([a-z°]+)\s+(?:in|to|em|para)\s+([a-z°]+)$").unwrap()
});

pub fn try_convert(query: &str) -> Option<QuickAnswer> {
    let caps = CONVERSION_RE.captures(query.trim())?;
    let value: f64 = caps.get(1)?.as_str().parse().ok()?;
    let from = normalize_unit(caps.get(2)?.as_str());
    let to = normalize_unit(caps.get(3)?.as_str());

    let result = convert(value, &from, &to)?;
    Some(QuickAnswer {
        kind: "conversion".to_string(),
        label: format!("{value} {from} →"),
        value: format!("{result} {to}"),
        hint: Some("Enter para copiar".to_string()),
    })
}

fn normalize_unit(unit: &str) -> String {
    match unit.to_lowercase().as_str() {
        "km" | "kilometer" | "kilometers" | "quilometros" | "quilômetros" => "km".to_string(),
        "mi" | "mile" | "miles" | "milhas" => "mi".to_string(),
        "m" | "meter" | "meters" | "metros" => "m".to_string(),
        "ft" | "feet" | "foot" | "pes" => "ft".to_string(),
        "kg" | "kilogram" | "kilograms" | "quilos" => "kg".to_string(),
        "lb" | "lbs" | "pound" | "pounds" | "libras" => "lb".to_string(),
        "g" | "gram" | "grams" | "gramas" => "g".to_string(),
        "c" | "celsius" | "°c" => "c".to_string(),
        "f" | "fahrenheit" | "°f" => "f".to_string(),
        other => other.to_string(),
    }
}

fn convert(value: f64, from: &str, to: &str) -> Option<f64> {
    if from == to {
        return Some(value);
    }

    match (from, to) {
        ("km", "mi") => Some(value * 0.621371),
        ("mi", "km") => Some(value * 1.60934),
        ("m", "ft") => Some(value * 3.28084),
        ("ft", "m") => Some(value / 3.28084),
        ("kg", "lb") => Some(value * 2.20462),
        ("lb", "kg") => Some(value / 2.20462),
        ("g", "kg") => Some(value / 1000.0),
        ("kg", "g") => Some(value * 1000.0),
        ("c", "f") => Some(value * 9.0 / 5.0 + 32.0),
        ("f", "c") => Some((value - 32.0) * 5.0 / 9.0),
        _ => None,
    }
}
