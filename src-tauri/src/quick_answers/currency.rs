use regex::Regex;
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};

use super::QuickAnswer;

static CURRENCY_CODE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^([\d.,]+)\s*([a-z]{3})\s+(?:to|in|em|para)\s+([a-z]{3})$").unwrap()
});

static CURRENCY_NL_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^([\d.,]+)\s+(.+?)\s+(?:to|in|em|para)\s+(.+)$").unwrap()
});

static CACHE: LazyLock<Mutex<Option<(Instant, HashMap<String, f64>)>>> =
    LazyLock::new(|| Mutex::new(None));

pub fn try_convert(query: &str) -> Option<QuickAnswer> {
    let query = query.trim();
    if query.is_empty() {
        return None;
    }

    if let Some(caps) = CURRENCY_CODE_RE.captures(query) {
        let amount = parse_amount(caps.get(1)?.as_str())?;
        let from = caps.get(2)?.as_str().to_uppercase();
        let to = caps.get(3)?.as_str().to_uppercase();
        return build_answer(amount, &from, &to);
    }

    if let Some(caps) = CURRENCY_NL_RE.captures(query) {
        let amount = parse_amount(caps.get(1)?.as_str())?;
        let from = resolve_currency(caps.get(2)?.as_str().trim())?;
        let to = resolve_currency(caps.get(3)?.as_str().trim())?;
        return build_answer(amount, &from, &to);
    }

    None
}

fn parse_amount(raw: &str) -> Option<f64> {
    let normalized = raw.replace(',', ".");
    normalized.parse().ok()
}

fn resolve_currency(token: &str) -> Option<String> {
    let t = token
        .trim()
        .to_lowercase()
        .replace('ó', "o")
        .replace('á', "a")
        .replace('é', "e")
        .replace('í', "i")
        .replace('ú', "u")
        .replace('ç', "c");

    let code = match t.as_str() {
        "usd" | "us$" | "dolar" | "dolares" | "dollar" | "dollars" | "dolar americano"
        | "dolares americanos" => "USD",
        "brl" | "r$" | "real" | "reais" | "real brasileiro" | "reais brasileiros" => "BRL",
        "eur" | "euro" | "euros" => "EUR",
        "gbp" | "libra" | "libras" | "pound" | "pounds" | "libra esterlina" => "GBP",
        "jpy" | "iene" | "ienes" | "yen" => "JPY",
        "cad" | "dolar canadense" | "dolares canadenses" => "CAD",
        "aud" | "dolar australiano" | "dolares australianes" => "AUD",
        "chf" | "franco suico" | "francos suicos" => "CHF",
        "cny" | "yuan" | "yuans" => "CNY",
        "ars" | "peso argentino" | "pesos argentinos" => "ARS",
        "mxn" | "peso mexicano" | "pesos mexicanos" => "MXN",
        other if other.len() == 3 && other.chars().all(|c| c.is_ascii_alphabetic()) => {
            return Some(other.to_uppercase());
        }
        _ => return None,
    };

    Some(code.to_string())
}

fn build_answer(amount: f64, from: &str, to: &str) -> Option<QuickAnswer> {
    let rate = fetch_rate(from, to)?;
    let result = amount * rate;

    Some(QuickAnswer {
        kind: "currency".to_string(),
        label: format!("{amount} {from} →"),
        value: format!("{result:.2} {to}"),
        hint: Some("Enter para copiar".to_string()),
    })
}

fn fetch_rate(from: &str, to: &str) -> Option<f64> {
    if from == to {
        return Some(1.0);
    }

    let cache_key = format!("{from}:{to}");
    if let Ok(cache) = CACHE.lock() {
        if let Some((time, rates)) = cache.as_ref() {
            if time.elapsed() < Duration::from_secs(3600) {
                if let Some(rate) = rates.get(&cache_key) {
                    return Some(*rate);
                }
            }
        }
    }

    let url = format!("https://api.frankfurter.app/latest?amount=1&from={from}&to={to}");
    let response = reqwest::blocking::get(&url).ok()?;
    if !response.status().is_success() {
        return None;
    }
    let json: serde_json::Value = response.json().ok()?;
    let rate = json.get("rates")?.get(to)?.as_f64()?;

    if let Ok(mut cache) = CACHE.lock() {
        let mut rates = cache.take().map(|(_, r)| r).unwrap_or_default();
        rates.insert(cache_key, rate);
        *cache = Some((Instant::now(), rates));
    }

    Some(rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_iso_codes() {
        assert!(try_convert("15 usd to brl").is_some());
        assert!(try_convert("15 USD para BRL").is_some());
    }

    #[test]
    fn parses_portuguese_names() {
        assert!(try_convert("15 dolares para reais").is_some());
        assert!(try_convert("15 dolares em reais").is_some());
        assert!(try_convert("100 reais para dolares").is_some());
    }
}
