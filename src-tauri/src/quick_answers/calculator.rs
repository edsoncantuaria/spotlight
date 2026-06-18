use evalexpr::eval;
use regex::Regex;
use std::sync::LazyLock;

use super::QuickAnswer;

static MATH_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[\d\s+\-*/().^%,]+$").unwrap()
});

pub fn try_evaluate(query: &str) -> Option<QuickAnswer> {
    let trimmed = query.trim();
    if trimmed.is_empty() || !MATH_PATTERN.is_match(trimmed) {
        return None;
    }
    if !trimmed.chars().any(|c| c.is_ascii_digit()) {
        return None;
    }

    let expr = trimmed.replace("sqrt", "sqrt").replace('^', "^");
    match eval(expr.as_str()) {
        Ok(value) => {
            let n = value.as_number().ok()?;
            Some(QuickAnswer {
                kind: "calculator".to_string(),
                label: "=".to_string(),
                value: format_number(n),
                hint: Some("Enter para copiar".to_string()),
            })
        }
        Err(_) => None,
    }
}

fn format_number(n: f64) -> String {
    if (n - n.round()).abs() < f64::EPSILON {
        format!("{}", n.round() as i64)
    } else {
        format!("{n:.6}")
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::try_evaluate;

    #[test]
    fn evaluates_integer_expressions() {
        let result = try_evaluate("2+2");
        assert!(result.is_some());
        assert_eq!(result.unwrap().value, "4");
    }
}
