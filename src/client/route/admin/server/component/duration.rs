use chrono::Duration;

/// Helper to format duration to string like "1h", "30m", "1h30m"
pub fn format_duration(d: &Duration) -> String {
    let total_seconds = d.num_seconds();
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;

    if hours > 0 && minutes > 0 {
        format!("{}h{}m", hours, minutes)
    } else if hours > 0 {
        format!("{}h", hours)
    } else if minutes > 0 {
        format!("{}m", minutes)
    } else {
        format!("{}s", total_seconds)
    }
}

/// Helper to parse duration string like "1h", "30m", "2h30m"
pub fn parse_duration(s: &str) -> Option<Duration> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }

    let s = s.to_lowercase();
    let mut total_seconds = 0i64;
    let mut current_num = String::new();
    let mut has_valid_unit = false;

    for ch in s.chars() {
        if ch.is_ascii_digit() {
            current_num.push(ch);
        } else if ch == 'h' {
            if current_num.is_empty() {
                return None; // 'h' without number
            }
            if let Ok(hours) = current_num.parse::<i64>() {
                total_seconds += hours * 3600;
                current_num.clear();
                has_valid_unit = true;
            } else {
                return None;
            }
        } else if ch == 'm' {
            if current_num.is_empty() {
                return None; // 'm' without number
            }
            if let Ok(minutes) = current_num.parse::<i64>() {
                total_seconds += minutes * 60;
                current_num.clear();
                has_valid_unit = true;
            } else {
                return None;
            }
        } else if ch == 's' {
            if current_num.is_empty() {
                return None; // 's' without number
            }
            if let Ok(seconds) = current_num.parse::<i64>() {
                total_seconds += seconds;
                current_num.clear();
                has_valid_unit = true;
            } else {
                return None;
            }
        } else if ch.is_whitespace() {
            // Allow whitespace
            continue;
        } else {
            // Invalid character
            return None;
        }
    }

    // Check if there are leftover digits (number without unit)
    if !current_num.is_empty() {
        return None;
    }

    if total_seconds > 0 && has_valid_unit {
        Some(Duration::seconds(total_seconds))
    } else {
        None
    }
}

/// Validates duration input - returns None if valid or empty, Some(error) if invalid
pub fn validate_duration_input(input: &str) -> Option<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None; // Empty is valid (optional field)
    }

    // Try to parse it
    if parse_duration(trimmed).is_some() {
        None // Valid duration
    } else {
        Some("Invalid format. Use: 1h, 30m, 1h30m, etc.".to_string())
    }
}
