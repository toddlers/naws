use chrono::DateTime;

pub fn format_date(date_str: &str) -> String {
    if let Ok(dt) = DateTime::parse_from_rfc3339(date_str) {
        dt.format("%Y-%m-%d %H:%M UTC").to_string()
    } else if let Ok(dt) = DateTime::parse_from_rfc3339(date_str) {
        dt.format("%Y-%m-%d %H:%M").to_string()
    } else {
        if date_str.len() > 10 {
            // take just the date
            date_str.chars().take(16).collect()
        } else {
            date_str.to_string()
        }
    }
}
