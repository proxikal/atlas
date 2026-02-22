//! Date and Time standard library functions
//!
//! Provides datetime creation, component access, arithmetic, and conversion operations.

use crate::span::Span;
use crate::stdlib::collections::hash::HashKey;
use crate::stdlib::collections::hashmap::AtlasHashMap;
use crate::value::RuntimeError;
use crate::value::Value;
use chrono::{Datelike, Local, TimeZone, Timelike, Utc, Weekday};
use chrono_tz::Tz;
use std::sync::Arc;

// ============================================================================
// DateTime Construction
// ============================================================================

/// Get current UTC time
///
/// Returns: DateTime representing current moment in UTC
///
/// Example:
/// ```atlas
/// let now = dateTimeNow();
/// ```atlas
pub fn date_time_now(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if !args.is_empty() {
        return Err(RuntimeError::TypeError {
            msg: "dateTimeNow: expected 0 arguments".to_string(),
            span,
        });
    }

    let now = Utc::now();
    Ok(Value::DateTime(Arc::new(now)))
}

/// Create DateTime from Unix timestamp (seconds since epoch)
///
/// Args:
/// - timestamp: number (seconds since Unix epoch)
///
/// Returns: DateTime
///
/// Example:
/// ```atlas
/// let dt = dateTimeFromTimestamp(1609459200); // 2021-01-01 00:00:00 UTC
/// ```atlas
pub fn date_time_from_timestamp(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::TypeError {
            msg: "dateTimeFromTimestamp: expected 1 argument (timestamp)".to_string(),
            span,
        });
    }

    let timestamp = expect_number(&args[0], "timestamp", span)?;
    let timestamp_i64 = timestamp as i64;

    match Utc.timestamp_opt(timestamp_i64, 0) {
        chrono::LocalResult::Single(dt) => Ok(Value::DateTime(Arc::new(dt))),
        _ => Err(RuntimeError::TypeError {
            msg: format!("dateTimeFromTimestamp: invalid timestamp: {}", timestamp),
            span,
        }),
    }
}

/// Create DateTime from components
///
/// Args:
/// - year: number
/// - month: number (1-12)
/// - day: number (1-31)
/// - hour: number (0-23)
/// - minute: number (0-59)
/// - second: number (0-59)
///
/// Returns: DateTime in UTC
///
/// Example:
/// ```atlas
/// let dt = dateTimeFromComponents(2024, 1, 15, 10, 30, 0);
/// ```atlas
pub fn date_time_from_components(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 6 {
        return Err(RuntimeError::TypeError { msg: "dateTimeFromComponents: expected 6 arguments (year, month, day, hour, minute, second)".to_string(), span });
    }

    let year = expect_number(&args[0], "year", span)? as i32;
    let month = expect_number(&args[1], "month", span)? as u32;
    let day = expect_number(&args[2], "day", span)? as u32;
    let hour = expect_number(&args[3], "hour", span)? as u32;
    let minute = expect_number(&args[4], "minute", span)? as u32;
    let second = expect_number(&args[5], "second", span)? as u32;

    // Validate ranges
    if !(1..=12).contains(&month) {
        return Err(RuntimeError::TypeError {
            msg: format!("dateTimeFromComponents: month must be 1-12, got {}", month),
            span,
        });
    }

    if !(1..=31).contains(&day) {
        return Err(RuntimeError::TypeError {
            msg: format!("dateTimeFromComponents: day must be 1-31, got {}", day),
            span,
        });
    }

    if hour >= 24 {
        return Err(RuntimeError::TypeError {
            msg: format!("dateTimeFromComponents: hour must be 0-23, got {}", hour),
            span,
        });
    }

    if minute >= 60 {
        return Err(RuntimeError::TypeError {
            msg: format!(
                "dateTimeFromComponents: minute must be 0-59, got {}",
                minute
            ),
            span,
        });
    }

    if second >= 60 {
        return Err(RuntimeError::TypeError {
            msg: format!(
                "dateTimeFromComponents: second must be 0-59, got {}",
                second
            ),
            span,
        });
    }

    match Utc.with_ymd_and_hms(year, month, day, hour, minute, second) {
        chrono::LocalResult::Single(dt) => Ok(Value::DateTime(Arc::new(dt))),
        _ => Err(RuntimeError::TypeError {
            msg: format!(
                "dateTimeFromComponents: invalid date/time: {}-{:02}-{:02} {:02}:{:02}:{:02}",
                year, month, day, hour, minute, second
            ),
            span,
        }),
    }
}

/// Parse ISO 8601 datetime string
///
/// Args:
/// - text: string (ISO 8601 format, e.g., "2024-01-15T10:30:00Z")
///
/// Returns: DateTime
///
/// Example:
/// ```atlas
/// let dt = dateTimeParseIso("2024-01-15T10:30:00Z");
/// ```atlas
pub fn date_time_parse_iso(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::TypeError {
            msg: "dateTimeParseIso: expected 1 argument (text)".to_string(),
            span,
        });
    }

    let text = expect_string(&args[0], "text", span)?;

    match text.parse::<chrono::DateTime<Utc>>() {
        Ok(dt) => Ok(Value::DateTime(Arc::new(dt))),
        Err(e) => Err(RuntimeError::TypeError {
            msg: format!("dateTimeParseIso: failed to parse '{}': {}", text, e),
            span,
        }),
    }
}

/// Get current UTC time (alias for dateTimeNow)
///
/// Returns: DateTime representing current moment in UTC
///
/// Example:
/// ```atlas
/// let utc = dateTimeUtc();
/// ```atlas
pub fn date_time_utc(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    date_time_now(args, span)
}

// ============================================================================
// Component Access
// ============================================================================

/// Extract year from DateTime
///
/// Args:
/// - dt: DateTime
///
/// Returns: number (year)
pub fn date_time_year(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::TypeError {
            msg: "dateTimeYear: expected 1 argument (datetime)".to_string(),
            span,
        });
    }

    let dt = expect_datetime(&args[0], "datetime", span)?;
    Ok(Value::Number(dt.year() as f64))
}

/// Extract month from DateTime
///
/// Args:
/// - dt: DateTime
///
/// Returns: number (1-12)
pub fn date_time_month(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::TypeError {
            msg: "dateTimeMonth: expected 1 argument (datetime)".to_string(),
            span,
        });
    }

    let dt = expect_datetime(&args[0], "datetime", span)?;
    Ok(Value::Number(dt.month() as f64))
}

/// Extract day from DateTime
///
/// Args:
/// - dt: DateTime
///
/// Returns: number (1-31)
pub fn date_time_day(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::TypeError {
            msg: "dateTimeDay: expected 1 argument (datetime)".to_string(),
            span,
        });
    }

    let dt = expect_datetime(&args[0], "datetime", span)?;
    Ok(Value::Number(dt.day() as f64))
}

/// Extract hour from DateTime
///
/// Args:
/// - dt: DateTime
///
/// Returns: number (0-23)
pub fn date_time_hour(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::TypeError {
            msg: "dateTimeHour: expected 1 argument (datetime)".to_string(),
            span,
        });
    }

    let dt = expect_datetime(&args[0], "datetime", span)?;
    Ok(Value::Number(dt.hour() as f64))
}

/// Extract minute from DateTime
///
/// Args:
/// - dt: DateTime
///
/// Returns: number (0-59)
pub fn date_time_minute(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::TypeError {
            msg: "dateTimeMinute: expected 1 argument (datetime)".to_string(),
            span,
        });
    }

    let dt = expect_datetime(&args[0], "datetime", span)?;
    Ok(Value::Number(dt.minute() as f64))
}

/// Extract second from DateTime
///
/// Args:
/// - dt: DateTime
///
/// Returns: number (0-59)
pub fn date_time_second(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::TypeError {
            msg: "dateTimeSecond: expected 1 argument (datetime)".to_string(),
            span,
        });
    }

    let dt = expect_datetime(&args[0], "datetime", span)?;
    Ok(Value::Number(dt.second() as f64))
}

/// Get day of week from DateTime
///
/// Args:
/// - dt: DateTime
///
/// Returns: number (1=Monday, 7=Sunday, ISO 8601 convention)
pub fn date_time_weekday(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::TypeError {
            msg: "dateTimeWeekday: expected 1 argument (datetime)".to_string(),
            span,
        });
    }

    let dt = expect_datetime(&args[0], "datetime", span)?;
    let weekday = dt.weekday();

    // Convert to ISO 8601: Monday=1, Sunday=7
    let iso_weekday = match weekday {
        Weekday::Mon => 1,
        Weekday::Tue => 2,
        Weekday::Wed => 3,
        Weekday::Thu => 4,
        Weekday::Fri => 5,
        Weekday::Sat => 6,
        Weekday::Sun => 7,
    };

    Ok(Value::Number(iso_weekday as f64))
}

/// Get day of year from DateTime
///
/// Args:
/// - dt: DateTime
///
/// Returns: number (1-366)
pub fn date_time_day_of_year(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::TypeError {
            msg: "dateTimeDayOfYear: expected 1 argument (datetime)".to_string(),
            span,
        });
    }

    let dt = expect_datetime(&args[0], "datetime", span)?;
    Ok(Value::Number(dt.ordinal() as f64))
}

// ============================================================================
// Time Arithmetic
// ============================================================================

/// Add seconds to DateTime
///
/// Args:
/// - dt: DateTime
/// - seconds: number (can be negative to subtract)
///
/// Returns: new DateTime
pub fn date_time_add_seconds(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::TypeError {
            msg: "dateTimeAddSeconds: expected 2 arguments (datetime, seconds)".to_string(),
            span,
        });
    }

    let dt = expect_datetime(&args[0], "datetime", span)?;
    let seconds = expect_number(&args[1], "seconds", span)?;

    let duration = chrono::Duration::seconds(seconds as i64);
    match dt.checked_add_signed(duration) {
        Some(new_dt) => Ok(Value::DateTime(Arc::new(new_dt))),
        None => Err(RuntimeError::TypeError {
            msg: "dateTimeAddSeconds: overflow when adding seconds".to_string(),
            span,
        }),
    }
}

/// Add minutes to DateTime
///
/// Args:
/// - dt: DateTime
/// - minutes: number (can be negative to subtract)
///
/// Returns: new DateTime
pub fn date_time_add_minutes(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::TypeError {
            msg: "dateTimeAddMinutes: expected 2 arguments (datetime, minutes)".to_string(),
            span,
        });
    }

    let dt = expect_datetime(&args[0], "datetime", span)?;
    let minutes = expect_number(&args[1], "minutes", span)?;

    let duration = chrono::Duration::minutes(minutes as i64);
    match dt.checked_add_signed(duration) {
        Some(new_dt) => Ok(Value::DateTime(Arc::new(new_dt))),
        None => Err(RuntimeError::TypeError {
            msg: "dateTimeAddMinutes: overflow when adding minutes".to_string(),
            span,
        }),
    }
}

/// Add hours to DateTime
///
/// Args:
/// - dt: DateTime
/// - hours: number (can be negative to subtract)
///
/// Returns: new DateTime
pub fn date_time_add_hours(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::TypeError {
            msg: "dateTimeAddHours: expected 2 arguments (datetime, hours)".to_string(),
            span,
        });
    }

    let dt = expect_datetime(&args[0], "datetime", span)?;
    let hours = expect_number(&args[1], "hours", span)?;

    let duration = chrono::Duration::hours(hours as i64);
    match dt.checked_add_signed(duration) {
        Some(new_dt) => Ok(Value::DateTime(Arc::new(new_dt))),
        None => Err(RuntimeError::TypeError {
            msg: "dateTimeAddHours: overflow when adding hours".to_string(),
            span,
        }),
    }
}

/// Add days to DateTime
///
/// Args:
/// - dt: DateTime
/// - days: number (can be negative to subtract)
///
/// Returns: new DateTime
pub fn date_time_add_days(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::TypeError {
            msg: "dateTimeAddDays: expected 2 arguments (datetime, days)".to_string(),
            span,
        });
    }

    let dt = expect_datetime(&args[0], "datetime", span)?;
    let days = expect_number(&args[1], "days", span)?;

    let duration = chrono::Duration::days(days as i64);
    match dt.checked_add_signed(duration) {
        Some(new_dt) => Ok(Value::DateTime(Arc::new(new_dt))),
        None => Err(RuntimeError::TypeError {
            msg: "dateTimeAddDays: overflow when adding days".to_string(),
            span,
        }),
    }
}

/// Calculate difference between two DateTimes in seconds
///
/// Args:
/// - dt1: DateTime
/// - dt2: DateTime
///
/// Returns: number (dt1 - dt2 in seconds, can be negative)
pub fn date_time_diff(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::TypeError {
            msg: "dateTimeDiff: expected 2 arguments (datetime1, datetime2)".to_string(),
            span,
        });
    }

    let dt1 = expect_datetime(&args[0], "datetime1", span)?;
    let dt2 = expect_datetime(&args[1], "datetime2", span)?;

    let diff = dt1.signed_duration_since(dt2);
    Ok(Value::Number(diff.num_seconds() as f64))
}

/// Compare two DateTimes
///
/// Args:
/// - dt1: DateTime
/// - dt2: DateTime
///
/// Returns: number (-1 if dt1 < dt2, 0 if equal, 1 if dt1 > dt2)
pub fn date_time_compare(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::TypeError {
            msg: "dateTimeCompare: expected 2 arguments (datetime1, datetime2)".to_string(),
            span,
        });
    }

    let dt1 = expect_datetime(&args[0], "datetime1", span)?;
    let dt2 = expect_datetime(&args[1], "datetime2", span)?;

    let result = if dt1 < dt2 {
        -1.0
    } else if dt1 > dt2 {
        1.0
    } else {
        0.0
    };

    Ok(Value::Number(result))
}

// ============================================================================
// Conversion
// ============================================================================

/// Convert DateTime to Unix timestamp (seconds since epoch)
///
/// Args:
/// - dt: DateTime
///
/// Returns: number (seconds since Unix epoch)
pub fn date_time_to_timestamp(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::TypeError {
            msg: "dateTimeToTimestamp: expected 1 argument (datetime)".to_string(),
            span,
        });
    }

    let dt = expect_datetime(&args[0], "datetime", span)?;
    Ok(Value::Number(dt.timestamp() as f64))
}

/// Convert DateTime to ISO 8601 string
///
/// Args:
/// - dt: DateTime
///
/// Returns: string (ISO 8601 format)
pub fn date_time_to_iso(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::TypeError {
            msg: "dateTimeToIso: expected 1 argument (datetime)".to_string(),
            span,
        });
    }

    let dt = expect_datetime(&args[0], "datetime", span)?;
    Ok(Value::string(dt.to_rfc3339()))
}

// ============================================================================
// Advanced Formatting
// ============================================================================

/// Format DateTime with custom format string
///
/// Args:
/// - dt: DateTime
/// - format: string (strftime format)
///
/// Returns: string (formatted datetime)
///
/// Format specifiers:
/// - %Y: Year (2024)
/// - %m: Month (01-12)
/// - %d: Day (01-31)
/// - %H: Hour 24h (00-23)
/// - %M: Minute (00-59)
/// - %S: Second (00-59)
/// - %A: Weekday name (Monday)
/// - %B: Month name (January)
/// - %z: Timezone offset (+0000)
/// - %Z: Timezone name (UTC)
///
/// Example:
/// ```atlas
/// let formatted = dateTimeFormat(dt, "%Y-%m-%d %H:%M:%S");
/// ```atlas
pub fn date_time_format(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::TypeError {
            msg: "dateTimeFormat: expected 2 arguments (datetime, format)".to_string(),
            span,
        });
    }

    let dt = expect_datetime(&args[0], "datetime", span)?;
    let format = expect_string(&args[1], "format", span)?;

    let formatted = dt.format(&format).to_string();
    Ok(Value::string(formatted))
}

/// Format DateTime to RFC 3339 string
///
/// Args:
/// - dt: DateTime
///
/// Returns: string (RFC 3339 format)
///
/// Example:
/// ```atlas
/// let rfc = dateTimeToRfc3339(dt); // "2024-01-15T10:30:00+00:00"
/// ```atlas
pub fn date_time_to_rfc3339(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::TypeError {
            msg: "dateTimeToRfc3339: expected 1 argument (datetime)".to_string(),
            span,
        });
    }

    let dt = expect_datetime(&args[0], "datetime", span)?;
    Ok(Value::string(dt.to_rfc3339()))
}

/// Format DateTime to RFC 2822 string (email format)
///
/// Args:
/// - dt: DateTime
///
/// Returns: string (RFC 2822 format)
///
/// Example:
/// ```atlas
/// let rfc = dateTimeToRfc2822(dt); // "Mon, 15 Jan 2024 10:30:00 +0000"
/// ```atlas
pub fn date_time_to_rfc2822(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::TypeError {
            msg: "dateTimeToRfc2822: expected 1 argument (datetime)".to_string(),
            span,
        });
    }

    let dt = expect_datetime(&args[0], "datetime", span)?;
    Ok(Value::string(dt.to_rfc2822()))
}

/// Format DateTime with custom format (alias for dateTimeFormat)
///
/// Args:
/// - dt: DateTime
/// - format: string (strftime format)
///
/// Returns: string (formatted datetime)
pub fn date_time_to_custom(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    date_time_format(args, span)
}

// ============================================================================
// Advanced Parsing
// ============================================================================

/// Parse DateTime with custom format string
///
/// Args:
/// - text: string
/// - format: string (strftime format)
///
/// Returns: DateTime
///
/// Example:
/// ```atlas
/// let dt = dateTimeParse("2024-01-15 10:30:00", "%Y-%m-%d %H:%M:%S");
/// ```atlas
pub fn date_time_parse(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::TypeError {
            msg: "dateTimeParse: expected 2 arguments (text, format)".to_string(),
            span,
        });
    }

    let text = expect_string(&args[0], "text", span)?;
    let format = expect_string(&args[1], "format", span)?;

    match chrono::NaiveDateTime::parse_from_str(&text, &format) {
        Ok(naive_dt) => {
            let dt = Utc.from_utc_datetime(&naive_dt);
            Ok(Value::DateTime(Arc::new(dt)))
        }
        Err(e) => Err(RuntimeError::TypeError {
            msg: format!(
                "dateTimeParse: failed to parse '{}' with format '{}': {}",
                text, format, e
            ),
            span,
        }),
    }
}

/// Parse RFC 3339 datetime string
///
/// Args:
/// - text: string (RFC 3339 format)
///
/// Returns: DateTime
///
/// Example:
/// ```atlas
/// let dt = dateTimeParseRfc3339("2024-01-15T10:30:00+00:00");
/// ```atlas
pub fn date_time_parse_rfc3339(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::TypeError {
            msg: "dateTimeParseRfc3339: expected 1 argument (text)".to_string(),
            span,
        });
    }

    let text = expect_string(&args[0], "text", span)?;

    match chrono::DateTime::parse_from_rfc3339(&text) {
        Ok(dt) => Ok(Value::DateTime(Arc::new(dt.with_timezone(&Utc)))),
        Err(e) => Err(RuntimeError::TypeError {
            msg: format!("dateTimeParseRfc3339: failed to parse '{}': {}", text, e),
            span,
        }),
    }
}

/// Parse RFC 2822 datetime string
///
/// Args:
/// - text: string (RFC 2822 format)
///
/// Returns: DateTime
///
/// Example:
/// ```atlas
/// let dt = dateTimeParseRfc2822("Mon, 15 Jan 2024 10:30:00 +0000");
/// ```atlas
pub fn date_time_parse_rfc2822(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::TypeError {
            msg: "dateTimeParseRfc2822: expected 1 argument (text)".to_string(),
            span,
        });
    }

    let text = expect_string(&args[0], "text", span)?;

    match chrono::DateTime::parse_from_rfc2822(&text) {
        Ok(dt) => Ok(Value::DateTime(Arc::new(dt.with_timezone(&Utc)))),
        Err(e) => Err(RuntimeError::TypeError {
            msg: format!("dateTimeParseRfc2822: failed to parse '{}': {}", text, e),
            span,
        }),
    }
}

/// Try parsing with multiple format strings
///
/// Args:
/// - text: string
/// - formats: Array of format strings
///
/// Returns: DateTime (first successful parse)
///
/// Example:
/// ```atlas
/// let dt = dateTimeTryParse("2024-01-15", ["%Y-%m-%d", "%d/%m/%Y"]);
/// ```atlas
pub fn date_time_try_parse(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::TypeError {
            msg: "dateTimeTryParse: expected 2 arguments (text, formats)".to_string(),
            span,
        });
    }

    let text = expect_string(&args[0], "text", span)?;
    let formats_array = expect_array(&args[1], "formats", span)?;

    for format_value in formats_array.as_slice().iter() {
        if let Value::String(format_str) = format_value {
            if let Ok(naive_dt) = chrono::NaiveDateTime::parse_from_str(&text, format_str.as_ref())
            {
                let dt = Utc.from_utc_datetime(&naive_dt);
                return Ok(Value::DateTime(Arc::new(dt)));
            }
        }
    }

    Err(RuntimeError::TypeError {
        msg: format!(
            "dateTimeTryParse: failed to parse '{}' with any provided format",
            text
        ),
        span,
    })
}

// ============================================================================
// Timezone Operations
// ============================================================================

/// Convert DateTime to UTC
///
/// Args:
/// - dt: DateTime
///
/// Returns: DateTime in UTC
///
/// Example:
/// ```atlas
/// let utc = dateTimeToUtc(dt);
/// ```atlas
pub fn date_time_to_utc(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::TypeError {
            msg: "dateTimeToUtc: expected 1 argument (datetime)".to_string(),
            span,
        });
    }

    let dt = expect_datetime(&args[0], "datetime", span)?;
    Ok(Value::DateTime(Arc::new(dt)))
}

/// Convert DateTime to local timezone
///
/// Args:
/// - dt: DateTime
///
/// Returns: DateTime in local timezone (converted back to UTC representation)
///
/// Example:
/// ```atlas
/// let local = dateTimeToLocal(dt);
/// ```atlas
pub fn date_time_to_local(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::TypeError {
            msg: "dateTimeToLocal: expected 1 argument (datetime)".to_string(),
            span,
        });
    }

    let dt = expect_datetime(&args[0], "datetime", span)?;
    let local = dt.with_timezone(&Local);
    Ok(Value::DateTime(Arc::new(local.with_timezone(&Utc))))
}

/// Convert DateTime to named timezone
///
/// Args:
/// - dt: DateTime
/// - tz: string (IANA timezone name)
///
/// Returns: DateTime in specified timezone (converted back to UTC representation)
///
/// Example:
/// ```atlas
/// let ny = dateTimeToTimezone(dt, "America/New_York");
/// ```atlas
pub fn date_time_to_timezone(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::TypeError {
            msg: "dateTimeToTimezone: expected 2 arguments (datetime, timezone)".to_string(),
            span,
        });
    }

    let dt = expect_datetime(&args[0], "datetime", span)?;
    let tz_name = expect_string(&args[1], "timezone", span)?;

    let tz: Tz = match tz_name.parse() {
        Ok(tz) => tz,
        Err(_) => {
            return Err(RuntimeError::TypeError {
                msg: format!("dateTimeToTimezone: invalid timezone name: '{}'", tz_name),
                span,
            })
        }
    };

    let converted = dt.with_timezone(&tz);
    Ok(Value::DateTime(Arc::new(converted.with_timezone(&Utc))))
}

/// Get timezone name from DateTime
///
/// Args:
/// - dt: DateTime
///
/// Returns: string (timezone name, always "UTC" for current implementation)
///
/// Example:
/// ```atlas
/// let tz = dateTimeGetTimezone(dt); // "UTC"
/// ```atlas
pub fn date_time_get_timezone(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::TypeError {
            msg: "dateTimeGetTimezone: expected 1 argument (datetime)".to_string(),
            span,
        });
    }

    let _dt = expect_datetime(&args[0], "datetime", span)?;
    Ok(Value::string("UTC".to_string()))
}

/// Get UTC offset from DateTime in seconds
///
/// Args:
/// - dt: DateTime
///
/// Returns: number (offset in seconds, 0 for UTC)
///
/// Example:
/// ```atlas
/// let offset = dateTimeGetOffset(dt); // 0
/// ```atlas
pub fn date_time_get_offset(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::TypeError {
            msg: "dateTimeGetOffset: expected 1 argument (datetime)".to_string(),
            span,
        });
    }

    let _dt = expect_datetime(&args[0], "datetime", span)?;
    // All DateTimes in Atlas are in UTC, so offset is always 0
    Ok(Value::Number(0.0))
}

/// Create DateTime in specific timezone
///
/// Args:
/// - dt: DateTime
/// - tz: string (IANA timezone name)
///
/// Returns: DateTime interpreted in specified timezone
///
/// Example:
/// ```atlas
/// let ny = dateTimeInTimezone(dt, "America/New_York");
/// ```atlas
pub fn date_time_in_timezone(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::TypeError {
            msg: "dateTimeInTimezone: expected 2 arguments (datetime, timezone)".to_string(),
            span,
        });
    }

    let dt = expect_datetime(&args[0], "datetime", span)?;
    let tz_name = expect_string(&args[1], "timezone", span)?;

    let tz: Tz = match tz_name.parse() {
        Ok(tz) => tz,
        Err(_) => {
            return Err(RuntimeError::TypeError {
                msg: format!("dateTimeInTimezone: invalid timezone name: '{}'", tz_name),
                span,
            })
        }
    };

    // Get naive datetime components and reconstruct in target timezone
    let naive = dt.naive_utc();
    match tz.from_local_datetime(&naive).single() {
        Some(local_dt) => Ok(Value::DateTime(Arc::new(local_dt.with_timezone(&Utc)))),
        None => Err(RuntimeError::TypeError {
            msg: format!(
                "dateTimeInTimezone: ambiguous or invalid datetime in timezone '{}'",
                tz_name
            ),
            span,
        }),
    }
}

// ============================================================================
// Duration Operations
// ============================================================================

/// Create duration from seconds
///
/// Args:
/// - seconds: number (can be negative)
///
/// Returns: HashMap with duration components
///
/// Example:
/// ```atlas
/// let dur = durationFromSeconds(3665); // {days: 0, hours: 1, minutes: 1, seconds: 5}
/// ```atlas
pub fn duration_from_seconds(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::TypeError {
            msg: "durationFromSeconds: expected 1 argument (seconds)".to_string(),
            span,
        });
    }

    let total_seconds = expect_number(&args[0], "seconds", span)? as i64;
    let duration = create_duration_map(total_seconds);
    Ok(duration)
}

/// Create duration from minutes
///
/// Args:
/// - minutes: number (can be negative)
///
/// Returns: HashMap with duration components
///
/// Example:
/// ```atlas
/// let dur = durationFromMinutes(90); // {days: 0, hours: 1, minutes: 30, seconds: 0}
/// ```atlas
pub fn duration_from_minutes(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::TypeError {
            msg: "durationFromMinutes: expected 1 argument (minutes)".to_string(),
            span,
        });
    }

    let minutes = expect_number(&args[0], "minutes", span)? as i64;
    let total_seconds = minutes * 60;
    let duration = create_duration_map(total_seconds);
    Ok(duration)
}

/// Create duration from hours
///
/// Args:
/// - hours: number (can be negative)
///
/// Returns: HashMap with duration components
///
/// Example:
/// ```atlas
/// let dur = durationFromHours(25); // {days: 1, hours: 1, minutes: 0, seconds: 0}
/// ```atlas
pub fn duration_from_hours(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::TypeError {
            msg: "durationFromHours: expected 1 argument (hours)".to_string(),
            span,
        });
    }

    let hours = expect_number(&args[0], "hours", span)? as i64;
    let total_seconds = hours * 3600;
    let duration = create_duration_map(total_seconds);
    Ok(duration)
}

/// Create duration from days
///
/// Args:
/// - days: number (can be negative)
///
/// Returns: HashMap with duration components
///
/// Example:
/// ```atlas
/// let dur = durationFromDays(2); // {days: 2, hours: 0, minutes: 0, seconds: 0}
/// ```atlas
pub fn duration_from_days(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::TypeError {
            msg: "durationFromDays: expected 1 argument (days)".to_string(),
            span,
        });
    }

    let days = expect_number(&args[0], "days", span)? as i64;
    let total_seconds = days * 86400;
    let duration = create_duration_map(total_seconds);
    Ok(duration)
}

/// Format duration as human-readable string
///
/// Args:
/// - duration: HashMap with {days, hours, minutes, seconds}
///
/// Returns: string (e.g., "2d 3h 30m 15s")
///
/// Example:
/// ```atlas
/// let formatted = durationFormat(dur); // "1h 30m"
/// ```atlas
pub fn duration_format(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::TypeError {
            msg: "durationFormat: expected 1 argument (duration)".to_string(),
            span,
        });
    }

    let duration_map = expect_hashmap(&args[0], "duration", span)?;
    let map = duration_map.inner();

    let days = get_number_from_map(map, "days", span)?;
    let hours = get_number_from_map(map, "hours", span)?;
    let minutes = get_number_from_map(map, "minutes", span)?;
    let seconds = get_number_from_map(map, "seconds", span)?;

    let mut parts = Vec::new();
    if days != 0 {
        parts.push(format!("{}d", days.abs()));
    }
    if hours != 0 {
        parts.push(format!("{}h", hours.abs()));
    }
    if minutes != 0 {
        parts.push(format!("{}m", minutes.abs()));
    }
    if seconds != 0 || parts.is_empty() {
        parts.push(format!("{}s", seconds.abs()));
    }

    let result = if days < 0 || hours < 0 || minutes < 0 || seconds < 0 {
        format!("-{}", parts.join(" "))
    } else {
        parts.join(" ")
    };

    Ok(Value::string(result))
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Expect a number value
fn expect_number(value: &Value, arg_name: &str, span: Span) -> Result<f64, RuntimeError> {
    match value {
        Value::Number(n) => Ok(*n),
        _ => Err(RuntimeError::TypeError {
            msg: format!(
                "expected number for '{}', got {}",
                arg_name,
                value.type_name()
            ),
            span,
        }),
    }
}

/// Expect a string value
fn expect_string(value: &Value, arg_name: &str, span: Span) -> Result<String, RuntimeError> {
    match value {
        Value::String(s) => Ok(s.as_ref().clone()),
        _ => Err(RuntimeError::TypeError {
            msg: format!(
                "expected string for '{}', got {}",
                arg_name,
                value.type_name()
            ),
            span,
        }),
    }
}

/// Expect a DateTime value
fn expect_datetime(
    value: &Value,
    arg_name: &str,
    span: Span,
) -> Result<chrono::DateTime<Utc>, RuntimeError> {
    match value {
        Value::DateTime(dt) => Ok(*dt.as_ref()),
        _ => Err(RuntimeError::TypeError {
            msg: format!(
                "expected datetime for '{}', got {}",
                arg_name,
                value.type_name()
            ),
            span,
        }),
    }
}

/// Expect an Array value
fn expect_array<'a>(
    value: &'a Value,
    arg_name: &str,
    span: Span,
) -> Result<&'a crate::value::ValueArray, RuntimeError> {
    match value {
        Value::Array(arr) => Ok(arr),
        _ => Err(RuntimeError::TypeError {
            msg: format!(
                "expected array for '{}', got {}",
                arg_name,
                value.type_name()
            ),
            span,
        }),
    }
}

/// Expect a HashMap value
fn expect_hashmap<'a>(
    value: &'a Value,
    arg_name: &str,
    span: Span,
) -> Result<&'a crate::value::ValueHashMap, RuntimeError> {
    match value {
        Value::HashMap(map) => Ok(map),
        _ => Err(RuntimeError::TypeError {
            msg: format!(
                "expected hashmap for '{}', got {}",
                arg_name,
                value.type_name()
            ),
            span,
        }),
    }
}

/// Get number value from HashMap
fn get_number_from_map(map: &AtlasHashMap, key: &str, span: Span) -> Result<i64, RuntimeError> {
    let hash_key = HashKey::String(Arc::new(key.to_string()));
    match map.get(&hash_key) {
        Some(Value::Number(n)) => Ok(*n as i64),
        Some(_) => Err(RuntimeError::TypeError {
            msg: format!("expected number for key '{}' in duration map", key),
            span,
        }),
        None => Ok(0),
    }
}

/// Create duration HashMap from total seconds
fn create_duration_map(total_seconds: i64) -> Value {
    let is_negative = total_seconds < 0;
    let abs_seconds = total_seconds.abs();

    let days = abs_seconds / 86400;
    let remainder = abs_seconds % 86400;
    let hours = remainder / 3600;
    let remainder = remainder % 3600;
    let minutes = remainder / 60;
    let seconds = remainder % 60;

    let mut map = AtlasHashMap::new();
    map.insert(
        HashKey::String(Arc::new("days".to_string())),
        Value::Number(if is_negative {
            -(days as f64)
        } else {
            days as f64
        }),
    );
    map.insert(
        HashKey::String(Arc::new("hours".to_string())),
        Value::Number(if is_negative {
            -(hours as f64)
        } else {
            hours as f64
        }),
    );
    map.insert(
        HashKey::String(Arc::new("minutes".to_string())),
        Value::Number(if is_negative {
            -(minutes as f64)
        } else {
            minutes as f64
        }),
    );
    map.insert(
        HashKey::String(Arc::new("seconds".to_string())),
        Value::Number(if is_negative {
            -(seconds as f64)
        } else {
            seconds as f64
        }),
    );

    Value::HashMap(crate::value::ValueHashMap::from_atlas(map))
}
