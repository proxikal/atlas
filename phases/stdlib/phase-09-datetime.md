# Phase 09: Date and Time API

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** Stdlib infrastructure must exist.

**Verification:**
```bash
ls crates/atlas-runtime/src/stdlib/mod.rs
cargo test stdlib
```

**What's needed:**
- Stdlib infrastructure from v0.1
- chrono crate for time handling
- String formatting support

**If missing:** Basic stdlib should exist from v0.1

---

## Objective
Implement comprehensive date and time API with parsing, formatting, arithmetic, and timezone support - enabling time-based applications and timestamp handling essential for real-world programs.

## Files
**Create:** `crates/atlas-runtime/src/stdlib/datetime.rs` (~900 lines)
**Update:** `crates/atlas-runtime/src/value.rs` (~100 lines DateTime value)
**Update:** `Cargo.toml` (add chrono dependency)
**Update:** `docs/stdlib.md` (~400 lines datetime docs)
**Tests:** `crates/atlas-runtime/tests/datetime_tests.rs` (~700 lines)

## Dependencies
- chrono crate for datetime handling
- Stdlib infrastructure
- String formatting
- Number types for timestamps

## Implementation

### DateTime Construction
Create datetime from components year, month, day, hour, minute, second. Parse from ISO 8601 string. Parse with custom format string. Current time with now function. Unix timestamp to datetime. UTC and local timezone constructors. Validate date components. Handle invalid dates gracefully.

### Date Components Access
Extract components from datetime. Year, month, day accessors. Hour, minute, second accessors. Weekday and day of year. Week number. Quarter. Immutable accessors. Timezone information.

### Time Arithmetic
Add and subtract durations. Duration type for time spans. Add days, hours, minutes, seconds. Difference between datetimes. Compare datetimes. Time until or since. Duration formatting.

### Formatting and Parsing
Format datetime to string with patterns. ISO 8601 format. RFC 3339 format. Custom format strings. Locale-aware formatting (future). Parse string to datetime. Handle parsing errors. Multiple format support.

### Timezone Handling
UTC timezone operations. Local timezone conversion. Named timezone support. Timezone offset. Daylight saving awareness. Convert between timezones.

## Tests (TDD - Use rstest)
1. Create datetime from components
2. Parse ISO 8601 string
3. Format to ISO 8601
4. Current time now
5. Unix timestamp conversion
6. Date component access
7. Time arithmetic addition
8. Datetime difference
9. Compare datetimes
10. Timezone conversion
11. Custom format parsing
12. Invalid date handling
13. Leap year handling
14. Edge cases
15. Performance with many dates

**Minimum test count:** 60 tests

## Acceptance
- DateTime creation works
- Parsing and formatting functional
- Time arithmetic correct
- Timezone support works
- Component access accurate
- 60+ tests pass
- Documentation complete
- cargo test passes
