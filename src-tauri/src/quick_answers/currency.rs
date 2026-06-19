use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use super::QuickAnswer;

static CURRENCY_CODE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^([\d.,]+)\s*([a-z]{3})\s+(?:to|in|em|para)\s+([a-z]{3})$").unwrap()
});

static CURRENCY_NL_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^([\d.,]+)\s+(.+?)\s+(?:to|in|em|para)\s+(.+)$").unwrap()
});

static CACHE: LazyLock<Mutex<Option<(Instant, HashMap<String, f64>)>>> =
    LazyLock::new(|| Mutex::new(None));

static HTTP: LazyLock<reqwest::blocking::Client> = LazyLock::new(|| {
    reqwest::blocking::Client::builder()
        .user_agent("Spotlight/1.0 (Linux)")
        .redirect(reqwest::redirect::Policy::limited(10))
        .timeout(Duration::from_secs(4))
        .build()
        .expect("http client")
});

pub fn warm_cache() {
    load_disk_cache();
    for (from, to) in [("USD", "BRL"), ("EUR", "BRL"), ("USD", "EUR"), ("GBP", "BRL")] {
        let _ = fetch_rate(from, to);
    }
}

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
    let Some(rate) = fetch_rate(from, to) else {
        return Some(QuickAnswer {
            kind: "currency".to_string(),
            label: format!("{amount} {from} →"),
            value: "Taxa indisponível".to_string(),
            hint: Some("Sem conexão — tente novamente".to_string()),
        });
    };
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

    if let Some(rate) = read_disk_rate(&cache_key) {
        store_rate(cache_key.clone(), rate);
        return Some(rate);
    }

    let rate = fetch_frankfurter(from, to).or_else(|| fetch_open_er_api(from, to))?;

    store_rate(cache_key, rate);
    Some(rate)
}

fn store_rate(cache_key: String, rate: f64) {
    if let Ok(mut cache) = CACHE.lock() {
        let mut rates = cache.take().map(|(_, r)| r).unwrap_or_default();
        rates.insert(cache_key.clone(), rate);
        *cache = Some((Instant::now(), rates));
    }
    write_disk_rate(&cache_key, rate);
}

#[derive(serde::Serialize, serde::Deserialize)]
struct DiskRates {
    updated_at: u64,
    rates: HashMap<String, f64>,
}

fn load_disk_cache() {
    let Some(path) = crate::paths::rates_cache_file() else {
        return;
    };
    let Ok(data) = fs::read_to_string(&path) else {
        return;
    };
    let Ok(disk) = serde_json::from_str::<DiskRates>(&data) else {
        return;
    };
    if unix_now().saturating_sub(disk.updated_at) > 86400 {
        return;
    }
    if let Ok(mut cache) = CACHE.lock() {
        *cache = Some((Instant::now(), disk.rates));
    }
}

fn read_disk_rate(key: &str) -> Option<f64> {
    let path = crate::paths::rates_cache_file()?;
    let data = fs::read_to_string(&path).ok()?;
    let disk: DiskRates = serde_json::from_str(&data).ok()?;
    if unix_now().saturating_sub(disk.updated_at) > 86400 {
        return None;
    }
    disk.rates.get(key).copied()
}

fn write_disk_rate(key: &str, rate: f64) {
    let Some(path) = crate::paths::rates_cache_file() else {
        return;
    };
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    let mut disk = fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str::<DiskRates>(&s).ok())
        .unwrap_or(DiskRates {
            updated_at: unix_now(),
            rates: HashMap::new(),
        });

    disk.updated_at = unix_now();
    disk.rates.insert(key.to_string(), rate);
    if let Ok(json) = serde_json::to_string_pretty(&disk) {
        let _ = fs::write(path, json);
    }
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn fetch_frankfurter(from: &str, to: &str) -> Option<f64> {
    let urls = [
        format!("https://api.frankfurter.dev/v1/latest?base={from}&symbols={to}"),
        format!("https://api.frankfurter.app/latest?amount=1&from={from}&to={to}"),
    ];

    for url in urls {
        let Ok(response) = HTTP.get(&url).send() else {
            continue;
        };
        if !response.status().is_success() {
            continue;
        }
        let Ok(json) = response.json::<serde_json::Value>() else {
            continue;
        };
        if let Some(rate) = json.get("rates").and_then(|r| r.get(to)).and_then(|v| v.as_f64()) {
            return Some(rate);
        }
    }
    None
}

fn fetch_open_er_api(from: &str, to: &str) -> Option<f64> {
    let url = format!("https://open.er-api.com/v6/latest/{from}");
    let response = HTTP.get(&url).send().ok()?;
    if !response.status().is_success() {
        return None;
    }
    let json: serde_json::Value = response.json().ok()?;
    if json.get("result")?.as_str()? != "success" {
        return None;
    }
    json.get("rates")?.get(to)?.as_f64()
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

    #[test]
    fn parses_mixed_pt_en() {
        assert!(try_convert("10 usd para reais").is_some());
        assert!(try_convert("10 usd to brl").is_some());
        assert!(try_convert("10 dolares para reais").is_some());
    }

    #[test]
    #[ignore = "requer rede"]
    fn fetch_usd_brl_live() {
        assert!(fetch_rate("USD", "BRL").is_some());
    }
}
