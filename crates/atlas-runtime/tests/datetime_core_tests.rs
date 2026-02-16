//! Tests for datetime core functionality (Phase-09a)
//!
//! Tests construction, component access, arithmetic, and conversion operations.

use atlas_runtime::Atlas;

// ============================================================================
// Test Helpers
// ============================================================================

fn eval_ok(code: &str) -> String {
    let atlas = Atlas::new();
    let result = atlas.eval(code).expect("Execution should succeed");
    result.to_string()
}

fn eval_expect_error(code: &str) -> bool {
    let atlas = Atlas::new();
    atlas.eval(code).is_err()
}

// ============================================================================
// Construction Tests (8 tests)
// ============================================================================

#[test]
fn test_date_time_now() {
    let code = r#"
        let dt = dateTimeNow();
        typeof(dt)
    "#;
    assert_eq!(eval_ok(code), "datetime");
}

#[test]
fn test_date_time_from_timestamp() {
    // Unix epoch for 2021-01-01 00:00:00 UTC = 1609459200
    let code = r#"
        let dt = dateTimeFromTimestamp(1609459200);
        let y = dateTimeYear(dt);
        let m = dateTimeMonth(dt);
        let d = dateTimeDay(dt);
        y * 10000 + m * 100 + d
    "#;
    assert_eq!(eval_ok(code), "20210101");
}

#[test]
fn test_date_time_from_components() {
    let code = r#"
        let dt = dateTimeFromComponents(2024, 2, 14, 15, 30, 45);
        let y = dateTimeYear(dt);
        let mo = dateTimeMonth(dt);
        let d = dateTimeDay(dt);
        let h = dateTimeHour(dt);
        let mi = dateTimeMinute(dt);
        let s = dateTimeSecond(dt);
        y * 10000000000 + mo * 100000000 + d * 1000000 + h * 10000 + mi * 100 + s
    "#;
    assert_eq!(eval_ok(code), "20240214153045");
}

#[test]
fn test_date_time_parse_iso() {
    let code = r#"
        let dt = dateTimeParseIso("2024-01-15T10:30:00Z");
        let y = dateTimeYear(dt);
        let m = dateTimeMonth(dt);
        let d = dateTimeDay(dt);
        y * 10000 + m * 100 + d
    "#;
    assert_eq!(eval_ok(code), "20240115");
}

#[test]
fn test_date_time_from_components_invalid_month() {
    assert!(eval_expect_error(
        "dateTimeFromComponents(2024, 13, 1, 0, 0, 0);"
    ));
}

#[test]
fn test_date_time_from_components_invalid_day() {
    assert!(eval_expect_error(
        "dateTimeFromComponents(2024, 1, 32, 0, 0, 0);"
    ));
}

#[test]
fn test_date_time_parse_iso_invalid() {
    assert!(eval_expect_error(r#"dateTimeParseIso("not-a-date");"#));
}

#[test]
fn test_date_time_negative_timestamp() {
    // Negative timestamp (before Unix epoch)
    let code = r#"
        let dt = dateTimeFromTimestamp(-86400);
        dateTimeYear(dt)
    "#;
    assert_eq!(eval_ok(code), "1969");
}

// ============================================================================
// Component Access Tests (10 tests)
// ============================================================================

#[test]
fn test_date_time_year() {
    let code = r#"
        let dt = dateTimeFromComponents(2024, 6, 15, 10, 30, 45);
        dateTimeYear(dt)
    "#;
    assert_eq!(eval_ok(code), "2024");
}

#[test]
fn test_date_time_month() {
    let code = r#"
        let dt = dateTimeFromComponents(2024, 6, 15, 10, 30, 45);
        dateTimeMonth(dt)
    "#;
    assert_eq!(eval_ok(code), "6");
}

#[test]
fn test_date_time_day() {
    let code = r#"
        let dt = dateTimeFromComponents(2024, 6, 15, 10, 30, 45);
        dateTimeDay(dt)
    "#;
    assert_eq!(eval_ok(code), "15");
}

#[test]
fn test_date_time_hour() {
    let code = r#"
        let dt = dateTimeFromComponents(2024, 6, 15, 10, 30, 45);
        dateTimeHour(dt)
    "#;
    assert_eq!(eval_ok(code), "10");
}

#[test]
fn test_date_time_minute() {
    let code = r#"
        let dt = dateTimeFromComponents(2024, 6, 15, 10, 30, 45);
        dateTimeMinute(dt)
    "#;
    assert_eq!(eval_ok(code), "30");
}

#[test]
fn test_date_time_second() {
    let code = r#"
        let dt = dateTimeFromComponents(2024, 6, 15, 10, 30, 45);
        dateTimeSecond(dt)
    "#;
    assert_eq!(eval_ok(code), "45");
}

#[test]
fn test_date_time_weekday() {
    // 2024-01-01 is a Monday (weekday = 1)
    let code = r#"
        let dt = dateTimeFromComponents(2024, 1, 1, 0, 0, 0);
        dateTimeWeekday(dt)
    "#;
    assert_eq!(eval_ok(code), "1"); // Monday
}

#[test]
fn test_date_time_weekday_sunday() {
    // 2024-01-07 is a Sunday (weekday = 7)
    let code = r#"
        let dt = dateTimeFromComponents(2024, 1, 7, 0, 0, 0);
        dateTimeWeekday(dt)
    "#;
    assert_eq!(eval_ok(code), "7"); // Sunday
}

#[test]
fn test_date_time_day_of_year() {
    // January 15 is day 15
    let code = r#"
        let dt = dateTimeFromComponents(2024, 1, 15, 0, 0, 0);
        dateTimeDayOfYear(dt)
    "#;
    assert_eq!(eval_ok(code), "15");
}

#[test]
fn test_date_time_day_of_year_leap() {
    // 2024 is a leap year, March 1 is day 61 (31 + 29 + 1)
    let code = r#"
        let dt = dateTimeFromComponents(2024, 3, 1, 0, 0, 0);
        dateTimeDayOfYear(dt)
    "#;
    assert_eq!(eval_ok(code), "61");
}

// ============================================================================
// Arithmetic Tests (8 tests)
// ============================================================================

#[test]
fn test_date_time_add_seconds_positive() {
    let code = r#"
        let dt = dateTimeFromComponents(2024, 1, 1, 12, 0, 0);
        let dt2 = dateTimeAddSeconds(dt, 90);
        dateTimeSecond(dt2)
    "#;
    assert_eq!(eval_ok(code), "30");
}

#[test]
fn test_date_time_add_seconds_negative() {
    let code = r#"
        let dt = dateTimeFromComponents(2024, 1, 1, 12, 0, 0);
        let dt2 = dateTimeAddSeconds(dt, -30);
        let s = dateTimeSecond(dt2);
        let m = dateTimeMinute(dt2);
        m * 100 + s
    "#;
    assert_eq!(eval_ok(code), "5930"); // 59:30
}

#[test]
fn test_date_time_add_days() {
    let code = r#"
        let dt = dateTimeFromComponents(2024, 1, 15, 0, 0, 0);
        let dt2 = dateTimeAddDays(dt, 10);
        dateTimeDay(dt2)
    "#;
    assert_eq!(eval_ok(code), "25");
}

#[test]
fn test_date_time_add_hours_and_minutes() {
    let code = r#"
        let dt = dateTimeFromComponents(2024, 1, 1, 10, 30, 0);
        let dt2 = dateTimeAddHours(dt, 2);
        let dt3 = dateTimeAddMinutes(dt2, 45);
        let h = dateTimeHour(dt3);
        let m = dateTimeMinute(dt3);
        h * 100 + m
    "#;
    assert_eq!(eval_ok(code), "1315"); // 13:15
}

#[test]
fn test_date_time_diff() {
    let code = r#"
        let dt1 = dateTimeFromComponents(2024, 1, 1, 12, 0, 0);
        let dt2 = dateTimeFromComponents(2024, 1, 1, 10, 0, 0);
        dateTimeDiff(dt1, dt2)
    "#;
    assert_eq!(eval_ok(code), "7200"); // 2 hours = 7200 seconds
}

#[test]
fn test_date_time_diff_negative() {
    let code = r#"
        let dt1 = dateTimeFromComponents(2024, 1, 1, 10, 0, 0);
        let dt2 = dateTimeFromComponents(2024, 1, 1, 12, 0, 0);
        dateTimeDiff(dt1, dt2)
    "#;
    assert_eq!(eval_ok(code), "-7200"); // -2 hours
}

#[test]
fn test_date_time_compare_equal() {
    let code = r#"
        let dt1 = dateTimeFromComponents(2024, 1, 1, 12, 0, 0);
        let dt2 = dateTimeFromComponents(2024, 1, 1, 12, 0, 0);
        dateTimeCompare(dt1, dt2)
    "#;
    assert_eq!(eval_ok(code), "0");
}

#[test]
fn test_date_time_compare_less_greater() {
    let code_less = r#"
        let dt1 = dateTimeFromComponents(2024, 1, 1, 10, 0, 0);
        let dt2 = dateTimeFromComponents(2024, 1, 1, 12, 0, 0);
        dateTimeCompare(dt1, dt2)
    "#;
    assert_eq!(eval_ok(code_less), "-1");

    let code_greater = r#"
        let dt1 = dateTimeFromComponents(2024, 1, 1, 12, 0, 0);
        let dt2 = dateTimeFromComponents(2024, 1, 1, 10, 0, 0);
        dateTimeCompare(dt1, dt2)
    "#;
    assert_eq!(eval_ok(code_greater), "1");
}

// ============================================================================
// Conversion Tests (4 tests)
// ============================================================================

#[test]
fn test_date_time_to_timestamp_roundtrip() {
    let code = r#"
        let timestamp = 1609459200;  // 2021-01-01 00:00:00 UTC
        let dt = dateTimeFromTimestamp(timestamp);
        dateTimeToTimestamp(dt)
    "#;
    assert_eq!(eval_ok(code), "1609459200");
}

#[test]
fn test_date_time_to_iso() {
    let code = r#"
        let dt = dateTimeFromComponents(2024, 1, 15, 10, 30, 0);
        let iso = dateTimeToIso(dt);
        substring(iso, 0, 19)
    "#;
    assert_eq!(eval_ok(code), "2024-01-15T10:30:00");
}

#[test]
fn test_iso_parse_roundtrip() {
    let code = r#"
        let iso = "2024-06-15T14:30:45Z";
        let dt = dateTimeParseIso(iso);
        let iso2 = dateTimeToIso(dt);
        substring(iso2, 0, 19)
    "#;
    assert_eq!(eval_ok(code), "2024-06-15T14:30:45");
}

#[test]
fn test_timestamp_edge_cases() {
    // Test epoch
    let code_epoch = r#"
        let dt = dateTimeFromTimestamp(0);
        dateTimeYear(dt)
    "#;
    assert_eq!(eval_ok(code_epoch), "1970");

    // Test far future
    let code_future = r#"
        let dt = dateTimeFromTimestamp(2000000000);
        dateTimeYear(dt)
    "#;
    assert_eq!(eval_ok(code_future), "2033");
}

// ============================================================================
// Additional Edge Case Tests (2 tests)
// ============================================================================

#[test]
fn test_date_time_utc_alias() {
    // dateTimeUtc should be an alias for dateTimeNow
    let code = r#"
        let dt = dateTimeUtc();
        typeof(dt)
    "#;
    assert_eq!(eval_ok(code), "datetime");
}

#[test]
fn test_large_time_span_arithmetic() {
    // Test adding many days
    let code = r#"
        let dt = dateTimeFromComponents(2024, 1, 1, 0, 0, 0);
        let dt2 = dateTimeAddDays(dt, 365);
        let m = dateTimeMonth(dt2);
        let d = dateTimeDay(dt2);
        m * 100 + d
    "#;
    // 2024 is a leap year, so 365 days from Jan 1 is Dec 31
    assert_eq!(eval_ok(code), "1231"); // Month 12, Day 31
}
