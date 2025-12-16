use chrono::{NaiveDateTime, Utc};
use dioxus::prelude::*;

/// UTC DateTime input component with format: YYYY-MM-DD HH:MM or "now"
/// Ensures 24-hour time format and UTC timezone
#[component]
pub fn UtcDateTimeInput(
    /// Current datetime value signal in format "YYYY-MM-DD HH:MM" or "now"
    mut value: Signal<String>,
    /// Placeholder text
    #[props(default = "YYYY-MM-DD HH:MM".to_string())]
    placeholder: String,
    /// Whether the input is required
    #[props(default = false)]
    required: bool,
    /// Whether the input is disabled
    #[props(default = false)]
    disabled: bool,
    /// Additional CSS classes
    #[props(default = "".to_string())]
    class: String,
    /// Allow times in the past
    #[props(default = false)]
    allow_past: bool,
    /// Minimum allowed datetime (in UTC)
    #[props(default = None)]
    min_datetime: Option<chrono::DateTime<Utc>>,
) -> Element {
    let mut is_valid = use_signal(|| true);
    let mut error_message = use_signal(|| String::new());

    // Validate datetime format and values
    let mut validate = move |input: &str| -> bool {
        if input.is_empty() {
            if required {
                error_message.set("Date and time are required".to_string());
                return false;
            }
            error_message.set(String::new());
            return true;
        }

        // Handle "now" shorthand (case-insensitive)
        if input.trim().eq_ignore_ascii_case("now") {
            error_message.set(String::new());
            return true;
        }

        // Expected format: YYYY-MM-DD HH:MM
        let parts: Vec<&str> = input.split(' ').collect();
        if parts.len() != 2 {
            error_message.set("Format must be: YYYY-MM-DD HH:MM".to_string());
            return false;
        }

        let date_part = parts[0];
        let time_part = parts[1];

        // Validate date part (YYYY-MM-DD)
        let date_components: Vec<&str> = date_part.split('-').collect();
        if date_components.len() != 3 {
            error_message.set("Date must be: YYYY-MM-DD".to_string());
            return false;
        }

        let year = date_components[0].parse::<i32>();
        let month = date_components[1].parse::<u32>();
        let day = date_components[2].parse::<u32>();

        if year.is_err() || month.is_err() || day.is_err() {
            error_message.set("Invalid date values".to_string());
            return false;
        }

        let year = year.unwrap();
        let month = month.unwrap();
        let day = day.unwrap();

        if year < 2000 || year > 2100 {
            error_message.set("Year must be between 2000 and 2100".to_string());
            return false;
        }

        if month < 1 || month > 12 {
            error_message.set("Month must be between 01 and 12".to_string());
            return false;
        }

        if day < 1 || day > 31 {
            error_message.set("Day must be between 01 and 31".to_string());
            return false;
        }

        // Basic month-day validation
        if (month == 4 || month == 6 || month == 9 || month == 11) && day > 30 {
            error_message.set("This month only has 30 days".to_string());
            return false;
        }

        if month == 2 && day > 29 {
            error_message.set("February can't have more than 29 days".to_string());
            return false;
        }

        // Validate time part (HH:MM)
        let time_components: Vec<&str> = time_part.split(':').collect();
        if time_components.len() != 2 {
            error_message.set("Time must be: HH:MM (24-hour format)".to_string());
            return false;
        }

        let hour = time_components[0].parse::<u32>();
        let minute = time_components[1].parse::<u32>();

        if hour.is_err() || minute.is_err() {
            error_message.set("Invalid time values".to_string());
            return false;
        }

        let hour = hour.unwrap();
        let minute = minute.unwrap();

        if hour > 23 {
            error_message.set("Hour must be between 00 and 23".to_string());
            return false;
        }

        if minute > 59 {
            error_message.set("Minute must be between 00 and 59".to_string());
            return false;
        }

        // Validate that the datetime is not in the past (unless allow_past is true)
        if let Ok(naive_dt) = NaiveDateTime::parse_from_str(input, "%Y-%m-%d %H:%M") {
            let input_dt = naive_dt.and_utc();

            // Check against minimum datetime if provided
            if let Some(min_dt) = min_datetime {
                if input_dt < min_dt {
                    error_message.set(format!(
                        "Fleet time cannot be set earlier than the original time ({})",
                        min_dt.format("%Y-%m-%d %H:%M UTC")
                    ));
                    return false;
                }
            }

            // Check against current time if not allowing past times
            if !allow_past {
                let now = Utc::now();
                if input_dt < now {
                    error_message.set("Fleet time cannot be in the past".to_string());
                    return false;
                }
            }
        }

        error_message.set(String::new());
        true
    };

    rsx! {
        div {
            class: "flex flex-col gap-1",
            input {
                r#type: "text",
                class: if is_valid() {
                    format!("input input-bordered w-full {}", class)
                } else {
                    format!("input input-bordered input-error w-full {}", class)
                },
                placeholder: "{placeholder}",
                value: "{value}",
                disabled,
                required,
                maxlength: 16,
                oninput: move |e| {
                    let new_value = e.value();
                    value.set(new_value.clone());
                    is_valid.set(validate(&new_value));
                },
                onblur: move |_| {
                    is_valid.set(validate(&value()));
                }
            }
            if !is_valid() && !error_message().is_empty() {
                div {
                    class: "text-xs text-error mt-1",
                    "{error_message()}"
                }
            }
            div {
                class: "text-xs opacity-60 mt-1",
                "Format: YYYY-MM-DD HH:MM (UTC, 24-hour time) or \"now\""
            }
        }
    }
}
