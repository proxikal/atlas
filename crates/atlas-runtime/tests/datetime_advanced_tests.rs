//! Integration tests for advanced datetime functionality (Phase-09b)
//!
//! Tests cover:
//! - Advanced formatting (custom formats, RFC 3339/2822)
//! - Advanced parsing (custom formats, RFC 3339/2822, try-parse)
//! - Timezone operations (conversion, offset, named zones)
//! - Duration operations (creation, formatting)

use atlas_runtime::{Atlas, Value};

// ============================================================================
// Advanced Formatting Tests (8 tests)
// ============================================================================

#[test]
fn test_datetime_format_custom_pattern() {
    let runtime = Atlas::new();
    let result = runtime
        .eval("let dt = dateTimeFromComponents(2024, 1, 15, 10, 30, 45); dateTimeFormat(dt, \"%Y-%m-%d %H:%M:%S\")")
        .unwrap();
    assert_eq!(result, Value::string("2024-01-15 10:30:45".to_string()));
}

#[test]
fn test_datetime_format_with_weekday_month_names() {
    let runtime = Atlas::new();
    let result = runtime
        .eval("let dt = dateTimeFromComponents(2024, 1, 15, 10, 30, 0); dateTimeFormat(dt, \"%A, %B %d, %Y\")")
        .unwrap();
    assert_eq!(
        result,
        Value::string("Monday, January 15, 2024".to_string())
    );
}

#[test]
fn test_datetime_to_rfc3339() {
    let runtime = Atlas::new();
    let result = runtime
        .eval("let dt = dateTimeFromComponents(2024, 1, 15, 10, 30, 0); dateTimeToRfc3339(dt)")
        .unwrap();
    // RFC 3339 format includes timezone
    match result {
        Value::String(s) => {
            assert!(s.starts_with("2024-01-15T10:30:00"));
            assert!(s.contains("+00:00") || s.contains("Z"));
        }
        _ => panic!("Expected string result"),
    }
}

#[test]
fn test_datetime_to_rfc2822() {
    let runtime = Atlas::new();
    let result = runtime
        .eval("let dt = dateTimeFromComponents(2024, 1, 15, 10, 30, 0); dateTimeToRfc2822(dt)")
        .unwrap();
    // RFC 2822 format: "Mon, 15 Jan 2024 10:30:00 +0000"
    match result {
        Value::String(s) => {
            assert!(s.contains("15 Jan 2024"));
            assert!(s.contains("10:30:00"));
        }
        _ => panic!("Expected string result"),
    }
}

#[test]
fn test_datetime_format_with_timezone_offset() {
    let runtime = Atlas::new();
    let result = runtime
        .eval("let dt = dateTimeFromComponents(2024, 1, 15, 10, 30, 0); dateTimeFormat(dt, \"%Y-%m-%d %z\")")
        .unwrap();
    match result {
        Value::String(s) => {
            assert!(s.starts_with("2024-01-15"));
            assert!(s.contains("+0000") || s.contains("+00:00"));
        }
        _ => panic!("Expected string result"),
    }
}

#[test]
fn test_datetime_format_year_only() {
    let runtime = Atlas::new();
    let result = runtime
        .eval("let dt = dateTimeFromComponents(2024, 6, 15, 0, 0, 0); dateTimeFormat(dt, \"%Y\")")
        .unwrap();
    assert_eq!(result, Value::string("2024".to_string()));
}

#[test]
fn test_datetime_format_time_only() {
    let runtime = Atlas::new();
    let result = runtime
        .eval("let dt = dateTimeFromComponents(2024, 1, 15, 14, 30, 45); dateTimeFormat(dt, \"%H:%M:%S\")")
        .unwrap();
    assert_eq!(result, Value::string("14:30:45".to_string()));
}

#[test]
fn test_datetime_to_custom_alias() {
    let runtime = Atlas::new();
    let result = runtime
        .eval("let dt = dateTimeFromComponents(2024, 1, 15, 10, 30, 0); dateTimeToCustom(dt, \"%Y/%m/%d\")")
        .unwrap();
    assert_eq!(result, Value::string("2024/01/15".to_string()));
}

// ============================================================================
// Advanced Parsing Tests (8 tests)
// ============================================================================

#[test]
fn test_datetime_parse_with_custom_format() {
    let runtime = Atlas::new();
    let result = runtime
        .eval("let dt = dateTimeParse(\"2024-01-15 10:30:00\", \"%Y-%m-%d %H:%M:%S\"); dateTimeYear(dt)")
        .unwrap();
    assert_eq!(result, Value::Number(2024.0));
}

#[test]
fn test_datetime_parse_different_format() {
    let runtime = Atlas::new();
    let result = runtime
        .eval("let dt = dateTimeParse(\"15/01/2024 10:30:00\", \"%d/%m/%Y %H:%M:%S\"); dateTimeMonth(dt)")
        .unwrap();
    assert_eq!(result, Value::Number(1.0));
}

#[test]
fn test_datetime_parse_rfc3339() {
    let runtime = Atlas::new();
    let result = runtime
        .eval("let dt = dateTimeParseRfc3339(\"2024-01-15T10:30:00+00:00\"); dateTimeYear(dt)")
        .unwrap();
    assert_eq!(result, Value::Number(2024.0));
}

#[test]
fn test_datetime_parse_rfc2822() {
    let runtime = Atlas::new();
    let result = runtime
        .eval("let dt = dateTimeParseRfc2822(\"Mon, 15 Jan 2024 10:30:00 +0000\"); dateTimeDay(dt)")
        .unwrap();
    assert_eq!(result, Value::Number(15.0));
}

#[test]
fn test_datetime_try_parse_first_format_succeeds() {
    let runtime = Atlas::new();
    let result = runtime
        .eval("let dt = dateTimeTryParse(\"2024-01-15 10:30:00\", [\"%Y-%m-%d %H:%M:%S\", \"%d/%m/%Y\"]); dateTimeYear(dt)")
        .unwrap();
    assert_eq!(result, Value::Number(2024.0));
}

#[test]
fn test_datetime_try_parse_second_format_succeeds() {
    let runtime = Atlas::new();
    let result = runtime
        .eval("let dt = dateTimeTryParse(\"15/01/2024 10:00:00\", [\"%Y-%m-%d %H:%M:%S\", \"%d/%m/%Y %H:%M:%S\"]); dateTimeMonth(dt)")
        .unwrap();
    assert_eq!(result, Value::Number(1.0));
}

#[test]
fn test_datetime_parse_invalid_format_error() {
    let runtime = Atlas::new();
    let result = runtime.eval("dateTimeParse(\"invalid\", \"%Y-%m-%d\")");
    assert!(result.is_err());
}

#[test]
fn test_datetime_try_parse_no_format_matches() {
    let runtime = Atlas::new();
    let result = runtime.eval("dateTimeTryParse(\"invalid\", [\"%Y-%m-%d\", \"%d/%m/%Y\"])");
    assert!(result.is_err());
}

// ============================================================================
// Timezone Operation Tests (10 tests)
// ============================================================================

#[test]
fn test_datetime_to_utc() {
    let runtime = Atlas::new();
    let result = runtime
        .eval("let dt = dateTimeFromComponents(2024, 1, 15, 10, 30, 0); let utc = dateTimeToUtc(dt); dateTimeYear(utc)")
        .unwrap();
    assert_eq!(result, Value::Number(2024.0));
}

#[test]
fn test_datetime_to_local() {
    let runtime = Atlas::new();
    let result = runtime
        .eval("let dt = dateTimeFromComponents(2024, 1, 15, 10, 30, 0); let local = dateTimeToLocal(dt); dateTimeYear(local)")
        .unwrap();
    assert_eq!(result, Value::Number(2024.0));
}

#[test]
fn test_datetime_to_timezone_america_new_york() {
    let runtime = Atlas::new();
    let result = runtime
        .eval("let dt = dateTimeFromComponents(2024, 1, 15, 10, 30, 0); let ny = dateTimeToTimezone(dt, \"America/New_York\"); dateTimeYear(ny)")
        .unwrap();
    assert_eq!(result, Value::Number(2024.0));
}

#[test]
fn test_datetime_to_timezone_europe_london() {
    let runtime = Atlas::new();
    let result = runtime
        .eval("let dt = dateTimeFromComponents(2024, 6, 15, 12, 0, 0); let london = dateTimeToTimezone(dt, \"Europe/London\"); dateTimeYear(london)")
        .unwrap();
    assert_eq!(result, Value::Number(2024.0));
}

#[test]
fn test_datetime_get_timezone() {
    let runtime = Atlas::new();
    let result = runtime
        .eval("let dt = dateTimeNow(); dateTimeGetTimezone(dt)")
        .unwrap();
    assert_eq!(result, Value::string("UTC".to_string()));
}

#[test]
fn test_datetime_get_offset() {
    let runtime = Atlas::new();
    let result = runtime
        .eval("let dt = dateTimeFromComponents(2024, 1, 15, 10, 30, 0); dateTimeGetOffset(dt)")
        .unwrap();
    // UTC offset is always 0
    assert_eq!(result, Value::Number(0.0));
}

#[test]
fn test_datetime_in_timezone() {
    let runtime = Atlas::new();
    let result = runtime
        .eval("let dt = dateTimeFromComponents(2024, 1, 15, 10, 30, 0); let tz = dateTimeInTimezone(dt, \"America/New_York\"); dateTimeHour(tz)")
        .unwrap();
    // Should have different hour due to timezone interpretation
    match result {
        Value::Number(h) => assert!(h >= 0.0 && h < 24.0),
        _ => panic!("Expected number"),
    }
}

#[test]
fn test_datetime_to_timezone_invalid_name_error() {
    let runtime = Atlas::new();
    let result =
        runtime.eval("let dt = dateTimeNow(); dateTimeToTimezone(dt, \"Invalid/Timezone\")");
    assert!(result.is_err());
}

#[test]
fn test_datetime_in_timezone_invalid_name_error() {
    let runtime = Atlas::new();
    let result = runtime.eval("let dt = dateTimeNow(); dateTimeInTimezone(dt, \"Invalid/Zone\")");
    assert!(result.is_err());
}

#[test]
fn test_datetime_timezone_roundtrip() {
    let runtime = Atlas::new();
    let result = runtime
        .eval("let dt1 = dateTimeFromComponents(2024, 1, 15, 12, 0, 0); let ny = dateTimeToTimezone(dt1, \"America/New_York\"); let dt2 = dateTimeToUtc(ny); dateTimeCompare(dt1, dt2)")
        .unwrap();
    assert_eq!(result, Value::Number(0.0)); // Should be equal
}

// ============================================================================
// Duration Operation Tests (4 tests)
// ============================================================================

#[test]
fn test_duration_from_seconds() {
    let runtime = Atlas::new();
    let result = runtime
        .eval("let dur = durationFromSeconds(3665); let opt = hashMapGet(dur, \"hours\"); unwrap(opt)")
        .unwrap();
    assert_eq!(result, Value::Number(1.0)); // 3665 seconds = 1 hour, 1 minute, 5 seconds
}

#[test]
fn test_duration_from_minutes() {
    let runtime = Atlas::new();
    let result = runtime
        .eval(
            "let dur = durationFromMinutes(90); let opt = hashMapGet(dur, \"hours\"); unwrap(opt)",
        )
        .unwrap();
    assert_eq!(result, Value::Number(1.0)); // 90 minutes = 1 hour, 30 minutes
}

#[test]
fn test_duration_from_hours() {
    let runtime = Atlas::new();
    let result = runtime
        .eval("let dur = durationFromHours(25); let opt = hashMapGet(dur, \"days\"); unwrap(opt)")
        .unwrap();
    assert_eq!(result, Value::Number(1.0)); // 25 hours = 1 day, 1 hour
}

#[test]
fn test_duration_from_days() {
    let runtime = Atlas::new();
    let result = runtime
        .eval("let dur = durationFromDays(2); let opt = hashMapGet(dur, \"days\"); unwrap(opt)")
        .unwrap();
    assert_eq!(result, Value::Number(2.0));
}

#[test]
fn test_duration_format_positive() {
    let runtime = Atlas::new();
    let result = runtime
        .eval("let dur = durationFromSeconds(3665); durationFormat(dur)")
        .unwrap();
    assert_eq!(result, Value::string("1h 1m 5s".to_string()));
}

#[test]
fn test_duration_format_days_hours() {
    let runtime = Atlas::new();
    let result = runtime
        .eval("let dur = durationFromHours(25); durationFormat(dur)")
        .unwrap();
    assert_eq!(result, Value::string("1d 1h".to_string()));
}

#[test]
fn test_duration_format_negative() {
    let runtime = Atlas::new();
    let result = runtime
        .eval("let dur = durationFromSeconds(-3600); durationFormat(dur)")
        .unwrap();
    assert_eq!(result, Value::string("-1h".to_string()));
}

#[test]
fn test_duration_format_zero_seconds() {
    let runtime = Atlas::new();
    let result = runtime
        .eval("let dur = durationFromSeconds(0); durationFormat(dur)")
        .unwrap();
    assert_eq!(result, Value::string("0s".to_string()));
}
