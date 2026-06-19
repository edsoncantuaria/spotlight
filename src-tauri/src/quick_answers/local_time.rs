use chrono::Local;

use super::QuickAnswer;

pub fn try_local_time(query: &str) -> Option<QuickAnswer> {
    let q = query.trim().to_lowercase();
    let is_time = matches!(
        q.as_str(),
        "hora"
            | "time"
            | "horário"
            | "horario"
            | "que horas são"
            | "que horas sao"
            | "what time"
    );
    let is_date = matches!(
        q.as_str(),
        "data" | "date" | "hoje" | "today" | "dia" | "what date"
    );

    if is_time {
        let now = Local::now();
        return Some(QuickAnswer {
            kind: "time".to_string(),
            label: now.format("%A").to_string(),
            value: now.format("%H:%M:%S").to_string(),
            hint: Some(now.format("%d/%m/%Y").to_string()),
        });
    }

    if is_date {
        let now = Local::now();
        return Some(QuickAnswer {
            kind: "time".to_string(),
            label: "Hoje".to_string(),
            value: now.format("%d/%m/%Y").to_string(),
            hint: Some(now.format("%A, %H:%M").to_string()),
        });
    }

    None
}
