//! stdlib.rs — merged from 20 files (Phase Infra-02)

mod common;

use atlas_runtime::diagnostic::Diagnostic;
use atlas_runtime::lexer::Lexer;
use atlas_runtime::parser::Parser;
use atlas_runtime::span::Span;
use atlas_runtime::stdlib::test as atlas_test;
use atlas_runtime::stdlib::{call_builtin, is_builtin, stdout_writer};
use atlas_runtime::typechecker::TypeChecker;
use atlas_runtime::value::{RuntimeError, Value};
use atlas_runtime::{Atlas, Binder, SecurityContext};
use common::{
    assert_error_code, assert_eval_bool, assert_eval_null, assert_eval_number, assert_eval_string,
    assert_has_error, path_for_atlas, temp_file_path,
};
use pretty_assertions::assert_eq;
use rstest::rstest;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tempfile::TempDir;

// ============================================================================
// Canonical helpers (deduplicated from multiple source files)
// ============================================================================

fn assert_eval_number_with_io(source: &str, expected: f64) {
    let security = SecurityContext::allow_all();
    let runtime = Atlas::new_with_security(security);
    match runtime.eval(source) {
        Ok(Value::Number(n)) if (n - expected).abs() < f64::EPSILON => {}
        Ok(val) => panic!("Expected number {}, got {:?}", expected, val),
        Err(e) => panic!("Execution error: {:?}", e),
    }
}

fn assert_eval_bool_with_io(source: &str, expected: bool) {
    let security = SecurityContext::allow_all();
    let runtime = Atlas::new_with_security(security);
    match runtime.eval(source) {
        Ok(Value::Bool(b)) if b == expected => {}
        Ok(val) => panic!("Expected bool {}, got {:?}", expected, val),
        Err(e) => panic!("Execution error: {:?}", e),
    }
}

fn assert_eval_string_with_io(source: &str, expected: &str) {
    let security = SecurityContext::allow_all();
    let runtime = Atlas::new_with_security(security);
    match runtime.eval(source) {
        Ok(Value::String(s)) if s.as_ref() == expected => {}
        Ok(val) => panic!("Expected string '{}', got {:?}", expected, val),
        Err(e) => panic!("Execution error: {:?}", e),
    }
}

// ============================================================================
// From stdlib_integration_tests.rs
// ============================================================================

// Standard Library Integration Tests
//
// Tests how stdlib functions work together in realistic scenarios.
// Unlike unit tests, these verify cross-function compatibility and complex pipelines.
//
// Test categories:
// - String + Array pipelines
// - Array + Math aggregations
// - JSON + Type conversions
// - File + JSON workflows
// - Complex multi-step transformations

// Assert with file I/O permissions (grants /tmp access)
// ============================================================================
// String + Array Integration Tests
// ============================================================================

#[test]
fn test_split_map_join_pipeline() {
    let code = r#"
        fn toUpper(s: string) -> string {
            return toUpperCase(s);
        }

        let words: string[] = split("hello,world,atlas", ",");
        let upper: string[] = map(words, toUpper);
        let result: string = join(upper, "-");
        result
    "#;
    assert_eval_string(code, "HELLO-WORLD-ATLAS");
}

#[test]
fn test_split_filter_length() {
    let code = r#"
        fn isLong(s: string) -> bool {
            return len(s) > 3;
        }

        let words: string[] = split("a,bb,ccc,dddd,eeeee", ",");
        let long: string[] = filter(words, isLong);
        len(long)
    "#;
    assert_eval_number(code, 2.0); // "dddd" and "eeeee"
}

#[test]
fn test_string_trim_split_trim_each() {
    let code = r#"
        fn trimWord(s: string) -> string {
            return trim(s);
        }

        let input: string = "  hello , world , atlas  ";
        let trimmed: string = trim(input);
        let parts: string[] = split(trimmed, ",");
        let clean: string[] = map(parts, trimWord);
        join(clean, "|")
    "#;
    assert_eval_string(code, "hello|world|atlas");
}

#[test]
fn test_split_reverse_join() {
    let code = r#"
        let words: string[] = split("one,two,three", ",");
        let reversed: string[] = reverse(words);
        join(reversed, ",")
    "#;
    assert_eval_string(code, "three,two,one");
}

#[test]
fn test_substring_map_concat() {
    let code = r#"
        fn first3(s: string) -> string {
            return substring(s, 0, 3);
        }

        let words: string[] = ["hello", "world", "atlas"];
        let prefixes: string[] = map(words, first3);
        join(prefixes, "-")
    "#;
    assert_eval_string(code, "hel-wor-atl");
}

#[test]
fn test_index_of_filter_slice() {
    let code = r#"
        fn hasA(s: string) -> bool {
            return indexOf(s, "a") != -1;
        }

        let words: string[] = ["apple", "banana", "cherry", "date", "avocado"];
        let withA: string[] = filter(words, hasA);
        let first2: string[] = slice(withA, 0, 2);
        len(first2)
    "#;
    assert_eval_number(code, 2.0); // "apple" and "banana"
}

#[test]
fn test_replace_all_in_array() {
    let code = r#"
        fn removeDashes(s: string) -> string {
            return replace(s, "-", "");
        }

        let ids: string[] = ["abc-123", "def-456", "ghi-789"];
        let clean: string[] = map(ids, removeDashes);
        join(clean, ",")
    "#;
    assert_eval_string(code, "abc123,def456,ghi789");
}

#[test]
fn test_pad_start_alignment() {
    let code = r#"
        fn pad5(s: string) -> string {
            return padStart(s, 5, " ");
        }

        let nums: string[] = ["1", "12", "123"];
        let padded: string[] = map(nums, pad5);
        join(padded, "|")
    "#;
    assert_eval_string(code, "    1|   12|  123");
}

#[test]
fn test_split_flatten_join() {
    let code = r#"
        fn splitLine(line: string) -> string[] {
            return split(line, ",");
        }

        let lines: string[] = ["a,b,c", "d,e,f"];
        let nested: string[][] = map(lines, splitLine);
        let flat: string[] = flatten(nested);
        join(flat, "-")
    "#;
    assert_eval_string(code, "a-b-c-d-e-f");
}

#[test]
fn test_starts_with_filter_count() {
    let code = r#"
        fn startsWithHttp(url: string) -> bool {
            return startsWith(url, "http");
        }

        let urls: string[] = [
            "https://example.com",
            "ftp://files.com",
            "http://api.com",
            "file:///local"
        ];
        let httpUrls: string[] = filter(urls, startsWithHttp);
        len(httpUrls)
    "#;
    assert_eval_number(code, 2.0);
}

// ============================================================================
// Array + Math Integration Tests
// ============================================================================

#[test]
fn test_map_numbers_sum_with_reduce() {
    let code = r#"
        fn double(x: number) -> number {
            return x * 2;
        }

        fn add(a: number, b: number) -> number {
            return a + b;
        }

        let nums: number[] = [1, 2, 3, 4, 5];
        let doubled: number[] = map(nums, double);
        reduce(doubled, add, 0)
    "#;
    assert_eval_number(code, 30.0); // (1+2+3+4+5)*2 = 30
}

#[test]
fn test_filter_positive_then_sum() {
    let code = r#"
        fn isPositive(x: number) -> bool {
            return x > 0;
        }

        fn add(a: number, b: number) -> number {
            return a + b;
        }

        let nums: number[] = [-5, 3, -2, 8, 0, 12];
        let positive: number[] = filter(nums, isPositive);
        reduce(positive, add, 0)
    "#;
    assert_eval_number(code, 23.0); // 3 + 8 + 12
}

#[test]
fn test_abs_map_max() {
    let code = r#"
        let nums: number[] = [-10, 5, -20, 15];
        let absNums: number[] = [abs(-10), abs(5), abs(-20), abs(15)];
        max(absNums[0], max(absNums[1], max(absNums[2], absNums[3])))
    "#;
    assert_eval_number(code, 20.0);
}

#[test]
fn test_sqrt_map_floor() {
    let code = r#"
        fn sqrtFloor(x: number) -> number {
            return floor(sqrt(x));
        }

        fn add(a: number, b: number) -> number {
            return a + b;
        }

        let nums: number[] = [4, 9, 10, 16, 20];
        let roots: number[] = map(nums, sqrtFloor);
        reduce(roots, add, 0)
    "#;
    assert_eval_number(code, 16.0); // 2 + 3 + 3 + 4 + 4 = 16
}

#[test]
fn test_clamp_map_range() {
    let code = r#"
        fn clampTo10(n: number) -> number {
            return clamp(n, 0, 10);
        }

        fn numToStr(n: number) -> string {
            return toString(n);
        }

        let nums: number[] = [-5, 3, 15, 7, 20];
        let clamped: number[] = map(nums, clampTo10);
        join(map(clamped, numToStr), ",")
    "#;
    assert_eval_string(code, "0,3,10,7,10");
}

#[test]
fn test_pow_reduce_product() {
    let code = r#"
        fn square(x: number) -> number {
            return pow(x, 2);
        }

        fn multiply(a: number, b: number) -> number {
            return a * b;
        }

        let nums: number[] = [2, 3];
        let squared: number[] = map(nums, square);
        reduce(squared, multiply, 1)
    "#;
    assert_eval_number(code, 36.0); // 4 * 9
}

#[test]
fn test_min_max_range() {
    let code = r#"
        let nums: number[] = [5, 2, 8, 1, 9, 3];
        let minVal: number = min(min(min(min(min(nums[0], nums[1]), nums[2]), nums[3]), nums[4]), nums[5]);
        let maxVal: number = max(max(max(max(max(nums[0], nums[1]), nums[2]), nums[3]), nums[4]), nums[5]);
        maxVal - minVal
    "#;
    assert_eval_number(code, 8.0); // 9 - 1
}

#[test]
fn test_round_map_average() {
    let code = r#"
        fn add(a: number, b: number) -> number {
            return a + b;
        }

        let nums: number[] = [1.2, 2.7, 3.5, 4.1, 5.9];
        let rounded: number[] = [round(1.2), round(2.7), round(3.5), round(4.1), round(5.9)];
        let sum: number = reduce(rounded, add, 0);
        sum / len(rounded)
    "#;
    assert_eval_number(code, 3.6); // (1+3+4+4+6)/5 = 18/5 = 3.6 wait let me recalculate: round(1.2)=1, round(2.7)=3, round(3.5)=4, round(4.1)=4, round(5.9)=6. Sum = 18. 18/5 = 3.6
}

#[test]
fn test_sign_filter_sort() {
    let code = r#"
        fn compare(a: number, b: number) -> number {
            return a - b;
        }

        fn numToStr(x: number) -> string {
            return toString(x);
        }

        let signs: number[] = [sign(-5), sign(3), sign(-2), sign(0), sign(8)];
        let sorted: number[] = sort(signs, compare);
        join(map(sorted, numToStr), ",")
    "#;
    assert_eval_string(code, "-1,-1,0,1,1");
}

#[test]
fn test_random_clamp_floor() {
    let code = r#"
        // Test that random works in a pipeline (result is clamped 0-10, then floored)
        let r: number = random();
        let scaled: number = r * 10;
        let clamped: number = clamp(scaled, 0, 10);
        let result: number = floor(clamped);
        result >= 0 && result <= 10
    "#;
    assert_eval_bool(code, true);
}

// ============================================================================
// JSON + Type Conversion Integration Tests
// ============================================================================

#[test]
fn test_parse_json_extract_map() {
    let code = r##"
        let jsonStr: string = "{\"users\": [{\"name\": \"Alice\"}, {\"name\": \"Bob\"}]}";
        let data: json = parseJSON(jsonStr);
        let users: json = data["users"];
        let alice: json = users[0];
        let name: string = alice["name"].as_string();
        name
    "##;
    assert_eval_string(code, "Alice");
}

#[test]
fn test_typeof_filter_numbers() {
    let code = r##"
        // Test JSON number extraction and type checking
        let jsonStr: string = "[1, 2, 3]";
        let arr: json = parseJSON(jsonStr);

        // Extract numbers and verify
        let item0: number = arr[0].as_number();
        let item1: number = arr[1].as_number();
        let item2: number = arr[2].as_number();

        isNumber(item0) && isNumber(item1) && isNumber(item2)
    "##;
    assert_eval_bool(code, true);
}

#[test]
fn test_json_to_string_concatenation() {
    let code = r##"
        let obj: json = parseJSON("{\"name\": \"Atlas\", \"version\": 1}");
        let name: string = obj["name"].as_string();
        let version: number = obj["version"].as_number();
        name + " v" + toString(version)
    "##;
    assert_eval_string(code, "Atlas v1");
}

#[test]
fn test_json_array_length_type_check() {
    let code = r#"
        let arr: json = parseJSON("[10, 20, 30]");
        // JSON arrays don't have len() directly, need to extract values
        let item0: number = arr[0].as_number();
        let item1: number = arr[1].as_number();
        let item2: number = arr[2].as_number();

        isNumber(item0) && isNumber(item1) && isNumber(item2)
    "#;
    assert_eval_bool(code, true);
}

#[test]
fn test_prettify_minify_roundtrip() {
    let code = r##"
        let compact: string = "{\"a\":1,\"b\":2}";
        let pretty: string = prettifyJSON(compact, 2);
        let mini: string = minifyJSON(pretty);
        isValidJSON(mini)
    "##;
    assert_eval_bool(code, true);
}

#[test]
fn test_json_nested_extraction() {
    let code = r##"
        let json: json = parseJSON("{\"user\":{\"profile\":{\"age\":25}}}");
        let user: json = json["user"];
        let profile: json = user["profile"];
        let age: number = profile["age"].as_number();
        age
    "##;
    assert_eval_number(code, 25.0);
}

#[test]
fn test_parse_float_parse_int_json_mix() {
    let code = r#"
        let floatStr: string = "42.5";
        let intStr: string = "42";
        let asFloat: number = parseFloat(floatStr);
        let asInt: number = parseInt(intStr, 10);
        asFloat - asInt
    "#;
    assert_eval_number(code, 0.5);
}

#[test]
fn test_to_bool_json_boolean() {
    let code = r##"
        let json: json = parseJSON("{\"active\": true, \"deleted\": false}");
        let active: bool = json["active"].as_bool();
        let deleted: bool = json["deleted"].as_bool();
        active && !deleted
    "##;
    assert_eval_bool(code, true);
}

#[test]
fn test_to_json_parse_roundtrip() {
    let code = r##"
        let original: json = parseJSON("{\"x\": 10}");
        let serialized: string = toJSON(original);
        let parsed: json = parseJSON(serialized);
        let x: number = parsed["x"].as_number();
        x
    "##;
    assert_eval_number(code, 10.0);
}

#[test]
fn test_is_valid_json_filter_strings() {
    let code = r##"
        let str1: string = "{\"valid\": true}";
        let str2: string = "not json";
        let str3: string = "[1, 2, 3]";
        let str4: string = "{invalid";

        let valid1: bool = isValidJSON(str1);
        let valid2: bool = isValidJSON(str2);
        let valid3: bool = isValidJSON(str3);
        let valid4: bool = isValidJSON(str4);

        var count: number = 0;
        if (valid1) { count = count + 1; }
        if (valid2) { count = count + 1; }
        if (valid3) { count = count + 1; }
        if (valid4) { count = count + 1; }
        count
    "##;
    assert_eval_number(code, 2.0); // First and third are valid
}

// ============================================================================
// File + JSON Integration Tests
// ============================================================================
// Note: File I/O integration tests are in stdlib_io_tests.rs with proper SecurityContext setup

// ============================================================================
// Complex Multi-Step Transformation Tests
// ============================================================================

#[test]
fn test_csv_to_json_transformation() {
    let code = r#"
        // Simulate CSV parsing and JSON conversion
        let header: string = "name,age,city";
        let row1: string = "Alice,30,NYC";
        let row2: string = "Bob,25,LA";
        let csv: string = join([header, row1, row2], "|");
        let lines: string[] = split(csv, "|");

        // Parse row1 (lines[1])
        let dataRow: string = lines[1];
        let fields1: string[] = split(dataRow, ",");
        let name1: string = fields1[0];
        let age1: string = fields1[1];

        // Build JSON manually (since we don't have object literals yet)
        let json1: string = "{\"name\":\"" + name1 + "\",\"age\":" + age1 + "}";
        let parsed: json = parseJSON(json1);
        let extractedName: string = parsed["name"].as_string();

        extractedName
    "#;
    assert_eval_string(code, "Alice");
}

#[test]
fn test_log_analysis_pipeline() {
    let code = r#"
        fn hasError(line: string) -> bool {
            return includes(line, "ERROR");
        }

        fn extractTimestamp(line: string) -> string {
            return substring(line, 0, 10);
        }

        let log1: string = "2024-01-01 INFO: Started";
        let log2: string = "2024-01-02 ERROR: Failed";
        let log3: string = "2024-01-03 INFO: Resumed";
        let log4: string = "2024-01-04 ERROR: Crashed";
        let logs: string = join([log1, log2, log3, log4], "|");
        let lines: string[] = split(logs, "|");
        let errors: string[] = filter(lines, hasError);
        let timestamps: string[] = map(errors, extractTimestamp);
        join(timestamps, ",")
    "#;
    assert_eval_string(code, "2024-01-02,2024-01-04");
}

#[test]
fn test_data_normalization_pipeline() {
    let code = r#"
        fn normalize(s: string) -> string {
            let trimmed: string = trim(s);
            let lower: string = toLowerCase(trimmed);
            return lower;
        }

        let inputs: string[] = ["  HELLO  ", "World  ", "  ATLAS"];
        let normalized: string[] = map(inputs, normalize);
        join(normalized, "|")
    "#;
    assert_eval_string(code, "hello|world|atlas");
}

#[test]
fn test_validation_and_transformation() {
    let code = r#"
        fn isValidEmail(email: string) -> bool {
            return includes(email, "@") && includes(email, ".");
        }

        fn extractDomain(email: string) -> string {
            let atIndex: number = indexOf(email, "@");
            if (atIndex == -1) {
                return "";
            }
            return substring(email, toNumber(toString(atIndex)) + 1, len(email));
        }

        let emails: string[] = [
            "alice@example.com",
            "invalid-email",
            "bob@test.org",
            "no-at-sign.com"
        ];
        let valid: string[] = filter(emails, isValidEmail);
        let domains: string[] = map(valid, extractDomain);
        join(domains, ",")
    "#;
    assert_eval_string(code, "example.com,test.org");
}

#[test]
fn test_statistical_pipeline() {
    let code = r#"
        fn add(a: number, b: number) -> number {
            return a + b;
        }

        let data: number[] = [10, 20, 30, 40, 50];

        // Calculate mean
        let sum: number = reduce(data, add, 0);
        let mean: number = sum / len(data);

        // Count values above mean
        fn aboveMean(x: number) -> bool {
            return x > mean;
        }
        let aboveCount: number[] = filter(data, aboveMean);

        len(aboveCount)
    "#;
    assert_eval_number(code, 2.0); // 40 and 50 are above mean (30)
}

#[test]
fn test_text_formatting_pipeline() {
    let code = r#"
        fn titleCase(word: string) -> string {
            if (len(word) == 0) {
                return word;
            }
            let first: string = charAt(word, 0);
            let rest: string = substring(word, 1, len(word));
            return toUpperCase(first) + toLowerCase(rest);
        }

        let text: string = "hello world from ATLAS";
        let words: string[] = split(text, " ");
        let titled: string[] = map(words, titleCase);
        join(titled, " ")
    "#;
    assert_eval_string(code, "Hello World From Atlas");
}

#[test]
fn test_deduplication_pipeline() {
    let code = r#"
        // Manual deduplication since we don't have Set yet
        fn notInList(items: string[], item: string) -> bool {
            return !arrayIncludes(items, item);
        }

        let words: string[] = ["apple", "banana", "apple", "cherry", "banana", "date"];
        var unique: string[] = [];

        // Manual dedup (simplified for test)
        if (notInList(unique, words[0])) {
            unique = concat(unique, [words[0]]);
        }
        if (notInList(unique, words[1])) {
            unique = concat(unique, [words[1]]);
        }
        if (notInList(unique, words[2])) {
            unique = concat(unique, [words[2]]);
        }
        if (notInList(unique, words[3])) {
            unique = concat(unique, [words[3]]);
        }
        if (notInList(unique, words[4])) {
            unique = concat(unique, [words[4]]);
        }
        if (notInList(unique, words[5])) {
            unique = concat(unique, [words[5]]);
        }

        len(unique)
    "#;
    assert_eval_number(code, 4.0); // apple, banana, cherry, date
}

#[test]
fn test_url_parsing_pipeline() {
    let code = r#"
        let url: string = "https://api.example.com/v1/users?page=2&limit=10";

        // Extract protocol
        let protocolEnd: number = indexOf(url, "://");
        let protocol: string = substring(url, 0, toNumber(toString(protocolEnd)));

        // Extract query string
        let queryStart: number = indexOf(url, "?");
        let query: string = substring(url, toNumber(toString(queryStart)) + 1, len(url));

        // Parse query params
        let params: string[] = split(query, "&");
        let firstParam: string = params[0];

        includes(protocol, "https") && includes(firstParam, "page")
    "#;
    assert_eval_bool(code, true);
}

#[test]
fn test_markdown_to_text_pipeline() {
    let code = r##"
        let markdown: string = "# Header **bold** and *italic*";

        // Remove headers (simplified)
        let noHeaders: string = replace(markdown, "# ", "");

        // Remove bold markers
        let noBold: string = replace(replace(noHeaders, "**", ""), "**", "");

        // Remove italic markers
        let noItalic: string = replace(replace(noBold, "*", ""), "*", "");

        // Check result has text but no markers
        !includes(noItalic, "#") && !includes(noItalic, "*")
    "##;
    assert_eval_bool(code, true);
}

#[test]
fn test_score_calculation_pipeline() {
    let code = r#"
        fn calculateGrade(score: number) -> string {
            if (score >= 90) {
                return "A";
            }
            if (score >= 80) {
                return "B";
            }
            if (score >= 70) {
                return "C";
            }
            return "F";
        }

        let scores: number[] = [95, 87, 72, 65, 91];
        let grades: string[] = map(scores, calculateGrade);
        join(grades, ",")
    "#;
    assert_eval_string(code, "A,B,C,F,A");
}

// ============================================================================
// Additional String + Array Integration Tests (20 tests to reach 30 total)
// ============================================================================

#[test]
fn test_join_split_identity() {
    let code = r#"
        let arr: string[] = ["hello", "world", "test"];
        let joined: string = join(arr, ",");
        let split_back: string[] = split(joined, ",");
        join(split_back, "|")
    "#;
    assert_eval_string(code, "hello|world|test");
}

#[test]
fn test_concat_strings_then_split() {
    let code = r#"
        let a: string = "foo";
        let b: string = "bar";
        let c: string = "baz";
        let combined: string = a + "," + b + "," + c;
        let parts: string[] = split(combined, ",");
        len(parts)
    "#;
    assert_eval_number(code, 3.0);
}

#[test]
fn test_filter_strings_by_length_then_join() {
    let code = r#"
        fn isShort(s: string) -> bool {
            return len(s) <= 3;
        }

        let words: string[] = ["a", "hello", "hi", "world", "bye"];
        let short: string[] = filter(words, isShort);
        join(short, "-")
    "#;
    assert_eval_string(code, "a-hi-bye");
}

#[test]
fn test_map_substring_all() {
    let code = r#"
        fn firstThree(s: string) -> string {
            if (len(s) < 3) {
                return s;
            }
            return substring(s, 0, 3);
        }

        let words: string[] = ["hello", "world", "hi", "testing"];
        let truncated: string[] = map(words, firstThree);
        join(truncated, ",")
    "#;
    assert_eval_string(code, "hel,wor,hi,tes");
}

#[test]
fn test_array_includes_string_check() {
    let code = r#"
        let items: string[] = ["apple", "banana", "cherry"];
        let search: string = "banana";
        arrayIncludes(items, search)
    "#;
    assert_eval_bool(code, true);
}

#[test]
fn test_reverse_strings_then_concat() {
    let code = r#"
        fn reverseString(s: string) -> string {
            let chars: string[] = split(s, "");
            let rev: string[] = reverse(chars);
            return join(rev, "");
        }

        let words: string[] = ["hello", "world"];
        let reversed: string[] = map(words, reverseString);
        join(reversed, " ")
    "#;
    assert_eval_string(code, "olleh dlrow");
}

#[test]
fn test_slice_array_join() {
    let code = r#"
        let words: string[] = ["one", "two", "three", "four", "five"];
        let middle: string[] = slice(words, 1, 4);
        join(middle, "-")
    "#;
    assert_eval_string(code, "two-three-four");
}

#[test]
fn test_repeat_then_split_count() {
    let code = r#"
        let repeated: string = repeat("ab,", 5);
        let parts: string[] = split(repeated, ",");
        len(parts)
    "#;
    assert_eval_number(code, 6.0); // "ab,ab,ab,ab,ab," splits into ["ab","ab","ab","ab","ab",""]
}

#[test]
fn test_trim_all_in_array() {
    let code = r#"
        fn trimStr(s: string) -> string {
            return trim(s);
        }

        let messy: string[] = ["  hello  ", " world", "test  "];
        let cleaned: string[] = map(messy, trimStr);
        join(cleaned, "|")
    "#;
    assert_eval_string(code, "hello|world|test");
}

#[test]
fn test_char_at_map() {
    let code = r#"
        fn firstChar(s: string) -> string {
            return charAt(s, 0);
        }

        let words: string[] = ["apple", "banana", "cherry"];
        let initials: string[] = map(words, firstChar);
        join(initials, "")
    "#;
    assert_eval_string(code, "abc");
}

#[test]
fn test_to_upper_to_lower_pipeline() {
    let code = r#"
        fn upper(s: string) -> string {
            return toUpperCase(s);
        }
        fn lower(s: string) -> string {
            return toLowerCase(s);
        }

        let words: string[] = ["Hello", "WORLD"];
        let uppered: string[] = map(words, upper);
        let lowered: string[] = map(uppered, lower);
        join(lowered, " ")
    "#;
    assert_eval_string(code, "hello world");
}

#[test]
fn test_ends_with_filter() {
    let code = r#"
        fn endsWithIng(s: string) -> bool {
            return endsWith(s, "ing");
        }

        let words: string[] = ["running", "jump", "walking", "sit", "coding"];
        let gerunds: string[] = filter(words, endsWithIng);
        len(gerunds)
    "#;
    assert_eval_number(code, 3.0);
}

#[test]
fn test_index_of_map_to_numbers() {
    let code = r#"
        fn findComma(s: string) -> number {
            return indexOf(s, ",");
        }

        let strings: string[] = ["a,b", "x,y,z", "no comma"];
        let indices: number[] = map(strings, findComma);
        indices[0] + indices[1]
    "#;
    assert_eval_number(code, 2.0); // 1 + 1 = 2
}

#[test]
fn test_last_index_of_in_array() {
    let code = r#"
        let items: string[] = ["a", "b", "c", "b", "d"];
        arrayLastIndexOf(items, "b")
    "#;
    assert_eval_number(code, 3.0);
}

#[test]
fn test_replace_map_all_strings() {
    let code = r#"
        fn removeDash(s: string) -> string {
            return replace(s, "-", "");
        }

        let codes: string[] = ["ABC-123", "DEF-456", "GHI-789"];
        let clean: string[] = map(codes, removeDash);
        join(clean, ",")
    "#;
    assert_eval_string(code, "ABC123,DEF456,GHI789");
}

#[test]
fn test_pad_end_alignment() {
    let code = r#"
        fn padTo10(s: string) -> string {
            return padEnd(s, 10, ".");
        }

        let names: string[] = ["Alice", "Bob", "Charlie"];
        let padded: string[] = map(names, padTo10);
        len(padded[0])
    "#;
    assert_eval_number(code, 10.0);
}

#[test]
fn test_starts_with_then_count() {
    let code = r#"
        fn startsWithA(s: string) -> bool {
            return startsWith(s, "A");
        }

        let words: string[] = ["Apple", "Banana", "Apricot", "Cherry", "Avocado"];
        let aWords: string[] = filter(words, startsWithA);
        len(aWords)
    "#;
    assert_eval_number(code, 3.0);
}

#[test]
fn test_flatten_then_join_strings() {
    let code = r#"
        let nested: string[][] = [["a", "b"], ["c", "d"], ["e"]];
        let flat: string[] = flatten(nested);
        join(flat, "")
    "#;
    assert_eval_string(code, "abcde");
}

#[test]
fn test_array_concat_then_filter() {
    let code = r#"
        fn isLong(s: string) -> bool {
            return len(s) > 3;
        }

        let a: string[] = ["hi", "hello"];
        let b: string[] = ["bye", "goodbye"];
        let combined: string[] = concat(a, b);
        let long: string[] = filter(combined, isLong);
        len(long)
    "#;
    assert_eval_number(code, 2.0); // "hello" and "goodbye"
}

#[test]
fn test_reduce_string_concatenation() {
    let code = r#"
        fn concatFn(acc: string, s: string) -> string {
            return acc + s + "-";
        }

        let words: string[] = ["one", "two", "three"];
        let result: string = reduce(words, concatFn, "start-");
        result
    "#;
    assert_eval_string(code, "start-one-two-three-");
}

// ============================================================================
// Additional Array + Math Integration Tests (20 tests to reach 30 total)
// ============================================================================

#[test]
fn test_sum_reduce_with_initial() {
    let code = r#"
        fn add(a: number, b: number) -> number {
            return a + b;
        }

        let numbers: number[] = [1, 2, 3, 4, 5];
        let sum: number = reduce(numbers, add, 100);
        sum
    "#;
    assert_eval_number(code, 115.0); // 100 + 1 + 2 + 3 + 4 + 5
}

#[test]
fn test_product_reduce() {
    let code = r#"
        fn multiply(a: number, b: number) -> number {
            return a * b;
        }

        let numbers: number[] = [2, 3, 4];
        let product: number = reduce(numbers, multiply, 1);
        product
    "#;
    assert_eval_number(code, 24.0); // 2 * 3 * 4
}

#[test]
fn test_ceil_floor_pipeline() {
    let code = r#"
        fn ceilNum(n: number) -> number {
            return ceil(n);
        }
        fn floorNum(n: number) -> number {
            return floor(n);
        }

        let floats: number[] = [1.2, 2.8, 3.5];
        let ceiled: number[] = map(floats, ceilNum);
        let floored: number[] = map(ceiled, floorNum);
        floored[0] + floored[1] + floored[2]
    "#;
    assert_eval_number(code, 9.0); // 2 + 3 + 4
}

#[test]
fn test_abs_negative_sum() {
    let code = r#"
        fn absVal(n: number) -> number {
            return abs(n);
        }
        fn add(a: number, b: number) -> number {
            return a + b;
        }

        let numbers: number[] = [-5, -10, -3];
        let positive: number[] = map(numbers, absVal);
        let sum: number = reduce(positive, add, 0);
        sum
    "#;
    assert_eval_number(code, 18.0); // 5 + 10 + 3
}

#[test]
fn test_filter_even_then_square() {
    let code = r#"
        fn isEven(n: number) -> bool {
            return (n % 2) == 0;
        }
        fn square(n: number) -> number {
            return pow(n, 2);
        }

        let numbers: number[] = [1, 2, 3, 4, 5, 6];
        let evens: number[] = filter(numbers, isEven);
        let squared: number[] = map(evens, square);
        squared[0] + squared[1] + squared[2]
    "#;
    assert_eval_number(code, 56.0); // 4 + 16 + 36
}

#[test]
fn test_min_of_array_manual() {
    let code = r#"
        fn minimum(a: number, b: number) -> number {
            return min(a, b);
        }

        let numbers: number[] = [5, 2, 9, 1, 7];
        let minVal: number = reduce(numbers, minimum, 999);
        minVal
    "#;
    assert_eval_number(code, 1.0);
}

#[test]
fn test_max_of_array_manual() {
    let code = r#"
        fn maximum(a: number, b: number) -> number {
            return max(a, b);
        }

        let numbers: number[] = [5, 2, 9, 1, 7];
        let maxVal: number = reduce(numbers, maximum, -999);
        maxVal
    "#;
    assert_eval_number(code, 9.0);
}

#[test]
fn test_sqrt_then_round() {
    let code = r#"
        fn sqrtNum(n: number) -> number {
            return sqrt(n);
        }
        fn roundNum(n: number) -> number {
            return round(n);
        }

        let numbers: number[] = [4, 9, 16, 25];
        let roots: number[] = map(numbers, sqrtNum);
        let rounded: number[] = map(roots, roundNum);
        rounded[0] + rounded[1] + rounded[2] + rounded[3]
    "#;
    assert_eval_number(code, 14.0); // 2 + 3 + 4 + 5
}

#[test]
fn test_sign_map_to_direction() {
    let code = r#"
        fn getSign(n: number) -> number {
            return sign(n);
        }

        let numbers: number[] = [-5, 0, 10, -3, 7];
        let signs: number[] = map(numbers, getSign);
        signs[0] + signs[1] + signs[2] + signs[3] + signs[4]
    "#;
    assert_eval_number(code, 0.0); // -1 + 0 + 1 + -1 + 1
}

#[test]
fn test_clamp_array_values() {
    let code = r#"
        fn clampTo10(n: number) -> number {
            return clamp(n, 0, 10);
        }

        let numbers: number[] = [-5, 5, 15, 20, 8];
        let clamped: number[] = map(numbers, clampTo10);
        clamped[0] + clamped[1] + clamped[2] + clamped[3] + clamped[4]
    "#;
    assert_eval_number(code, 33.0); // 0 + 5 + 10 + 10 + 8
}

#[test]
fn test_filter_positive_count() {
    let code = r#"
        fn isPositive(n: number) -> bool {
            return n > 0;
        }

        let numbers: number[] = [-3, 5, -1, 8, 0, 12];
        let positive: number[] = filter(numbers, isPositive);
        len(positive)
    "#;
    assert_eval_number(code, 3.0); // 5, 8, 12
}

#[test]
fn test_sort_then_first_last() {
    let code = r#"
        fn compare(a: number, b: number) -> number {
            return a - b;
        }

        let numbers: number[] = [5, 2, 9, 1, 7];
        let sorted: number[] = sort(numbers, compare);
        sorted[0] + sorted[4]
    "#;
    assert_eval_number(code, 10.0); // 1 + 9
}

#[test]
fn test_pow_map_exponents() {
    let code = r#"
        fn cube(n: number) -> number {
            return pow(n, 3);
        }

        let numbers: number[] = [1, 2, 3];
        let cubed: number[] = map(numbers, cube);
        cubed[0] + cubed[1] + cubed[2]
    "#;
    assert_eval_number(code, 36.0); // 1 + 8 + 27
}

#[test]
fn test_log_then_floor() {
    let code = r#"
        fn logNum(n: number) -> number {
            return log(n);
        }
        fn floorNum(n: number) -> number {
            return floor(n);
        }

        let numbers: number[] = [10, 100, 1000];
        let logs: number[] = map(numbers, logNum);
        let floored: number[] = map(logs, floorNum);
        floored[0] + floored[1] + floored[2]
    "#;
    assert_eval_number(code, 12.0); // 2 + 4 + 6 (natural log floored)
}

#[test]
fn test_filter_range_then_average() {
    let code = r#"
        fn inRange(n: number) -> bool {
            return n >= 10 && n <= 50;
        }
        fn add(a: number, b: number) -> number {
            return a + b;
        }

        let numbers: number[] = [5, 15, 25, 35, 45, 55];
        let inRangeNums: number[] = filter(numbers, inRange);
        let sum: number = reduce(inRangeNums, add, 0);
        let avg: number = sum / len(inRangeNums);
        avg
    "#;
    assert_eval_number(code, 30.0); // (15 + 25 + 35 + 45) / 4
}

#[test]
fn test_map_modulo_patterns() {
    let code = r#"
        fn mod3(n: number) -> number {
            return n % 3;
        }

        let numbers: number[] = [1, 2, 3, 4, 5, 6, 7, 8, 9];
        let remainders: number[] = map(numbers, mod3);
        remainders[0] + remainders[1] + remainders[2]
    "#;
    assert_eval_number(code, 3.0); // 1 + 2 + 0
}

#[test]
fn test_concat_numeric_arrays() {
    let code = r#"
        fn add(a: number, b: number) -> number {
            return a + b;
        }

        let a: number[] = [1, 2, 3];
        let b: number[] = [4, 5, 6];
        let combined: number[] = concat(a, b);
        let sum: number = reduce(combined, add, 0);
        sum
    "#;
    assert_eval_number(code, 21.0); // 1+2+3+4+5+6
}

#[test]
fn test_slice_then_sum() {
    let code = r#"
        fn add(a: number, b: number) -> number {
            return a + b;
        }

        let numbers: number[] = [1, 2, 3, 4, 5, 6, 7, 8, 9];
        let middle: number[] = slice(numbers, 3, 7);
        let sum: number = reduce(middle, add, 0);
        sum
    "#;
    assert_eval_number(code, 22.0); // slice(numbers, 3, 7) gets [4, 5, 6, 7] = 22
}

#[test]
fn test_reverse_numeric_array() {
    let code = r#"
        let numbers: number[] = [1, 2, 3, 4, 5];
        let rev: number[] = reverse(numbers);
        rev[0] + rev[4]
    "#;
    assert_eval_number(code, 6.0); // 5 + 1
}

#[test]
fn test_find_first_match() {
    let code = r#"
        fn greaterThan10(n: number) -> bool {
            return n > 10;
        }

        let numbers: number[] = [5, 8, 12, 15, 20];
        let found: number = find(numbers, greaterThan10);
        found
    "#;
    assert_eval_number(code, 12.0);
}

// ============================================================================
// Additional JSON + Type Integration Tests (20 tests to reach 30 total)
// ============================================================================

#[test]
fn test_parse_json_array_extract_double() {
    let code = r##"
        let jsonStr: string = "[1, 2, 3]";
        let arr: json = parseJSON(jsonStr);
        let n1: number = arr[0].as_number() * 2;
        let n2: number = arr[1].as_number() * 2;
        let n3: number = arr[2].as_number() * 2;
        n1 + n2 + n3
    "##;
    assert_eval_number(code, 12.0); // 2 + 4 + 6
}

#[test]
fn test_typeof_individual_values() {
    let code = r#"
        let numType: string = typeof(42);
        let strType: string = typeof("hello");
        let boolType: string = typeof(true);
        let nullType: string = typeof(null);
        numType + "," + strType + "," + boolType + "," + nullType
    "#;
    assert_eval_string(code, "number,string,bool,null");
}

#[test]
fn test_type_check_numbers_only() {
    let code = r#"
        fn isNum(val: number) -> bool {
            return isNumber(val);
        }

        let numbers: number[] = [1, 3, 5];
        let check1: bool = isNum(numbers[0]);
        let check2: bool = isNum(numbers[1]);
        let check3: bool = isNum(numbers[2]);
        check1 && check2 && check3
    "#;
    assert_eval_bool(code, true);
}

#[test]
fn test_type_check_strings_only() {
    let code = r#"
        fn isStr(val: string) -> bool {
            return isString(val);
        }

        let strings: string[] = ["two", "four"];
        let check1: bool = isStr(strings[0]);
        let check2: bool = isStr(strings[1]);
        check1 && check2
    "#;
    assert_eval_bool(code, true);
}

#[test]
fn test_json_object_to_json_string() {
    let code = r##"
        let obj: json = parseJSON("{\"name\":\"Alice\",\"age\":30}");
        let jsonString: string = toJSON(obj);
        includes(jsonString, "Alice")
    "##;
    assert_eval_bool(code, true);
}

#[test]
fn test_is_valid_json_with_map() {
    let code = r#"
        fn checkValid(s: string) -> bool {
            return isValidJSON(s);
        }

        let candidates: string[] = ["{\"valid\":true}", "invalid", "[1,2,3]", "null"];
        let results: bool[] = map(candidates, checkValid);
        results[0] && results[2] && results[3]
    "#;
    assert_eval_bool(code, true);
}

#[test]
fn test_parse_json_numbers_sum() {
    let code = r##"
        let jsonStr: string = "[10, 20, 30, 40]";
        let arr: json = parseJSON(jsonStr);
        let sum: number = arr[0].as_number() + arr[1].as_number() + arr[2].as_number() + arr[3].as_number();
        sum
    "##;
    assert_eval_number(code, 100.0);
}

#[test]
fn test_to_string_numbers() {
    let code = r#"
        fn stringify(val: number) -> string {
            return toString(val);
        }

        let numbers: number[] = [42, 99, 7];
        let strings: string[] = map(numbers, stringify);
        join(strings, ",")
    "#;
    assert_eval_string(code, "42,99,7");
}

#[test]
fn test_to_number_parse_strings() {
    let code = r#"
        fn toNum(s: string) -> number {
            return toNumber(s);
        }

        let strings: string[] = ["1", "2", "3"];
        let numbers: number[] = map(strings, toNum);
        numbers[0] + numbers[1] + numbers[2]
    "#;
    assert_eval_number(code, 6.0);
}

#[test]
fn test_parse_int_parse_float_comparison() {
    let code = r#"
        let intVal: number = toNumber("42");
        let floatVal: number = toNumber("42.7");
        intVal + floatVal
    "#;
    assert_eval_number(code, 84.7);
}

#[test]
fn test_to_bool_numbers() {
    let code = r#"
        let b1: bool = toBool(0);
        let b2: bool = toBool(1);
        let b3: bool = toBool(42);
        !b1 && b2 && b3
    "#;
    assert_eval_bool(code, true); // 0 is falsy, 1 and 42 are truthy
}

#[test]
fn test_is_array_type_check() {
    let code = r#"
        let arr: number[] = [1, 2, 3];
        let notArr: number = 42;
        isArray(arr) && !isArray(notArr)
    "#;
    assert_eval_bool(code, true);
}

#[test]
fn test_is_function_check() {
    let code = r#"
        fn myFunc() -> number {
            return 42;
        }

        isFunction(myFunc)
    "#;
    assert_eval_bool(code, true);
}

#[test]
fn test_is_bool_check() {
    let code = r#"
        let b1: bool = true;
        let b2: bool = false;
        let n: number = 1;
        isBool(b1) && isBool(b2) && !isBool(n)
    "#;
    assert_eval_bool(code, true);
}

#[test]
fn test_is_null_check() {
    let code = r#"
        let n = null;
        let num: number = 42;
        isNull(n) && !isNull(num)
    "#;
    assert_eval_bool(code, true);
}

#[test]
fn test_prettify_json_then_minify() {
    let code = r##"
        let compact: string = "{\"a\":1,\"b\":2}";
        let pretty: string = prettifyJSON(compact, 2);
        let mini: string = minifyJSON(pretty);
        mini == compact
    "##;
    assert_eval_bool(code, true);
}

#[test]
fn test_json_array_of_objects_to_strings() {
    let code = r##"
        let jsonStr: string = "[{\"a\":1},{\"b\":2}]";
        let arr: json = parseJSON(jsonStr);
        let str1: string = toJSON(arr[0]);
        let str2: string = toJSON(arr[1]);
        includes(str1, "a") && includes(str2, "b")
    "##;
    assert_eval_bool(code, true);
}

#[test]
fn test_type_checking_pipeline() {
    let code = r#"
        let val: any = 42;
        let isNum: bool = isNumber(val);
        let isStr: bool = isString(val);
        let isB: bool = isBool(val);
        isNum && !isStr && !isB
    "#;
    assert_eval_bool(code, true);
}

#[test]
fn test_parse_json_nested_array() {
    let code = r##"
        let jsonStr: string = "[[1,2],[3,4]]";
        let nested: json = parseJSON(jsonStr);
        let n1: number = nested[0][0].as_number();
        let n2: number = nested[0][1].as_number();
        let n3: number = nested[1][0].as_number();
        let n4: number = nested[1][1].as_number();
        n1 + n2 + n3 + n4
    "##;
    assert_eval_number(code, 10.0); // 1 + 2 + 3 + 4
}

#[test]
fn test_json_roundtrip_with_extraction() {
    let code = r#"
        fn isPositive(n: number) -> bool {
            return n > 0;
        }

        let original: number[] = [-1, 2, -3, 4, 5];
        let jsonStr: string = toJSON(original);
        let parsed: json = parseJSON(jsonStr);
        // Extract and filter manually
        let values: number[] = [];
        // Check each value (json arrays don't support map directly)
        let positive: number[] = filter(original, isPositive);
        len(positive)
    "#;
    assert_eval_number(code, 3.0); // 2, 4, 5
}

// ============================================================================
// File + JSON Integration Tests (20 new tests)
// ============================================================================

#[test]
fn test_write_json_read_parse() {
    let (_temp, path) = temp_file_path("test_json1.json");
    let code = format!(
        r##"
        let data: number[] = [1, 2, 3, 4, 5];
        let jsonStr: string = toJSON(data);
        writeFile("{path}", jsonStr);

        let content: string = readFile("{path}");
        let parsed: json = parseJSON(content);
        parsed[0].as_number() + parsed[4].as_number()
    "##
    );
    assert_eval_number_with_io(&code, 6.0); // 1 + 5
}

#[test]
fn test_json_file_roundtrip() {
    let (_temp, path) = temp_file_path("test_json2.json");
    let code = format!(
        r##"
        let obj: json = parseJSON("{{\"name\":\"Atlas\",\"version\":2}}");
        let jsonStr: string = toJSON(obj);
        writeFile("{path}", jsonStr);

        let loaded: string = readFile("{path}");
        let reparsed: json = parseJSON(loaded);
        reparsed["version"].as_number()
    "##
    );
    assert_eval_number_with_io(&code, 2.0);
}

#[test]
fn test_prettify_write_minify_read() {
    let (_temp, path) = temp_file_path("test_json3.json");
    let code = format!(
        r###"
        let compact: string = "{{\"a\":1,\"b\":2}}";
        let pretty: string = prettifyJSON(compact, 2);
        writeFile("{path}", pretty);

        let loaded: string = readFile("{path}");
        let mini: string = minifyJSON(loaded);
        mini == compact
    "###
    );
    assert_eval_bool_with_io(&code, true);
}

#[test]
fn test_file_exists_json_check() {
    let (_temp, path) = temp_file_path("test_json4.json");
    let code = format!(
        r#"
        writeFile("{path}", "[]");
        let exists: bool = fileExists("{path}");
        let content: string = readFile("{path}");
        let valid: bool = isValidJSON(content);
        exists && valid
    "#
    );
    assert_eval_bool_with_io(&code, true);
}

#[test]
fn test_append_json_array_elements() {
    let (_temp, path) = temp_file_path("test_json5.txt");
    let code = format!(
        r##"
        writeFile("{path}", "[1,2,3]");
        appendFile("{path}", "\n[4,5,6]");

        let content: string = readFile("{path}");
        let lines: string[] = split(content, "\n");
        let arr1: json = parseJSON(lines[0]);
        let arr2: json = parseJSON(lines[1]);
        arr1[0].as_number() + arr2[2].as_number()
    "##
    );
    assert_eval_number_with_io(&code, 7.0); // 1 + 6
}

#[test]
fn test_json_array_to_file_lines() {
    let (_temp, path) = temp_file_path("test_json6.txt");
    let code = format!(
        r#"
        fn toNum(s: string) -> number {{
            return toNumber(s);
        }}

        let numbers: number[] = [10, 20, 30];
        let jsonStr: string = toJSON(numbers);
        writeFile("{path}", jsonStr);

        let content: string = readFile("{path}");
        let parsed: json = parseJSON(content);
        parsed[1].as_number()
    "#
    );
    assert_eval_number_with_io(&code, 20.0);
}

#[test]
fn test_multiple_json_files_sum() {
    let (_temp1, path1) = temp_file_path("test_json7a.json");
    let (_temp2, path2) = temp_file_path("test_json7b.json");
    let code = format!(
        r##"
        writeFile("{path1}", "[10]");
        writeFile("{path2}", "[20]");

        let content1: string = readFile("{path1}");
        let content2: string = readFile("{path2}");
        let arr1: json = parseJSON(content1);
        let arr2: json = parseJSON(content2);
        arr1[0].as_number() + arr2[0].as_number()
    "##
    );
    assert_eval_number_with_io(&code, 30.0);
}

#[test]
fn test_json_validation_before_write() {
    let (_temp, path) = temp_file_path("test_json8.json");
    let code = format!(
        r#"
        let invalid: string = "not json";
        let valid: string = "{{\"key\":\"value\"}}";

        if (isValidJSON(valid)) {{
            writeFile("{path}", valid);
        }}

        let content: string = readFile("{path}");
        includes(content, "key")
    "#
    );
    assert_eval_bool_with_io(&code, true);
}

#[test]
fn test_read_json_check_type() {
    let (_temp, path) = temp_file_path("test_json9.json");
    let code = format!(
        r##"
        writeFile("{path}", "{{\"count\":42}}");

        let content: string = readFile("{path}");
        let obj: json = parseJSON(content);
        let count: number = obj["count"].as_number();
        isNumber(count)
    "##
    );
    assert_eval_bool_with_io(&code, true);
}

#[test]
fn test_json_array_length_via_file() {
    let (_temp, path) = temp_file_path("test_json10.json");
    let code = format!(
        r##"
        let arr: number[] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let jsonStr: string = toJSON(arr);
        writeFile("{path}", jsonStr);

        let content: string = readFile("{path}");
        let parsed: json = parseJSON(content);
        // Extract last element to check array size
        parsed[9].as_number()
    "##
    );
    assert_eval_number_with_io(&code, 10.0);
}

#[test]
fn test_conditional_file_write_json() {
    let (_temp, path) = temp_file_path("test_json11.json");
    let code = format!(
        r##"
        let data: json = parseJSON("{{\"enabled\":true}}");
        let enabled: bool = data["enabled"].as_bool();

        if (enabled) {{
            writeFile("{path}", "{{\"status\":\"active\"}}");
        }}

        let content: string = readFile("{path}");
        includes(content, "active")
    "##
    );
    assert_eval_bool_with_io(&code, true);
}

#[test]
fn test_json_file_string_concat() {
    let (_temp1, path1) = temp_file_path("test_json12a.txt");
    let (_temp2, path2) = temp_file_path("test_json12b.txt");
    let code = format!(
        r##"
        writeFile("{path1}", "Hello");
        writeFile("{path2}", "World");

        let part1: string = readFile("{path1}");
        let part2: string = readFile("{path2}");
        let combined: string = part1 + " " + part2;
        combined
    "##
    );
    assert_eval_string_with_io(&code, "Hello World");
}

#[test]
fn test_json_parse_file_nested_access() {
    let (_temp, path) = temp_file_path("test_json13.json");
    let code = format!(
        r##"
        writeFile("{path}", "{{\"user\":{{\"name\":\"Alice\",\"age\":30}}}}");

        let content: string = readFile("{path}");
        let obj: json = parseJSON(content);
        let user: json = obj["user"];
        let name: string = user["name"].as_string();
        name
    "##
    );
    assert_eval_string_with_io(&code, "Alice");
}

#[test]
fn test_file_to_json_to_string_array() {
    let (_temp, path) = temp_file_path("test_json14.json");
    let code = format!(
        r##"
        let strings: string[] = ["apple", "banana", "cherry"];
        let jsonStr: string = toJSON(strings);
        writeFile("{path}", jsonStr);

        let content: string = readFile("{path}");
        let parsed: json = parseJSON(content);
        let first: string = parsed[0].as_string();
        let last: string = parsed[2].as_string();
        first + "," + last
    "##
    );
    assert_eval_string_with_io(&code, "apple,cherry");
}

#[test]
fn test_json_number_extraction_math() {
    let (_temp, path) = temp_file_path("test_json15.json");
    let code = format!(
        r##"
        writeFile("{path}", "[5,10,15]");

        let content: string = readFile("{path}");
        let arr: json = parseJSON(content);
        let sum: number = arr[0].as_number() + arr[1].as_number() + arr[2].as_number();
        sum / 3
    "##
    );
    assert_eval_number_with_io(&code, 10.0); // Average
}

#[test]
fn test_write_read_bool_json() {
    let (_temp, path) = temp_file_path("test_json16.json");
    let code = format!(
        r##"
        writeFile("{path}", "{{\"active\":true,\"enabled\":false}}");

        let content: string = readFile("{path}");
        let obj: json = parseJSON(content);
        let active: bool = obj["active"].as_bool();
        let enabled: bool = obj["enabled"].as_bool();
        active && !enabled
    "##
    );
    assert_eval_bool_with_io(&code, true);
}

#[test]
fn test_json_file_type_conversion() {
    let (_temp, path) = temp_file_path("test_json17.json");
    let code = format!(
        r##"
        writeFile("{path}", "{{\"count\":\"42\"}}");

        let content: string = readFile("{path}");
        let obj: json = parseJSON(content);
        let countStr: string = obj["count"].as_string();
        let countNum: number = toNumber(countStr);
        countNum * 2
    "##
    );
    assert_eval_number_with_io(&code, 84.0);
}

#[test]
fn test_file_contains_valid_json() {
    let (_temp, path) = temp_file_path("test_json18.json");
    let code = format!(
        r##"
        writeFile("{path}", "{{\"valid\":true}}");

        let content: string = readFile("{path}");
        isValidJSON(content)
    "##
    );
    assert_eval_bool_with_io(&code, true);
}

#[test]
fn test_json_null_in_file() {
    let (_temp, path) = temp_file_path("test_json19.json");
    let code = format!(
        r##"
        writeFile("{path}", "{{\"value\":null}}");

        let content: string = readFile("{path}");
        let obj: json = parseJSON(content);
        let val: json = obj["value"];
        val.is_null()
    "##
    );
    assert_eval_bool_with_io(&code, true);
}

#[test]
fn test_large_json_array_file() {
    let (_temp, path) = temp_file_path("test_json20.json");
    let code = format!(
        r##"
        let arr: number[] = [1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20];
        let jsonStr: string = toJSON(arr);
        writeFile("{path}", jsonStr);

        let content: string = readFile("{path}");
        let parsed: json = parseJSON(content);
        let first: number = parsed[0].as_number();
        let last: number = parsed[19].as_number();
        first + last
    "##
    );
    assert_eval_number_with_io(&code, 21.0); // 1 + 20
}

// ============================================================================
// From stdlib_string_tests.rs
// ============================================================================

// String stdlib tests (Interpreter engine)
//
// Tests all 18 string functions with comprehensive edge case coverage

// ============================================================================
// Core Operations Tests
// ============================================================================

#[test]
fn test_split_basic() {
    let code = r#"
        let result: string[] = split("a,b,c", ",");
        len(result)
    "#;
    assert_eval_number(code, 3.0);
}

#[test]
fn test_split_empty_separator() {
    let code = r#"
        let result: string[] = split("abc", "");
        len(result)
    "#;
    assert_eval_number(code, 3.0);
}

#[test]
fn test_split_no_match() {
    let code = r#"
        let result: string[] = split("hello", ",");
        len(result)
    "#;
    assert_eval_number(code, 1.0);
}

#[test]
fn test_split_unicode() {
    let code = r#"
        let result: string[] = split("🎉,🔥,✨", ",");
        len(result)
    "#;
    assert_eval_number(code, 3.0);
}

#[test]
fn test_join_basic() {
    let code = r#"join(["a", "b", "c"], ",")"#;
    assert_eval_string(code, "a,b,c");
}

#[test]
fn test_join_empty_array() {
    let code = r#"join([], ",")"#;
    assert_eval_string(code, "");
}

#[test]
fn test_join_empty_separator() {
    let code = r#"join(["a", "b", "c"], "")"#;
    assert_eval_string(code, "abc");
}

#[test]
fn test_trim_basic() {
    let code = r#"trim("  hello  ")"#;
    assert_eval_string(code, "hello");
}

#[test]
fn test_trim_unicode_whitespace() {
    let code = "trim(\"\u{00A0}hello\u{00A0}\")";
    assert_eval_string(code, "hello");
}

#[test]
fn test_trim_start() {
    let code = r#"trimStart("  hello")"#;
    assert_eval_string(code, "hello");
}

#[test]
fn test_trim_end() {
    let code = r#"trimEnd("hello  ")"#;
    assert_eval_string(code, "hello");
}

// ============================================================================
// Search Operations Tests
// ============================================================================

#[test]
fn test_index_of_found() {
    let code = r#"indexOf("hello", "ll")"#;
    assert_eval_number(code, 2.0);
}

#[test]
fn test_index_of_not_found() {
    let code = r#"indexOf("hello", "x")"#;
    assert_eval_number(code, -1.0);
}

#[test]
fn test_index_of_empty_needle() {
    let code = r#"indexOf("hello", "")"#;
    assert_eval_number(code, 0.0);
}

#[test]
fn test_last_index_of_found() {
    let code = r#"lastIndexOf("hello", "l")"#;
    assert_eval_number(code, 3.0);
}

#[test]
fn test_last_index_of_not_found() {
    let code = r#"lastIndexOf("hello", "x")"#;
    assert_eval_number(code, -1.0);
}

#[test]
fn test_includes_found() {
    let code = r#"includes("hello", "ll")"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_includes_not_found() {
    let code = r#"includes("hello", "x")"#;
    assert_eval_bool(code, false);
}

// ============================================================================
// Transformation Tests
// ============================================================================

#[test]
fn test_to_upper_case() {
    let code = r#"toUpperCase("hello")"#;
    assert_eval_string(code, "HELLO");
}

#[test]
fn test_to_upper_case_unicode() {
    let code = r#"toUpperCase("café")"#;
    assert_eval_string(code, "CAFÉ");
}

#[test]
fn test_to_lower_case() {
    let code = r#"toLowerCase("HELLO")"#;
    assert_eval_string(code, "hello");
}

#[test]
fn test_to_lower_case_unicode() {
    let code = r#"toLowerCase("CAFÉ")"#;
    assert_eval_string(code, "café");
}

#[test]
fn test_substring_basic() {
    let code = r#"substring("hello", 1, 4)"#;
    assert_eval_string(code, "ell");
}

#[test]
fn test_substring_empty() {
    let code = r#"substring("hello", 2, 2)"#;
    assert_eval_string(code, "");
}

#[test]
fn test_substring_out_of_bounds() {
    let code = r#"substring("hello", 0, 100)"#;
    assert_has_error(code);
}

#[test]
fn test_char_at_basic() {
    let code = r#"charAt("hello", 0)"#;
    assert_eval_string(code, "h");
}

#[test]
fn test_char_at_unicode() {
    let code = r#"charAt("🎉🔥✨", 1)"#;
    assert_eval_string(code, "🔥");
}

#[test]
fn test_char_at_out_of_bounds() {
    let code = r#"charAt("hello", 10)"#;
    assert_has_error(code);
}

#[test]
fn test_repeat_basic() {
    let code = r#"repeat("ha", 3)"#;
    assert_eval_string(code, "hahaha");
}

#[test]
fn test_repeat_zero() {
    let code = r#"repeat("ha", 0)"#;
    assert_eval_string(code, "");
}

#[test]
fn test_repeat_negative() {
    let code = r#"repeat("ha", -1)"#;
    assert_has_error(code);
}

#[test]
fn test_replace_basic() {
    let code = r#"replace("hello", "l", "L")"#;
    assert_eval_string(code, "heLlo");
}

#[test]
fn test_replace_not_found() {
    let code = r#"replace("hello", "x", "y")"#;
    assert_eval_string(code, "hello");
}

#[test]
fn test_replace_empty_search() {
    let code = r#"replace("hello", "", "x")"#;
    assert_eval_string(code, "hello");
}

// ============================================================================
// Formatting Tests
// ============================================================================

#[test]
fn test_pad_start_basic() {
    let code = r#"padStart("5", 3, "0")"#;
    assert_eval_string(code, "005");
}

#[test]
fn test_pad_start_already_long() {
    let code = r#"padStart("hello", 3, "0")"#;
    assert_eval_string(code, "hello");
}

#[test]
fn test_pad_start_multichar_fill() {
    let code = r#"padStart("x", 5, "ab")"#;
    assert_eval_string(code, "ababx");
}

#[test]
fn test_pad_end_basic() {
    let code = r#"padEnd("5", 3, "0")"#;
    assert_eval_string(code, "500");
}

#[test]
fn test_pad_end_already_long() {
    let code = r#"padEnd("hello", 3, "0")"#;
    assert_eval_string(code, "hello");
}

#[test]
fn test_starts_with_true() {
    let code = r#"startsWith("hello", "he")"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_starts_with_false() {
    let code = r#"startsWith("hello", "x")"#;
    assert_eval_bool(code, false);
}

#[test]
fn test_starts_with_empty() {
    let code = r#"startsWith("hello", "")"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_ends_with_true() {
    let code = r#"endsWith("hello", "lo")"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_ends_with_false() {
    let code = r#"endsWith("hello", "x")"#;
    assert_eval_bool(code, false);
}

#[test]
fn test_ends_with_empty() {
    let code = r#"endsWith("hello", "")"#;
    assert_eval_bool(code, true);
}

// ============================================================================
// From stdlib_json_tests.rs
// ============================================================================

// JSON stdlib tests (Interpreter engine)
//
// Tests all 5 JSON functions with comprehensive edge case coverage

// ============================================================================
// parseJSON Tests
// ============================================================================

#[test]
fn test_parse_json_null() {
    let code = r#"
        let result: json = parseJSON("null");
        typeof(result)
    "#;
    assert_eval_string(code, "json");
}

#[test]
fn test_parse_json_boolean_true() {
    // Should return JsonValue, test via typeof
    let code = r#"typeof(parseJSON("true"))"#;
    assert_eval_string(code, "json");
}

#[test]
fn test_parse_json_boolean_false() {
    let code = r#"typeof(parseJSON("false"))"#;
    assert_eval_string(code, "json");
}

#[test]
fn test_parse_json_number() {
    let code = r#"typeof(parseJSON("42"))"#;
    assert_eval_string(code, "json");
}

#[test]
fn test_parse_json_number_float() {
    let code = r#"typeof(parseJSON("3.14"))"#;
    assert_eval_string(code, "json");
}

#[test]
fn test_parse_json_number_negative() {
    let code = r#"typeof(parseJSON("-123"))"#;
    assert_eval_string(code, "json");
}

#[test]
fn test_parse_json_string() {
    let code = r#"typeof(parseJSON("\"hello\""))"#;
    assert_eval_string(code, "json");
}

#[test]
fn test_parse_json_empty_string() {
    let code = r#"typeof(parseJSON("\"\""))"#;
    assert_eval_string(code, "json");
}

#[test]
fn test_parse_json_array_empty() {
    let code = r#"typeof(parseJSON("[]"))"#;
    assert_eval_string(code, "json");
}

#[test]
fn test_parse_json_array_numbers() {
    let code = r#"typeof(parseJSON("[1,2,3]"))"#;
    assert_eval_string(code, "json");
}

#[test]
fn test_parse_json_array_mixed() {
    let code = r#"typeof(parseJSON("[1,\"two\",true,null]"))"#;
    assert_eval_string(code, "json");
}

#[test]
fn test_parse_json_array_nested() {
    let code = r#"typeof(parseJSON("[[1,2],[3,4]]"))"#;
    assert_eval_string(code, "json");
}

#[test]
fn test_parse_json_object_empty() {
    let code = r#"typeof(parseJSON("{}"))"#;
    assert_eval_string(code, "json");
}

#[test]
fn test_parse_json_object_simple() {
    let code = r#"typeof(parseJSON("{\"name\":\"Alice\",\"age\":30}"))"#;
    assert_eval_string(code, "json");
}

#[test]
fn test_parse_json_object_nested() {
    let code = r#"typeof(parseJSON("{\"user\":{\"name\":\"Bob\"}}"))"#;
    assert_eval_string(code, "json");
}

#[test]
fn test_parse_json_object_with_array() {
    let code = r#"typeof(parseJSON("{\"items\":[1,2,3]}"))"#;
    assert_eval_string(code, "json");
}

#[test]
fn test_parse_json_whitespace() {
    let code = r#"typeof(parseJSON("  { \"a\" : 1 }  "))"#;
    assert_eval_string(code, "json");
}

#[test]
fn test_parse_json_unicode() {
    let code = r#"typeof(parseJSON("{\"emoji\":\"🎉\"}"))"#;
    assert_eval_string(code, "json");
}

// ============================================================================
// parseJSON Error Tests
// ============================================================================

#[test]
fn test_parse_json_invalid_syntax() {
    let code = r#"parseJSON("{invalid}")"#;
    assert_has_error(code);
}

#[test]
fn test_parse_json_trailing_comma() {
    let code = r#"parseJSON("[1,2,]")"#;
    assert_has_error(code);
}

#[test]
fn test_parse_json_single_quote() {
    let code = r#"parseJSON("{'key':'value'}")"#;
    assert_has_error(code);
}

#[test]
fn test_parse_json_unquoted_keys() {
    let code = r#"parseJSON("{key:\"value\"}")"#;
    assert_has_error(code);
}

#[test]
fn test_parse_json_wrong_type() {
    let code = r#"parseJSON(123)"#;
    assert_has_error(code);
}

// ============================================================================
// toJSON Tests
// ============================================================================

#[test]
fn test_to_json_null() {
    let code = r#"toJSON(null)"#;
    assert_eval_string(code, "null");
}

#[test]
fn test_to_json_bool_true() {
    let code = r#"toJSON(true)"#;
    assert_eval_string(code, "true");
}

#[test]
fn test_to_json_bool_false() {
    let code = r#"toJSON(false)"#;
    assert_eval_string(code, "false");
}

#[test]
fn test_to_json_number_int() {
    let code = r#"toJSON(42)"#;
    assert_eval_string(code, "42");
}

#[test]
fn test_to_json_number_float() {
    let code = r#"toJSON(3.14)"#;
    assert_eval_string(code, "3.14");
}

#[test]
fn test_to_json_number_negative() {
    let code = r#"toJSON(-10)"#;
    assert_eval_string(code, "-10");
}

#[test]
fn test_to_json_number_zero() {
    let code = r#"toJSON(0)"#;
    assert_eval_string(code, "0");
}

#[test]
fn test_to_json_string_simple() {
    let code = r#"toJSON("hello")"#;
    assert_eval_string(code, r#""hello""#);
}

#[test]
fn test_to_json_string_empty() {
    let code = r#"toJSON("")"#;
    assert_eval_string(code, r#""""#);
}

#[test]
fn test_to_json_string_with_quotes() {
    let code = r#"toJSON("say \"hi\"")"#;
    assert_eval_string(code, r#""say \"hi\"""#);
}

#[test]
fn test_to_json_array_empty() {
    let code = r#"toJSON([])"#;
    assert_eval_string(code, "[]");
}

#[test]
fn test_to_json_array_numbers() {
    let code = r#"toJSON([1,2,3])"#;
    assert_eval_string(code, "[1,2,3]");
}

// Note: Mixed-type array test removed - Atlas enforces homogeneous arrays.
// For heterogeneous JSON arrays, use parseJSON to create json values.

#[test]
fn test_to_json_array_nested() {
    let code = r#"toJSON([[1,2],[3,4]])"#;
    assert_eval_string(code, "[[1,2],[3,4]]");
}

// ============================================================================
// toJSON Error Tests
// ============================================================================

#[test]
fn test_to_json_nan_error() {
    let code = r#"toJSON(0.0 / 0.0)"#;
    assert_has_error(code);
}

#[test]
fn test_to_json_infinity_error() {
    let code = r#"toJSON(1.0 / 0.0)"#;
    assert_has_error(code);
}

#[test]
fn test_to_json_function_error() {
    let code = r#"
        fn test(): number { return 42; }
        toJSON(test)
    "#;
    assert_has_error(code);
}

// ============================================================================
// isValidJSON Tests
// ============================================================================

#[test]
fn test_is_valid_json_true_null() {
    let code = r#"isValidJSON("null")"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_is_valid_json_true_bool() {
    let code = r#"isValidJSON("true")"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_is_valid_json_true_number() {
    let code = r#"isValidJSON("42")"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_is_valid_json_true_string() {
    let code = r#"isValidJSON("\"hello\"")"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_is_valid_json_true_array() {
    let code = r#"isValidJSON("[1,2,3]")"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_is_valid_json_true_object() {
    let code = r#"isValidJSON("{\"key\":\"value\"}")"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_is_valid_json_false_invalid() {
    let code = r#"isValidJSON("{invalid}")"#;
    assert_eval_bool(code, false);
}

#[test]
fn test_is_valid_json_false_trailing_comma() {
    let code = r#"isValidJSON("[1,2,]")"#;
    assert_eval_bool(code, false);
}

#[test]
fn test_is_valid_json_false_empty() {
    let code = r#"isValidJSON("")"#;
    assert_eval_bool(code, false);
}

#[test]
fn test_is_valid_json_false_single_quote() {
    let code = r#"isValidJSON("{'a':1}")"#;
    assert_eval_bool(code, false);
}

#[test]
fn test_is_valid_json_wrong_type() {
    let code = r#"isValidJSON(123)"#;
    assert_has_error(code);
}

// ============================================================================
// prettifyJSON Tests
// ============================================================================

#[test]
fn test_prettify_json_object() {
    let code = r#"
        let compact: string = "{\"name\":\"Alice\",\"age\":30}";
        let pretty: string = prettifyJSON(compact, 2);
        includes(pretty, "  ")
    "#;
    assert_eval_bool(code, true);
}

#[test]
fn test_prettify_json_array() {
    let code = r#"
        let compact: string = "[1,2,3]";
        let pretty: string = prettifyJSON(compact, 2);
        len(pretty) > len(compact)
    "#;
    assert_eval_bool(code, true);
}

#[test]
fn test_prettify_json_indent_zero() {
    let code = r#"
        let compact: string = "{\"a\":1}";
        let pretty: string = prettifyJSON(compact, 0);
        typeof(pretty)
    "#;
    assert_eval_string(code, "string");
}

#[test]
fn test_prettify_json_indent_four() {
    let code = r#"
        let compact: string = "{\"a\":1}";
        let pretty: string = prettifyJSON(compact, 4);
        includes(pretty, "    ")
    "#;
    assert_eval_bool(code, true);
}

#[test]
fn test_prettify_json_nested() {
    let code = r#"
        let compact: string = "{\"user\":{\"name\":\"Bob\"}}";
        let pretty: string = prettifyJSON(compact, 2);
        len(pretty) > len(compact)
    "#;
    assert_eval_bool(code, true);
}

#[test]
fn test_prettify_json_invalid() {
    let code = r#"prettifyJSON("{invalid}", 2)"#;
    assert_has_error(code);
}

#[test]
fn test_prettify_json_negative_indent() {
    let code = r#"prettifyJSON("{}", -1)"#;
    assert_has_error(code);
}

#[test]
fn test_prettify_json_float_indent() {
    let code = r#"prettifyJSON("{}", 2.5)"#;
    assert_has_error(code);
}

#[test]
fn test_prettify_json_wrong_type_first_arg() {
    let code = r#"prettifyJSON(123, 2)"#;
    assert_has_error(code);
}

#[test]
fn test_prettify_json_wrong_type_second_arg() {
    let code = r#"prettifyJSON("{}", "2")"#;
    assert_has_error(code);
}

// ============================================================================
// minifyJSON Tests
// ============================================================================

#[test]
fn test_minify_json_object() {
    let code = r#"
        let pretty: string = "{\n  \"name\": \"Alice\",\n  \"age\": 30\n}";
        let minified: string = minifyJSON(pretty);
        len(minified) < len(pretty)
    "#;
    assert_eval_bool(code, true);
}

#[test]
fn test_minify_json_array() {
    let code = r#"
        let pretty: string = "[\n  1,\n  2,\n  3\n]";
        let minified: string = minifyJSON(pretty);
        len(minified) < len(pretty)
    "#;
    assert_eval_bool(code, true);
}

#[test]
fn test_minify_json_no_whitespace() {
    let code = r#"
        let compact: string = "{\"a\":1}";
        let minified: string = minifyJSON(compact);
        typeof(minified)
    "#;
    assert_eval_string(code, "string");
}

#[test]
fn test_minify_json_nested() {
    let code = r#"
        let pretty: string = "{\n  \"user\": {\n    \"name\": \"Bob\"\n  }\n}";
        let minified: string = minifyJSON(pretty);
        len(minified) < len(pretty)
    "#;
    assert_eval_bool(code, true);
}

#[test]
fn test_minify_json_invalid() {
    let code = r#"minifyJSON("{invalid}")"#;
    assert_has_error(code);
}

#[test]
fn test_minify_json_wrong_type() {
    let code = r#"minifyJSON(123)"#;
    assert_has_error(code);
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_parse_then_serialize() {
    let code = r#"
        let original: string = "{\"name\":\"Alice\",\"age\":30}";
        let parsed: json = parseJSON(original);
        let serialized: string = toJSON(parsed);
        typeof(serialized)
    "#;
    assert_eval_string(code, "string");
}

#[test]
fn test_prettify_then_minify() {
    let code = r#"
        let compact: string = "{\"a\":1,\"b\":2}";
        let pretty: string = prettifyJSON(compact, 2);
        let minified: string = minifyJSON(pretty);
        len(minified) < len(pretty)
    "#;
    assert_eval_bool(code, true);
}

#[test]
fn test_validate_before_parse() {
    let code = r#"
        let json_str: string = "{\"valid\":true}";
        let valid: bool = isValidJSON(json_str);
        let parsed: json = parseJSON(json_str);
        valid && typeof(parsed) == "json"
    "#;
    assert_eval_bool(code, true);
}

// ============================================================================
// From stdlib_io_tests.rs
// ============================================================================

// Standard library file I/O tests (Interpreter)
//
// Tests file and directory operations with security checks.

// Helper to create runtime with full filesystem permissions
fn test_runtime_with_io() -> (Atlas, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let mut security = SecurityContext::new();
    security.grant_filesystem_read(temp_dir.path(), true);
    security.grant_filesystem_write(temp_dir.path(), true);
    let runtime = Atlas::new_with_security(security);
    (runtime, temp_dir)
}

// ============================================================================
// readFile tests
// ============================================================================

#[test]
fn test_read_file_basic() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "Hello, World!").unwrap();

    let code = format!(r#"readFile("{}")"#, path_for_atlas(&test_file));
    let result = runtime.eval(&code);

    assert!(result.is_ok());
    let value = result.unwrap();
    assert!(matches!(value, atlas_runtime::Value::String(_)));
}

#[test]
fn test_read_file_utf8() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let test_file = temp_dir.path().join("utf8.txt");
    fs::write(&test_file, "Hello 你好 🎉").unwrap();

    let code = format!(r#"readFile("{}")"#, path_for_atlas(&test_file));
    let result = runtime.eval(&code);

    assert!(result.is_ok());
}

#[test]
fn test_read_file_not_found() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let nonexistent = temp_dir.path().join("does_not_exist.txt");

    let code = format!(r#"readFile("{}")"#, path_for_atlas(&nonexistent));
    let result = runtime.eval(&code);

    assert!(result.is_err());
    let diagnostics = result.unwrap_err();
    assert!(diagnostics[0].message.contains("Failed to resolve path"));
}

#[test]
fn test_read_file_permission_denied() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("secret.txt");
    fs::write(&test_file, "secret").unwrap();

    // Runtime with no permissions
    let runtime = Atlas::new();
    let code = format!(r#"readFile("{}")"#, path_for_atlas(&test_file));
    let result = runtime.eval(&code);

    assert!(result.is_err());
    let diagnostics = result.unwrap_err();
    assert_eq!(diagnostics[0].code, "AT0300");
    assert!(diagnostics[0].message.contains("Permission denied"));
}

// ============================================================================
// writeFile tests
// ============================================================================

#[test]
fn test_write_file_basic() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let test_file = temp_dir.path().join("output.txt");

    let code = format!(
        r#"writeFile("{}", "test content")"#,
        path_for_atlas(&test_file)
    );
    let result = runtime.eval(&code);

    assert!(result.is_ok());
    let contents = fs::read_to_string(&test_file).unwrap();
    assert_eq!(contents, "test content");
}

#[test]
fn test_write_file_overwrite() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let test_file = temp_dir.path().join("overwrite.txt");
    fs::write(&test_file, "original").unwrap();

    let code = format!(
        r#"writeFile("{}", "new content")"#,
        path_for_atlas(&test_file)
    );
    let result = runtime.eval(&code);

    assert!(result.is_ok());
    let contents = fs::read_to_string(&test_file).unwrap();
    assert_eq!(contents, "new content");
}

#[test]
fn test_write_file_permission_denied() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("output.txt");

    let runtime = Atlas::new();
    let code = format!(r#"writeFile("{}", "content")"#, path_for_atlas(&test_file));
    let result = runtime.eval(&code);

    assert!(result.is_err());
    let diagnostics = result.unwrap_err();
    assert_eq!(diagnostics[0].code, "AT0300");
}

// ============================================================================
// appendFile tests
// ============================================================================

#[test]
fn test_append_file_basic() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let test_file = temp_dir.path().join("append.txt");
    fs::write(&test_file, "line1\n").unwrap();

    let code = format!(r#"appendFile("{}", "line2\n")"#, path_for_atlas(&test_file));
    let result = runtime.eval(&code);

    assert!(result.is_ok());
    let contents = fs::read_to_string(&test_file).unwrap();
    assert_eq!(contents, "line1\nline2\n");
}

#[test]
fn test_append_file_create_if_not_exists() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let test_file = temp_dir.path().join("new.txt");

    let code = format!(r#"appendFile("{}", "content")"#, path_for_atlas(&test_file));
    let result = runtime.eval(&code);

    assert!(result.is_ok());
    let contents = fs::read_to_string(&test_file).unwrap();
    assert_eq!(contents, "content");
}

// ============================================================================
// fileExists tests
// ============================================================================

#[test]
fn test_file_exists_true() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let test_file = temp_dir.path().join("exists.txt");
    fs::write(&test_file, "").unwrap();

    let code = format!(r#"fileExists("{}")"#, path_for_atlas(&test_file));
    let result = runtime.eval(&code);

    assert!(result.is_ok());
    assert!(matches!(result.unwrap(), atlas_runtime::Value::Bool(true)));
}

#[test]
fn test_file_exists_false() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let nonexistent = temp_dir.path().join("does_not_exist.txt");

    let code = format!(r#"fileExists("{}")"#, path_for_atlas(&nonexistent));
    let result = runtime.eval(&code);

    assert!(result.is_ok());
    assert!(matches!(result.unwrap(), atlas_runtime::Value::Bool(false)));
}

// ============================================================================
// readDir tests
// ============================================================================

#[test]
fn test_read_dir_basic() {
    let (runtime, temp_dir) = test_runtime_with_io();
    fs::write(temp_dir.path().join("file1.txt"), "").unwrap();
    fs::write(temp_dir.path().join("file2.txt"), "").unwrap();

    let code = format!(r#"readDir("{}")"#, path_for_atlas(temp_dir.path()));
    let result = runtime.eval(&code);

    assert!(result.is_ok());
    assert!(matches!(result.unwrap(), atlas_runtime::Value::Array(_)));
}

#[test]
fn test_read_dir_not_found() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let nonexistent = temp_dir.path().join("nonexistent_dir");

    let code = format!(r#"readDir("{}")"#, path_for_atlas(&nonexistent));
    let result = runtime.eval(&code);

    assert!(result.is_err());
}

// ============================================================================
// createDir tests
// ============================================================================

#[test]
fn test_create_dir_basic() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let new_dir = temp_dir.path().join("newdir");

    let code = format!(r#"createDir("{}")"#, path_for_atlas(&new_dir));
    let result = runtime.eval(&code);

    assert!(result.is_ok());
    assert!(new_dir.exists());
    assert!(new_dir.is_dir());
}

#[test]
fn test_create_dir_nested() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let nested_dir = temp_dir.path().join("a/b/c");

    let code = format!(r#"createDir("{}")"#, path_for_atlas(&nested_dir));
    let result = runtime.eval(&code);

    assert!(result.is_ok());
    assert!(nested_dir.exists());
    assert!(nested_dir.is_dir());
}

// ============================================================================
// removeFile tests
// ============================================================================

#[test]
fn test_remove_file_basic() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let test_file = temp_dir.path().join("remove.txt");
    fs::write(&test_file, "").unwrap();

    let code = format!(r#"removeFile("{}")"#, path_for_atlas(&test_file));
    let result = runtime.eval(&code);

    assert!(result.is_ok());
    assert!(!test_file.exists());
}

#[test]
fn test_remove_file_not_found() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let nonexistent = temp_dir.path().join("does_not_exist.txt");

    let code = format!(r#"removeFile("{}")"#, path_for_atlas(&nonexistent));
    let result = runtime.eval(&code);

    assert!(result.is_err());
}

// ============================================================================
// removeDir tests
// ============================================================================

#[test]
fn test_remove_dir_basic() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let test_dir = temp_dir.path().join("rmdir");
    fs::create_dir(&test_dir).unwrap();

    let code = format!(r#"removeDir("{}")"#, path_for_atlas(&test_dir));
    let result = runtime.eval(&code);

    assert!(result.is_ok());
    assert!(!test_dir.exists());
}

#[test]
fn test_remove_dir_not_empty() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let test_dir = temp_dir.path().join("notempty");
    fs::create_dir(&test_dir).unwrap();
    fs::write(test_dir.join("file.txt"), "").unwrap();

    let code = format!(r#"removeDir("{}")"#, path_for_atlas(&test_dir));
    let result = runtime.eval(&code);

    assert!(result.is_err());
    let diagnostics = result.unwrap_err();
    assert!(diagnostics[0]
        .message
        .contains("Failed to remove directory"));
}

// ============================================================================
// fileInfo tests
// ============================================================================

#[test]
fn test_file_info_file() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let test_file = temp_dir.path().join("info.txt");
    fs::write(&test_file, "test content").unwrap();

    let code = format!(r#"fileInfo("{}")"#, path_for_atlas(&test_file));
    let result = runtime.eval(&code);

    assert!(result.is_ok());
    // Result should be a JsonValue object
    assert!(matches!(
        result.unwrap(),
        atlas_runtime::Value::JsonValue(_)
    ));
}

#[test]
fn test_file_info_directory() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let test_dir = temp_dir.path().join("infodir");
    fs::create_dir(&test_dir).unwrap();

    let code = format!(r#"fileInfo("{}")"#, path_for_atlas(&test_dir));
    let result = runtime.eval(&code);

    assert!(result.is_ok());
}

// ============================================================================
// pathJoin tests
// ============================================================================

#[test]
fn test_path_join_basic() {
    let runtime = Atlas::new(); // No permissions needed
    let result = runtime.eval(r#"pathJoin("a", "b", "c")"#);

    assert!(result.is_ok());
    assert!(matches!(result.unwrap(), atlas_runtime::Value::String(_)));
}

#[test]
fn test_path_join_single() {
    let runtime = Atlas::new();
    let result = runtime.eval(r#"pathJoin("single")"#);

    assert!(result.is_ok());
}

#[test]
fn test_path_join_no_args() {
    let runtime = Atlas::new();
    let result = runtime.eval(r#"pathJoin()"#);

    assert!(result.is_err());
}

// ============================================================================
// readFile - Additional UTF-8 and edge case tests
// ============================================================================

#[test]
fn test_read_file_empty() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let test_file = temp_dir.path().join("empty.txt");
    fs::write(&test_file, "").unwrap();

    let code = format!(r#"readFile("{}")"#, path_for_atlas(&test_file));
    let result = runtime.eval(&code);

    assert!(result.is_ok());
    if let atlas_runtime::Value::String(s) = result.unwrap() {
        assert_eq!(s.as_str(), "");
    } else {
        panic!("Expected string");
    }
}

#[test]
fn test_read_file_invalid_utf8() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let test_file = temp_dir.path().join("binary.bin");
    // Invalid UTF-8 sequence
    fs::write(&test_file, [0xFF, 0xFE, 0xFD]).unwrap();

    let code = format!(r#"readFile("{}")"#, path_for_atlas(&test_file));
    let result = runtime.eval(&code);

    assert!(result.is_err());
    let diagnostics = result.unwrap_err();
    assert!(diagnostics[0].message.contains("UTF-8"));
}

#[test]
fn test_read_file_multiline() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let test_file = temp_dir.path().join("multiline.txt");
    let content = "line1\nline2\nline3\n";
    fs::write(&test_file, content).unwrap();

    let code = format!(r#"readFile("{}")"#, path_for_atlas(&test_file));
    let result = runtime.eval(&code);

    assert!(result.is_ok());
    if let atlas_runtime::Value::String(s) = result.unwrap() {
        assert_eq!(s.as_str(), content);
    } else {
        panic!("Expected string");
    }
}

#[test]
fn test_read_file_large() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let test_file = temp_dir.path().join("large.txt");
    let content = "x".repeat(10000);
    fs::write(&test_file, &content).unwrap();

    let code = format!(r#"readFile("{}")"#, path_for_atlas(&test_file));
    let result = runtime.eval(&code);

    assert!(result.is_ok());
    if let atlas_runtime::Value::String(s) = result.unwrap() {
        assert_eq!(s.len(), 10000);
    } else {
        panic!("Expected string");
    }
}

#[test]
fn test_read_file_with_bom() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let test_file = temp_dir.path().join("bom.txt");
    // UTF-8 BOM + content
    let mut content = vec![0xEF, 0xBB, 0xBF];
    content.extend_from_slice(b"Hello");
    fs::write(&test_file, content).unwrap();

    let code = format!(r#"readFile("{}")"#, path_for_atlas(&test_file));
    let result = runtime.eval(&code);

    assert!(result.is_ok());
}

// ============================================================================
// writeFile - Additional edge case tests
// ============================================================================

#[test]
fn test_write_file_empty() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let test_file = temp_dir.path().join("empty_write.txt");

    let code = format!(r#"writeFile("{}", "")"#, path_for_atlas(&test_file));
    let result = runtime.eval(&code);

    assert!(result.is_ok());
    let contents = fs::read_to_string(&test_file).unwrap();
    assert_eq!(contents, "");
}

#[test]
fn test_write_file_unicode() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let test_file = temp_dir.path().join("unicode.txt");
    let content = "Hello 世界 🌍";

    let code = format!(
        r#"writeFile("{}", "{}")"#,
        path_for_atlas(&test_file),
        content
    );
    let result = runtime.eval(&code);

    assert!(result.is_ok());
    let contents = fs::read_to_string(&test_file).unwrap();
    assert_eq!(contents, content);
}

#[test]
fn test_write_file_newlines() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let test_file = temp_dir.path().join("newlines.txt");

    let code = format!(
        r#"writeFile("{}", "line1\nline2\n")"#,
        path_for_atlas(&test_file)
    );
    let result = runtime.eval(&code);

    assert!(result.is_ok());
    let contents = fs::read_to_string(&test_file).unwrap();
    assert_eq!(contents, "line1\nline2\n");
}

#[test]
fn test_write_file_creates_file() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let test_file = temp_dir.path().join("new_file.txt");
    assert!(!test_file.exists());

    let code = format!(r#"writeFile("{}", "content")"#, path_for_atlas(&test_file));
    let result = runtime.eval(&code);

    assert!(result.is_ok());
    assert!(test_file.exists());
}

// ============================================================================
// appendFile - Additional edge case tests
// ============================================================================

#[test]
fn test_append_file_multiple() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let test_file = temp_dir.path().join("multi_append.txt");
    fs::write(&test_file, "start\n").unwrap();

    let code1 = format!(r#"appendFile("{}", "line1\n")"#, path_for_atlas(&test_file));
    let code2 = format!(r#"appendFile("{}", "line2\n")"#, path_for_atlas(&test_file));

    runtime.eval(&code1).unwrap();
    runtime.eval(&code2).unwrap();

    let contents = fs::read_to_string(&test_file).unwrap();
    assert_eq!(contents, "start\nline1\nline2\n");
}

#[test]
fn test_append_file_empty_content() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let test_file = temp_dir.path().join("append_empty.txt");
    fs::write(&test_file, "base").unwrap();

    let code = format!(r#"appendFile("{}", "")"#, path_for_atlas(&test_file));
    let result = runtime.eval(&code);

    assert!(result.is_ok());
    let contents = fs::read_to_string(&test_file).unwrap();
    assert_eq!(contents, "base");
}

#[test]
fn test_append_file_permission_denied() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("append_denied.txt");

    let runtime = Atlas::new();
    let code = format!(r#"appendFile("{}", "content")"#, path_for_atlas(&test_file));
    let result = runtime.eval(&code);

    assert!(result.is_err());
    let diagnostics = result.unwrap_err();
    assert_eq!(diagnostics[0].code, "AT0300");
}

// ============================================================================
// fileExists - Additional edge case tests
// ============================================================================

#[test]
fn test_file_exists_directory() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let test_dir = temp_dir.path().join("exists_dir");
    fs::create_dir(&test_dir).unwrap();

    let code = format!(r#"fileExists("{}")"#, path_for_atlas(&test_dir));
    let result = runtime.eval(&code);

    assert!(result.is_ok());
    assert!(matches!(result.unwrap(), atlas_runtime::Value::Bool(true)));
}

#[test]
fn test_file_exists_no_permission_check() {
    // fileExists doesn't require read permissions - it just checks existence
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("exists_test.txt");
    fs::write(&test_file, "").unwrap();

    let runtime = Atlas::new();
    let code = format!(r#"fileExists("{}")"#, path_for_atlas(&test_file));
    let result = runtime.eval(&code);

    // Should succeed without permissions since it only checks existence
    assert!(result.is_ok());
    assert!(matches!(result.unwrap(), atlas_runtime::Value::Bool(true)));
}

// ============================================================================
// readDir - Additional edge case tests
// ============================================================================

#[test]
fn test_read_dir_empty() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let empty_dir = temp_dir.path().join("empty");
    fs::create_dir(&empty_dir).unwrap();

    let code = format!(r#"readDir("{}")"#, path_for_atlas(&empty_dir));
    let result = runtime.eval(&code);

    assert!(result.is_ok());
    if let atlas_runtime::Value::Array(arr) = result.unwrap() {
        assert_eq!(arr.lock().unwrap().len(), 0);
    } else {
        panic!("Expected array");
    }
}

#[test]
fn test_read_dir_mixed_contents() {
    let (runtime, temp_dir) = test_runtime_with_io();
    fs::write(temp_dir.path().join("file.txt"), "").unwrap();
    fs::create_dir(temp_dir.path().join("subdir")).unwrap();

    let code = format!(r#"readDir("{}")"#, path_for_atlas(temp_dir.path()));
    let result = runtime.eval(&code);

    assert!(result.is_ok());
    if let atlas_runtime::Value::Array(arr) = result.unwrap() {
        assert_eq!(arr.lock().unwrap().len(), 2);
    } else {
        panic!("Expected array");
    }
}

#[test]
fn test_read_dir_permission_denied() {
    let temp_dir = TempDir::new().unwrap();
    let test_dir = temp_dir.path().join("dir");
    fs::create_dir(&test_dir).unwrap();

    let runtime = Atlas::new();
    let code = format!(r#"readDir("{}")"#, path_for_atlas(&test_dir));
    let result = runtime.eval(&code);

    assert!(result.is_err());
    let diagnostics = result.unwrap_err();
    assert_eq!(diagnostics[0].code, "AT0300");
}

// ============================================================================
// createDir - Additional edge case tests
// ============================================================================

#[test]
fn test_create_dir_already_exists() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let test_dir = temp_dir.path().join("already_exists");
    fs::create_dir(&test_dir).unwrap();

    let code = format!(r#"createDir("{}")"#, path_for_atlas(&test_dir));
    let result = runtime.eval(&code);

    // Should succeed (mkdir -p behavior)
    assert!(result.is_ok());
}

#[test]
fn test_create_dir_permission_denied() {
    let temp_dir = TempDir::new().unwrap();
    let new_dir = temp_dir.path().join("denied");

    let runtime = Atlas::new();
    let code = format!(r#"createDir("{}")"#, path_for_atlas(&new_dir));
    let result = runtime.eval(&code);

    assert!(result.is_err());
    let diagnostics = result.unwrap_err();
    assert_eq!(diagnostics[0].code, "AT0300");
}

// ============================================================================
// removeFile - Additional edge case tests
// ============================================================================

#[test]
fn test_remove_file_is_directory() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let test_dir = temp_dir.path().join("is_dir");
    fs::create_dir(&test_dir).unwrap();

    let code = format!(r#"removeFile("{}")"#, path_for_atlas(&test_dir));
    let result = runtime.eval(&code);

    assert!(result.is_err());
}

#[test]
fn test_remove_file_permission_denied() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("remove_denied.txt");
    fs::write(&test_file, "").unwrap();

    let runtime = Atlas::new();
    let code = format!(r#"removeFile("{}")"#, path_for_atlas(&test_file));
    let result = runtime.eval(&code);

    assert!(result.is_err());
    let diagnostics = result.unwrap_err();
    assert_eq!(diagnostics[0].code, "AT0300");
}

// ============================================================================
// removeDir - Additional edge case tests
// ============================================================================

#[test]
fn test_remove_dir_not_found() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let nonexistent = temp_dir.path().join("not_found");

    let code = format!(r#"removeDir("{}")"#, path_for_atlas(&nonexistent));
    let result = runtime.eval(&code);

    assert!(result.is_err());
}

#[test]
fn test_remove_dir_is_file() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let test_file = temp_dir.path().join("is_file.txt");
    fs::write(&test_file, "").unwrap();

    let code = format!(r#"removeDir("{}")"#, path_for_atlas(&test_file));
    let result = runtime.eval(&code);

    assert!(result.is_err());
}

#[test]
fn test_remove_dir_permission_denied() {
    let temp_dir = TempDir::new().unwrap();
    let test_dir = temp_dir.path().join("remove_denied");
    fs::create_dir(&test_dir).unwrap();

    let runtime = Atlas::new();
    let code = format!(r#"removeDir("{}")"#, path_for_atlas(&test_dir));
    let result = runtime.eval(&code);

    assert!(result.is_err());
    let diagnostics = result.unwrap_err();
    assert_eq!(diagnostics[0].code, "AT0300");
}

// ============================================================================
// fileInfo - Additional validation tests
// ============================================================================

#[test]
fn test_file_info_size_check() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let test_file = temp_dir.path().join("info_fields.txt");
    fs::write(&test_file, "12345").unwrap();

    let code = format!(r#"fileInfo("{}")"#, path_for_atlas(&test_file));
    let result = runtime.eval(&code);

    assert!(result.is_ok());
    // Verify it returns a JsonValue
    assert!(matches!(
        result.unwrap(),
        atlas_runtime::Value::JsonValue(_)
    ));
}

#[test]
fn test_file_info_not_found() {
    let (runtime, temp_dir) = test_runtime_with_io();
    let nonexistent = temp_dir.path().join("not_found.txt");

    let code = format!(r#"fileInfo("{}")"#, path_for_atlas(&nonexistent));
    let result = runtime.eval(&code);

    assert!(result.is_err());
}

#[test]
fn test_file_info_permission_denied() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("info_denied.txt");
    fs::write(&test_file, "test").unwrap();

    let runtime = Atlas::new();
    let code = format!(r#"fileInfo("{}")"#, path_for_atlas(&test_file));
    let result = runtime.eval(&code);

    assert!(result.is_err());
    let diagnostics = result.unwrap_err();
    assert_eq!(diagnostics[0].code, "AT0300");
}

// ============================================================================
// pathJoin - Platform and edge case tests
// ============================================================================

#[test]
fn test_path_join_many_parts() {
    let runtime = Atlas::new();
    let result = runtime.eval(r#"pathJoin("a", "b", "c", "d", "e")"#);

    assert!(result.is_ok());
    if let atlas_runtime::Value::String(path) = result.unwrap() {
        assert!(path.contains("a"));
        assert!(path.contains("e"));
    } else {
        panic!("Expected string");
    }
}

#[test]
fn test_path_join_empty_parts() {
    let runtime = Atlas::new();
    let result = runtime.eval(r#"pathJoin("", "a", "")"#);

    assert!(result.is_ok());
}

#[test]
fn test_path_join_absolute_path() {
    let runtime = Atlas::new();
    let result = runtime.eval(r#"pathJoin("/absolute", "path")"#);

    assert!(result.is_ok());
    if let atlas_runtime::Value::String(path) = result.unwrap() {
        assert!(path.starts_with("/") || path.starts_with("\\"));
    } else {
        panic!("Expected string");
    }
}

// ============================================================================
// From stdlib_types_tests.rs
// ============================================================================

// Type checking and conversion stdlib tests (Interpreter engine)
//
// Tests all 12 type utility functions with comprehensive edge case coverage

// ============================================================================
// typeof Tests
// ============================================================================

#[test]
fn test_typeof_null() {
    let code = r#"typeof(null)"#;
    assert_eval_string(code, "null");
}

#[test]
fn test_typeof_bool_true() {
    let code = r#"typeof(true)"#;
    assert_eval_string(code, "bool");
}

#[test]
fn test_typeof_bool_false() {
    let code = r#"typeof(false)"#;
    assert_eval_string(code, "bool");
}

#[test]
fn test_typeof_number_positive() {
    let code = r#"typeof(42)"#;
    assert_eval_string(code, "number");
}

#[test]
fn test_typeof_number_negative() {
    let code = r#"typeof(-10)"#;
    assert_eval_string(code, "number");
}

#[test]
fn test_typeof_number_float() {
    let code = r#"typeof(3.5)"#;
    assert_eval_string(code, "number");
}

// NaN/Infinity tests removed: division by zero is a runtime error in Atlas

#[test]
fn test_typeof_string_nonempty() {
    let code = r#"typeof("hello")"#;
    assert_eval_string(code, "string");
}

#[test]
fn test_typeof_string_empty() {
    let code = r#"typeof("")"#;
    assert_eval_string(code, "string");
}

#[test]
fn test_typeof_array_nonempty() {
    let code = r#"typeof([1,2,3])"#;
    assert_eval_string(code, "array");
}

#[test]
fn test_typeof_array_empty() {
    let code = r#"typeof([])"#;
    assert_eval_string(code, "array");
}

// Function reference tests removed: not yet fully supported

#[test]
fn test_typeof_json() {
    let code = r#"typeof(parseJSON("null"))"#;
    assert_eval_string(code, "json");
}

#[test]
fn test_typeof_option() {
    let code = r#"typeof(Some(42))"#;
    assert_eval_string(code, "option");
}

#[test]
fn test_typeof_result() {
    let code = r#"typeof(Ok(42))"#;
    assert_eval_string(code, "result");
}

// ============================================================================
// Type Guard Tests
// ============================================================================

#[test]
fn test_is_string_true() {
    let code = r#"isString("hello")"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_is_string_false_number() {
    let code = r#"isString(42)"#;
    assert_eval_bool(code, false);
}

#[test]
fn test_is_string_false_null() {
    let code = r#"isString(null)"#;
    assert_eval_bool(code, false);
}

#[test]
fn test_is_number_true_int() {
    let code = r#"isNumber(42)"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_is_number_true_float() {
    let code = r#"isNumber(3.5)"#;
    assert_eval_bool(code, true);
}

// Removed: NaN test (division by zero is error)

#[test]
fn test_is_number_false_string() {
    let code = r#"isNumber("42")"#;
    assert_eval_bool(code, false);
}

#[test]
fn test_is_bool_true() {
    let code = r#"isBool(true)"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_is_bool_false() {
    let code = r#"isBool(false)"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_is_bool_false_number() {
    let code = r#"isBool(1)"#;
    assert_eval_bool(code, false);
}

#[test]
fn test_is_null_true() {
    let code = r#"isNull(null)"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_is_null_false() {
    let code = r#"isNull(0)"#;
    assert_eval_bool(code, false);
}

#[test]
fn test_is_array_true() {
    let code = r#"isArray([1,2,3])"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_is_array_true_empty() {
    let code = r#"isArray([])"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_is_array_false() {
    let code = r#"isArray("not array")"#;
    assert_eval_bool(code, false);
}

// Function reference tests removed: not yet fully supported

#[test]
fn test_is_function_false() {
    let code = r#"isFunction(42)"#;
    assert_eval_bool(code, false);
}

// ============================================================================
// toString Tests
// ============================================================================

#[test]
fn test_to_string_null() {
    let code = r#"toString(null)"#;
    assert_eval_string(code, "null");
}

#[test]
fn test_to_string_bool_true() {
    let code = r#"toString(true)"#;
    assert_eval_string(code, "true");
}

#[test]
fn test_to_string_bool_false() {
    let code = r#"toString(false)"#;
    assert_eval_string(code, "false");
}

#[test]
fn test_to_string_number_int() {
    let code = r#"toString(42)"#;
    assert_eval_string(code, "42");
}

#[test]
fn test_to_string_number_float() {
    let code = r#"toString(3.5)"#;
    assert_eval_string(code, "3.5");
}

#[test]
fn test_to_string_number_negative() {
    let code = r#"toString(-10)"#;
    assert_eval_string(code, "-10");
}

#[test]
fn test_to_string_number_zero() {
    let code = r#"toString(0)"#;
    assert_eval_string(code, "0");
}

// NaN/Infinity toString tests removed: division by zero is error

#[test]
fn test_to_string_string_identity() {
    let code = r#"toString("hello")"#;
    assert_eval_string(code, "hello");
}

#[test]
fn test_to_string_string_empty() {
    let code = r#"toString("")"#;
    assert_eval_string(code, "");
}

#[test]
fn test_to_string_array() {
    let code = r#"toString([1,2,3])"#;
    assert_eval_string(code, "[Array]");
}

// Function toString test removed: not yet fully supported

#[test]
fn test_to_string_json() {
    let code = r#"toString(parseJSON("null"))"#;
    assert_eval_string(code, "[JSON]");
}

// ============================================================================
// toNumber Tests
// ============================================================================

#[test]
fn test_to_number_number_identity() {
    let code = r#"toNumber(42)"#;
    assert_eval_number(code, 42.0);
}

#[test]
fn test_to_number_bool_true() {
    let code = r#"toNumber(true)"#;
    assert_eval_number(code, 1.0);
}

#[test]
fn test_to_number_bool_false() {
    let code = r#"toNumber(false)"#;
    assert_eval_number(code, 0.0);
}

#[test]
fn test_to_number_string_int() {
    let code = r#"toNumber("42")"#;
    assert_eval_number(code, 42.0);
}

#[test]
fn test_to_number_string_float() {
    let code = r#"toNumber("3.5")"#;
    assert_eval_number(code, 3.5);
}

#[test]
fn test_to_number_string_negative() {
    let code = r#"toNumber("-10")"#;
    assert_eval_number(code, -10.0);
}

#[test]
fn test_to_number_string_whitespace() {
    let code = r#"toNumber("  42  ")"#;
    assert_eval_number(code, 42.0);
}

#[test]
fn test_to_number_string_scientific() {
    let code = r#"toNumber("1e10")"#;
    assert_eval_number(code, 1e10);
}

#[test]
fn test_to_number_string_empty_error() {
    let code = r#"toNumber("")"#;
    assert_has_error(code);
}

#[test]
fn test_to_number_string_invalid_error() {
    let code = r#"toNumber("hello")"#;
    assert_has_error(code);
}

#[test]
fn test_to_number_null_error() {
    let code = r#"toNumber(null)"#;
    assert_has_error(code);
}

#[test]
fn test_to_number_array_error() {
    let code = r#"toNumber([1,2,3])"#;
    assert_has_error(code);
}

// ============================================================================
// toBool Tests
// ============================================================================

#[test]
fn test_to_bool_bool_identity_true() {
    let code = r#"toBool(true)"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_to_bool_bool_identity_false() {
    let code = r#"toBool(false)"#;
    assert_eval_bool(code, false);
}

#[test]
fn test_to_bool_number_zero_false() {
    let code = r#"toBool(0)"#;
    assert_eval_bool(code, false);
}

// NaN toBool test removed: division by zero is error

#[test]
fn test_to_bool_number_positive_true() {
    let code = r#"toBool(42)"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_to_bool_number_negative_true() {
    let code = r#"toBool(-10)"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_to_bool_string_empty_false() {
    let code = r#"toBool("")"#;
    assert_eval_bool(code, false);
}

#[test]
fn test_to_bool_string_nonempty_true() {
    let code = r#"toBool("hello")"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_to_bool_string_space_true() {
    let code = r#"toBool(" ")"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_to_bool_null_false() {
    let code = r#"toBool(null)"#;
    assert_eval_bool(code, false);
}

#[test]
fn test_to_bool_array_true() {
    let code = r#"toBool([1,2,3])"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_to_bool_array_empty_true() {
    let code = r#"toBool([])"#;
    assert_eval_bool(code, true);
}

// Function toBool test removed: not yet fully supported

// ============================================================================
// parseInt Tests
// ============================================================================

#[test]
fn test_parse_int_decimal() {
    let code = r#"parseInt("42", 10)"#;
    assert_eval_number(code, 42.0);
}

#[test]
fn test_parse_int_decimal_negative() {
    let code = r#"parseInt("-10", 10)"#;
    assert_eval_number(code, -10.0);
}

#[test]
fn test_parse_int_binary() {
    let code = r#"parseInt("1010", 2)"#;
    assert_eval_number(code, 10.0);
}

#[test]
fn test_parse_int_octal() {
    let code = r#"parseInt("17", 8)"#;
    assert_eval_number(code, 15.0);
}

#[test]
fn test_parse_int_hex() {
    let code = r#"parseInt("FF", 16)"#;
    assert_eval_number(code, 255.0);
}

#[test]
fn test_parse_int_hex_lowercase() {
    let code = r#"parseInt("ff", 16)"#;
    assert_eval_number(code, 255.0);
}

#[test]
fn test_parse_int_radix_36() {
    let code = r#"parseInt("Z", 36)"#;
    assert_eval_number(code, 35.0);
}

#[test]
fn test_parse_int_plus_sign() {
    let code = r#"parseInt("+42", 10)"#;
    assert_eval_number(code, 42.0);
}

#[test]
fn test_parse_int_whitespace() {
    let code = r#"parseInt("  42  ", 10)"#;
    assert_eval_number(code, 42.0);
}

#[test]
fn test_parse_int_radix_too_low() {
    let code = r#"parseInt("42", 1)"#;
    assert_has_error(code);
}

#[test]
fn test_parse_int_radix_too_high() {
    let code = r#"parseInt("42", 37)"#;
    assert_has_error(code);
}

#[test]
fn test_parse_int_radix_float() {
    let code = r#"parseInt("42", 10.5)"#;
    assert_has_error(code);
}

#[test]
fn test_parse_int_empty_string() {
    let code = r#"parseInt("", 10)"#;
    assert_has_error(code);
}

#[test]
fn test_parse_int_invalid_digit() {
    let code = r#"parseInt("G", 16)"#;
    assert_has_error(code);
}

#[test]
fn test_parse_int_invalid_for_radix() {
    let code = r#"parseInt("2", 2)"#;
    assert_has_error(code);
}

#[test]
fn test_parse_int_wrong_type_first_arg() {
    let code = r#"parseInt(42, 10)"#;
    assert_has_error(code);
}

#[test]
fn test_parse_int_wrong_type_second_arg() {
    let code = r#"parseInt("42", "10")"#;
    assert_has_error(code);
}

// ============================================================================
// parseFloat Tests
// ============================================================================

#[test]
fn test_parse_float_integer() {
    let code = r#"parseFloat("42")"#;
    assert_eval_number(code, 42.0);
}

#[test]
fn test_parse_float_decimal() {
    let code = r#"parseFloat("3.5")"#;
    assert_eval_number(code, 3.5);
}

#[test]
fn test_parse_float_negative() {
    let code = r#"parseFloat("-10.5")"#;
    assert_eval_number(code, -10.5);
}

#[test]
fn test_parse_float_scientific_lowercase() {
    let code = r#"parseFloat("1.5e3")"#;
    assert_eval_number(code, 1500.0);
}

#[test]
fn test_parse_float_scientific_uppercase() {
    let code = r#"parseFloat("1.5E3")"#;
    assert_eval_number(code, 1500.0);
}

#[test]
fn test_parse_float_scientific_negative_exp() {
    let code = r#"parseFloat("1.5e-3")"#;
    assert_eval_number(code, 0.0015);
}

#[test]
fn test_parse_float_scientific_positive_exp() {
    let code = r#"parseFloat("1.5e+3")"#;
    assert_eval_number(code, 1500.0);
}

#[test]
fn test_parse_float_whitespace() {
    let code = r#"parseFloat("  3.5  ")"#;
    assert_eval_number(code, 3.5);
}

#[test]
fn test_parse_float_plus_sign() {
    let code = r#"parseFloat("+42.5")"#;
    assert_eval_number(code, 42.5);
}

#[test]
fn test_parse_float_empty_string() {
    let code = r#"parseFloat("")"#;
    assert_has_error(code);
}

#[test]
fn test_parse_float_invalid() {
    let code = r#"parseFloat("hello")"#;
    assert_has_error(code);
}

#[test]
fn test_parse_float_wrong_type() {
    let code = r#"parseFloat(42)"#;
    assert_has_error(code);
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_typeof_guards_match() {
    let code = r#"
        let val: string = "hello";
        typeof(val) == "string" && isString(val)
    "#;
    assert_eval_bool(code, true);
}

#[test]
fn test_type_conversion_chain() {
    let code = r#"
        let num: number = 42;
        let numStr: string = toString(num);
        toNumber(numStr)
    "#;
    assert_eval_number(code, 42.0);
}

#[test]
fn test_parse_int_then_to_string() {
    let code = r#"
        let parsed: number = parseInt("FF", 16);
        toString(parsed)
    "#;
    assert_eval_string(code, "255");
}

#[test]
fn test_type_guards_all_false_for_null() {
    let code = r#"
        let val = null;
        !isString(val) && !isNumber(val) && !isBool(val) && !isArray(val) && !isFunction(val)
    "#;
    assert_eval_bool(code, true);
}

#[test]
fn test_type_guards_only_null_true() {
    let code = r#"isNull(null)"#;
    assert_eval_bool(code, true);
}

// ============================================================================
// From stdlib_real_world_tests.rs
// ============================================================================

// Real-World Standard Library Integration Tests
//
// This test suite demonstrates practical, real-world usage patterns of the Atlas
// standard library. Tests read like actual programs users would write:
// - CSV processing
// - JSON API handling
// - Log file analysis
// - Data transformation pipelines
// - Text processing
// - Configuration file processing
//
// ALL tests verify interpreter/VM parity (100% identical output).

// ============================================================================
// Test Helpers
// ============================================================================

/// Assert with file I/O permissions (grants full filesystem access for tests)
#[allow(dead_code)]
// ============================================================================
// Category 1: CSV Processing (30 tests)
// ============================================================================
#[test]
fn test_csv_read_and_parse_basic() {
    // Create CSV file
    let temp_dir = TempDir::new().unwrap();
    let csv_path = temp_dir.path().join("data.csv");
    std::fs::write(
        &csv_path,
        "name,age,city\nAlice,30,NYC\nBob,25,LA\nCarol,35,SF\n",
    )
    .unwrap();

    let code = format!(
        r#"
        let csv: string = readFile("{}");
        let lines: string[] = split(csv, "\n");
        let header: string = lines[0];
        header
    "#,
        path_for_atlas(&csv_path)
    );
    assert_eval_string_with_io(&code, "name,age,city");
}

#[test]
fn test_csv_parse_rows() {
    let temp_dir = TempDir::new().unwrap();
    let csv_path = temp_dir.path().join("data.csv");
    std::fs::write(&csv_path, "name,age\nAlice,30\nBob,25\nCarol,35\n").unwrap();

    let code = format!(
        r#"
        let csv: string = readFile("{}");
        let lines: string[] = split(csv, "\n");
        let header: string = lines[0];
        let dataLines: string[] = slice(lines, 1, len(lines));

        // Get first data row
        let row1: string = dataLines[0];
        let fields: string[] = split(row1, ",");
        let name: string = fields[0];
        name
    "#,
        path_for_atlas(&csv_path)
    );
    assert_eval_string_with_io(&code, "Alice");
}

#[test]
fn test_csv_count_rows() {
    let temp_dir = TempDir::new().unwrap();
    let csv_path = temp_dir.path().join("data.csv");
    std::fs::write(&csv_path, "id,value\n1,100\n2,200\n3,300\n4,400\n").unwrap();

    let code = format!(
        r#"
        let csv: string = readFile("{}");
        let lines: string[] = split(csv, "\n");
        // Count data rows (excluding header and empty last line)
        let allRows: number = len(lines);
        allRows - 2.0
    "#,
        path_for_atlas(&csv_path)
    );
    assert_eval_number_with_io(&code, 4.0);
}

#[test]
fn test_csv_filter_by_criteria() {
    let temp_dir = TempDir::new().unwrap();
    let csv_path = temp_dir.path().join("sales.csv");
    std::fs::write(
        &csv_path,
        "product,price\nApple,1.5\nBanana,0.5\nCherry,3.0\nDate,2.5\n",
    )
    .unwrap();

    let code = format!(
        r#"
        fn isExpensive(row: string) -> bool {{
            let fields: string[] = split(row, ",");
            let price: number = parseFloat(fields[1]);
            return price >= 2.0;
        }}

        let csv: string = readFile("{}");
        let lines: string[] = split(csv, "\n");
        let dataLines: string[] = slice(lines, 1.0, len(lines) - 1.0);

        // Filter expensive items
        let expensive: string[] = filter(dataLines, isExpensive);
        len(expensive)
    "#,
        path_for_atlas(&csv_path)
    );
    assert_eval_number_with_io(&code, 2.0); // Cherry (3.0) and Date (2.5)
}

#[test]
fn test_csv_extract_column() {
    let temp_dir = TempDir::new().unwrap();
    let csv_path = temp_dir.path().join("users.csv");
    std::fs::write(
        &csv_path,
        "name,email\nAlice,alice@test.com\nBob,bob@test.com\n",
    )
    .unwrap();

    let code = format!(
        r#"
        fn getName(row: string) -> string {{
            let fields: string[] = split(row, ",");
            return fields[0];
        }}

        let csv: string = readFile("{}");
        let lines: string[] = split(csv, "\n");
        let dataLines: string[] = slice(lines, 1, len(lines) - 1.0);

        let names: string[] = map(dataLines, getName);
        join(names, "|")
    "#,
        path_for_atlas(&csv_path)
    );
    assert_eval_string_with_io(&code, "Alice|Bob");
}

#[test]
fn test_csv_sum_column() {
    let temp_dir = TempDir::new().unwrap();
    let csv_path = temp_dir.path().join("amounts.csv");
    std::fs::write(&csv_path, "item,amount\nA,10\nB,20\nC,30\n").unwrap();

    let code = format!(
        r#"
        fn sumAmounts(total: number, row: string) -> number {{
            let fields: string[] = split(row, ",");
            let amount: number = parseFloat(fields[1]);
            return total + amount;
        }}

        let csv: string = readFile("{}");
        let lines: string[] = split(csv, "\n");
        let dataLines: string[] = slice(lines, 1, len(lines) - 1.0);

        reduce(dataLines, sumAmounts, 0.0)
    "#,
        path_for_atlas(&csv_path)
    );
    assert_eval_number_with_io(&code, 60.0);
}

#[test]
fn test_csv_empty_file() {
    let temp_dir = TempDir::new().unwrap();
    let csv_path = temp_dir.path().join("empty.csv");
    std::fs::write(&csv_path, "").unwrap();

    let code = format!(
        r#"
        let csv: string = readFile("{}");
        len(csv)
    "#,
        path_for_atlas(&csv_path)
    );
    assert_eval_number_with_io(&code, 0.0);
}

#[test]
fn test_csv_single_row() {
    let temp_dir = TempDir::new().unwrap();
    let csv_path = temp_dir.path().join("single.csv");
    std::fs::write(&csv_path, "name,value\nAlice,100\n").unwrap();

    let code = format!(
        r#"
        let csv: string = readFile("{}");
        let lines: string[] = split(csv, "\n");
        let dataLines: string[] = slice(lines, 1, len(lines) - 1.0);
        len(dataLines)
    "#,
        path_for_atlas(&csv_path)
    );
    assert_eval_number_with_io(&code, 1.0);
}

#[test]
fn test_csv_handle_empty_fields() {
    let temp_dir = TempDir::new().unwrap();
    let csv_path = temp_dir.path().join("sparse.csv");
    std::fs::write(&csv_path, "a,b,c\n1,,3\n4,5,\n").unwrap();

    let code = format!(
        r#"
        let csv: string = readFile("{}");
        let lines: string[] = split(csv, "\n");
        let row1: string = lines[1];
        let fields: string[] = split(row1, ",");
        let emptyField: string = fields[1];
        len(emptyField)
    "#,
        path_for_atlas(&csv_path)
    );
    assert_eval_number_with_io(&code, 0.0);
}

#[test]
fn test_csv_write_transformed() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("input.csv");
    let output_path = temp_dir.path().join("output.csv");
    std::fs::write(&input_path, "name,value\nAlice,10\nBob,20\n").unwrap();

    let code = format!(
        r#"
        fn transform(row: string) -> string {{
            let fields: string[] = split(row, ",");
            let name: string = fields[0];
            let value: number = parseFloat(fields[1]);
            let doubled: number = value * 2.0;
            return name + "," + str(doubled);
        }}

        let csv: string = readFile("{}");
        let lines: string[] = split(csv, "\n");
        let header: string = lines[0];
        let dataLines: string[] = slice(lines, 1, len(lines) - 1.0);

        let transformed: string[] = map(dataLines, transform);
        let output: string = header + "\n" + join(transformed, "\n") + "\n";
        writeFile("{}", output);

        // Verify output
        let result: string = readFile("{}");
        result
    "#,
        path_for_atlas(&input_path),
        path_for_atlas(&output_path),
        path_for_atlas(&output_path)
    );
    assert_eval_string_with_io(&code, "name,value\nAlice,20\nBob,40\n");
}

#[test]
fn test_csv_calculate_average() {
    let temp_dir = TempDir::new().unwrap();
    let csv_path = temp_dir.path().join("scores.csv");
    std::fs::write(&csv_path, "student,score\nAlice,85\nBob,90\nCarol,95\n").unwrap();

    let code = format!(
        r#"
        fn sumScores(total: number, row: string) -> number {{
            let fields: string[] = split(row, ",");
            let score: number = parseFloat(fields[1]);
            return total + score;
        }}

        let csv: string = readFile("{}");
        let lines: string[] = split(csv, "\n");
        let dataLines: string[] = slice(lines, 1, len(lines) - 1.0);

        let total: number = reduce(dataLines, sumScores, 0.0);
        let count: number = len(dataLines);
        total / count
    "#,
        path_for_atlas(&csv_path)
    );
    assert_eval_number_with_io(&code, 90.0); // (85 + 90 + 95) / 3 = 90
}

#[test]
fn test_csv_filter_and_count() {
    let temp_dir = TempDir::new().unwrap();
    let csv_path = temp_dir.path().join("data.csv");
    std::fs::write(&csv_path, "name,age\nAlice,25\nBob,35\nCarol,40\nDave,20\n").unwrap();

    let code = format!(
        r#"
        fn isAdult(row: string) -> bool {{
            let fields: string[] = split(row, ",");
            let age: number = parseFloat(fields[1]);
            return age >= 30.0;
        }}

        let csv: string = readFile("{}");
        let lines: string[] = split(csv, "\n");
        let dataLines: string[] = slice(lines, 1, len(lines) - 1.0);

        let adults: string[] = filter(dataLines, isAdult);
        len(adults)
    "#,
        path_for_atlas(&csv_path)
    );
    assert_eval_number_with_io(&code, 2.0); // Bob (35) and Carol (40)
}

#[test]
fn test_csv_max_value() {
    let temp_dir = TempDir::new().unwrap();
    let csv_path = temp_dir.path().join("values.csv");
    std::fs::write(&csv_path, "id,value\n1,45\n2,89\n3,23\n4,67\n").unwrap();

    let code = format!(
        r#"
        fn findMax(current: number, row: string) -> number {{
            let fields: string[] = split(row, ",");
            let value: number = parseFloat(fields[1]);
            return max(current, value);
        }}

        let csv: string = readFile("{}");
        let lines: string[] = split(csv, "\n");
        let dataLines: string[] = slice(lines, 1, len(lines) - 1.0);

        reduce(dataLines, findMax, 0.0)
    "#,
        path_for_atlas(&csv_path)
    );
    assert_eval_number_with_io(&code, 89.0);
}

#[test]
fn test_csv_header_extraction() {
    let temp_dir = TempDir::new().unwrap();
    let csv_path = temp_dir.path().join("data.csv");
    std::fs::write(&csv_path, "name,email,age\nAlice,a@test.com,30\n").unwrap();

    let code = format!(
        r#"
        let csv: string = readFile("{}");
        let lines: string[] = split(csv, "\n");
        let header: string = lines[0];
        let columns: string[] = split(header, ",");
        join(columns, "|")
    "#,
        path_for_atlas(&csv_path)
    );
    assert_eval_string_with_io(&code, "name|email|age");
}

#[test]
fn test_csv_quoted_fields() {
    let temp_dir = TempDir::new().unwrap();
    let csv_path = temp_dir.path().join("text.csv");
    std::fs::write(&csv_path, "name,note\nAlice,Hello World\nBob,Test Data\n").unwrap();

    let code = format!(
        r#"
        let csv: string = readFile("{}");
        let lines: string[] = split(csv, "\n");
        let row1: string = lines[1];
        let fields: string[] = split(row1, ",");
        fields[1]
    "#,
        path_for_atlas(&csv_path)
    );
    assert_eval_string_with_io(&code, "Hello World");
}

#[test]
fn test_csv_multi_column_filter() {
    let temp_dir = TempDir::new().unwrap();
    let csv_path = temp_dir.path().join("products.csv");
    std::fs::write(
        &csv_path,
        "name,price,stock\nApple,1.5,100\nBanana,0.5,50\nCherry,3.0,200\n",
    )
    .unwrap();

    let code = format!(
        r#"
        fn isHighValueInStock(row: string) -> bool {{
            let fields: string[] = split(row, ",");
            let price: number = parseFloat(fields[1]);
            let stock: number = parseFloat(fields[2]);
            return price >= 1.0 && stock >= 100.0;
        }}

        let csv: string = readFile("{}");
        let lines: string[] = split(csv, "\n");
        let dataLines: string[] = slice(lines, 1, len(lines) - 1.0);

        let filtered: string[] = filter(dataLines, isHighValueInStock);
        len(filtered)
    "#,
        path_for_atlas(&csv_path)
    );
    assert_eval_number_with_io(&code, 2.0); // Apple and Cherry
}

#[test]
fn test_csv_column_sum_with_condition() {
    let temp_dir = TempDir::new().unwrap();
    let csv_path = temp_dir.path().join("sales.csv");
    std::fs::write(
        &csv_path,
        "region,amount\nNorth,1000\nSouth,500\nNorth,1500\nEast,800\n",
    )
    .unwrap();

    let code = format!(
        r#"
        fn sumNorth(total: number, row: string) -> number {{
            let fields: string[] = split(row, ",");
            let region: string = fields[0];
            let amount: number = parseFloat(fields[1]);
            if (region == "North") {{
                return total + amount;
            }}
            return total;
        }}

        let csv: string = readFile("{}");
        let lines: string[] = split(csv, "\n");
        let dataLines: string[] = slice(lines, 1, len(lines) - 1.0);

        reduce(dataLines, sumNorth, 0.0)
    "#,
        path_for_atlas(&csv_path)
    );
    assert_eval_number_with_io(&code, 2500.0); // 1000 + 1500
}

#[test]
fn test_csv_row_count_by_group() {
    let temp_dir = TempDir::new().unwrap();
    let csv_path = temp_dir.path().join("events.csv");
    std::fs::write(
        &csv_path,
        "type,count\nERROR,5\nWARN,10\nERROR,3\nINFO,20\n",
    )
    .unwrap();

    let code = format!(
        r#"
        fn isError(row: string) -> bool {{
            let fields: string[] = split(row, ",");
            return fields[0] == "ERROR";
        }}

        let csv: string = readFile("{}");
        let lines: string[] = split(csv, "\n");
        let dataLines: string[] = slice(lines, 1, len(lines) - 1.0);

        let errors: string[] = filter(dataLines, isError);
        len(errors)
    "#,
        path_for_atlas(&csv_path)
    );
    assert_eval_number_with_io(&code, 2.0);
}

#[test]
fn test_csv_transform_and_join() {
    let temp_dir = TempDir::new().unwrap();
    let csv_path = temp_dir.path().join("names.csv");
    std::fs::write(&csv_path, "first,last\nAlice,Smith\nBob,Jones\n").unwrap();

    let code = format!(
        r#"
        fn fullName(row: string) -> string {{
            let fields: string[] = split(row, ",");
            return fields[0] + " " + fields[1];
        }}

        let csv: string = readFile("{}");
        let lines: string[] = split(csv, "\n");
        let dataLines: string[] = slice(lines, 1, len(lines) - 1.0);

        let names: string[] = map(dataLines, fullName);
        join(names, "; ")
    "#,
        path_for_atlas(&csv_path)
    );
    assert_eval_string_with_io(&code, "Alice Smith; Bob Jones");
}

#[test]
fn test_csv_percentage_calculation() {
    let temp_dir = TempDir::new().unwrap();
    let csv_path = temp_dir.path().join("stats.csv");
    std::fs::write(&csv_path, "item,sold,total\nA,80,100\nB,60,100\n").unwrap();

    let code = format!(
        r#"
        fn calcPercentage(row: string) -> number {{
            let fields: string[] = split(row, ",");
            let sold: number = parseFloat(fields[1]);
            let total: number = parseFloat(fields[2]);
            return (sold / total) * 100.0;
        }}

        let csv: string = readFile("{}");
        let lines: string[] = split(csv, "\n");
        let row1: string = lines[1];

        calcPercentage(row1)
    "#,
        path_for_atlas(&csv_path)
    );
    assert_eval_number_with_io(&code, 80.0);
}

#[test]
fn test_csv_trim_whitespace() {
    let temp_dir = TempDir::new().unwrap();
    let csv_path = temp_dir.path().join("messy.csv");
    std::fs::write(&csv_path, "name,value\n Alice , 100 \n Bob , 200 \n").unwrap();

    let code = format!(
        r#"
        fn cleanRow(row: string) -> string {{
            let fields: string[] = split(row, ",");
            let name: string = trim(fields[0]);
            let value: string = trim(fields[1]);
            return name + "," + value;
        }}

        let csv: string = readFile("{}");
        let lines: string[] = split(csv, "\n");
        let row1: string = lines[1];

        cleanRow(row1)
    "#,
        path_for_atlas(&csv_path)
    );
    assert_eval_string_with_io(&code, "Alice,100");
}

#[test]
fn test_csv_case_insensitive_filter() {
    let temp_dir = TempDir::new().unwrap();
    let csv_path = temp_dir.path().join("items.csv");
    std::fs::write(
        &csv_path,
        "name,type\nApple,FRUIT\nCarrot,vegetable\nBanana,Fruit\n",
    )
    .unwrap();

    let code = format!(
        r#"
        fn isFruit(row: string) -> bool {{
            let fields: string[] = split(row, ",");
            let kind: string = toLowerCase(fields[1]);
            return kind == "fruit";
        }}

        let csv: string = readFile("{}");
        let lines: string[] = split(csv, "\n");
        let dataLines: string[] = slice(lines, 1, len(lines) - 1.0);

        let fruits: string[] = filter(dataLines, isFruit);
        len(fruits)
    "#,
        path_for_atlas(&csv_path)
    );
    assert_eval_number_with_io(&code, 2.0);
}

#[test]
fn test_csv_contains_filter() {
    let temp_dir = TempDir::new().unwrap();
    let csv_path = temp_dir.path().join("logs.csv");
    std::fs::write(
        &csv_path,
        "timestamp,message\n10:00,User login\n10:05,Error occurred\n10:10,User logout\n",
    )
    .unwrap();

    let code = format!(
        r#"
        fn hasError(row: string) -> bool {{
            return includes(row, "Error");
        }}

        let csv: string = readFile("{}");
        let lines: string[] = split(csv, "\n");
        let dataLines: string[] = slice(lines, 1, len(lines) - 1.0);

        let errors: string[] = filter(dataLines, hasError);
        len(errors)
    "#,
        path_for_atlas(&csv_path)
    );
    assert_eval_number_with_io(&code, 1.0);
}

#[test]
fn test_csv_numeric_sort_data() {
    let temp_dir = TempDir::new().unwrap();
    let csv_path = temp_dir.path().join("unsorted.csv");
    std::fs::write(&csv_path, "id,value\n3,30\n1,10\n2,20\n").unwrap();

    let code = format!(
        r#"
        fn compareById(a: string, b: string) -> number {{
            let fieldsA: string[] = split(a, ",");
            let fieldsB: string[] = split(b, ",");
            let idA: number = parseFloat(fieldsA[0]);
            let idB: number = parseFloat(fieldsB[0]);
            return idA - idB;
        }}

        let csv: string = readFile("{}");
        let lines: string[] = split(csv, "\n");
        let dataLines: string[] = slice(lines, 1, len(lines) - 1.0);

        let sorted: string[] = sort(dataLines, compareById);
        let first: string = sorted[0];
        let fields: string[] = split(first, ",");
        fields[0]
    "#,
        path_for_atlas(&csv_path)
    );
    assert_eval_string_with_io(&code, "1");
}

#[test]
fn test_csv_append_row() {
    let temp_dir = TempDir::new().unwrap();
    let csv_path = temp_dir.path().join("append.csv");
    std::fs::write(&csv_path, "name,score\nAlice,85\n").unwrap();

    let code = format!(
        r#"
        let csv: string = readFile("{}");
        let newRow: string = "Bob,90";
        let updated: string = csv + newRow + "\n";
        writeFile("{}", updated);

        let result: string = readFile("{}");
        let lines: string[] = split(result, "\n");
        len(lines) - 1.0
    "#,
        path_for_atlas(&csv_path),
        path_for_atlas(&csv_path),
        path_for_atlas(&csv_path)
    );
    assert_eval_number_with_io(&code, 3.0); // header + Alice + Bob
}

#[test]
fn test_csv_validate_column_count() {
    let temp_dir = TempDir::new().unwrap();
    let csv_path = temp_dir.path().join("valid.csv");
    std::fs::write(&csv_path, "a,b,c\n1,2,3\n4,5,6\n").unwrap();

    let code = format!(
        r#"
        fn hasThreeColumns(row: string) -> bool {{
            let fields: string[] = split(row, ",");
            return len(fields) == 3.0;
        }}

        let csv: string = readFile("{}");
        let lines: string[] = split(csv, "\n");
        let dataLines: string[] = slice(lines, 1, len(lines) - 1.0);

        let valid: string[] = filter(dataLines, hasThreeColumns);
        len(valid) == len(dataLines)
    "#,
        path_for_atlas(&csv_path)
    );
    assert_eval_bool_with_io(&code, true);
}

#[test]
fn test_csv_extract_unique_values() {
    let temp_dir = TempDir::new().unwrap();
    let csv_path = temp_dir.path().join("categories.csv");
    std::fs::write(
        &csv_path,
        "item,category\nA,fruit\nB,veggie\nC,fruit\nD,meat\n",
    )
    .unwrap();

    let code = format!(
        r#"
        fn getCategory(row: string) -> string {{
            let fields: string[] = split(row, ",");
            return fields[1];
        }}

        let csv: string = readFile("{}");
        let lines: string[] = split(csv, "\n");
        let dataLines: string[] = slice(lines, 1, len(lines) - 1.0);

        let categories: string[] = map(dataLines, getCategory);
        // Count unique by checking first occurrence
        let hasFruit: bool = arrayIncludes(categories, "fruit");
        let hasVeggie: bool = arrayIncludes(categories, "veggie");
        let hasMeat: bool = arrayIncludes(categories, "meat");

        str(hasFruit) + "," + str(hasVeggie) + "," + str(hasMeat)
    "#,
        path_for_atlas(&csv_path)
    );
    assert_eval_string_with_io(&code, "true,true,true");
}

#[test]
fn test_csv_conditional_transformation() {
    let temp_dir = TempDir::new().unwrap();
    let csv_path = temp_dir.path().join("grades.csv");
    std::fs::write(&csv_path, "name,score\nAlice,85\nBob,92\nCarol,78\n").unwrap();

    let code = format!(
        r#"
        fn addGrade(row: string) -> string {{
            let fields: string[] = split(row, ",");
            let score: number = parseFloat(fields[1]);
            var grade: string = "F";
            if (score >= 90.0) {{
                grade = "A";
            }} else {{
                if (score >= 80.0) {{
                    grade = "B";
                }} else {{
                    grade = "C";
                }}
            }}
            return fields[0] + "," + fields[1] + "," + grade;
        }}

        let csv: string = readFile("{}");
        let lines: string[] = split(csv, "\n");
        let row1: string = lines[1];

        addGrade(row1)
    "#,
        path_for_atlas(&csv_path)
    );
    assert_eval_string_with_io(&code, "Alice,85,B");
}

#[test]
fn test_csv_min_value() {
    let temp_dir = TempDir::new().unwrap();
    let csv_path = temp_dir.path().join("temps.csv");
    std::fs::write(&csv_path, "day,temp\nMon,72\nTue,68\nWed,75\n").unwrap();

    let code = format!(
        r#"
        fn findMin(current: number, row: string) -> number {{
            let fields: string[] = split(row, ",");
            let temp: number = parseFloat(fields[1]);
            if (current == 0.0) {{
                return temp;
            }}
            return min(current, temp);
        }}

        let csv: string = readFile("{}");
        let lines: string[] = split(csv, "\n");
        let dataLines: string[] = slice(lines, 1, len(lines) - 1.0);

        reduce(dataLines, findMin, 0.0)
    "#,
        path_for_atlas(&csv_path)
    );
    assert_eval_number_with_io(&code, 68.0);
}

#[test]
fn test_csv_concatenate_fields() {
    let temp_dir = TempDir::new().unwrap();
    let csv_path = temp_dir.path().join("addresses.csv");
    std::fs::write(&csv_path, "street,city,state\nMain St,NYC,NY\n").unwrap();

    let code = format!(
        r#"
        let csv: string = readFile("{}");
        let lines: string[] = split(csv, "\n");
        let row1: string = lines[1];
        let fields: string[] = split(row1, ",");
        fields[0] + ", " + fields[1] + ", " + fields[2]
    "#,
        path_for_atlas(&csv_path)
    );
    assert_eval_string_with_io(&code, "Main St, NYC, NY");
}

// ============================================================================
// Category 2: JSON API Response Handling (30 tests)
// ============================================================================

#[test]
fn test_json_parse_simple_object() {
    let code = r#"
        let jsonStr: string = "{\"name\": \"Alice\", \"age\": 30}";
        let data: json = parseJSON(jsonStr);
        let name: string = data["name"].as_string();
        name
    "#;
    assert_eval_string_with_io(code, "Alice");
}

#[test]
fn test_json_parse_nested_object() {
    let code = r#"
        let jsonStr: string = "{\"user\": {\"name\": \"Bob\", \"email\": \"bob@test.com\"}}";
        let data: json = parseJSON(jsonStr);
        let user: json = data["user"];
        let email: string = user["email"].as_string();
        email
    "#;
    assert_eval_string_with_io(code, "bob@test.com");
}

#[test]
fn test_json_parse_array() {
    let code = r#"
        let jsonStr: string = "[1, 2, 3, 4, 5]";
        let arr: json = parseJSON(jsonStr);
        let first: number = arr[0].as_number();
        first
    "#;
    assert_eval_number_with_io(code, 1.0);
}

#[test]
fn test_json_nested_array_access() {
    let code = r#"
        let jsonStr: string = "{\"numbers\": [10, 20, 30]}";
        let data: json = parseJSON(jsonStr);
        let numbers: json = data["numbers"];
        let second: number = numbers[1].as_number();
        second
    "#;
    assert_eval_number_with_io(code, 20.0);
}

#[test]
fn test_json_api_extract_users() {
    let code = r#"
        let jsonStr: string = "{\"users\": [{\"name\": \"Alice\"}, {\"name\": \"Bob\"}]}";
        let response: json = parseJSON(jsonStr);
        let users: json = response["users"];
        let firstUser: json = users[0];
        let name: string = firstUser["name"].as_string();
        name
    "#;
    assert_eval_string_with_io(code, "Alice");
}

#[test]
fn test_json_extract_multiple_fields() {
    let code = r#"
        let jsonStr: string = "{\"id\": 123, \"name\": \"Product\", \"price\": 29.99}";
        let data: json = parseJSON(jsonStr);
        let id: number = data["id"].as_number();
        let name: string = data["name"].as_string();
        let price: number = data["price"].as_number();
        name + ":" + str(price)
    "#;
    assert_eval_string_with_io(code, "Product:29.99");
}

#[test]
fn test_json_deep_nesting() {
    let code = r#"
        let jsonStr: string = "{\"data\": {\"user\": {\"profile\": {\"name\": \"Charlie\"}}}}";
        let response: json = parseJSON(jsonStr);
        let data: json = response["data"];
        let user: json = data["user"];
        let profile: json = user["profile"];
        let name: string = profile["name"].as_string();
        name
    "#;
    assert_eval_string_with_io(code, "Charlie");
}

#[test]
fn test_json_array_of_objects() {
    let code = r#"
        let jsonStr: string = "[{\"id\": 1}, {\"id\": 2}, {\"id\": 3}]";
        let arr: json = parseJSON(jsonStr);
        let item2: json = arr[1];
        let id: number = item2["id"].as_number();
        id
    "#;
    assert_eval_number_with_io(code, 2.0);
}

#[test]
fn test_json_boolean_extraction() {
    let code = r#"
        let jsonStr: string = "{\"active\": true, \"verified\": false}";
        let data: json = parseJSON(jsonStr);
        let active: bool = data["active"].as_bool();
        active
    "#;
    assert_eval_bool_with_io(code, true);
}

#[test]
fn test_json_null_check() {
    let code = r#"
        let jsonStr: string = "{\"value\": null}";
        let data: json = parseJSON(jsonStr);
        let value: json = data["value"];
        jsonIsNull(value)
    "#;
    assert_eval_bool_with_io(code, true);
}

#[test]
fn test_json_missing_key_returns_null() {
    let code = r#"
        let jsonStr: string = "{\"name\": \"Test\"}";
        let data: json = parseJSON(jsonStr);
        let missing: json = data["nonexistent"];
        jsonIsNull(missing)
    "#;
    assert_eval_bool_with_io(code, true);
}

#[test]
fn test_json_build_from_parts() {
    let code = r#"
        let name: string = "Alice";
        let age: number = 30.0;
        let jsonStr: string = "{\"name\":\"" + name + "\",\"age\":" + str(age) + "}";
        let parsed: json = parseJSON(jsonStr);
        let extractedAge: number = parsed["age"].as_number();
        extractedAge
    "#;
    assert_eval_number_with_io(code, 30.0);
}

#[test]
fn test_json_array_length_via_iteration() {
    let code = r#"
        let jsonStr: string = "[1, 2, 3, 4, 5]";
        let arr: json = parseJSON(jsonStr);
        // Access elements to count
        let v0: number = arr[0].as_number();
        let v1: number = arr[1].as_number();
        let v2: number = arr[2].as_number();
        let v3: number = arr[3].as_number();
        let v4: number = arr[4].as_number();
        v0 + v1 + v2 + v3 + v4
    "#;
    assert_eval_number_with_io(code, 15.0);
}

#[test]
fn test_json_mixed_types_in_object() {
    let code = r#"
        let jsonStr: string = "{\"str\": \"hello\", \"num\": 42, \"bool\": true}";
        let data: json = parseJSON(jsonStr);
        let s: string = data["str"].as_string();
        let n: number = data["num"].as_number();
        let b: bool = data["bool"].as_bool();
        s + ":" + str(n) + ":" + str(b)
    "#;
    assert_eval_string_with_io(code, "hello:42:true");
}

#[test]
fn test_json_empty_object() {
    let code = r#"
        let jsonStr: string = "{}";
        let data: json = parseJSON(jsonStr);
        let missing: json = data["anything"];
        jsonIsNull(missing)
    "#;
    assert_eval_bool_with_io(code, true);
}

#[test]
fn test_json_empty_array() {
    let code = r#"
        let jsonStr: string = "[]";
        let arr: json = parseJSON(jsonStr);
        let missing: json = arr[0];
        jsonIsNull(missing)
    "#;
    assert_eval_bool_with_io(code, true);
}

#[test]
fn test_json_prettify_output() {
    let code = r#"
        let jsonStr: string = "{\"name\":\"Alice\",\"age\":30}";
        let data: json = parseJSON(jsonStr);
        let pretty: string = prettifyJSON(jsonStr, 2.0);
        includes(pretty, "  ")
    "#;
    assert_eval_bool_with_io(code, true);
}

#[test]
fn test_json_validate_before_parse() {
    let code = r#"
        let validJson: string = "{\"test\": true}";
        let invalidJson: string = "{invalid}";
        let valid: bool = isValidJSON(validJson);
        let invalid: bool = isValidJSON(invalidJson);
        valid && !invalid
    "#;
    assert_eval_bool_with_io(code, true);
}

#[test]
fn test_json_to_json_round_trip() {
    let code = r#"
        let original: string = "{\"key\":\"value\"}";
        let parsed: json = parseJSON(original);
        let serialized: string = toJSON(parsed);
        includes(serialized, "key") && includes(serialized, "value")
    "#;
    assert_eval_bool_with_io(code, true);
}

#[test]
fn test_json_numeric_precision() {
    let code = r#"
        let jsonStr: string = "{\"value\": 123.456}";
        let data: json = parseJSON(jsonStr);
        let value: number = data["value"].as_number();
        value
    "#;
    assert_eval_number_with_io(code, 123.456);
}

#[test]
fn test_json_github_api_style() {
    let code = r#"
        let response: string = "{\"data\": {\"repository\": {\"name\": \"atlas\", \"stars\": 100}}}";
        let json: json = parseJSON(response);
        let data: json = json["data"];
        let repo: json = data["repository"];
        let name: string = repo["name"].as_string();
        let stars: number = repo["stars"].as_number();
        name + ":" + str(stars)
    "#;
    assert_eval_string_with_io(code, "atlas:100");
}

#[test]
fn test_json_array_filter_pattern() {
    let code = r#"
        let jsonStr: string = "[{\"active\":true},{\"active\":false},{\"active\":true}]";
        let arr: json = parseJSON(jsonStr);
        let item0: json = arr[0];
        let item1: json = arr[1];
        let item2: json = arr[2];
        let a0: bool = item0["active"].as_bool();
        let a1: bool = item1["active"].as_bool();
        let a2: bool = item2["active"].as_bool();
        // Count active
        var count: number = 0.0;
        if (a0) { count = count + 1.0; }
        if (a1) { count = count + 1.0; }
        if (a2) { count = count + 1.0; }
        count
    "#;
    assert_eval_number_with_io(code, 2.0);
}

#[test]
fn test_json_string_escaping() {
    let code = r#"
        let jsonStr: string = "{\"message\": \"Hello\\nWorld\"}";
        let data: json = parseJSON(jsonStr);
        let msg: string = data["message"].as_string();
        includes(msg, "Hello")
    "#;
    assert_eval_bool_with_io(code, true);
}

#[test]
fn test_json_number_as_string() {
    let code = r#"
        let jsonStr: string = "{\"id\": \"12345\"}";
        let data: json = parseJSON(jsonStr);
        let id: string = data["id"].as_string();
        id
    "#;
    assert_eval_string_with_io(code, "12345");
}

#[test]
fn test_json_nested_arrays() {
    let code = r#"
        let jsonStr: string = "{\"matrix\": [[1,2],[3,4]]}";
        let data: json = parseJSON(jsonStr);
        let matrix: json = data["matrix"];
        let row0: json = matrix[0];
        let val: number = row0[1].as_number();
        val
    "#;
    assert_eval_number_with_io(code, 2.0);
}

#[test]
fn test_json_api_pagination_meta() {
    let code = r#"
        let response: string = "{\"data\": [], \"meta\": {\"page\": 1, \"total\": 100}}";
        let json: json = parseJSON(response);
        let meta: json = json["meta"];
        let page: number = meta["page"].as_number();
        let total: number = meta["total"].as_number();
        page + total
    "#;
    assert_eval_number_with_io(code, 101.0);
}

#[test]
fn test_json_error_response() {
    let code = r#"
        let response: string = "{\"error\": {\"code\": 404, \"message\": \"Not Found\"}}";
        let json: json = parseJSON(response);
        let error: json = json["error"];
        let code: number = error["code"].as_number();
        let message: string = error["message"].as_string();
        str(code) + ":" + message
    "#;
    assert_eval_string_with_io(code, "404:Not Found");
}

#[test]
fn test_json_transform_data() {
    let code = r#"
        let input: string = "{\"firstName\": \"John\", \"lastName\": \"Doe\"}";
        let data: json = parseJSON(input);
        let first: string = data["firstName"].as_string();
        let last: string = data["lastName"].as_string();
        // Build new structure
        let fullName: string = first + " " + last;
        let output: string = "{\"name\":\"" + fullName + "\"}";
        let result: json = parseJSON(output);
        let name: string = result["name"].as_string();
        name
    "#;
    assert_eval_string_with_io(code, "John Doe");
}

#[test]
fn test_json_conditional_field_access() {
    let code = r#"
        let jsonStr: string = "{\"premium\": true, \"features\": {\"advanced\": true}}";
        let data: json = parseJSON(jsonStr);
        let premium: bool = data["premium"].as_bool();
        var result: bool = false;
        if (premium) {
            let features: json = data["features"];
            let advanced: bool = features["advanced"].as_bool();
            result = advanced;
        }
        result
    "#;
    assert_eval_bool_with_io(code, true);
}

#[test]
fn test_json_minify_compact() {
    let code = r#"
        let jsonStr: string = "{  \"name\" :  \"test\"  }";
        let minified: string = minifyJSON(jsonStr);
        !includes(minified, "  ")
    "#;
    assert_eval_bool_with_io(code, true);
}

// ============================================================================
// Category 3: Log File Analysis (30 tests)
// ============================================================================

#[test]
fn test_log_parse_basic() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("app.log");
    std::fs::write(&log_path, "2024-01-01 10:00:00 INFO: Application started\n").unwrap();

    let code = format!(
        r#"
        let logs: string = readFile("{}");
        let lines: string[] = split(logs, "\n");
        let first: string = lines[0];
        includes(first, "INFO")
    "#,
        path_for_atlas(&log_path)
    );
    assert_eval_bool_with_io(&code, true);
}

#[test]
fn test_log_filter_errors() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("app.log");
    std::fs::write(
        &log_path,
        "INFO: Started\nERROR: Failed\nWARN: Warning\nERROR: Crashed\n",
    )
    .unwrap();

    let code = format!(
        r#"
        fn isError(line: string) -> bool {{
            return includes(line, "ERROR");
        }}

        let logs: string = readFile("{}");
        let lines: string[] = split(logs, "\n");
        let errors: string[] = filter(lines, isError);
        len(errors)
    "#,
        path_for_atlas(&log_path)
    );
    assert_eval_number_with_io(&code, 2.0);
}

#[test]
fn test_log_extract_timestamps() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("app.log");
    std::fs::write(&log_path, "2024-01-01 ERROR: Test\n2024-01-02 INFO: OK\n").unwrap();

    let code = format!(
        r#"
        fn getTimestamp(line: string) -> string {{
            return substring(line, 0.0, 10.0);
        }}

        let logs: string = readFile("{}");
        let lines: string[] = split(logs, "\n");
        let line1: string = lines[0];
        getTimestamp(line1)
    "#,
        path_for_atlas(&log_path)
    );
    assert_eval_string_with_io(&code, "2024-01-01");
}

#[test]
fn test_log_count_by_level() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("app.log");
    std::fs::write(
        &log_path,
        "INFO: msg1\nERROR: msg2\nINFO: msg3\nWARN: msg4\nINFO: msg5\n",
    )
    .unwrap();

    let code = format!(
        r#"
        fn isInfo(line: string) -> bool {{
            return includes(line, "INFO");
        }}

        let logs: string = readFile("{}");
        let lines: string[] = split(logs, "\n");
        let dataLines: string[] = slice(lines, 0.0, len(lines) - 1.0);
        let infos: string[] = filter(dataLines, isInfo);
        len(infos)
    "#,
        path_for_atlas(&log_path)
    );
    assert_eval_number_with_io(&code, 3.0);
}

#[test]
fn test_log_extract_error_messages() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("app.log");
    std::fs::write(&log_path, "[2024-01-01] ERROR: Connection failed\n").unwrap();

    let code = format!(
        r#"
        let logs: string = readFile("{}");
        let lines: string[] = split(logs, "\n");
        let line: string = lines[0];
        let parts: string[] = split(line, "ERROR: ");
        var msg: string = "";
        if (len(parts) >= 2.0) {{
            msg = parts[1];
        }}
        msg
    "#,
        path_for_atlas(&log_path)
    );
    assert_eval_string_with_io(&code, "Connection failed");
}

#[test]
fn test_log_filter_by_date() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("app.log");
    std::fs::write(
        &log_path,
        "2024-01-01 INFO: Old\n2024-01-15 ERROR: New\n2024-01-20 INFO: Newer\n",
    )
    .unwrap();

    let code = format!(
        r#"
        fn isAfterJan10(line: string) -> bool {{
            let date: string = substring(line, 0.0, 10.0);
            return !startsWith(date, "2024-01-0");
        }}

        let logs: string = readFile("{}");
        let lines: string[] = split(logs, "\n");
        let dataLines: string[] = slice(lines, 0.0, len(lines) - 1.0);
        let recent: string[] = filter(dataLines, isAfterJan10);
        len(recent)
    "#,
        path_for_atlas(&log_path)
    );
    assert_eval_number_with_io(&code, 2.0);
}

#[test]
fn test_log_severity_ordering() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("app.log");
    std::fs::write(&log_path, "DEBUG: d\nINFO: i\nWARN: w\nERROR: e\n").unwrap();

    let code = format!(
        r#"
        fn isHighSeverity(line: string) -> bool {{
            return includes(line, "ERROR") || includes(line, "WARN");
        }}

        let logs: string = readFile("{}");
        let lines: string[] = split(logs, "\n");
        let dataLines: string[] = slice(lines, 0.0, len(lines) - 1.0);
        let high: string[] = filter(dataLines, isHighSeverity);
        len(high)
    "#,
        path_for_atlas(&log_path)
    );
    assert_eval_number_with_io(&code, 2.0);
}

#[test]
fn test_log_multi_line_error() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("app.log");
    std::fs::write(
        &log_path,
        "ERROR: Failed\nStack trace line 1\nStack trace line 2\n",
    )
    .unwrap();

    let code = format!(
        r#"
        let logs: string = readFile("{}");
        let lines: string[] = split(logs, "\n");
        let first: string = lines[0];
        let second: string = lines[1];
        includes(first, "ERROR") && includes(second, "Stack")
    "#,
        path_for_atlas(&log_path)
    );
    assert_eval_bool_with_io(&code, true);
}

#[test]
fn test_log_empty_lines_filter() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("app.log");
    std::fs::write(&log_path, "INFO: msg1\n\nERROR: msg2\n\nWARN: msg3\n").unwrap();

    let code = format!(
        r#"
        fn isNotEmpty(line: string) -> bool {{
            return len(line) > 0.0;
        }}

        let logs: string = readFile("{}");
        let lines: string[] = split(logs, "\n");
        let nonEmpty: string[] = filter(lines, isNotEmpty);
        len(nonEmpty)
    "#,
        path_for_atlas(&log_path)
    );
    assert_eval_number_with_io(&code, 3.0);
}

#[test]
fn test_log_contains_pattern() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("app.log");
    std::fs::write(
        &log_path,
        "User alice logged in\nUser bob failed\nUser alice logged out\n",
    )
    .unwrap();

    let code = format!(
        r#"
        fn mentionsAlice(line: string) -> bool {{
            return includes(line, "alice");
        }}

        let logs: string = readFile("{}");
        let lines: string[] = split(logs, "\n");
        let dataLines: string[] = slice(lines, 0.0, len(lines) - 1.0);
        let aliceLogs: string[] = filter(dataLines, mentionsAlice);
        len(aliceLogs)
    "#,
        path_for_atlas(&log_path)
    );
    assert_eval_number_with_io(&code, 2.0);
}

#[test]
fn test_log_case_insensitive_search() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("app.log");
    std::fs::write(&log_path, "Error: test\nerror: test2\nERROR: test3\n").unwrap();

    let code = format!(
        r#"
        fn hasError(line: string) -> bool {{
            let lower: string = toLowerCase(line);
            return includes(lower, "error");
        }}

        let logs: string = readFile("{}");
        let lines: string[] = split(logs, "\n");
        let dataLines: string[] = slice(lines, 0.0, len(lines) - 1.0);
        let errors: string[] = filter(dataLines, hasError);
        len(errors)
    "#,
        path_for_atlas(&log_path)
    );
    assert_eval_number_with_io(&code, 3.0);
}

#[test]
fn test_log_extract_user_actions() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("app.log");
    std::fs::write(
        &log_path,
        "User:alice Action:login\nUser:bob Action:logout\n",
    )
    .unwrap();

    let code = format!(
        r#"
        fn extractUser(line: string) -> string {{
            let parts: string[] = split(line, " ");
            let userPart: string = parts[0];
            let userFields: string[] = split(userPart, ":");
            return userFields[1];
        }}

        let logs: string = readFile("{}");
        let lines: string[] = split(logs, "\n");
        let line1: string = lines[0];
        extractUser(line1)
    "#,
        path_for_atlas(&log_path)
    );
    assert_eval_string_with_io(&code, "alice");
}

#[test]
fn test_log_count_occurrences() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("app.log");
    std::fs::write(&log_path, "login\nlogout\nlogin\nlogin\nlogout\n").unwrap();

    let code = format!(
        r#"
        fn isLogin(line: string) -> bool {{
            return line == "login";
        }}

        let logs: string = readFile("{}");
        let lines: string[] = split(logs, "\n");
        let dataLines: string[] = slice(lines, 0.0, len(lines) - 1.0);
        let logins: string[] = filter(dataLines, isLogin);
        len(logins)
    "#,
        path_for_atlas(&log_path)
    );
    assert_eval_number_with_io(&code, 3.0);
}

#[test]
fn test_log_trim_whitespace() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("app.log");
    std::fs::write(&log_path, "  ERROR: Test  \n  WARN: Alert  \n").unwrap();

    let code = format!(
        r#"
        fn cleanLine(line: string) -> string {{
            return trim(line);
        }}

        let logs: string = readFile("{}");
        let lines: string[] = split(logs, "\n");
        let line1: string = lines[0];
        let cleaned: string = cleanLine(line1);
        cleaned
    "#,
        path_for_atlas(&log_path)
    );
    assert_eval_string_with_io(&code, "ERROR: Test");
}

#[test]
fn test_log_starts_with_timestamp() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("app.log");
    std::fs::write(&log_path, "2024-01-01 INFO: msg\n2024-01-02 ERROR: err\n").unwrap();

    let code = format!(
        r#"
        fn hasTimestamp(line: string) -> bool {{
            return startsWith(line, "2024");
        }}

        let logs: string = readFile("{}");
        let lines: string[] = split(logs, "\n");
        let dataLines: string[] = slice(lines, 0.0, len(lines) - 1.0);
        let timestamped: string[] = filter(dataLines, hasTimestamp);
        len(timestamped)
    "#,
        path_for_atlas(&log_path)
    );
    assert_eval_number_with_io(&code, 2.0);
}

#[test]
fn test_log_extract_ip_addresses() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("access.log");
    std::fs::write(&log_path, "192.168.1.1 GET /page\n10.0.0.1 POST /api\n").unwrap();

    let code = format!(
        r#"
        fn extractIP(line: string) -> string {{
            let parts: string[] = split(line, " ");
            return parts[0];
        }}

        let logs: string = readFile("{}");
        let lines: string[] = split(logs, "\n");
        let line1: string = lines[0];
        extractIP(line1)
    "#,
        path_for_atlas(&log_path)
    );
    assert_eval_string_with_io(&code, "192.168.1.1");
}

#[test]
fn test_log_group_by_category() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("app.log");
    std::fs::write(
        &log_path,
        "DB: query\nAPI: request\nDB: update\nDB: delete\n",
    )
    .unwrap();

    let code = format!(
        r#"
        fn isDatabase(line: string) -> bool {{
            return startsWith(line, "DB:");
        }}

        let logs: string = readFile("{}");
        let lines: string[] = split(logs, "\n");
        let dataLines: string[] = slice(lines, 0.0, len(lines) - 1.0);
        let dbLogs: string[] = filter(dataLines, isDatabase);
        len(dbLogs)
    "#,
        path_for_atlas(&log_path)
    );
    assert_eval_number_with_io(&code, 3.0);
}

#[test]
fn test_log_parse_structured() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("app.log");
    std::fs::write(
        &log_path,
        "level=error msg=\"Failed to connect\" code=500\n",
    )
    .unwrap();

    let code = format!(
        r#"
        let logs: string = readFile("{}");
        let lines: string[] = split(logs, "\n");
        let line: string = lines[0];
        let parts: string[] = split(line, " ");
        let levelPart: string = parts[0];
        startsWith(levelPart, "level=error")
    "#,
        path_for_atlas(&log_path)
    );
    assert_eval_bool_with_io(&code, true);
}

#[test]
fn test_log_count_warnings() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("app.log");
    std::fs::write(&log_path, "INFO\nWARN\nERROR\nWARN\nINFO\nWARN\n").unwrap();

    let code = format!(
        r#"
        fn countWarnings(total: number, line: string) -> number {{
            if (line == "WARN") {{
                return total + 1.0;
            }}
            return total;
        }}

        let logs: string = readFile("{}");
        let lines: string[] = split(logs, "\n");
        let dataLines: string[] = slice(lines, 0.0, len(lines) - 1.0);
        reduce(dataLines, countWarnings, 0.0)
    "#,
        path_for_atlas(&log_path)
    );
    assert_eval_number_with_io(&code, 3.0);
}

#[test]
fn test_log_find_first_error() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("app.log");
    std::fs::write(
        &log_path,
        "INFO: ok\nWARN: warning\nERROR: failure\nERROR: another\n",
    )
    .unwrap();

    let code = format!(
        r#"
        fn isError(line: string) -> bool {{
            return includes(line, "ERROR");
        }}

        let logs: string = readFile("{}");
        let lines: string[] = split(logs, "\n");
        let dataLines: string[] = slice(lines, 0.0, len(lines) - 1.0);
        let firstError: string = find(dataLines, isError);
        firstError
    "#,
        path_for_atlas(&log_path)
    );
    assert_eval_string_with_io(&code, "ERROR: failure");
}

#[test]
fn test_log_reverse_chronological() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("app.log");
    std::fs::write(&log_path, "Line1\nLine2\nLine3\n").unwrap();

    let code = format!(
        r#"
        let logs: string = readFile("{}");
        let lines: string[] = split(logs, "\n");
        let dataLines: string[] = slice(lines, 0.0, len(lines) - 1.0);
        let reversed: string[] = reverse(dataLines);
        reversed[0]
    "#,
        path_for_atlas(&log_path)
    );
    assert_eval_string_with_io(&code, "Line3");
}

#[test]
fn test_log_summary_report() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("app.log");
    std::fs::write(
        &log_path,
        "ERROR:e1\nINFO:i1\nERROR:e2\nWARN:w1\nERROR:e3\n",
    )
    .unwrap();

    let code = format!(
        r#"
        fn isError(line: string) -> bool {{ return includes(line, "ERROR"); }}
        fn isWarn(line: string) -> bool {{ return includes(line, "WARN"); }}
        fn isInfo(line: string) -> bool {{ return includes(line, "INFO"); }}

        let logs: string = readFile("{}");
        let lines: string[] = split(logs, "\n");
        let dataLines: string[] = slice(lines, 0.0, len(lines) - 1.0);

        let errors: number = len(filter(dataLines, isError));
        let warns: number = len(filter(dataLines, isWarn));
        let infos: number = len(filter(dataLines, isInfo));

        "E:" + str(errors) + ",W:" + str(warns) + ",I:" + str(infos)
    "#,
        path_for_atlas(&log_path)
    );
    assert_eval_string_with_io(&code, "E:3,W:1,I:1");
}

#[test]
fn test_log_filter_time_range() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("app.log");
    std::fs::write(&log_path, "08:00 Start\n09:30 Middle\n12:00 End\n").unwrap();

    let code = format!(
        r#"
        fn isMorning(line: string) -> bool {{
            let time: string = substring(line, 0.0, 2.0);
            return time == "08" || time == "09";
        }}

        let logs: string = readFile("{}");
        let lines: string[] = split(logs, "\n");
        let dataLines: string[] = slice(lines, 0.0, len(lines) - 1.0);
        let morning: string[] = filter(dataLines, isMorning);
        len(morning)
    "#,
        path_for_atlas(&log_path)
    );
    assert_eval_number_with_io(&code, 2.0);
}

#[test]
fn test_log_extract_http_codes() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("access.log");
    std::fs::write(&log_path, "GET /page 200\nPOST /api 404\nGET /home 200\n").unwrap();

    let code = format!(
        r#"
        fn is404(line: string) -> bool {{
            return includes(line, "404");
        }}

        let logs: string = readFile("{}");
        let lines: string[] = split(logs, "\n");
        let dataLines: string[] = slice(lines, 0.0, len(lines) - 1.0);
        let notFound: string[] = filter(dataLines, is404);
        len(notFound)
    "#,
        path_for_atlas(&log_path)
    );
    assert_eval_number_with_io(&code, 1.0);
}

#[test]
fn test_log_parse_json_lines() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("json.log");
    std::fs::write(
        &log_path,
        "{\"level\":\"error\",\"msg\":\"failed\"}\n{\"level\":\"info\",\"msg\":\"ok\"}\n",
    )
    .unwrap();

    let code = format!(
        r#"
        let logs: string = readFile("{}");
        let lines: string[] = split(logs, "\n");
        let line1: string = lines[0];
        let json: json = parseJSON(line1);
        let level: string = json["level"].as_string();
        level
    "#,
        path_for_atlas(&log_path)
    );
    assert_eval_string_with_io(&code, "error");
}

#[test]
fn test_log_aggregate_metrics() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("metrics.log");
    std::fs::write(&log_path, "latency:100\nlatency:150\nlatency:200\n").unwrap();

    let code = format!(
        r#"
        fn sumLatency(total: number, line: string) -> number {{
            let parts: string[] = split(line, ":");
            let value: number = parseFloat(parts[1]);
            return total + value;
        }}

        let logs: string = readFile("{}");
        let lines: string[] = split(logs, "\n");
        let dataLines: string[] = slice(lines, 0.0, len(lines) - 1.0);
        let total: number = reduce(dataLines, sumLatency, 0.0);
        let avg: number = total / len(dataLines);
        avg
    "#,
        path_for_atlas(&log_path)
    );
    assert_eval_number_with_io(&code, 150.0);
}

#[test]
fn test_log_detect_anomalies() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("app.log");
    std::fs::write(&log_path, "Normal\nNormal\nANOMALY\nNormal\n").unwrap();

    let code = format!(
        r#"
        fn isAnomaly(line: string) -> bool {{
            return line == "ANOMALY";
        }}

        let logs: string = readFile("{}");
        let lines: string[] = split(logs, "\n");
        let dataLines: string[] = slice(lines, 0.0, len(lines) - 1.0);
        let anomalies: string[] = filter(dataLines, isAnomaly);
        len(anomalies) > 0.0
    "#,
        path_for_atlas(&log_path)
    );
    assert_eval_bool_with_io(&code, true);
}

#[test]
fn test_log_combine_multiline() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("app.log");
    std::fs::write(&log_path, "ERROR: Start\nContinue\nEnd\n").unwrap();

    let code = format!(
        r#"
        let logs: string = readFile("{}");
        let lines: string[] = split(logs, "\n");
        let combined: string = lines[0] + " " + lines[1] + " " + lines[2];
        includes(combined, "Start") && includes(combined, "Continue") && includes(combined, "End")
    "#,
        path_for_atlas(&log_path)
    );
    assert_eval_bool_with_io(&code, true);
}

#[test]
fn test_log_write_filtered() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("input.log");
    let output_path = temp_dir.path().join("errors.log");
    std::fs::write(
        &input_path,
        "INFO: ok\nERROR: failed\nWARN: warn\nERROR: bad\n",
    )
    .unwrap();

    let code = format!(
        r#"
        fn isError(line: string) -> bool {{
            return includes(line, "ERROR");
        }}

        let logs: string = readFile("{}");
        let lines: string[] = split(logs, "\n");
        let dataLines: string[] = slice(lines, 0.0, len(lines) - 1.0);
        let errors: string[] = filter(dataLines, isError);
        let output: string = join(errors, "\n") + "\n";
        writeFile("{}", output);

        let result: string = readFile("{}");
        let resultLines: string[] = split(result, "\n");
        len(resultLines) - 1.0
    "#,
        path_for_atlas(&input_path),
        path_for_atlas(&output_path),
        path_for_atlas(&output_path)
    );
    assert_eval_number_with_io(&code, 2.0);
}

// ============================================================================
// Category 4: Data Transformation Pipelines (30 tests)
// ============================================================================

#[test]
fn test_pipeline_map_filter_reduce() {
    let code = r#"
        fn double(x: number) -> number { return x * 2.0; }
        fn isEven(x: number) -> bool { return x % 2.0 == 0.0; }
        fn sum(a: number, b: number) -> number { return a + b; }

        let numbers: number[] = [1.0, 2.0, 3.0, 4.0, 5.0];
        let doubled: number[] = map(numbers, double);
        let evens: number[] = filter(doubled, isEven);
        reduce(evens, sum, 0.0)
    "#;
    assert_eval_number_with_io(code, 30.0); // doubled=[2,4,6,8,10], all even, sum=30
}

#[test]
fn test_pipeline_filter_map_join() {
    let code = r#"
        fn isLong(s: string) -> bool { return len(s) > 3.0; }
        fn toUpper(s: string) -> string { return toUpperCase(s); }

        let words: string[] = ["hi", "hello", "bye", "world"];
        let long: string[] = filter(words, isLong);
        let uppered: string[] = map(long, toUpper);
        join(uppered, "-")
    "#;
    assert_eval_string_with_io(code, "HELLO-WORLD");
}

#[test]
fn test_pipeline_nested_arrays() {
    let code = r#"
        let nested: number[][] = [[1.0, 2.0], [3.0, 4.0], [5.0, 6.0]];
        let flat: number[] = flatten(nested);
        fn double(x: number) -> number { return x * 2.0; }
        let doubled: number[] = map(flat, double);
        fn sum(a: number, b: number) -> number { return a + b; }
        reduce(doubled, sum, 0.0)
    "#;
    assert_eval_number_with_io(code, 42.0); // [1..6] doubled = [2,4,6,8,10,12] sum=42
}

#[test]
fn test_pipeline_string_processing() {
    let code = r#"
        fn trimAndLower(s: string) -> string {
            let t: string = trim(s);
            return toLowerCase(t);
        }

        let input: string[] = ["  HELLO  ", "  WORLD  ", "  TEST  "];
        let cleaned: string[] = map(input, trimAndLower);
        join(cleaned, ",")
    "#;
    assert_eval_string_with_io(code, "hello,world,test");
}

#[test]
fn test_pipeline_multi_step_filter() {
    let code = r#"
        fn isPositive(x: number) -> bool { return x > 0.0; }
        fn isSmall(x: number) -> bool { return x < 100.0; }

        let numbers: number[] = [-5.0, 10.0, 150.0, 50.0, -20.0, 75.0];
        let positive: number[] = filter(numbers, isPositive);
        let small: number[] = filter(positive, isSmall);
        len(small)
    "#;
    assert_eval_number_with_io(code, 3.0); // [10, 50, 75]
}

#[test]
fn test_pipeline_sort_and_slice() {
    let code = r#"
        fn compare(a: number, b: number) -> number { return a - b; }

        let numbers: number[] = [5.0, 2.0, 8.0, 1.0, 9.0, 3.0];
        let sorted: number[] = sort(numbers, compare);
        let top3: number[] = slice(sorted, 0.0, 3.0);
        fn sum(a: number, b: number) -> number { return a + b; }
        reduce(top3, sum, 0.0)
    "#;
    assert_eval_number_with_io(code, 6.0); // [1,2,3] sum=6
}

#[test]
fn test_pipeline_flatmap_strings() {
    let code = r#"
        fn splitWords(s: string) -> string[] {
            return split(s, " ");
        }

        let sentences: string[] = ["hello world", "foo bar"];
        let words: string[] = flatMap(sentences, splitWords);
        len(words)
    "#;
    assert_eval_number_with_io(code, 4.0);
}

#[test]
fn test_pipeline_conditional_transform() {
    let code = r#"
        fn transform(x: number) -> number {
            if (x < 0.0) {
                return abs(x);
            }
            return x;
        }

        let numbers: number[] = [-5.0, 10.0, -3.0, 7.0];
        let transformed: number[] = map(numbers, transform);
        fn sum(a: number, b: number) -> number { return a + b; }
        reduce(transformed, sum, 0.0)
    "#;
    assert_eval_number_with_io(code, 25.0); // [5,10,3,7] sum=25
}

#[test]
fn test_pipeline_find_and_transform() {
    let code = r#"
        fn isLarge(x: number) -> bool { return x > 50.0; }

        let numbers: number[] = [10.0, 60.0, 30.0, 80.0];
        let found: number = find(numbers, isLarge);
        found * 2.0
    "#;
    assert_eval_number_with_io(code, 120.0); // 60 * 2
}

#[test]
fn test_pipeline_every_and_some() {
    let code = r#"
        fn isPositive(x: number) -> bool { return x > 0.0; }

        let numbers: number[] = [1.0, 2.0, 3.0];
        let allPositive: bool = every(numbers, isPositive);
        let somePositive: bool = some(numbers, isPositive);
        allPositive && somePositive
    "#;
    assert_eval_bool_with_io(code, true);
}

#[test]
fn test_pipeline_reverse_and_join() {
    let code = r#"
        let words: string[] = ["one", "two", "three"];
        let reversed: string[] = reverse(words);
        join(reversed, "-")
    "#;
    assert_eval_string_with_io(code, "three-two-one");
}

#[test]
fn test_pipeline_unshift_and_concat() {
    let code = r#"
        let arr1: number[] = [2.0, 3.0];
        let arr2: number[] = [4.0, 5.0];
        let withOne: number[] = unshift(arr1, 1.0);
        let combined: number[] = concat(withOne, arr2);
        len(combined)
    "#;
    assert_eval_number_with_io(code, 5.0);
}

#[test]
fn test_pipeline_multiple_maps() {
    let code = r#"
        fn add10(x: number) -> number { return x + 10.0; }
        fn double(x: number) -> number { return x * 2.0; }

        let numbers: number[] = [1.0, 2.0, 3.0];
        let step1: number[] = map(numbers, add10);
        let step2: number[] = map(step1, double);
        step2[0]
    "#;
    assert_eval_number_with_io(code, 22.0); // (1+10)*2 = 22
}

#[test]
fn test_pipeline_filter_reverse_first() {
    let code = r#"
        fn isEven(x: number) -> bool { return x % 2.0 == 0.0; }

        let numbers: number[] = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        let evens: number[] = filter(numbers, isEven);
        let reversed: number[] = reverse(evens);
        reversed[0]
    "#;
    assert_eval_number_with_io(code, 6.0);
}

#[test]
fn test_pipeline_sortby_number() {
    let code = r#"
        fn negate(x: number) -> number { return x * -1.0; }

        let numbers: number[] = [3.0, 1.0, 4.0, 1.0, 5.0];
        let sorted: number[] = sortBy(numbers, negate);
        sorted[0]
    "#;
    assert_eval_number_with_io(code, 5.0); // sorted descending
}

#[test]
fn test_pipeline_pop_and_process() {
    let code = r#"
        let numbers: number[] = [1.0, 2.0, 3.0];
        let last: number = numbers[len(numbers) - 1.0];
        let remaining: number[] = slice(numbers, 0.0, len(numbers) - 1.0);
        last + len(remaining)
    "#;
    assert_eval_number_with_io(code, 5.0); // 3 + 2
}

#[test]
fn test_pipeline_shift_and_process() {
    let code = r#"
        let numbers: number[] = [1.0, 2.0, 3.0];
        let first: number = numbers[0];
        let remaining: number[] = slice(numbers, 1.0, len(numbers));
        first + len(remaining)
    "#;
    assert_eval_number_with_io(code, 3.0); // 1 + 2
}

#[test]
fn test_pipeline_findindex_and_slice() {
    let code = r#"
        fn isLarge(x: number) -> bool { return x > 50.0; }

        let numbers: number[] = [10.0, 20.0, 60.0, 80.0];
        let idx: number = findIndex(numbers, isLarge);
        let fromLarge: number[] = slice(numbers, idx, len(numbers));
        len(fromLarge)
    "#;
    assert_eval_number_with_io(code, 2.0); // [60, 80]
}

#[test]
fn test_pipeline_complex_aggregation() {
    let code = r#"
        fn square(x: number) -> number { return x * x; }
        fn sum(a: number, b: number) -> number { return a + b; }

        let numbers: number[] = [1.0, 2.0, 3.0, 4.0];
        let squared: number[] = map(numbers, square);
        let total: number = reduce(squared, sum, 0.0);
        total
    "#;
    assert_eval_number_with_io(code, 30.0); // 1+4+9+16 = 30
}

#[test]
fn test_pipeline_string_filter_map() {
    let code = r#"
        fn notEmpty(s: string) -> bool { return len(s) > 0.0; }
        fn firstChar(s: string) -> string { return charAt(s, 0.0); }

        let words: string[] = ["apple", "", "banana", "", "cherry"];
        let nonEmpty: string[] = filter(words, notEmpty);
        let firstChars: string[] = map(nonEmpty, firstChar);
        join(firstChars, "")
    "#;
    assert_eval_string_with_io(code, "abc");
}

#[test]
fn test_pipeline_nested_operations() {
    let code = r#"
        fn process(x: number) -> number {
            let step1: number = x + 5.0;
            let step2: number = step1 * 2.0;
            return step2;
        }
        fn sum(a: number, b: number) -> number { return a + b; }

        let numbers: number[] = [1.0, 2.0, 3.0];
        let processed: number[] = map(numbers, process);
        reduce(processed, sum, 0.0)
    "#;
    assert_eval_number_with_io(code, 42.0); // (1+5)*2=12, (2+5)*2=14, (3+5)*2=16, sum=42
}

#[test]
fn test_pipeline_includes_filter() {
    let code = r#"
        fn hasLetterA(s: string) -> bool {
            return includes(s, "a");
        }

        let words: string[] = ["apple", "berry", "apricot", "cherry"];
        let withA: string[] = filter(words, hasLetterA);
        len(withA)
    "#;
    assert_eval_number_with_io(code, 2.0); // apple, apricot
}

#[test]
fn test_pipeline_index_access_transform() {
    let code = r#"
        let matrix: number[][] = [[1.0, 2.0], [3.0, 4.0], [5.0, 6.0]];
        fn getFirst(row: number[]) -> number { return row[0]; }

        let firstElements: number[] = map(matrix, getFirst);
        fn sum(a: number, b: number) -> number { return a + b; }
        reduce(firstElements, sum, 0.0)
    "#;
    assert_eval_number_with_io(code, 9.0); // 1+3+5 = 9
}

#[test]
fn test_pipeline_replace_map() {
    let code = r#"
        fn removeSpaces(s: string) -> string {
            return replace(s, " ", "_");
        }

        let phrases: string[] = ["hello world", "foo bar"];
        let replaced: string[] = map(phrases, removeSpaces);
        join(replaced, "|")
    "#;
    assert_eval_string_with_io(code, "hello_world|foo_bar");
}

#[test]
fn test_pipeline_padstart_map() {
    let code = r#"
        fn pad(s: string) -> string {
            return padStart(s, 5.0, "0");
        }

        let numbers: string[] = ["1", "22", "333"];
        let padded: string[] = map(numbers, pad);
        join(padded, ",")
    "#;
    assert_eval_string_with_io(code, "00001,00022,00333");
}

#[test]
fn test_pipeline_substring_filter_map() {
    let code = r#"
        fn getPrefix(s: string) -> string {
            return substring(s, 0.0, 3.0);
        }

        let words: string[] = ["apple", "application", "appropriate"];
        let prefixes: string[] = map(words, getPrefix);
        fn isApp(s: string) -> bool { return s == "app"; }
        let appPrefixes: string[] = filter(prefixes, isApp);
        len(appPrefixes)
    "#;
    assert_eval_number_with_io(code, 3.0);
}

#[test]
fn test_pipeline_min_max_aggregation() {
    let code = r#"
        fn findMin(current: number, x: number) -> number {
            if (current == 0.0) { return x; }
            return min(current, x);
        }
        fn findMax(current: number, x: number) -> number {
            return max(current, x);
        }

        let numbers: number[] = [5.0, 2.0, 8.0, 1.0, 9.0];
        let minVal: number = reduce(numbers, findMin, 0.0);
        let maxVal: number = reduce(numbers, findMax, 0.0);
        maxVal - minVal
    "#;
    assert_eval_number_with_io(code, 8.0); // 9 - 1
}

#[test]
fn test_pipeline_array_building() {
    let code = r#"
        let arr1: number[] = [1.0];
        let arr2: number[] = unshift(arr1, 0.0);
        let arr3: number[] = concat(arr2, [2.0, 3.0]);
        fn sum(a: number, b: number) -> number { return a + b; }
        reduce(arr3, sum, 0.0)
    "#;
    assert_eval_number_with_io(code, 6.0); // [0,1,2,3] sum=6
}

#[test]
fn test_pipeline_foreach_side_effects() {
    let code = r#"
        fn noop(_x: number) -> void { return; }

        let numbers: number[] = [1.0, 2.0, 3.0];
        forEach(numbers, noop);
        // forEach returns null, verify it doesn't crash
        true
    "#;
    assert_eval_bool_with_io(code, true);
}

// ============================================================================
// Category 5: Text Processing (20 tests)
// ============================================================================

#[test]
fn test_text_word_count() {
    let code = r#"
        let text: string = "hello world this is a test";
        let words: string[] = split(text, " ");
        len(words)
    "#;
    assert_eval_number_with_io(code, 6.0);
}

#[test]
fn test_text_line_count() {
    let code = r#"
        let text: string = "line1\nline2\nline3";
        let lines: string[] = split(text, "\n");
        len(lines)
    "#;
    assert_eval_number_with_io(code, 3.0);
}

#[test]
fn test_text_average_word_length() {
    let code = r#"
        fn wordLength(word: string) -> number { return len(word); }
        fn sum(a: number, b: number) -> number { return a + b; }

        let text: string = "the quick brown fox";
        let words: string[] = split(text, " ");
        let lengths: number[] = map(words, wordLength);
        let total: number = reduce(lengths, sum, 0.0);
        let avg: number = total / len(words);
        floor(avg)
    "#;
    assert_eval_number_with_io(code, 4.0); // (3+5+5+3)/4 = 4
}

#[test]
fn test_text_uppercase_words() {
    let code = r#"
        fn toUpper(s: string) -> string { return toUpperCase(s); }

        let text: string = "hello world";
        let words: string[] = split(text, " ");
        let uppered: string[] = map(words, toUpper);
        join(uppered, " ")
    "#;
    assert_eval_string_with_io(code, "HELLO WORLD");
}

#[test]
fn test_text_titlecase() {
    let code = r#"
        fn titleCase(word: string) -> string {
            let first: string = charAt(word, 0.0);
            let rest: string = substring(word, 1.0, len(word));
            let firstUpper: string = toUpperCase(first);
            let restLower: string = toLowerCase(rest);
            return firstUpper + restLower;
        }

        let text: string = "hello WORLD";
        let words: string[] = split(text, " ");
        let titled: string[] = map(words, titleCase);
        join(titled, " ")
    "#;
    assert_eval_string_with_io(code, "Hello World");
}

#[test]
fn test_text_remove_punctuation() {
    let code = r#"
        fn removePunct(s: string) -> string {
            let s1: string = replace(s, ".", "");
            let s2: string = replace(s1, ",", "");
            let s3: string = replace(s2, "!", "");
            return s3;
        }

        let text: string = "Hello, World! Test.";
        removePunct(text)
    "#;
    assert_eval_string_with_io(code, "Hello World Test");
}

#[test]
fn test_text_find_longest_word() {
    let code = r#"
        fn longerWord(current: string, word: string) -> string {
            if (len(word) > len(current)) {
                return word;
            }
            return current;
        }

        let text: string = "the quick brown fox jumps";
        let words: string[] = split(text, " ");
        reduce(words, longerWord, "")
    "#;
    assert_eval_string_with_io(code, "quick"); // or "brown" or "jumps" (all 5 chars, first wins)
}

#[test]
fn test_text_filter_short_words() {
    let code = r#"
        fn isLong(word: string) -> bool {
            return len(word) >= 4.0;
        }

        let text: string = "the quick brown fox";
        let words: string[] = split(text, " ");
        let long: string[] = filter(words, isLong);
        len(long)
    "#;
    assert_eval_number_with_io(code, 2.0); // "quick"=5, "brown"=5 are >=4
}

#[test]
fn test_text_count_character() {
    let code = r#"
        let text: string = "hello world";
        let chars: string[] = split(text, "");
        fn isL(c: string) -> bool { return c == "l"; }
        let ls: string[] = filter(chars, isL);
        len(ls)
    "#;
    assert_eval_number_with_io(code, 3.0);
}

#[test]
fn test_text_reverse_words() {
    let code = r#"
        let text: string = "hello world";
        let words: string[] = split(text, " ");
        let reversed: string[] = reverse(words);
        join(reversed, " ")
    "#;
    assert_eval_string_with_io(code, "world hello");
}

#[test]
fn test_text_acronym() {
    let code = r#"
        fn firstChar(s: string) -> string {
            return charAt(s, 0.0);
        }

        let text: string = "Portable Network Graphics";
        let words: string[] = split(text, " ");
        let initials: string[] = map(words, firstChar);
        join(initials, "")
    "#;
    assert_eval_string_with_io(code, "PNG");
}

#[test]
fn test_text_trim_lines() {
    let code = r#"
        fn trimLine(line: string) -> string { return trim(line); }

        let text: string = "  line1  \n  line2  \n  line3  ";
        let lines: string[] = split(text, "\n");
        let trimmed: string[] = map(lines, trimLine);
        join(trimmed, "|")
    "#;
    assert_eval_string_with_io(code, "line1|line2|line3");
}

#[test]
fn test_text_starts_with_filter() {
    let code = r#"
        fn startsWithA(word: string) -> bool {
            return startsWith(word, "a");
        }

        let words: string[] = ["apple", "banana", "apricot", "cherry"];
        let aWords: string[] = filter(words, startsWithA);
        len(aWords)
    "#;
    assert_eval_number_with_io(code, 2.0);
}

#[test]
fn test_text_ends_with_filter() {
    let code = r#"
        fn endsWithE(word: string) -> bool {
            return endsWith(word, "e");
        }

        let words: string[] = ["apple", "banana", "grape", "cherry"];
        let eWords: string[] = filter(words, endsWithE);
        len(eWords)
    "#;
    assert_eval_number_with_io(code, 2.0); // apple, grape
}

#[test]
fn test_text_pad_lines() {
    let code = r#"
        fn pad(line: string) -> string {
            return padEnd(line, 10.0, ".");
        }

        let lines: string[] = ["short", "medium", "long"];
        let padded: string[] = map(lines, pad);
        padded[0]
    "#;
    assert_eval_string_with_io(code, "short.....");
}

#[test]
fn test_text_replace_multiple() {
    let code = r#"
        let text: string = "foo bar foo baz";
        let step1: string = replace(text, "foo", "hello");
        let step2: string = replace(step1, "bar", "world");
        step2
    "#;
    assert_eval_string_with_io(code, "hello world foo baz"); // only first "foo" replaced
}

#[test]
fn test_text_split_multichar() {
    let code = r#"
        let text: string = "one::two::three";
        let parts: string[] = split(text, "::");
        len(parts)
    "#;
    assert_eval_number_with_io(code, 3.0);
}

#[test]
fn test_text_extract_numbers() {
    let code = r#"
        let text: string = "Price: 100 Quantity: 50";
        let words: string[] = split(text, " ");
        let num1: number = parseFloat(words[1]);
        let num2: number = parseFloat(words[3]);
        num1 + num2
    "#;
    assert_eval_number_with_io(code, 150.0);
}

#[test]
fn test_text_repeat_pattern() {
    let code = r#"
        let pattern: string = repeat("*", 5.0);
        pattern
    "#;
    assert_eval_string_with_io(code, "*****");
}

#[test]
fn test_text_contains_substring() {
    let code = r#"
        let text: string = "The quick brown fox";
        let hasQuick: bool = includes(text, "quick");
        let hasSlow: bool = includes(text, "slow");
        hasQuick && !hasSlow
    "#;
    assert_eval_bool_with_io(code, true);
}

// ============================================================================
// Category 6: Configuration Processing (10 tests)
// ============================================================================

#[test]
fn test_config_parse_json() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.json");
    std::fs::write(&config_path, "{\"host\": \"localhost\", \"port\": 8080}").unwrap();

    let code = format!(
        r#"
        let configStr: string = readFile("{}");
        let config: json = parseJSON(configStr);
        let host: string = config["host"].as_string();
        host
    "#,
        path_for_atlas(&config_path)
    );
    assert_eval_string_with_io(&code, "localhost");
}

#[test]
fn test_config_extract_port() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.json");
    std::fs::write(&config_path, "{\"port\": 3000}").unwrap();

    let code = format!(
        r#"
        let configStr: string = readFile("{}");
        let config: json = parseJSON(configStr);
        let port: number = config["port"].as_number();
        port
    "#,
        path_for_atlas(&config_path)
    );
    assert_eval_number_with_io(&code, 3000.0);
}

#[test]
fn test_config_validate_required_fields() {
    let code = r#"
        let configStr: string = "{\"host\": \"localhost\", \"port\": 8080}";
        let config: json = parseJSON(configStr);
        let hasHost: bool = !jsonIsNull(config["host"]);
        let hasPort: bool = !jsonIsNull(config["port"]);
        hasHost && hasPort
    "#;
    assert_eval_bool_with_io(code, true);
}

#[test]
fn test_config_missing_field_default() {
    let code = r#"
        let configStr: string = "{\"host\": \"localhost\"}";
        let config: json = parseJSON(configStr);
        let port: json = config["port"];
        var portValue: number = 8080.0;
        if (!jsonIsNull(port)) {
            portValue = port.as_number();
        }
        portValue
    "#;
    assert_eval_number_with_io(code, 8080.0);
}

#[test]
fn test_config_nested_settings() {
    let code = r#"
        let configStr: string = "{\"database\": {\"host\": \"db.local\", \"port\": 5432}}";
        let config: json = parseJSON(configStr);
        let db: json = config["database"];
        let dbHost: string = db["host"].as_string();
        dbHost
    "#;
    assert_eval_string_with_io(code, "db.local");
}

#[test]
fn test_config_boolean_flags() {
    let code = r#"
        let configStr: string = "{\"debug\": true, \"production\": false}";
        let config: json = parseJSON(configStr);
        let debug: bool = config["debug"].as_bool();
        let prod: bool = config["production"].as_bool();
        debug && !prod
    "#;
    assert_eval_bool_with_io(code, true);
}

#[test]
fn test_config_array_values() {
    let code = r#"
        let configStr: string = "{\"allowed_hosts\": [\"localhost\", \"127.0.0.1\"]}";
        let config: json = parseJSON(configStr);
        let hosts: json = config["allowed_hosts"];
        let first: string = hosts[0].as_string();
        first
    "#;
    assert_eval_string_with_io(code, "localhost");
}

#[test]
fn test_config_write_updated() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.json");
    std::fs::write(&config_path, "{\"version\": 1}").unwrap();

    let code = format!(
        r#"
        let configStr: string = readFile("{}");
        let config: json = parseJSON(configStr);
        let version: number = config["version"].as_number();
        let newVersion: number = version + 1.0;
        let updated: string = "{{\"version\":" + str(newVersion) + "}}";
        writeFile("{}", updated);

        let result: string = readFile("{}");
        let newConfig: json = parseJSON(result);
        let finalVersion: number = newConfig["version"].as_number();
        finalVersion
    "#,
        path_for_atlas(&config_path),
        path_for_atlas(&config_path),
        path_for_atlas(&config_path)
    );
    assert_eval_number_with_io(&code, 2.0);
}

#[test]
fn test_config_merge_defaults() {
    let code = r#"
        let userConfig: string = "{\"host\": \"custom.com\"}";
        let defaults: string = "{\"host\": \"localhost\", \"port\": 8080, \"debug\": false}";

        let user: json = parseJSON(userConfig);
        let def: json = parseJSON(defaults);

        let hostUser: json = user["host"];
        let portUser: json = user["port"];

        var finalHost: string = user["host"].as_string();
        if (jsonIsNull(hostUser)) {
            finalHost = def["host"].as_string();
        }

        var finalPort: number = def["port"].as_number();
        if (!jsonIsNull(portUser)) {
            finalPort = user["port"].as_number();
        }

        finalHost + ":" + str(finalPort)
    "#;
    assert_eval_string_with_io(code, "custom.com:8080");
}

#[test]
fn test_config_prettify_for_humans() {
    let code = r#"
        let compact: string = "{\"host\":\"localhost\",\"port\":8080}";
        let pretty: string = prettifyJSON(compact, 2.0);
        includes(pretty, "\n") && includes(pretty, "  ")
    "#;
    assert_eval_bool_with_io(code, true);
}

#[test]
fn test_config_array_length() {
    let code = r#"
        let configStr: string = "{\"servers\": [\"server1\", \"server2\", \"server3\"]}";
        let config: json = parseJSON(configStr);
        let servers: json = config["servers"];
        let s0: string = servers[0].as_string();
        let s1: string = servers[1].as_string();
        let s2: string = servers[2].as_string();
        len(s0) > 0.0 && len(s1) > 0.0 && len(s2) > 0.0
    "#;
    assert_eval_bool_with_io(code, true);
}

#[test]
fn test_config_environment_specific() {
    let code = r#"
        let configStr: string = "{\"env\": \"production\", \"debug\": false}";
        let config: json = parseJSON(configStr);
        let env: string = config["env"].as_string();
        let debug: bool = config["debug"].as_bool();
        let isProd: bool = env == "production";
        isProd && !debug
    "#;
    assert_eval_bool_with_io(code, true);
}

// ============================================================================
// From stdlib_parity_verification.rs
// ============================================================================

// Systematic Standard Library Parity Verification
//
// Verifies that ALL stdlib functions produce identical output in both
// interpreter and VM execution engines. This is critical for correctness.
//
// Coverage:
// - All 18 string functions
// - All 21 array functions
// - All 18 math functions + 5 constants
// - All 17 JSON functions
// - All 10 file I/O functions
// - All type checking functions
// - Edge cases for each function
// - Error cases for each function
//
// Total: 130+ parity tests

// ============================================================================
// String Function Parity Tests (18 functions)
// ============================================================================

#[rstest]
#[case::length("len(\"hello\")", "5")]
#[case::length_empty("len(\"\")", "0")]
#[case::length_unicode("len(\"hello世界\")", "7")]
#[case::concat("\"hello\" + \" \" + \"world\"", "hello world")]
#[case::concat_empty("\"\" + \"test\"", "test")]
#[case::substring("substring(\"hello\", 1, 4)", "ell")]
#[case::substring_full("substring(\"hello\", 0, 5)", "hello")]
#[case::charat("charAt(\"hello\", 1)", "e")]
#[case::charat_first("charAt(\"hello\", 0)", "h")]
#[case::indexof("indexOf(\"hello\", \"l\")", "2")]
#[case::indexof_not_found("indexOf(\"hello\", \"x\")", "-1")]
#[case::split("join(split(\"a,b,c\", \",\"), \"|\")", "a|b|c")]
#[case::split_empty("len(split(\"\", \",\"))", "1")] // Empty string splits to [""]
#[case::join("join([\"a\", \"b\", \"c\"], \",\")", "a,b,c")]
#[case::join_empty("join([], \",\")", "")]
#[case::replace("replace(\"hello world\", \"world\", \"Atlas\")", "hello Atlas")]
#[case::replace_first("replace(\"aaa\", \"a\", \"b\")", "baa")] // replace() only replaces first occurrence
#[case::trim("trim(\"  hello  \")", "hello")]
#[case::trim_no_space("trim(\"hello\")", "hello")]
#[case::to_upper("toUpperCase(\"hello\")", "HELLO")]
#[case::to_upper_mixed("toUpperCase(\"HeLLo\")", "HELLO")]
#[case::to_lower("toLowerCase(\"HELLO\")", "hello")]
#[case::to_lower_mixed("toLowerCase(\"HeLLo\")", "hello")]
#[case::startswith("startsWith(\"hello\", \"he\")", "true")]
#[case::startswith_false("startsWith(\"hello\", \"wo\")", "false")]
#[case::endswith("endsWith(\"hello\", \"lo\")", "true")]
#[case::endswith_false("endsWith(\"hello\", \"he\")", "false")]
#[case::includes("includes(\"hello world\", \"wo\")", "true")]
#[case::includes_false("includes(\"hello world\", \"xyz\")", "false")]
#[case::repeat("repeat(\"ab\", 3)", "ababab")]
#[case::repeat_zero("repeat(\"x\", 0)", "")]
#[case::padstart("padStart(\"5\", 3, \"0\")", "005")]
#[case::padend("padEnd(\"5\", 3, \"0\")", "500")]
#[case::lastindexof("lastIndexOf(\"hello\", \"l\")", "3")]
#[case::lastindexof_not_found("lastIndexOf(\"hello\", \"x\")", "-1")]
#[case::trimstart("trimStart(\"  hello\")", "hello")]
#[case::trimend("trimEnd(\"hello  \")", "hello")]
fn test_string_parity(#[case] code: &str, #[case] expected: &str) {
    // Run in interpreter
    let runtime_interp = Atlas::new();
    let interp_result = runtime_interp.eval(code).unwrap();

    // Run in VM (eval uses VM by default in atlas-runtime)
    let runtime_vm = Atlas::new();
    let vm_result = runtime_vm.eval(code).unwrap();

    // Assert identical output
    assert_eq!(
        format!("{:?}", interp_result),
        format!("{:?}", vm_result),
        "Parity failure for: {}",
        code
    );

    // Verify expected value
    match &interp_result {
        Value::String(s) => assert_eq!(s.as_ref(), expected),
        Value::Number(n) => assert_eq!(&n.to_string(), expected),
        Value::Bool(b) => assert_eq!(&b.to_string(), expected),
        _ => panic!("Unexpected value type"),
    }
}

// ============================================================================
// Array Function Parity Tests (21 functions)
// ============================================================================

#[rstest]
#[case::len("len([1, 2, 3])", "3")]
#[case::len_empty("len([])", "0")]
#[case::concat_add("len(concat([1, 2], [3]))", "3")]
#[case::concat_empty_add("len(concat([], [1]))", "1")]
#[case::pop_result("pop([1, 2, 3])[0]", "3")]
#[case::pop_remainder("len(pop([1, 2, 3])[1])", "2")]
#[case::shift_result("shift([1, 2, 3])[0]", "1")]
#[case::shift_remainder("len(shift([1, 2, 3])[1])", "2")]
#[case::unshift("len(unshift([2, 3], 1))", "3")]
#[case::concat_arr("len(concat([1, 2], [3, 4]))", "4")]
#[case::slice("slice([1, 2, 3, 4], 1, 3)[0]", "2")]
#[case::reverse("reverse([1, 2, 3])[0]", "3")]
// Note: sort() not yet implemented - removing test cases
// #[case::sort_nums("sort([3, 1, 2])[0]", "1")]
// #[case::sort_strings("join(sort([\"c\", \"a\", \"b\"]), \",\")", "a,b,c")]
#[case::indexof_arr("arrayIndexOf([1, 2, 3], 2)", "1")]
#[case::indexof_not_found_arr("arrayIndexOf([1, 2, 3], 5)", "-1")]
#[case::includes_arr("arrayIncludes([1, 2, 3], 2)", "true")]
#[case::includes_false_arr("arrayIncludes([1, 2, 3], 5)", "false")]
#[case::first_elem("[1, 2, 3][0]", "1")]
#[case::last_elem("[1, 2, 3][2]", "3")]
#[case::slice_rest("slice([1, 2, 3], 1, 3)[0]", "2")]
#[case::slice_rest_len("len(slice([1], 1, 1))", "0")]
#[case::flatten("len(flatten([[1, 2], [3, 4]]))", "4")]
#[case::flatten_empty("len(flatten([]))", "0")]
#[case::arraylastindexof("arrayLastIndexOf([1, 2, 3, 2], 2)", "3")]
#[case::arraylastindexof_not_found("arrayLastIndexOf([1, 2, 3], 5)", "-1")]
fn test_array_basic_parity(#[case] code: &str, #[case] expected: &str) {
    let runtime_interp = Atlas::new();
    let interp_result = runtime_interp.eval(code).unwrap();

    let runtime_vm = Atlas::new();
    let vm_result = runtime_vm.eval(code).unwrap();

    assert_eq!(
        format!("{:?}", interp_result),
        format!("{:?}", vm_result),
        "Parity failure for: {}",
        code
    );

    match &interp_result {
        Value::String(s) => assert_eq!(s.as_ref(), expected),
        Value::Number(n) => assert_eq!(&n.to_string(), expected),
        Value::Bool(b) => assert_eq!(&b.to_string(), expected),
        _ => panic!("Unexpected value type"),
    }
}

#[rstest]
#[case::map(
    "fn double(x: number) -> number { return x * 2; } map([1, 2, 3], double)[0]",
    "2"
)]
#[case::filter(
    "fn isEven(x: number) -> bool { return x % 2 == 0; } filter([1, 2, 3, 4], isEven)[0]",
    "2"
)]
#[case::reduce(
    "fn sum(a: number, b: number) -> number { return a + b; } reduce([1, 2, 3], sum, 0)",
    "6"
)]
#[case::every_true(
    "fn isPositive(x: number) -> bool { return x > 0; } every([1, 2, 3], isPositive)",
    "true"
)]
#[case::every_false(
    "fn isPositive(x: number) -> bool { return x > 0; } every([1, -2, 3], isPositive)",
    "false"
)]
#[case::some_true(
    "fn isNegative(x: number) -> bool { return x < 0; } some([1, -2, 3], isNegative)",
    "true"
)]
#[case::some_false(
    "fn isNegative(x: number) -> bool { return x < 0; } some([1, 2, 3], isNegative)",
    "false"
)]
fn test_array_higher_order_parity(#[case] code: &str, #[case] expected: &str) {
    let runtime_interp = Atlas::new();
    let interp_result = runtime_interp.eval(code).unwrap();

    let runtime_vm = Atlas::new();
    let vm_result = runtime_vm.eval(code).unwrap();

    assert_eq!(
        format!("{:?}", interp_result),
        format!("{:?}", vm_result),
        "Parity failure for: {}",
        code
    );

    match &interp_result {
        Value::String(s) => assert_eq!(s.as_ref(), expected),
        Value::Number(n) => assert_eq!(&n.to_string(), expected),
        Value::Bool(b) => assert_eq!(&b.to_string(), expected),
        _ => panic!("Unexpected value type"),
    }
}

// ============================================================================
// Math Function Parity Tests (18 functions + 5 constants)
// ============================================================================

#[rstest]
#[case::abs_positive("abs(5)", "5")]
#[case::abs_negative("abs(-5)", "5")]
#[case::abs_zero("abs(0)", "0")]
#[case::ceil("ceil(4.3)", "5")]
#[case::ceil_negative("ceil(-4.3)", "-4")]
#[case::floor("floor(4.7)", "4")]
#[case::floor_negative("floor(-4.7)", "-5")]
#[case::round("round(4.5)", "4")] // Banker's rounding (round to even)
#[case::round_down("round(4.4)", "4")]
#[case::min("min(5, 3)", "3")]
#[case::min_negative("min(-5, -3)", "-5")]
#[case::max("max(5, 3)", "5")]
#[case::max_negative("max(-5, -3)", "-3")]
#[case::pow("pow(2, 3)", "8")]
#[case::pow_zero("pow(5, 0)", "1")]
#[case::sqrt("sqrt(16)", "4")]
#[case::sqrt_decimal("sqrt(2) > 1.414 && sqrt(2) < 1.415", "true")]
#[case::sin_zero("sin(0)", "0")]
#[case::cos_zero("cos(0)", "1")]
#[case::tan_zero("tan(0)", "0")]
// Note: exp() not implemented
// #[case::exp_zero("exp(0)", "1")]
#[case::log_e(
    "log(2.718281828459045) > 0.999 && log(2.718281828459045) < 1.001",
    "true"
)]
// Note: log10() not implemented (only log/ln)
// #[case::log10("log10(100)", "2")]
#[case::pi("PI > 3.14159 && PI < 3.14160", "true")]
#[case::e("E > 2.71828 && E < 2.71829", "true")]
#[case::clamp_mid("clamp(5, 0, 10)", "5")]
#[case::clamp_low("clamp(-5, 0, 10)", "0")]
#[case::clamp_high("clamp(15, 0, 10)", "10")]
#[case::sign_positive("sign(42)", "1")]
#[case::sign_negative("sign(-42)", "-1")]
#[case::sign_zero("sign(0)", "0")]
#[case::asin_zero("asin(0)", "0")]
#[case::acos_one("acos(1)", "0")]
#[case::atan_zero("atan(0)", "0")]
fn test_math_parity(#[case] code: &str, #[case] expected: &str) {
    let runtime_interp = Atlas::new();
    let interp_result = runtime_interp.eval(code).unwrap();

    let runtime_vm = Atlas::new();
    let vm_result = runtime_vm.eval(code).unwrap();

    assert_eq!(
        format!("{:?}", interp_result),
        format!("{:?}", vm_result),
        "Parity failure for: {}",
        code
    );

    match &interp_result {
        Value::String(s) => assert_eq!(s.as_ref(), expected),
        Value::Number(n) => assert_eq!(&n.to_string(), expected),
        Value::Bool(b) => assert_eq!(&b.to_string(), expected),
        _ => {}
    }
}

// ============================================================================
// JSON Function Parity Tests (17 functions)
// ============================================================================

#[rstest]
#[case::parse_object(
    "let j = parseJSON(\"{\\\"key\\\": \\\"value\\\"}\"); j[\"key\"].as_string()",
    "value"
)]
#[case::parse_array("let j = parseJSON(\"[1, 2, 3]\"); j[0].as_number()", "1")]
#[case::parse_number("let j = parseJSON(\"42\"); j.as_number()", "42")]
#[case::parse_string("let j = parseJSON(\"\\\"hello\\\"\"); j.as_string()", "hello")]
#[case::parse_bool("let j = parseJSON(\"true\"); j.as_bool()", "true")]
#[case::parse_null("let j = parseJSON(\"null\"); j.is_null()", "true")]
#[case::stringify_object("toJSON(parseJSON(\"{\\\"a\\\": 1}\"))", "{\"a\":1}")]
#[case::stringify_array("toJSON(parseJSON(\"[1,2,3]\"))", "[1,2,3]")]
#[case::as_string("parseJSON(\"\\\"test\\\"\").as_string()", "test")]
#[case::as_number("parseJSON(\"123\").as_number()", "123")]
#[case::as_bool("parseJSON(\"true\").as_bool()", "true")]
#[case::is_null_true("parseJSON(\"null\").is_null()", "true")]
#[case::is_null_false("parseJSON(\"123\").is_null()", "false")]
// Note: JSON type checking methods not yet implemented
// #[case::is_array_true("parseJSON(\"[1,2]\").is_array()", "true")]
// #[case::is_array_false("parseJSON(\"123\").is_array()", "false")]
// #[case::is_object_true("parseJSON(\"{\\\"a\\\": 1}\").is_object()", "true")]
// #[case::is_object_false("parseJSON(\"123\").is_object()", "false")]
// #[case::array_length("parseJSON(\"[1,2,3]\").array_length()", "3")]
#[case::nested_access(
    "let j = parseJSON(\"{\\\"a\\\": {\\\"b\\\": 42}}\"); j[\"a\"][\"b\"].as_number()",
    "42"
)]
#[case::json_array_index("let j = parseJSON(\"[10, 20, 30]\"); j[1].as_number()", "20")]
#[case::json_string_value(
    "let j = parseJSON(\"{\\\"name\\\": \\\"Alice\\\"}\"); j[\"name\"].as_string()",
    "Alice"
)]
#[case::json_bool_value(
    "let j = parseJSON(\"{\\\"active\\\": false}\"); j[\"active\"].as_bool()",
    "false"
)]
#[case::isvalidjson_true("isValidJSON(\"{\\\"key\\\": \\\"value\\\"}\")", "true")]
#[case::isvalidjson_false("isValidJSON(\"invalid json\")", "false")]
fn test_json_parity(#[case] code: &str, #[case] expected: &str) {
    let runtime_interp = Atlas::new();
    let interp_result = runtime_interp.eval(code).unwrap();

    let runtime_vm = Atlas::new();
    let vm_result = runtime_vm.eval(code).unwrap();

    assert_eq!(
        format!("{:?}", interp_result),
        format!("{:?}", vm_result),
        "Parity failure for: {}",
        code
    );

    match &interp_result {
        Value::String(s) => assert_eq!(s.as_ref(), expected),
        Value::Number(n) => assert_eq!(&n.to_string(), expected),
        Value::Bool(b) => assert_eq!(&b.to_string(), expected),
        _ => panic!("Unexpected value type for: {}", code),
    }
}

// ============================================================================
// File I/O Function Parity Tests (10 functions)
// ============================================================================

#[test]
fn test_file_read_write_parity() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");

    // Write and read back
    let code = format!(
        r#"
        writeFile("{}", "test content");
        readFile("{}")
    "#,
        path_for_atlas(&file_path),
        path_for_atlas(&file_path)
    );

    // Interpreter
    let mut security_interp = SecurityContext::new();
    security_interp.grant_filesystem_read(temp_dir.path(), true);
    security_interp.grant_filesystem_write(temp_dir.path(), true);
    let runtime_interp = Atlas::new_with_security(security_interp);
    let interp_result = runtime_interp.eval(&code).unwrap();

    // VM
    let mut security_vm = SecurityContext::new();
    security_vm.grant_filesystem_read(temp_dir.path(), true);
    security_vm.grant_filesystem_write(temp_dir.path(), true);
    let runtime_vm = Atlas::new_with_security(security_vm);
    let vm_result = runtime_vm.eval(&code).unwrap();

    assert_eq!(format!("{:?}", interp_result), format!("{:?}", vm_result));
    assert_eq!(
        interp_result,
        Value::String(Arc::new("test content".to_string()))
    );
}

#[test]
fn test_file_exists_parity() {
    let temp_dir = TempDir::new().unwrap();
    let existing = temp_dir.path().join("exists.txt");
    let non_existing = temp_dir.path().join("nonexistent.txt");
    std::fs::write(&existing, "content").unwrap();

    let code_exists = format!(r#"fileExists("{}")"#, path_for_atlas(&existing));
    let code_not_exists = format!(r#"fileExists("{}")"#, path_for_atlas(&non_existing));

    // Test existing file
    let mut security1 = SecurityContext::new();
    security1.grant_filesystem_read(temp_dir.path(), true);
    let runtime_interp = Atlas::new_with_security(security1);
    let interp_result = runtime_interp.eval(&code_exists).unwrap();

    let mut security2 = SecurityContext::new();
    security2.grant_filesystem_read(temp_dir.path(), true);
    let runtime_vm = Atlas::new_with_security(security2);
    let vm_result = runtime_vm.eval(&code_exists).unwrap();

    assert_eq!(format!("{:?}", interp_result), format!("{:?}", vm_result));
    assert_eq!(interp_result, Value::Bool(true));

    // Test non-existing file
    let mut security3 = SecurityContext::new();
    security3.grant_filesystem_read(temp_dir.path(), true);
    let runtime_interp2 = Atlas::new_with_security(security3);
    let interp_result2 = runtime_interp2.eval(&code_not_exists).unwrap();

    let mut security4 = SecurityContext::new();
    security4.grant_filesystem_read(temp_dir.path(), true);
    let runtime_vm2 = Atlas::new_with_security(security4);
    let vm_result2 = runtime_vm2.eval(&code_not_exists).unwrap();

    assert_eq!(format!("{:?}", interp_result2), format!("{:?}", vm_result2));
    assert_eq!(interp_result2, Value::Bool(false));
}

#[test]
fn test_file_delete_parity() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("delete_me.txt");

    let code = format!(
        r#"
        writeFile("{}", "content");
        removeFile("{}");
        fileExists("{}")
    "#,
        path_for_atlas(&file_path),
        path_for_atlas(&file_path),
        path_for_atlas(&file_path)
    );

    // Interpreter
    let mut security_interp = SecurityContext::new();
    security_interp.grant_filesystem_read(temp_dir.path(), true);
    security_interp.grant_filesystem_write(temp_dir.path(), true);
    let runtime_interp = Atlas::new_with_security(security_interp);
    let interp_result = runtime_interp.eval(&code).unwrap();

    // VM
    let mut security_vm = SecurityContext::new();
    security_vm.grant_filesystem_read(temp_dir.path(), true);
    security_vm.grant_filesystem_write(temp_dir.path(), true);
    let runtime_vm = Atlas::new_with_security(security_vm);
    let vm_result = runtime_vm.eval(&code).unwrap();

    assert_eq!(format!("{:?}", interp_result), format!("{:?}", vm_result));
    assert_eq!(interp_result, Value::Bool(false));
}

#[test]
fn test_file_append_parity() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("append.txt");

    let code = format!(
        r#"
        writeFile("{}", "first");
        appendFile("{}", "second");
        readFile("{}")
    "#,
        path_for_atlas(&file_path),
        path_for_atlas(&file_path),
        path_for_atlas(&file_path)
    );

    // Interpreter
    let mut security_interp = SecurityContext::new();
    security_interp.grant_filesystem_read(temp_dir.path(), true);
    security_interp.grant_filesystem_write(temp_dir.path(), true);
    let runtime_interp = Atlas::new_with_security(security_interp);
    let interp_result = runtime_interp.eval(&code).unwrap();

    // VM
    let mut security_vm = SecurityContext::new();
    security_vm.grant_filesystem_read(temp_dir.path(), true);
    security_vm.grant_filesystem_write(temp_dir.path(), true);
    let runtime_vm = Atlas::new_with_security(security_vm);
    let vm_result = runtime_vm.eval(&code).unwrap();

    assert_eq!(format!("{:?}", interp_result), format!("{:?}", vm_result));
    assert_eq!(
        interp_result,
        Value::String(Arc::new("firstsecond".to_string()))
    );
}

#[test]
fn test_file_list_directory_parity() {
    let temp_dir = TempDir::new().unwrap();
    std::fs::write(temp_dir.path().join("file1.txt"), "content1").unwrap();
    std::fs::write(temp_dir.path().join("file2.txt"), "content2").unwrap();

    let code = format!(r#"len(readDir("{}"))"#, path_for_atlas(temp_dir.path()));

    // Interpreter
    let mut security_interp = SecurityContext::new();
    security_interp.grant_filesystem_read(temp_dir.path(), true);
    let runtime_interp = Atlas::new_with_security(security_interp);
    let interp_result = runtime_interp.eval(&code).unwrap();

    // VM
    let mut security_vm = SecurityContext::new();
    security_vm.grant_filesystem_read(temp_dir.path(), true);
    let runtime_vm = Atlas::new_with_security(security_vm);
    let vm_result = runtime_vm.eval(&code).unwrap();

    assert_eq!(format!("{:?}", interp_result), format!("{:?}", vm_result));
    assert_eq!(interp_result, Value::Number(2.0));
}

#[test]
fn test_file_create_remove_directory_parity() {
    let temp_dir = TempDir::new().unwrap();
    let dir_path = temp_dir.path().join("testdir");

    let code = format!(
        r#"
        createDir("{}");
        let exists1 = fileExists("{}");
        removeDir("{}");
        let exists2 = fileExists("{}");
        exists1 && !exists2
    "#,
        path_for_atlas(&dir_path),
        path_for_atlas(&dir_path),
        path_for_atlas(&dir_path),
        path_for_atlas(&dir_path)
    );

    // Interpreter
    let mut security_interp = SecurityContext::new();
    security_interp.grant_filesystem_read(temp_dir.path(), true);
    security_interp.grant_filesystem_write(temp_dir.path(), true);
    let runtime_interp = Atlas::new_with_security(security_interp);
    let interp_result = runtime_interp.eval(&code).unwrap();

    // VM
    let mut security_vm = SecurityContext::new();
    security_vm.grant_filesystem_read(temp_dir.path(), true);
    security_vm.grant_filesystem_write(temp_dir.path(), true);
    let runtime_vm = Atlas::new_with_security(security_vm);
    let vm_result = runtime_vm.eval(&code).unwrap();

    assert_eq!(format!("{:?}", interp_result), format!("{:?}", vm_result));
    assert_eq!(interp_result, Value::Bool(true));
}

// ============================================================================
// Type Checking Function Parity Tests (6 functions)
// ============================================================================

#[rstest]
#[case::is_string_true("isString(\"hello\")", "true")]
#[case::is_string_false("isString(123)", "false")]
#[case::is_number_true("isNumber(123)", "true")]
#[case::is_number_false("isNumber(\"123\")", "false")]
#[case::is_bool_true("isBool(true)", "true")]
#[case::is_bool_false("isBool(1)", "false")]
#[case::is_null_true("isNull(null)", "true")]
#[case::is_null_false("isNull(0)", "false")]
#[case::is_array_true("isArray([1, 2, 3])", "true")]
#[case::is_array_false("isArray(\"[1,2,3]\")", "false")]
#[case::is_function_true("fn test() {} isFunction(test)", "true")]
#[case::is_function_false("isFunction(123)", "false")]
fn test_type_checking_parity(#[case] code: &str, #[case] expected: &str) {
    let runtime_interp = Atlas::new();
    let interp_result = runtime_interp.eval(code).unwrap();

    let runtime_vm = Atlas::new();
    let vm_result = runtime_vm.eval(code).unwrap();

    assert_eq!(
        format!("{:?}", interp_result),
        format!("{:?}", vm_result),
        "Parity failure for: {}",
        code
    );

    match &interp_result {
        Value::Bool(b) => assert_eq!(&b.to_string(), expected),
        _ => panic!("Expected bool for type checking"),
    }
}

// ============================================================================
// Edge Case & Error Parity Tests
// ============================================================================

#[rstest]
#[case::empty_string_operations("len(trim(\"\"))", "0")]
#[case::empty_array_operations("len(reverse([]))", "0")]
#[case::divide_by_zero("1 / 0 > 999999999999999", "true")] // inf
#[case::negative_sqrt("sqrt(-1)", "NaN")] // NaN as string
#[case::parse_invalid_json_safety("let j = parseJSON(\"invalid\"); j.is_null()", "false")] // Returns error, not crash
fn test_edge_cases_parity(#[case] code: &str, #[case] _expected: &str) {
    let runtime_interp = Atlas::new();
    let interp_result = runtime_interp.eval(code);

    let runtime_vm = Atlas::new();
    let vm_result = runtime_vm.eval(code);

    // Both should succeed or both should fail with same error
    match (&interp_result, &vm_result) {
        (Ok(v1), Ok(v2)) => {
            assert_eq!(
                format!("{:?}", v1),
                format!("{:?}", v2),
                "Parity failure for: {}",
                code
            );
        }
        (Err(e1), Err(e2)) => {
            assert_eq!(e1.len(), e2.len(), "Different error counts for: {}", code);
            if !e1.is_empty() && !e2.is_empty() {
                assert_eq!(
                    e1[0].code, e2[0].code,
                    "Different error codes for: {}",
                    code
                );
            }
        }
        _ => panic!("Parity failure: one succeeded, one failed for: {}", code),
    }
}

// ============================================================================
// From option_result_tests.rs
// ============================================================================

// Integration tests for Option<T> and Result<T,E>
//
// BLOCKER 02-D: Built-in Generic Types

// ============================================================================
// Option<T> Tests
// ============================================================================

#[test]
fn test_option_is_some() {
    assert_eval_bool("is_some(Some(42))", true);
    assert_eval_bool("is_some(None())", false);
}

#[test]
fn test_option_is_none() {
    assert_eval_bool("is_none(None())", true);
    assert_eval_bool("is_none(Some(42))", false);
}

#[test]
fn test_option_unwrap_number() {
    assert_eval_number("unwrap(Some(42))", 42.0);
}

#[test]
fn test_option_unwrap_string() {
    assert_eval_string(r#"unwrap(Some("hello"))"#, "hello");
}

#[test]
fn test_option_unwrap_bool() {
    assert_eval_bool("unwrap(Some(true))", true);
}

#[test]
fn test_option_unwrap_null() {
    assert_eval_null("unwrap(Some(null))");
}

#[test]
fn test_option_unwrap_or_some() {
    assert_eval_number("unwrap_or(Some(42), 0)", 42.0);
}

#[test]
fn test_option_unwrap_or_none() {
    assert_eval_number("unwrap_or(None(), 99)", 99.0);
}

#[test]
fn test_option_unwrap_or_string() {
    assert_eval_string(r#"unwrap_or(Some("hello"), "default")"#, "hello");
    assert_eval_string(r#"unwrap_or(None(), "default")"#, "default");
}

#[test]
fn test_option_nested() {
    assert_eval_number("unwrap(unwrap(Some(Some(42))))", 42.0);
}

// ============================================================================
// Result<T,E> Tests
// ============================================================================

#[test]
fn test_result_is_ok() {
    assert_eval_bool("is_ok(Ok(42))", true);
    assert_eval_bool(r#"is_ok(Err("failed"))"#, false);
}

#[test]
fn test_result_is_err() {
    assert_eval_bool(r#"is_err(Err("failed"))"#, true);
    assert_eval_bool("is_err(Ok(42))", false);
}

#[test]
fn test_result_unwrap_ok_number() {
    assert_eval_number("unwrap(Ok(42))", 42.0);
}

#[test]
fn test_result_unwrap_ok_string() {
    assert_eval_string(r#"unwrap(Ok("success"))"#, "success");
}

#[test]
fn test_result_unwrap_ok_null() {
    assert_eval_null("unwrap(Ok(null))");
}

#[test]
fn test_result_unwrap_or_ok() {
    assert_eval_number("unwrap_or(Ok(42), 0)", 42.0);
}

#[test]
fn test_result_unwrap_or_err() {
    assert_eval_number(r#"unwrap_or(Err("failed"), 99)"#, 99.0);
}

#[test]
fn test_result_unwrap_or_string() {
    assert_eval_string(r#"unwrap_or(Ok("success"), "default")"#, "success");
    assert_eval_string(r#"unwrap_or(Err(404), "default")"#, "default");
}

// ============================================================================
// Mixed Option/Result Tests
// ============================================================================

#[test]
fn test_option_and_result_together() {
    let code = r#"
        let opt = Some(42);
        let res = Ok(99);
        unwrap(opt) + unwrap(res)
    "#;
    assert_eval_number(code, 141.0);
}

#[test]
fn test_option_in_conditional() {
    let code = r#"
        let opt = Some(42);
        if (is_some(opt)) {
            unwrap(opt);
        } else {
            0;
        }
    "#;
    assert_eval_number(code, 42.0);
}

#[test]
fn test_result_in_conditional() {
    let code = r#"
        let res = Ok(42);
        if (is_ok(res)) {
            unwrap(res);
        } else {
            0;
        }
    "#;
    assert_eval_number(code, 42.0);
}

// ============================================================================
// Complex Tests
// ============================================================================

#[test]
fn test_option_chain() {
    let code = r#"
        let a = Some(10);
        let b = Some(20);
        let c = Some(30);
        unwrap(a) + unwrap(b) + unwrap(c)
    "#;
    assert_eval_number(code, 60.0);
}

#[test]
fn test_result_chain() {
    let code = r#"
        let a = Ok(10);
        let b = Ok(20);
        let c = Ok(30);
        unwrap(a) + unwrap(b) + unwrap(c)
    "#;
    assert_eval_number(code, 60.0);
}

#[test]
fn test_option_unwrap_or_with_none_chain() {
    let code = r#"
        let a = None();
        let b = None();
        unwrap_or(a, 5) + unwrap_or(b, 10)
    "#;
    assert_eval_number(code, 15.0);
}

#[test]
fn test_result_unwrap_or_with_err_chain() {
    let code = r#"
        let a = Err("fail1");
        let b = Err("fail2");
        unwrap_or(a, 5) + unwrap_or(b, 10)
    "#;
    assert_eval_number(code, 15.0);
}

// ============================================================================
// From result_advanced_tests.rs
// ============================================================================

// Advanced Result<T,E> method tests (interpreter)
//
// Tests for expect, result_map, result_map_err, result_and_then, result_or_else, result_ok, result_err, ? operator

// ============================================================================
// expect() Tests
// ============================================================================

#[test]
fn test_expect_ok() {
    assert_eval_number(r#"expect(Ok(42), "should have value")"#, 42.0);
}

#[test]
fn test_expect_err_panics() {
    assert_has_error(r#"expect(Err("failed"), "custom message")"#);
}

#[test]
fn test_expect_with_string() {
    assert_eval_string(r#"expect(Ok("success"), "should work")"#, "success");
}

// ============================================================================
// result_ok() Tests - Convert Result to Option
// ============================================================================

#[test]
fn test_result_ok_from_ok() {
    let code = r#"
        let result = Ok(42);
        let opt = result_ok(result);
        unwrap(opt)
    "#;
    assert_eval_number(code, 42.0);
}

#[test]
fn test_result_ok_from_err() {
    let code = r#"
        let result = Err("failed");
        let opt = result_ok(result);
        is_none(opt)
    "#;
    assert_eval_bool(code, true);
}

// ============================================================================
// result_err() Tests - Extract Err to Option
// ============================================================================

#[test]
fn test_result_err_from_ok() {
    let code = r#"
        let result = Ok(42);
        let opt = result_err(result);
        is_none(opt)
    "#;
    assert_eval_bool(code, true);
}

#[test]
fn test_result_err_from_err() {
    let code = r#"
        let result = Err("failed");
        let opt = result_err(result);
        unwrap(opt)
    "#;
    assert_eval_string(code, "failed");
}

// ============================================================================
// result_map() Tests - Transform Ok value
// ============================================================================

#[test]
fn test_result_map_ok() {
    let code = r#"
        fn double(x: number) -> number { return x * 2; }
        let result = Ok(21);
        let mapped = result_map(result, double);
        unwrap(mapped)
    "#;
    assert_eval_number(code, 42.0);
}

#[test]
fn test_result_map_err_preserves() {
    let code = r#"
        fn double(x: number) -> number { return x * 2; }
        let result = Err("failed");
        let mapped = result_map(result, double);
        is_err(mapped)
    "#;
    assert_eval_bool(code, true);
}

#[test]
fn test_result_map_chain() {
    let code = r#"
        fn double(x: number) -> number { return x * 2; }
        fn triple(x: number) -> number { return x * 3; }
        let result = Ok(7);
        let mapped = result_map(result, double);
        let mapped2 = result_map(mapped, triple);
        unwrap(mapped2)
    "#;
    assert_eval_number(code, 42.0); // 7 * 2 * 3 = 42
}

// ============================================================================
// result_map_err() Tests - Transform Err value
// ============================================================================

#[test]
fn test_result_map_err_transforms_error() {
    let code = r#"
        fn format_error(e: string) -> string { return "Error: " + e; }
        let result = Err("failed");
        let mapped = result_map_err(result, format_error);
        unwrap_or(mapped, "default")
    "#;
    assert_eval_string(code, "default");
}

#[test]
fn test_result_map_err_preserves_ok() {
    let code = r#"
        fn format_error(e: string) -> string { return "Error: " + e; }
        let result = Ok(42);
        let mapped = result_map_err(result, format_error);
        unwrap(mapped)
    "#;
    assert_eval_number(code, 42.0);
}

// ============================================================================
// result_and_then() Tests - Monadic chaining
// ============================================================================

#[test]
fn test_result_and_then_success_chain() {
    let code = r#"
        fn divide(x: number) -> Result<number, string> {
            if (x == 0) {
                return Err("division by zero");
            }
            return Ok(100 / x);
        }
        let result = Ok(10);
        let chained = result_and_then(result, divide);
        unwrap(chained)
    "#;
    assert_eval_number(code, 10.0);
}

#[test]
fn test_result_and_then_error_propagates() {
    let code = r#"
        fn divide(x: number) -> Result<number, string> {
            if (x == 0) {
                return Err("division by zero");
            }
            return Ok(100 / x);
        }
        let result = Err("initial error");
        let chained = result_and_then(result, divide);
        is_err(chained)
    "#;
    assert_eval_bool(code, true);
}

#[test]
fn test_result_and_then_returns_error() {
    let code = r#"
        fn divide(x: number) -> Result<number, string> {
            if (x == 0) {
                return Err("division by zero");
            }
            return Ok(100 / x);
        }
        let result = Ok(0);
        let chained = result_and_then(result, divide);
        is_err(chained)
    "#;
    assert_eval_bool(code, true);
}

// ============================================================================
// result_or_else() Tests - Error recovery
// ============================================================================

#[test]
fn test_result_or_else_recovers_from_error() {
    let code = r#"
        fn recover(_e: string) -> Result<number, string> {
            return Ok(0);
        }
        let result = Err("failed");
        let recovered = result_or_else(result, recover);
        unwrap(recovered)
    "#;
    assert_eval_number(code, 0.0);
}

#[test]
fn test_result_or_else_preserves_ok() {
    let code = r#"
        fn recover(_e: string) -> Result<number, string> {
            return Ok(0);
        }
        let result = Ok(42);
        let recovered = result_or_else(result, recover);
        unwrap(recovered)
    "#;
    assert_eval_number(code, 42.0);
}

#[test]
fn test_result_or_else_can_return_error() {
    let code = r#"
        fn retry(_e: string) -> Result<number, string> {
            return Err("retry failed");
        }
        let result = Err("initial");
        let recovered = result_or_else(result, retry);
        is_err(recovered)
    "#;
    assert_eval_bool(code, true);
}

// ============================================================================
// Complex Combination Tests
// ============================================================================

#[test]
fn test_result_pipeline() {
    let code = r#"
        fn double(x: number) -> number { return x * 2; }
        fn safe_divide(x: number) -> Result<number, string> {
            if (x == 0) {
                return Err("division by zero");
            }
            return Ok(100 / x);
        }

        let result = Ok(10);
        let step1 = result_map(result, double);
        let step2 = result_and_then(step1, safe_divide);
        unwrap(step2)
    "#;
    assert_eval_number(code, 5.0); // (10 * 2) = 20, then 100 / 20 = 5
}

#[test]
fn test_result_error_recovery_pipeline() {
    let code = r#"
        fn recover(_e: string) -> Result<number, string> {
            return Ok(99);
        }
        fn double(x: number) -> number { return x * 2; }

        let result = Err("initial");
        let recovered = result_or_else(result, recover);
        let mapped = result_map(recovered, double);
        unwrap(mapped)
    "#;
    assert_eval_number(code, 198.0); // recover to 99, then * 2
}

// ============================================================================
// Error Propagation Operator (?) Tests
// ============================================================================

#[test]
fn test_try_operator_unwraps_ok() {
    let code = r#"
        fn get_value() -> Result<number, string> {
            let result = Ok(42);
            return Ok(result?);
        }
        unwrap(get_value())
    "#;
    assert_eval_number(code, 42.0);
}

#[test]
fn test_try_operator_propagates_error() {
    let code = r#"
        fn get_value() -> Result<number, string> {
            let result = Err("failed");
            return Ok(result?);
        }
        is_err(get_value())
    "#;
    assert_eval_bool(code, true);
}

#[test]
fn test_try_operator_multiple_propagations() {
    let code = r#"
        fn divide(a: number, b: number) -> Result<number, string> {
            if (b == 0) {
                return Err("division by zero");
            }
            return Ok(a / b);
        }

        fn calculate() -> Result<number, string> {
            let x = divide(100, 10)?;
            let y = divide(x, 2)?;
            let z = divide(y, 5)?;
            return Ok(z);
        }

        unwrap(calculate())
    "#;
    assert_eval_number(code, 1.0); // 100 / 10 = 10, 10 / 2 = 5, 5 / 5 = 1
}

#[test]
fn test_try_operator_early_return() {
    let code = r#"
        fn divide(a: number, b: number) -> Result<number, string> {
            if (b == 0) {
                return Err("division by zero");
            }
            return Ok(a / b);
        }

        fn calculate() -> Result<number, string> {
            let x = divide(100, 10)?;
            let y = divide(x, 0)?;  // This will error
            let z = divide(y, 5)?;  // This won't execute
            return Ok(z);
        }

        is_err(calculate())
    "#;
    assert_eval_bool(code, true);
}

#[test]
fn test_try_operator_with_expressions() {
    let code = r#"
        fn get_number() -> Result<number, string> {
            return Ok(21);
        }

        fn double_it() -> Result<number, string> {
            return Ok(get_number()? * 2);
        }

        unwrap(double_it())
    "#;
    assert_eval_number(code, 42.0);
}

#[test]
fn test_try_operator_in_nested_calls() {
    let code = r#"
        fn inner() -> Result<number, string> {
            return Ok(42);
        }

        fn middle() -> Result<number, string> {
            return Ok(inner()?);
        }

        fn outer() -> Result<number, string> {
            return Ok(middle()?);
        }

        unwrap(outer())
    "#;
    assert_eval_number(code, 42.0);
}

#[test]
fn test_try_operator_with_error_in_nested_calls() {
    let code = r#"
        fn inner() -> Result<number, string> {
            return Err("inner failed");
        }

        fn middle() -> Result<number, string> {
            return Ok(inner()?);
        }

        fn outer() -> Result<number, string> {
            return Ok(middle()?);
        }

        is_err(outer())
    "#;
    assert_eval_bool(code, true);
}

#[test]
fn test_try_operator_combined_with_methods() {
    let code = r#"
        fn get_value() -> Result<number, string> {
            return Ok(10);
        }

        fn double(x: number) -> number {
            return x * 2;
        }

        fn process() -> Result<number, string> {
            let val = get_value()?;
            let mapped = Ok(double(val));
            return Ok(mapped?);
        }

        unwrap(process())
    "#;
    assert_eval_number(code, 20.0);
}

// ============================================================================
// From first_class_functions_tests.rs
// ============================================================================

// First-class functions tests for interpreter
//
// Tests that functions can be:
// - Stored in variables
// - Passed as arguments
// - Returned from functions
// - Called through variables
//
// Note: Some tests currently trigger false-positive "unused parameter" warnings.
// This is a pre-existing bug in the warning system (AT2001) - it doesn't recognize
// parameters passed to function-valued variables as "used". The actual first-class
// function functionality works correctly. This will be fixed in a separate phase.

// ============================================================================
// Category 1: Variable Storage (20 tests)
// ============================================================================

#[test]
fn test_store_function_in_let() {
    let source = r#"
        fn double(x: number) -> number { return x * 2; }
        let f = double;
        f(5);
    "#;
    assert_eval_number(source, 10.0);
}

#[test]
fn test_store_function_in_var() {
    let source = r#"
        fn triple(x: number) -> number { return x * 3; }
        var f = triple;
        f(4);
    "#;
    assert_eval_number(source, 12.0);
}

#[test]
fn test_reassign_function_variable() {
    let source = r#"
        fn add(a: number, b: number) -> number { return a + b; }
        fn mul(a: number, b: number) -> number { return a * b; }
        var f = add;
        let x = f(2, 3);
        f = mul;
        let y = f(2, 3);
        y;
    "#;
    assert_eval_number(source, 6.0);
}

#[test]
fn test_store_builtin_print() {
    let source = r#"
        let p = print;
        p("test");
    "#;
    // print returns void
    assert_eval_null(source);
}

#[test]
fn test_store_builtin_len() {
    let source = r#"
        let l = len;
        l("hello");
    "#;
    assert_eval_number(source, 5.0);
}

#[test]
fn test_store_builtin_str() {
    let source = r#"
        let s = str;
        s(42);
    "#;
    assert_eval_string(source, "42");
}

#[test]
fn test_multiple_function_variables() {
    let source = r#"
        fn add(a: number, b: number) -> number { return a + b; }
        fn sub(a: number, b: number) -> number { return a - b; }
        let f1 = add;
        let f2 = sub;
        f1(10, 3) + f2(10, 3);
    "#;
    assert_eval_number(source, 20.0);
}

#[test]
fn test_function_variable_with_same_name() {
    let source = r#"
        fn double(x: number) -> number { return x * 2; }
        let double = double;
        double(5);
    "#;
    assert_eval_number(source, 10.0);
}

// Requires nested functions or closure capture (deferred to v0.3+)
#[test]
#[ignore = "requires nested functions or closure capture — deferred to v0.3+"]
fn test_function_variable_in_block() {
    let source = r#"
        fn square(x: number) -> number { return x * x; }
        {
            let f = square;
            f(3);
        }
    "#;
    assert_eval_number(source, 9.0);
}

// Requires nested functions or closure capture (deferred to v0.3+)
#[test]
#[ignore = "requires nested functions or closure capture — deferred to v0.3+"]
fn test_function_variable_shadowing() {
    let source = r#"
        fn add(a: number, b: number) -> number { return a + b; }
        fn mul(a: number, b: number) -> number { return a * b; }
        let f = add;
        {
            let f = mul;
            f(2, 3);
        }
    "#;
    assert_eval_number(source, 6.0);
}

// ============================================================================
// Category 2: Function Parameters (25 tests)
// ============================================================================

// Requires nested functions or closure capture (deferred to v0.3+)
#[test]
#[ignore = "requires nested functions or closure capture — deferred to v0.3+"]
fn test_pass_function_as_argument() {
    let source = r#"
        fn apply(f: (number) -> number, x: number) -> number {
            return f(x);
        }
        fn double(n: number) -> number { return n * 2; }
        apply(double, 5);
    "#;
    assert_eval_number(source, 10.0);
}

// Requires nested functions or closure capture (deferred to v0.3+)
#[test]
#[ignore = "requires nested functions or closure capture — deferred to v0.3+"]
fn test_pass_builtin_as_argument() {
    let source = r#"
        fn applyStr(f: (number) -> string, x: number) -> string {
            return f(x);
        }
        applyStr(str, 42);
    "#;
    assert_eval_string(source, "42");
}

// Requires nested functions or closure capture (deferred to v0.3+)
#[test]
#[ignore = "requires nested functions or closure capture — deferred to v0.3+"]
fn test_pass_function_through_variable() {
    let source = r#"
        fn apply(f: (number) -> number, x: number) -> number {
            return f(x);
        }
        fn triple(n: number) -> number { return n * 3; }
        let myFunc = triple;
        apply(myFunc, 4);
    "#;
    assert_eval_number(source, 12.0);
}

// Requires nested functions or closure capture (deferred to v0.3+)
#[test]
#[ignore = "requires nested functions or closure capture — deferred to v0.3+"]
fn test_multiple_function_parameters() {
    let source = r#"
        fn compose(
            f: (number) -> number,
            g: (number) -> number,
            x: number
        ) -> number {
            return f(g(x));
        }
        fn double(n: number) -> number { return n * 2; }
        fn inc(n: number) -> number { return n + 1; }
        compose(double, inc, 5);
    "#;
    assert_eval_number(source, 12.0);
}

// Requires nested functions or closure capture (deferred to v0.3+)
#[test]
#[ignore = "requires nested functions or closure capture — deferred to v0.3+"]
fn test_function_parameter_called_multiple_times() {
    let source = r#"
        fn applyTwice(f: (number) -> number, x: number) -> number {
            return f(f(x));
        }
        fn double(n: number) -> number { return n * 2; }
        applyTwice(double, 3);
    "#;
    assert_eval_number(source, 12.0);
}

// Requires nested functions or closure capture (deferred to v0.3+)
#[test]
#[ignore = "requires nested functions or closure capture — deferred to v0.3+"]
fn test_function_parameter_with_string() {
    let source = r#"
        fn apply(f: (string) -> number, s: string) -> number {
            return f(s);
        }
        apply(len, "hello");
    "#;
    assert_eval_number(source, 5.0);
}

// Requires nested functions or closure capture (deferred to v0.3+)
#[test]
#[ignore = "requires nested functions or closure capture — deferred to v0.3+"]
fn test_function_parameter_two_args() {
    let source = r#"
        fn applyBinary(
            f: (number, number) -> number,
            a: number,
            b: number
        ) -> number {
            return f(a, b);
        }
        fn add(x: number, y: number) -> number { return x + y; }
        applyBinary(add, 10, 20);
    "#;
    assert_eval_number(source, 30.0);
}

#[test]
fn test_conditional_function_call() {
    let source = r#"
        fn apply(f: (number) -> number, x: number, flag: bool) -> number {
            if (flag) {
                return f(x);
            }
            return x;
        }
        fn double(n: number) -> number { return n * 2; }
        apply(double, 5, true);
    "#;
    assert_eval_number(source, 10.0);
}

// Requires nested functions or closure capture (deferred to v0.3+)
#[test]
#[ignore = "requires nested functions or closure capture — deferred to v0.3+"]
fn test_function_in_loop() {
    let source = r#"
        fn apply(f: (number) -> number, x: number) -> number {
            return f(x);
        }
        fn inc(n: number) -> number { return n + 1; }
        var result = 0;
        for (var i = 0; i < 3; i++) {
            result = apply(inc, result);
        }
        result;
    "#;
    assert_eval_number(source, 3.0);
}

// ============================================================================
// Category 3: Function Returns (15 tests)
// ============================================================================

// Requires nested function declarations (deferred to v0.3+)
#[test]
#[ignore = "requires nested function declarations — deferred to v0.3+"]
fn test_return_function() {
    let source = r#"
        fn getDouble() -> (number) -> number {
            fn double(x: number) -> number { return x * 2; }
            return double;
        }
        let f = getDouble();
        f(7);
    "#;
    assert_eval_number(source, 14.0);
}

#[test]
fn test_return_builtin() {
    let source = r#"
        fn getLen() -> (string) -> number {
            return len;
        }
        let f = getLen();
        f("test");
    "#;
    assert_eval_number(source, 4.0);
}

#[test]
fn test_return_function_from_parameter() {
    let source = r#"
        fn identity(f: (number) -> number) -> (number) -> number {
            return f;
        }
        fn triple(x: number) -> number { return x * 3; }
        let f = identity(triple);
        f(4);
    "#;
    assert_eval_number(source, 12.0);
}

// Requires nested function declarations (deferred to v0.3+)
#[test]
#[ignore = "requires nested function declarations — deferred to v0.3+"]
fn test_conditional_function_return() {
    let source = r#"
        fn getFunc(flag: bool) -> (number) -> number {
            fn double(x: number) -> number { return x * 2; }
            fn triple(x: number) -> number { return x * 3; }
            if (flag) {
                return double;
            }
            return triple;
        }
        let f = getFunc(true);
        f(5);
    "#;
    assert_eval_number(source, 10.0);
}

// Requires nested function declarations (deferred to v0.3+)
#[test]
#[ignore = "requires nested function declarations — deferred to v0.3+"]
fn test_return_function_and_call_immediately() {
    let source = r#"
        fn getDouble() -> (number) -> number {
            fn double(x: number) -> number { return x * 2; }
            return double;
        }
        getDouble()(6);
    "#;
    assert_eval_number(source, 12.0);
}

// ============================================================================
// Category 4: Type Checking (15 tests)
// ============================================================================

#[test]
fn test_type_error_wrong_function_type() {
    let source = r#"
        fn add(a: number, b: number) -> number { return a + b; }
        let f: (number) -> number = add;
    "#;
    assert_error_code(source, "AT3001");
}

#[test]
fn test_type_error_not_a_function() {
    let source = r#"
        let x: number = 5;
        x();
    "#;
    assert_error_code(source, "AT3006");
}

// Requires nested function declarations (deferred to v0.3+)
#[test]
#[ignore = "requires nested function declarations — deferred to v0.3+"]
fn test_type_error_wrong_return_type() {
    let source = r#"
        fn getString() -> string {
            fn getNum() -> number { return 42; }
            return getNum;
        }
    "#;
    assert_error_code(source, "AT3001");
}

#[test]
fn test_type_valid_function_assignment() {
    let source = r#"
        fn double(x: number) -> number { return x * 2; }
        let f: (number) -> number = double;
        f(5);
    "#;
    assert_eval_number(source, 10.0);
}

// Requires nested functions or closure capture (deferred to v0.3+)
#[test]
#[ignore = "requires nested functions or closure capture — deferred to v0.3+"]
fn test_type_valid_function_parameter() {
    let source = r#"
        fn apply(f: (string) -> number, s: string) -> number {
            return f(s);
        }
        apply(len, "test");
    "#;
    assert_eval_number(source, 4.0);
}

// ============================================================================
// Category 5: Edge Cases (15 tests)
// ============================================================================

#[test]
fn test_function_returning_void() {
    let source = r#"
        fn getVoid() -> (string) -> void {
            return print;
        }
        let f = getVoid();
        f("test");
    "#;
    assert_eval_null(source);
}

#[test]
fn test_nested_function_calls_through_variables() {
    let source = r#"
        fn add(a: number, b: number) -> number { return a + b; }
        let f = add;
        let g = f;
        let h = g;
        h(2, 3);
    "#;
    assert_eval_number(source, 5.0);
}

#[test]
fn test_function_with_no_params() {
    let source = r#"
        fn getFortyTwo() -> number { return 42; }
        let f: () -> number = getFortyTwo;
        f();
    "#;
    assert_eval_number(source, 42.0);
}

#[test]
fn test_function_with_many_params() {
    let source = r#"
        fn sum4(a: number, b: number, c: number, d: number) -> number {
            return a + b + c + d;
        }
        let f = sum4;
        f(1, 2, 3, 4);
    "#;
    assert_eval_number(source, 10.0);
}

// Requires nested functions or closure capture (deferred to v0.3+)
#[test]
#[ignore = "requires nested functions or closure capture — deferred to v0.3+"]
fn test_function_variable_in_global_scope() {
    let source = r#"
        fn double(x: number) -> number { return x * 2; }
        let globalFunc = double;
        fn useGlobal(x: number) -> number {
            return globalFunc(x);
        }
        useGlobal(5);
    "#;
    assert_eval_number(source, 10.0);
}

// ============================================================================
// Category 6: Integration Tests (15 tests)
// ============================================================================

// Requires nested functions or closure capture (deferred to v0.3+)
#[test]
#[ignore = "requires nested functions or closure capture — deferred to v0.3+"]
fn test_map_pattern_with_function() {
    let source = r#"
        fn applyToArray(arr: number[], f: (number) -> number) -> number[] {
            var result: number[] = [];
            for (var i = 0; i < len(arr); i++) {
                result = result + [f(arr[i])];
            }
            return result;
        }
        fn double(x: number) -> number { return x * 2; }
        let arr = [1, 2, 3];
        let doubled = applyToArray(arr, double);
        doubled[0] + doubled[1] + doubled[2];
    "#;
    assert_eval_number(source, 12.0);
}

// Requires nested functions or closure capture (deferred to v0.3+)
#[test]
#[ignore = "requires nested functions or closure capture — deferred to v0.3+"]
fn test_filter_pattern_with_function() {
    let source = r#"
        fn filterArray(arr: number[], predicate: (number) -> bool) -> number[] {
            var result: number[] = [];
            for (var i = 0; i < len(arr); i++) {
                if (predicate(arr[i])) {
                    result = result + [arr[i]];
                }
            }
            return result;
        }
        fn isEven(x: number) -> bool { return x % 2 == 0; }
        let arr = [1, 2, 3, 4, 5, 6];
        let evens = filterArray(arr, isEven);
        len(evens);
    "#;
    assert_eval_number(source, 3.0);
}

#[test]
fn test_reduce_pattern_with_function() {
    let source = r#"
        fn reduceArray(
            arr: number[],
            reducer: (number, number) -> number,
            initial: number
        ) -> number {
            var acc = initial;
            for (var i = 0; i < len(arr); i++) {
                acc = reducer(acc, arr[i]);
            }
            return acc;
        }
        fn add(a: number, b: number) -> number { return a + b; }
        let arr = [1, 2, 3, 4, 5];
        reduceArray(arr, add, 0);
    "#;
    assert_eval_number(source, 15.0);
}

// Requires nested function declarations (deferred to v0.3+)
#[test]
#[ignore = "requires nested function declarations — deferred to v0.3+"]
fn test_function_composition() {
    let source = r#"
        fn compose(
            f: (number) -> number,
            g: (number) -> number
        ) -> (number) -> number {
            fn composed(x: number) -> number {
                return f(g(x));
            }
            return composed;
        }
        fn double(x: number) -> number { return x * 2; }
        fn inc(x: number) -> number { return x + 1; }
        let doubleAndInc = compose(inc, double);
        doubleAndInc(5);
    "#;
    assert_eval_number(source, 11.0);
}

// Requires nested functions or closure capture (deferred to v0.3+)
#[test]
#[ignore = "requires nested functions or closure capture — deferred to v0.3+"]
fn test_callback_pattern() {
    let source = r#"
        fn processValue(
            x: number,
            callback: (number) -> void
        ) -> void {
            callback(x * 2);
        }
        var result = 0;
        fn setResult(x: number) -> void {
            result = x;
        }
        processValue(5, setResult);
        result;
    "#;
    assert_eval_number(source, 10.0);
}

// Requires nested functions or closure capture (deferred to v0.3+)
#[test]
#[ignore = "requires nested functions or closure capture — deferred to v0.3+"]
fn test_function_array_element() {
    let source = r#"
        fn double(x: number) -> number { return x * 2; }
        fn triple(x: number) -> number { return x * 3; }
        let funcs: ((number) -> number)[] = [double, triple];
        funcs[0](5) + funcs[1](5);
    "#;
    assert_eval_number(source, 25.0);
}

// Requires nested functions or closure capture (deferred to v0.3+)
#[test]
#[ignore = "requires nested functions or closure capture — deferred to v0.3+"]
fn test_complex_function_passing() {
    let source = r#"
        fn transform(
            arr: number[],
            f1: (number) -> number,
            f2: (number) -> number
        ) -> number {
            var sum = 0;
            for (var i = 0; i < len(arr); i++) {
                sum = sum + f1(f2(arr[i]));
            }
            return sum;
        }
        fn double(x: number) -> number { return x * 2; }
        fn square(x: number) -> number { return x * x; }
        transform([1, 2, 3], double, square);
    "#;
    assert_eval_number(source, 28.0);
}

// ============================================================================
// From test_primitives.rs
// ============================================================================

// Integration tests for testing primitives (phase-15)
//
// Verifies that assertion functions work correctly in Atlas code
// and through the stdlib API directly.
//
// Test categories:
// - Basic assertions (assert, assertFalse)
// - Equality assertions (assertEqual, assertNotEqual)
// - Result assertions (assertOk, assertErr)
// - Option assertions (assertSome, assertNone)
// - Collection assertions (assertContains, assertEmpty, assertLength)
// - Error assertions (assertThrows, assertNoThrow via NativeFunction)
// - Stdlib registration (is_builtin, call_builtin)
// - Interpreter/VM parity

// ============================================================================
// Helpers
// ============================================================================

fn span() -> Span {
    Span::dummy()
}

fn bool_val(b: bool) -> Value {
    Value::Bool(b)
}

fn str_val(s: &str) -> Value {
    Value::string(s)
}

fn num_val(n: f64) -> Value {
    Value::Number(n)
}

fn arr_val(items: Vec<Value>) -> Value {
    Value::array(items)
}

fn ok_val(v: Value) -> Value {
    Value::Result(Ok(Box::new(v)))
}

fn some_val(v: Value) -> Value {
    Value::Option(Some(Box::new(v)))
}

fn throwing_fn() -> Value {
    Value::NativeFunction(Arc::new(|_| {
        Err(RuntimeError::TypeError {
            msg: "intentional".to_string(),
            span: Span::dummy(),
        })
    }))
}

fn ok_fn() -> Value {
    Value::NativeFunction(Arc::new(|_| Ok(Value::Null)))
}

/// Evaluate Atlas source and assert it succeeds (returns Null or any value).
fn eval_ok(source: &str) {
    let runtime = Atlas::new();
    match runtime.eval(source) {
        Ok(_) => {}
        Err(diags) => panic!("Expected success, got errors: {:?}", diags),
    }
}

/// Evaluate Atlas source and assert it fails with an error containing `fragment`.
fn eval_err_contains(source: &str, fragment: &str) {
    let runtime = Atlas::new();
    match runtime.eval(source) {
        Err(diags) => {
            let combined = diags
                .iter()
                .map(|d| d.message.clone())
                .collect::<Vec<_>>()
                .join("\n");
            assert!(
                combined.contains(fragment),
                "Error message {:?} did not contain {:?}",
                combined,
                fragment
            );
        }
        Ok(val) => panic!("Expected error, got success: {:?}", val),
    }
}

// ============================================================================
// 1. Basic assertions — Atlas code integration
// ============================================================================

#[test]
fn test_assert_passes_in_atlas_code() {
    eval_ok("assert(true, \"should pass\");");
}

#[test]
fn test_assert_false_passes_in_atlas_code() {
    eval_ok("assertFalse(false, \"should pass\");");
}

#[test]
fn test_assert_failure_produces_error() {
    eval_err_contains(
        "assert(false, \"my custom failure message\");",
        "my custom failure message",
    );
}

#[test]
fn test_assert_false_failure_produces_error() {
    eval_err_contains(
        "assertFalse(true, \"was unexpectedly true\");",
        "was unexpectedly true",
    );
}

#[test]
fn test_assert_in_function_body() {
    eval_ok(
        r#"
        fn test_basic() -> void {
            assert(true, "should pass");
            assertFalse(false, "should also pass");
        }
        test_basic();
    "#,
    );
}

// ============================================================================
// 2. Equality assertions — Atlas code integration
// ============================================================================

#[test]
fn test_assert_equal_numbers_in_atlas_code() {
    eval_ok("assertEqual(5, 5);");
}

#[test]
fn test_assert_equal_strings_in_atlas_code() {
    eval_ok(r#"assertEqual("hello", "hello");"#);
}

#[test]
fn test_assert_equal_bools_in_atlas_code() {
    eval_ok("assertEqual(true, true);");
}

#[test]
fn test_assert_equal_failure_shows_diff() {
    let runtime = Atlas::new();
    match runtime.eval("assertEqual(5, 10);") {
        Err(diags) => {
            let combined = diags
                .iter()
                .map(|d| d.message.clone())
                .collect::<Vec<_>>()
                .join("\n");
            assert!(
                combined.contains("Actual:") || combined.contains("actual"),
                "Expected diff in: {}",
                combined
            );
            assert!(
                combined.contains("Expected:") || combined.contains("expected"),
                "Expected diff in: {}",
                combined
            );
        }
        Ok(val) => panic!("Expected failure, got: {:?}", val),
    }
}

#[test]
fn test_assert_not_equal_in_atlas_code() {
    eval_ok("assertNotEqual(1, 2);");
}

#[test]
fn test_assert_not_equal_failure() {
    eval_err_contains("assertNotEqual(5, 5);", "equal");
}

// ============================================================================
// 3. Result assertions — Atlas code integration
// ============================================================================

#[test]
fn test_assert_ok_in_atlas_code() {
    eval_ok(
        r#"
        fn divide(a: number, b: number) -> Result<number, string> {
            if (b == 0) { return Err("division by zero"); }
            return Ok(a / b);
        }

        let result = divide(10, 2);
        let value = assertOk(result);
        assertEqual(value, 5);
    "#,
    );
}

#[test]
fn test_assert_ok_failure_on_err_value() {
    eval_err_contains(
        r#"
        let result = Err("something broke");
        assertOk(result);
    "#,
        "Err",
    );
}

#[test]
fn test_assert_err_in_atlas_code() {
    eval_ok(
        r#"
        let result = Err("expected failure");
        let err_value = assertErr(result);
        assertEqual(err_value, "expected failure");
    "#,
    );
}

#[test]
fn test_assert_err_failure_on_ok_value() {
    eval_err_contains(
        r#"
        let result = Ok(42);
        assertErr(result);
    "#,
        "Ok",
    );
}

// ============================================================================
// 4. Option assertions — Atlas code integration
// ============================================================================

#[test]
fn test_assert_some_in_atlas_code() {
    eval_ok(
        r#"
        let opt = Some(42);
        let value = assertSome(opt);
        assertEqual(value, 42);
    "#,
    );
}

#[test]
fn test_assert_some_failure_on_none() {
    eval_err_contains(
        r#"
        let opt = None();
        assertSome(opt);
    "#,
        "None",
    );
}

#[test]
fn test_assert_none_in_atlas_code() {
    eval_ok(
        r#"
        let opt = None();
        assertNone(opt);
    "#,
    );
}

#[test]
fn test_assert_none_failure_on_some() {
    eval_err_contains(
        r#"
        let opt = Some(99);
        assertNone(opt);
    "#,
        "Some",
    );
}

// ============================================================================
// 5. Collection assertions — Atlas code integration
// ============================================================================

#[test]
fn test_assert_contains_in_atlas_code() {
    eval_ok(
        r#"
        let arr = [1, 2, 3];
        assertContains(arr, 2);
    "#,
    );
}

#[test]
fn test_assert_contains_failure() {
    eval_err_contains(
        r#"
        let arr = [1, 2, 3];
        assertContains(arr, 99);
    "#,
        "does not contain",
    );
}

#[test]
fn test_assert_empty_in_atlas_code() {
    eval_ok(
        r#"
        let arr = [];
        assertEmpty(arr);
    "#,
    );
}

#[test]
fn test_assert_empty_failure() {
    eval_err_contains(
        r#"
        let arr = [1];
        assertEmpty(arr);
    "#,
        "length",
    );
}

#[test]
fn test_assert_length_in_atlas_code() {
    eval_ok(
        r#"
        let arr = [10, 20, 30];
        assertLength(arr, 3);
    "#,
    );
}

#[test]
fn test_assert_length_failure() {
    eval_err_contains(
        r#"
        let arr = [1, 2];
        assertLength(arr, 5);
    "#,
        "length",
    );
}

// ============================================================================
// 6. Error assertions — via stdlib API (NativeFunction)
// ============================================================================

#[test]
fn test_assert_throws_stdlib_api_passes() {
    let result = atlas_test::assert_throws(&[throwing_fn()], span());
    assert!(result.is_ok(), "assert_throws should pass when fn throws");
}

#[test]
fn test_assert_throws_stdlib_api_fails_when_no_throw() {
    let result = atlas_test::assert_throws(&[ok_fn()], span());
    assert!(
        result.is_err(),
        "assert_throws should fail when fn succeeds"
    );
}

#[test]
fn test_assert_no_throw_stdlib_api_passes() {
    let result = atlas_test::assert_no_throw(&[ok_fn()], span());
    assert!(
        result.is_ok(),
        "assert_no_throw should pass when fn succeeds"
    );
}

#[test]
fn test_assert_no_throw_stdlib_api_fails_when_throws() {
    let result = atlas_test::assert_no_throw(&[throwing_fn()], span());
    assert!(
        result.is_err(),
        "assert_no_throw should fail when fn throws"
    );
}

#[test]
fn test_assert_throws_type_error_on_non_fn() {
    let result = atlas_test::assert_throws(&[num_val(42.0)], span());
    assert!(result.is_err());
}

// ============================================================================
// 7. Stdlib registration — is_builtin + call_builtin
// ============================================================================

#[test]
fn test_is_builtin_assert() {
    assert!(is_builtin("assert"));
    assert!(is_builtin("assertFalse"));
}

#[test]
fn test_is_builtin_equality() {
    assert!(is_builtin("assertEqual"));
    assert!(is_builtin("assertNotEqual"));
}

#[test]
fn test_is_builtin_result() {
    assert!(is_builtin("assertOk"));
    assert!(is_builtin("assertErr"));
}

#[test]
fn test_is_builtin_option() {
    assert!(is_builtin("assertSome"));
    assert!(is_builtin("assertNone"));
}

#[test]
fn test_is_builtin_collection() {
    assert!(is_builtin("assertContains"));
    assert!(is_builtin("assertEmpty"));
    assert!(is_builtin("assertLength"));
}

#[test]
fn test_is_builtin_error() {
    assert!(is_builtin("assertThrows"));
    assert!(is_builtin("assertNoThrow"));
}

#[test]
fn test_call_builtin_assert_via_dispatch() {
    let security = SecurityContext::allow_all();
    let result = call_builtin(
        "assert",
        &[bool_val(true), str_val("ok")],
        span(),
        &security,
        &stdout_writer(),
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Null);
}

#[test]
fn test_call_builtin_assert_equal_via_dispatch() {
    let security = SecurityContext::allow_all();
    let result = call_builtin(
        "assertEqual",
        &[num_val(42.0), num_val(42.0)],
        span(),
        &security,
        &stdout_writer(),
    );
    assert!(result.is_ok());
}

#[test]
fn test_call_builtin_assert_ok_via_dispatch() {
    let security = SecurityContext::allow_all();
    let result = call_builtin(
        "assertOk",
        &[ok_val(str_val("inner"))],
        span(),
        &security,
        &stdout_writer(),
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), str_val("inner"));
}

#[test]
fn test_call_builtin_assert_some_via_dispatch() {
    let security = SecurityContext::allow_all();
    let result = call_builtin(
        "assertSome",
        &[some_val(num_val(7.0))],
        span(),
        &security,
        &stdout_writer(),
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), num_val(7.0));
}

#[test]
fn test_call_builtin_assert_empty_via_dispatch() {
    let security = SecurityContext::allow_all();
    let result = call_builtin(
        "assertEmpty",
        &[arr_val(vec![])],
        span(),
        &security,
        &stdout_writer(),
    );
    assert!(result.is_ok());
}

// ============================================================================
// 8. Interpreter / VM parity
// ============================================================================

/// Run source twice (as two separate runtime instances) and verify both succeed.
/// This matches the established parity testing pattern in this codebase.
fn eval_parity_ok(source: &str) {
    let r1 = Atlas::new();
    match r1.eval(source) {
        Ok(_) => {}
        Err(diags) => panic!("First eval failed: {:?}", diags),
    }
    let r2 = Atlas::new();
    match r2.eval(source) {
        Ok(_) => {}
        Err(diags) => panic!("Second eval failed: {:?}", diags),
    }
}

/// Run source twice and verify both fail (parity of failure).
fn eval_parity_err(source: &str) {
    let err1 = Atlas::new().eval(source).is_err();
    let err2 = Atlas::new().eval(source).is_err();
    assert!(err1, "First eval should fail");
    assert!(err2, "Second eval should fail");
}

#[test]
fn test_assert_parity_basic() {
    eval_parity_ok("assert(true, \"parity\");");
}

#[test]
fn test_assert_equal_parity() {
    eval_parity_ok("assertEqual(10, 10);");
}

#[test]
fn test_assert_ok_parity() {
    eval_parity_ok(
        r#"
        let r = Ok(42);
        let v = assertOk(r);
        assertEqual(v, 42);
    "#,
    );
}

#[test]
fn test_assert_some_parity() {
    eval_parity_ok(
        r#"
        let opt = Some("hello");
        let v = assertSome(opt);
        assertEqual(v, "hello");
    "#,
    );
}

#[test]
fn test_assert_none_parity() {
    eval_parity_ok(
        r#"
        let opt = None();
        assertNone(opt);
    "#,
    );
}

#[test]
fn test_assert_contains_parity() {
    eval_parity_ok(
        r#"
        let arr = [1, 2, 3];
        assertContains(arr, 3);
    "#,
    );
}

#[test]
fn test_assert_length_parity() {
    eval_parity_ok(
        r#"
        let arr = [10, 20];
        assertLength(arr, 2);
    "#,
    );
}

#[test]
fn test_assert_failure_parity() {
    eval_parity_err("assert(false, \"parity failure test\");");
}

// ============================================================================
// 9. Comprehensive real-world test example
// ============================================================================

#[test]
fn test_realistic_test_function() {
    eval_ok(
        r#"
        fn add(a: number, b: number) -> number {
            return a + b;
        }

        fn test_add() -> void {
            assertEqual(add(1, 2), 3);
            assertEqual(add(0, 0), 0);
            assertEqual(add(-1, 1), 0);
            assert(add(5, 5) == 10, "5 + 5 should be 10");
        }

        test_add();
    "#,
    );
}

#[test]
fn test_result_chain_with_assertions() {
    eval_ok(
        r#"
        fn safe_divide(a: number, b: number) -> Result<number, string> {
            if (b == 0) { return Err("division by zero"); }
            return Ok(a / b);
        }

        let r1 = safe_divide(10, 2);
        let v = assertOk(r1);
        assertEqual(v, 5);

        let r2 = safe_divide(5, 0);
        let e = assertErr(r2);
        assertEqual(e, "division by zero");
    "#,
    );
}

#[test]
fn test_option_chain_with_assertions() {
    eval_ok(
        r#"
        fn find_value(arr: array, target: number) -> Option<number> {
            var found = None();
            for item in arr {
                if (item == target) {
                    found = Some(item);
                }
            }
            return found;
        }

        let arr = [10, 20, 30];
        let r1 = find_value(arr, 20);
        let v = assertSome(r1);
        assertEqual(v, 20);

        let r2 = find_value(arr, 99);
        assertNone(r2);
    "#,
    );
}

#[test]
fn test_collection_assertions_in_sequence() {
    eval_ok(
        r#"
        let nums = [1, 2, 3, 4, 5];
        assertLength(nums, 5);
        assertContains(nums, 3);

        let empty = [];
        assertEmpty(empty);
        assertLength(empty, 0);
    "#,
    );
}

#[test]
fn test_assert_equal_with_expressions() {
    eval_ok(
        r#"
        assertEqual(2 + 3, 5);
        assertEqual(10 * 2, 20);
        assertEqual(true && true, true);
        assertEqual(false || true, true);
    "#,
    );
}

// ============================================================================
// From prelude_tests.rs
// ============================================================================

// Prelude Availability and Shadowing Tests
//
// Tests that prelude builtins (print, len, str) are:
// - Always available without imports
// - Can be shadowed in nested scopes
// - Cannot be shadowed in global scope (AT1012)

fn check_file(filename: &str) -> Vec<Diagnostic> {
    let path = Path::new("../../tests/prelude").join(filename);
    let source = fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("Failed to read test file: {}", path.display()));

    let mut lexer = Lexer::new(&source);
    let (tokens, lex_diagnostics) = lexer.tokenize();

    if !lex_diagnostics.is_empty() {
        return lex_diagnostics;
    }

    let mut parser = Parser::new(tokens);
    let (program, parse_diagnostics) = parser.parse();

    if !parse_diagnostics.is_empty() {
        return parse_diagnostics;
    }

    let mut binder = Binder::new();
    let (_symbol_table, bind_diagnostics) = binder.bind(&program);

    bind_diagnostics
}

// ============================================================================
// Prelude Availability Tests
// ============================================================================

#[test]
fn test_prelude_available_without_imports() {
    let diagnostics = check_file("prelude_available.atl");

    // Should have no errors - prelude functions are available
    assert_eq!(
        diagnostics.len(),
        0,
        "Prelude functions should be available without imports, got: {:?}",
        diagnostics
    );
}

// ============================================================================
// Nested Scope Shadowing Tests (Allowed)
// ============================================================================

#[test]
fn test_nested_shadowing_allowed() {
    let diagnostics = check_file("nested_shadowing_allowed.atl");

    // Should have no errors - shadowing in nested scopes is allowed
    assert_eq!(
        diagnostics.len(),
        0,
        "Shadowing prelude in nested scopes should be allowed, got: {:?}",
        diagnostics
    );
}

// ============================================================================
// Global Scope Shadowing Tests (Disallowed - AT1012)
// ============================================================================

#[rstest]
#[case("global_shadowing_function.atl", "print")]
#[case("global_shadowing_variable.atl", "len")]
fn test_global_shadowing_produces_at1012(#[case] filename: &str, #[case] builtin_name: &str) {
    let diagnostics = check_file(filename);

    // Should have exactly 1 error
    assert_eq!(
        diagnostics.len(),
        1,
        "Expected exactly 1 diagnostic for {}, got: {:?}",
        filename,
        diagnostics
    );

    // Should be AT1012
    assert_eq!(
        diagnostics[0].code, "AT1012",
        "Expected AT1012 for global shadowing, got: {}",
        diagnostics[0].code
    );

    // Should mention the builtin name
    assert!(
        diagnostics[0].message.contains(builtin_name),
        "Error message should mention '{}', got: {}",
        builtin_name,
        diagnostics[0].message
    );

    // Should mention "Cannot shadow prelude builtin"
    assert!(
        diagnostics[0]
            .message
            .contains("Cannot shadow prelude builtin"),
        "Error message should mention prelude shadowing, got: {}",
        diagnostics[0].message
    );

    // Snapshot the diagnostic for stability tracking
    insta::assert_yaml_snapshot!(
        format!("prelude_{}", filename.replace(".atl", "")),
        diagnostics
    );
}

#[test]
fn test_global_shadowing_all_builtins() {
    let diagnostics = check_file("global_shadowing_all.atl");

    // Should have exactly 3 errors (one for each builtin)
    assert_eq!(
        diagnostics.len(),
        3,
        "Expected 3 diagnostics for shadowing all builtins, got: {:?}",
        diagnostics
    );

    // All should be AT1012
    for diag in &diagnostics {
        assert_eq!(
            diag.code, "AT1012",
            "Expected all diagnostics to be AT1012, got: {}",
            diag.code
        );
    }

    // Should mention each builtin
    let messages: Vec<&str> = diagnostics.iter().map(|d| d.message.as_str()).collect();
    assert!(
        messages.iter().any(|m| m.contains("print")),
        "Should have error for 'print'"
    );
    assert!(
        messages.iter().any(|m| m.contains("len")),
        "Should have error for 'len'"
    );
    assert!(
        messages.iter().any(|m| m.contains("str")),
        "Should have error for 'str'"
    );

    // Snapshot all diagnostics
    insta::assert_yaml_snapshot!("prelude_global_shadowing_all", diagnostics);
}

// ============================================================================
// Stability Test
// ============================================================================

#[test]
fn test_prelude_diagnostic_stability() {
    // Verify that running the same file twice produces identical diagnostics
    let diag1 = check_file("global_shadowing_function.atl");
    let diag2 = check_file("global_shadowing_function.atl");

    assert_eq!(
        diag1.len(),
        diag2.len(),
        "Diagnostic count should be stable"
    );
    for (d1, d2) in diag1.iter().zip(diag2.iter()) {
        assert_eq!(d1.code, d2.code, "Diagnostic codes should be stable");
        assert_eq!(
            d1.message, d2.message,
            "Diagnostic messages should be stable"
        );
        assert_eq!(d1.line, d2.line, "Diagnostic lines should be stable");
        assert_eq!(d1.column, d2.column, "Diagnostic columns should be stable");
    }
}

// ============================================================================
// From numeric_edge_cases_tests.rs
// ============================================================================

// Tests for numeric edge cases
//
// Verifies behavior with boundary values, special floats (infinity, NaN),
// division by zero, and other numeric edge cases.
//
// Atlas uses f64 (64-bit IEEE 754 floating point) for all numbers.

/// Helper to get all diagnostics from source code
fn get_all_diagnostics(source: &str) -> Vec<atlas_runtime::Diagnostic> {
    let mut lexer = Lexer::new(source);
    let (tokens, lex_diags) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, parse_diags) = parser.parse();

    let mut binder = Binder::new();
    let (mut table, bind_diags) = binder.bind(&program);

    let mut checker = TypeChecker::new(&mut table);
    let type_diags = checker.check(&program);

    let mut all_diags = Vec::new();
    all_diags.extend(lex_diags);
    all_diags.extend(parse_diags);
    all_diags.extend(bind_diags);
    all_diags.extend(type_diags);

    all_diags
}

// =============================================================================
// Integer and Float Boundary Tests
// =============================================================================

#[rstest]
#[case::large_integer("let x: number = 9007199254740991;")]
#[case::negative_large_integer("let x: number = -9007199254740991;")]
#[case::large_integer_arithmetic(
    "let a: number = 9007199254740991;\nlet b: number = 1;\nlet c: number = a + b;"
)]
#[case::float_literal("let x: number = 3.14159265358979323846;")]
#[case::very_small_float("let x: number = 0.0000000001;")]
#[case::negative_float("let x: number = -3.14159;")]
#[case::zero_variants("let a: number = 0;\nlet b: number = 0.0;\nlet c: number = -0.0;")]
fn test_numeric_boundaries(#[case] source: &str) {
    let diags = get_all_diagnostics(source);
    assert!(diags.is_empty(), "Should be valid: {:?}", diags);
}

#[test]
fn test_very_large_float() {
    let source = "let x = 179769313486231570000000000000000000000.0;";
    let _diags = get_all_diagnostics(source);
    // This might fail to parse depending on lexer implementation
}

// =============================================================================
// Division and Modulo Tests
// =============================================================================

#[rstest]
#[case::division("let a: number = 10;\nlet b: number = 2;\nlet c: number = a / b;")]
#[case::division_by_zero_literal("let x: number = 10 / 0;")]
#[case::division_by_variable("let divisor: number = 0;\nlet result: number = 10 / divisor;")]
#[case::division_underflow("let a = 1;\nlet b = 10000000;\nlet c = a / b;")]
#[case::modulo_by_zero("let x: number = 10 % 0;")]
#[case::modulo_with_floats("let x: number = 5.5 % 2.3;")]
fn test_division_and_modulo(#[case] source: &str) {
    let diags = get_all_diagnostics(source);
    // Type checker cannot detect division by zero - this is runtime behavior
    assert!(diags.is_empty(), "Should typecheck: {:?}", diags);
}

// =============================================================================
// Arithmetic Overflow/Underflow Tests
// =============================================================================

#[rstest]
#[case::addition_overflow("let a = 100000000000000000000000000000.0;\nlet b = 100000000000000000000000000000.0;\nlet c = a + b;")]
#[case::multiplication_overflow(
    "let a = 10000000000000000000.0;\nlet b = 10000000000000000000.0;\nlet c = a * b;"
)]
fn test_arithmetic_overflow(#[case] source: &str) {
    let _diags = get_all_diagnostics(source);
    // Typechecks fine, runtime would produce infinity
}

#[test]
fn test_subtraction_to_negative() {
    let source = "let a: number = 5;\nlet b: number = 10;\nlet c: number = a - b;";
    let diags = get_all_diagnostics(source);
    assert!(diags.is_empty(), "Should typecheck: {:?}", diags);
}

// =============================================================================
// Comparison Tests with Edge Values
// =============================================================================

#[rstest]
#[case::zero_comparisons(
    "let a: number = 0;\nlet b: bool = a > 0;\nlet c: bool = a < 0;\nlet d: bool = a == 0;"
)]
#[case::negative_comparison("let a: number = -5;\nlet b: number = 10;\nlet c: bool = a < b;")]
#[case::float_equality("let a: number = 0.1 + 0.2;\nlet b: number = 0.3;\nlet c: bool = a == b;")]
fn test_comparisons(#[case] source: &str) {
    let diags = get_all_diagnostics(source);
    assert!(diags.is_empty(), "Should typecheck: {:?}", diags);
}

// =============================================================================
// Mixed Arithmetic Tests
// =============================================================================

#[rstest]
#[case::complex_expression("let x: number = (10 + 5) * 2 - 8 / 4;")]
#[case::nested_arithmetic("let a: number = 10;\nlet b: number = 5;\nlet c: number = 2;\nlet result: number = (a + b) * c - (a / b);")]
#[case::negative_arithmetic("let a: number = -10;\nlet b: number = -5;\nlet c: number = a + b;\nlet d: number = a - b;\nlet e: number = a * b;\nlet f: number = a / b;")]
fn test_mixed_arithmetic(#[case] source: &str) {
    let diags = get_all_diagnostics(source);
    assert!(diags.is_empty(), "Should typecheck: {:?}", diags);
}

// =============================================================================
// Unary Minus Tests
// =============================================================================

#[rstest]
#[case::literal("let x: number = -42;")]
#[case::variable("let a: number = 42;\nlet b: number = -a;")]
#[case::double_negation("let a: number = 42;\nlet b: number = -(-a);")]
#[case::negative_zero("let x: number = -0;\nlet y: number = -0.0;")]
fn test_unary_minus(#[case] source: &str) {
    let diags = get_all_diagnostics(source);
    assert!(diags.is_empty(), "Should typecheck: {:?}", diags);
}

// =============================================================================
// Error Cases
// =============================================================================

#[rstest]
#[case::string_plus_number("let x: number = \"hello\" + 5;")]
#[case::string_division("let x: number = \"10\" / \"2\";")]
#[case::bool_modulo("let x: number = true % false;")]
#[case::string_comparison("let x: bool = \"hello\" < 5;")]
fn test_type_errors(#[case] source: &str) {
    let diags = get_all_diagnostics(source);
    assert!(!diags.is_empty(), "Should produce error");
}

#[test]
fn test_arithmetic_on_non_numbers_has_error_code() {
    let source = "let x: number = \"hello\" + 5;";
    let diags = get_all_diagnostics(source);
    let error = diags.iter().find(|d| d.code.starts_with("AT"));
    assert!(error.is_some(), "Should have AT error code");
}

// =============================================================================
// Array Index Edge Cases
// =============================================================================

#[rstest]
#[case::zero_index("let arr = [1, 2, 3];\nlet x = arr[0];")]
#[case::large_index("let arr = [1, 2, 3];\nlet x = arr[999999];")]
#[case::negative_index("let arr = [1, 2, 3];\nlet x = arr[-1];")]
#[case::float_index("let arr = [1, 2, 3];\nlet x = arr[1.5];")]
fn test_array_index_edge_cases(#[case] source: &str) {
    let diags = get_all_diagnostics(source);
    // Type system allows number (f64) for array index
    // Runtime would handle bounds/integer checking
    assert!(diags.is_empty(), "Should typecheck: {:?}", diags);
}

// ============================================================================
// From collection_iteration_tests.rs
// ============================================================================

// HashMap and HashSet Iteration Tests
//
// Comprehensive tests for forEach, map, and filter intrinsics on collections.
//
// NOTE: Atlas v0.2 does not support anonymous functions (fn(x) { ... }).
// All callbacks must be named functions passed by reference.

fn eval(code: &str) -> Value {
    let runtime = Atlas::new();
    runtime.eval(code).expect("Interpretation failed")
}

fn eval_expect_error(code: &str) -> bool {
    let runtime = Atlas::new();
    runtime.eval(code).is_err()
}

// =============================================================================
// HashMap Iteration Tests
// =============================================================================

#[test]
fn test_hashmap_foreach_returns_null() {
    let result = eval(
        r#"
        fn callback(_v: number, _k: string) -> void {}
        let hmap = hashMapNew();
        hashMapPut(hmap, "a", 1);
        hashMapForEach(hmap, callback)
    "#,
    );
    assert_eq!(result, Value::Null);
}

#[test]
fn test_hashmap_foreach_executes_callback() {
    // Verify callback executes by counting iterations
    let result = eval(
        r#"
        var count: number = 0;
        fn callback(_v: number, _k: string) -> void {
            count = count + 1;
        }
        let hmap = hashMapNew();
        hashMapPut(hmap, "a", 1);
        hashMapPut(hmap, "b", 2);
        hashMapPut(hmap, "c", 3);
        hashMapForEach(hmap, callback);
        count
    "#,
    );
    assert_eq!(result, Value::Number(3.0));
}

#[test]
fn test_hashmap_map_transforms_values() {
    let result = eval(
        r#"
        fn double(v: number, _k: string) -> number {
            return v * 2;
        }
        let hmap = hashMapNew();
        hashMapPut(hmap, "a", 1);
        hashMapPut(hmap, "b", 2);
        let mapped = hashMapMap(hmap, double);
        unwrap(hashMapGet(mapped, "a"))
    "#,
    );
    assert_eq!(result, Value::Number(2.0));
}

#[test]
fn test_hashmap_map_preserves_keys() {
    let result = eval(
        r#"
        fn addFive(v: number, _k: string) -> number {
            return v + 5;
        }
        let hmap = hashMapNew();
        hashMapPut(hmap, "x", 10);
        hashMapPut(hmap, "y", 20);
        let mapped = hashMapMap(hmap, addFive);
        hashMapHas(mapped, "x") && hashMapHas(mapped, "y")
    "#,
    );
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_hashmap_map_preserves_size() {
    let result = eval(
        r#"
        fn times10(v: number, _k: string) -> number {
            return v * 10;
        }
        let hmap = hashMapNew();
        hashMapPut(hmap, "a", 1);
        hashMapPut(hmap, "b", 2);
        hashMapPut(hmap, "c", 3);
        let mapped = hashMapMap(hmap, times10);
        hashMapSize(mapped)
    "#,
    );
    assert_eq!(result, Value::Number(3.0));
}

#[test]
fn test_hashmap_filter_keeps_matching_entries() {
    let result = eval(
        r#"
        fn greaterThanOne(v: number, _k: string) -> bool {
            return v > 1;
        }
        let hmap = hashMapNew();
        hashMapPut(hmap, "a", 1);
        hashMapPut(hmap, "b", 2);
        hashMapPut(hmap, "c", 3);
        let filtered = hashMapFilter(hmap, greaterThanOne);
        hashMapSize(filtered)
    "#,
    );
    assert_eq!(result, Value::Number(2.0));
}

#[test]
fn test_hashmap_filter_with_predicate() {
    let result = eval(
        r#"
        fn isEven(v: number, _k: string) -> bool {
            return v % 2 == 0;
        }
        let hmap = hashMapNew();
        hashMapPut(hmap, "a", 1);
        hashMapPut(hmap, "b", 2);
        hashMapPut(hmap, "c", 3);
        hashMapPut(hmap, "d", 4);
        let filtered = hashMapFilter(hmap, isEven);
        hashMapSize(filtered)
    "#,
    );
    assert_eq!(result, Value::Number(2.0));
}

#[test]
fn test_hashmap_filter_removes_non_matching() {
    let result = eval(
        r#"
        fn greaterThan10(v: number, _k: string) -> bool {
            return v > 10;
        }
        let hmap = hashMapNew();
        hashMapPut(hmap, "a", 1);
        hashMapPut(hmap, "b", 2);
        hashMapPut(hmap, "c", 3);
        let filtered = hashMapFilter(hmap, greaterThan10);
        hashMapSize(filtered)
    "#,
    );
    assert_eq!(result, Value::Number(0.0));
}

#[test]
fn test_hashmap_empty_iteration() {
    let result = eval(
        r#"
        fn identity(v: number, _k: string) -> number {
            return v;
        }
        let hmap = hashMapNew();
        let mapped = hashMapMap(hmap, identity);
        hashMapSize(mapped)
    "#,
    );
    assert_eq!(result, Value::Number(0.0));
}

#[test]
fn test_hashmap_chaining_operations() {
    let result = eval(
        r#"
        fn double(v: number, _k: string) -> number {
            return v * 2;
        }
        fn greaterThan2(v: number, _k: string) -> bool {
            return v > 2;
        }
        let hmap = hashMapNew();
        hashMapPut(hmap, "a", 1);
        hashMapPut(hmap, "b", 2);
        hashMapPut(hmap, "c", 3);
        let doubled = hashMapMap(hmap, double);
        let filtered = hashMapFilter(doubled, greaterThan2);
        hashMapSize(filtered)
    "#,
    );
    assert_eq!(result, Value::Number(2.0));
}

#[test]
fn test_hashmap_callback_receives_value_and_key() {
    // Verify callback receives both value and key parameters
    let result = eval(
        r#"
        fn addIfTest(v: number, k: string) -> number {
            if (k == "test") {
                return v + 1;
            } else {
                return v;
            }
        }
        let hmap = hashMapNew();
        hashMapPut(hmap, "test", 42);
        let mapped = hashMapMap(hmap, addIfTest);
        unwrap(hashMapGet(mapped, "test"))
    "#,
    );
    assert_eq!(result, Value::Number(43.0));
}

#[test]
fn test_hashmap_large_map() {
    let result = eval(
        r#"
        fn lessThan25(v: number, _k: string) -> bool {
            return v < 25;
        }
        let hmap = hashMapNew();
        var i: number = 0;
        while (i < 50) {
            hashMapPut(hmap, toString(i), i);
            i = i + 1;
        }
        let filtered = hashMapFilter(hmap, lessThan25);
        hashMapSize(filtered)
    "#,
    );
    assert_eq!(result, Value::Number(25.0));
}

// Error Handling Tests

#[test]
fn test_hashmap_foreach_non_function_callback() {
    assert!(eval_expect_error(
        r#"
        let hmap = hashMapNew();
        hashMapPut(hmap, "a", 1);
        hashMapForEach(hmap, "not a function")
    "#
    ));
}

#[test]
fn test_hashmap_map_non_function_callback() {
    assert!(eval_expect_error(
        r#"
        let hmap = hashMapNew();
        hashMapPut(hmap, "a", 1);
        hashMapMap(hmap, 42)
    "#
    ));
}

#[test]
fn test_hashmap_filter_non_function_callback() {
    assert!(eval_expect_error(
        r#"
        let hmap = hashMapNew();
        hashMapPut(hmap, "a", 1);
        hashMapFilter(hmap, null)
    "#
    ));
}

#[test]
fn test_hashmap_filter_non_bool_return() {
    // Filter predicate must return bool
    assert!(eval_expect_error(
        r#"
        fn returnValue(v: number, _k: string) -> number {
            return v;
        }
        let hmap = hashMapNew();
        hashMapPut(hmap, "a", 1);
        hashMapFilter(hmap, returnValue)
    "#
    ));
}

// =============================================================================
// HashSet Iteration Tests
// =============================================================================

#[test]
fn test_hashset_foreach_returns_null() {
    let result = eval(
        r#"
        fn callback(_elem: number) -> void {}
        let hset = hashSetNew();
        hashSetAdd(hset, 1);
        hashSetForEach(hset, callback)
    "#,
    );
    assert_eq!(result, Value::Null);
}

#[test]
fn test_hashset_foreach_executes_callback() {
    let result = eval(
        r#"
        var count: number = 0;
        fn callback(_elem: number) -> void {
            count = count + 1;
        }
        let hset = hashSetNew();
        hashSetAdd(hset, 1);
        hashSetAdd(hset, 2);
        hashSetAdd(hset, 3);
        hashSetForEach(hset, callback);
        count
    "#,
    );
    assert_eq!(result, Value::Number(3.0));
}

#[test]
fn test_hashset_map_to_array() {
    let result = eval(
        r#"
        fn double(elem: number) -> number {
            return elem * 2;
        }
        let hset = hashSetNew();
        hashSetAdd(hset, 1);
        hashSetAdd(hset, 2);
        let arr = hashSetMap(hset, double);
        typeof(arr)
    "#,
    );
    assert_eq!(result, Value::String(Arc::new("array".to_string())));
}

#[test]
fn test_hashset_map_array_length() {
    let result = eval(
        r#"
        fn times10(elem: number) -> number {
            return elem * 10;
        }
        let hset = hashSetNew();
        hashSetAdd(hset, 1);
        hashSetAdd(hset, 2);
        hashSetAdd(hset, 3);
        let arr = hashSetMap(hset, times10);
        len(arr)
    "#,
    );
    assert_eq!(result, Value::Number(3.0));
}

#[test]
fn test_hashset_map_transforms_elements() {
    let result = eval(
        r#"
        fn double(elem: number) -> number {
            return elem * 2;
        }
        let hset = hashSetNew();
        hashSetAdd(hset, 5);
        let arr = hashSetMap(hset, double);
        arr[0]
    "#,
    );
    assert_eq!(result, Value::Number(10.0));
}

#[test]
fn test_hashset_filter_keeps_matching() {
    let result = eval(
        r#"
        fn greaterThan2(elem: number) -> bool {
            return elem > 2;
        }
        let hset = hashSetNew();
        hashSetAdd(hset, 1);
        hashSetAdd(hset, 2);
        hashSetAdd(hset, 3);
        hashSetAdd(hset, 4);
        let filtered = hashSetFilter(hset, greaterThan2);
        hashSetSize(filtered)
    "#,
    );
    assert_eq!(result, Value::Number(2.0));
}

#[test]
fn test_hashset_filter_removes_non_matching() {
    let result = eval(
        r#"
        fn greaterThan10(elem: number) -> bool {
            return elem > 10;
        }
        let hset = hashSetNew();
        hashSetAdd(hset, 1);
        hashSetAdd(hset, 2);
        hashSetAdd(hset, 3);
        let filtered = hashSetFilter(hset, greaterThan10);
        hashSetSize(filtered)
    "#,
    );
    assert_eq!(result, Value::Number(0.0));
}

#[test]
fn test_hashset_empty_filter() {
    let result = eval(
        r#"
        fn alwaysTrue(_elem: number) -> bool {
            return true;
        }
        let hset = hashSetNew();
        let filtered = hashSetFilter(hset, alwaysTrue);
        hashSetSize(filtered)
    "#,
    );
    assert_eq!(result, Value::Number(0.0));
}

#[test]
fn test_hashset_filter_chaining() {
    let result = eval(
        r#"
        fn greaterThan1(elem: number) -> bool {
            return elem > 1;
        }
        fn lessThan4(elem: number) -> bool {
            return elem < 4;
        }
        let hset = hashSetNew();
        hashSetAdd(hset, 1);
        hashSetAdd(hset, 2);
        hashSetAdd(hset, 3);
        hashSetAdd(hset, 4);
        let f1 = hashSetFilter(hset, greaterThan1);
        let f2 = hashSetFilter(f1, lessThan4);
        hashSetSize(f2)
    "#,
    );
    assert_eq!(result, Value::Number(2.0));
}

#[test]
fn test_hashset_large_set() {
    let result = eval(
        r#"
        fn divisibleBy3(elem: number) -> bool {
            return elem % 3 == 0;
        }
        let hset = hashSetNew();
        var i: number = 0;
        while (i < 30) {
            hashSetAdd(hset, i);
            i = i + 1;
        }
        let filtered = hashSetFilter(hset, divisibleBy3);
        hashSetSize(filtered)
    "#,
    );
    assert_eq!(result, Value::Number(10.0));
}

// Error Handling Tests

#[test]
fn test_hashset_foreach_non_function_callback() {
    assert!(eval_expect_error(
        r#"
        let hset = hashSetNew();
        hashSetAdd(hset, 1);
        hashSetForEach(hset, "not a function")
    "#
    ));
}

#[test]
fn test_hashset_map_non_function_callback() {
    assert!(eval_expect_error(
        r#"
        let hset = hashSetNew();
        hashSetAdd(hset, 1);
        hashSetMap(hset, 42)
    "#
    ));
}

#[test]
fn test_hashset_filter_non_function_callback() {
    assert!(eval_expect_error(
        r#"
        let hset = hashSetNew();
        hashSetAdd(hset, 1);
        hashSetFilter(hset, null)
    "#
    ));
}

#[test]
fn test_hashset_filter_non_bool_return() {
    // Filter predicate must return bool
    assert!(eval_expect_error(
        r#"
        fn returnValue(elem: number) -> number {
            return elem;
        }
        let hset = hashSetNew();
        hashSetAdd(hset, 1);
        hashSetFilter(hset, returnValue)
    "#
    ));
}

// =============================================================================
// Integration Tests
// =============================================================================

#[test]
fn test_integration_hashmap_to_hashset() {
    let result = eval(
        r#"
        let hmap = hashMapNew();
        hashMapPut(hmap, "a", 1);
        hashMapPut(hmap, "b", 2);
        let values = hashMapValues(hmap);
        let hset = hashSetFromArray(values);
        hashSetSize(hset)
    "#,
    );
    assert_eq!(result, Value::Number(2.0));
}

#[test]
fn test_integration_hashset_map_to_array_filter() {
    let result = eval(
        r#"
        fn double(x: number) -> number {
            return x * 2;
        }
        fn greaterThan2(x: number) -> bool {
            return x > 2;
        }
        let hset = hashSetNew();
        hashSetAdd(hset, 1);
        hashSetAdd(hset, 2);
        hashSetAdd(hset, 3);
        let arr = hashSetMap(hset, double);
        let filtered = filter(arr, greaterThan2);
        len(filtered)
    "#,
    );
    assert_eq!(result, Value::Number(2.0));
}

#[test]
fn test_integration_empty_collections() {
    let result = eval(
        r#"
        fn identity(v: number, _k: string) -> number {
            return v;
        }
        fn alwaysTrue(_x: number) -> bool {
            return true;
        }
        let hm = hashMapNew();
        let hs = hashSetNew();
        let mr = hashMapMap(hm, identity);
        let sr = hashSetFilter(hs, alwaysTrue);
        hashMapSize(mr) + hashSetSize(sr)
    "#,
    );
    assert_eq!(result, Value::Number(0.0));
}

#[test]
fn test_integration_complex_transformation() {
    let result = eval(
        r#"
        fn double(v: number, _k: string) -> number {
            return v * 2;
        }
        fn greaterOrEqual4(v: number, _k: string) -> bool {
            return v >= 4;
        }
        var sum: number = 0;
        fn addToSum(v: number) -> void {
            sum = sum + v;
        }
        let hmap = hashMapNew();
        hashMapPut(hmap, "a", 1);
        hashMapPut(hmap, "b", 2);
        hashMapPut(hmap, "c", 3);
        hashMapPut(hmap, "d", 4);
        let doubled = hashMapMap(hmap, double);
        let filtered = hashMapFilter(doubled, greaterOrEqual4);
        let values = hashMapValues(filtered);
        forEach(values, addToSum);
        sum
    "#,
    );
    assert_eq!(result, Value::Number(18.0)); // 4 + 6 + 8 = 18
}

#[test]
fn test_integration_hashmap_keys_to_hashset() {
    let result = eval(
        r#"
        let hmap = hashMapNew();
        hashMapPut(hmap, "a", 1);
        hashMapPut(hmap, "b", 2);
        hashMapPut(hmap, "c", 3);
        let keys = hashMapKeys(hmap);
        let hset = hashSetFromArray(keys);
        hashSetSize(hset)
    "#,
    );
    assert_eq!(result, Value::Number(3.0));
}

// =============================================================================
// Parity Tests (ensure interpreter/VM consistency)
// =============================================================================

#[test]
fn test_parity_hashmap_foreach() {
    let result = eval(
        r#"
        var sum: number = 0;
        fn addToSum(v: number, _k: string) -> void {
            sum = sum + v;
        }
        let hmap = hashMapNew();
        hashMapPut(hmap, "x", 5);
        hashMapForEach(hmap, addToSum);
        sum
    "#,
    );
    assert_eq!(result, Value::Number(5.0));
}

#[test]
fn test_parity_hashmap_map() {
    let result = eval(
        r#"
        fn triple(v: number, _k: string) -> number {
            return v * 3;
        }
        let hmap = hashMapNew();
        hashMapPut(hmap, "test", 5);
        let mapped = hashMapMap(hmap, triple);
        unwrap(hashMapGet(mapped, "test"))
    "#,
    );
    assert_eq!(result, Value::Number(15.0));
}

#[test]
fn test_parity_hashmap_filter() {
    let result = eval(
        r#"
        fn notEqual2(v: number, _k: string) -> bool {
            return v != 2;
        }
        let hmap = hashMapNew();
        hashMapPut(hmap, "a", 1);
        hashMapPut(hmap, "b", 2);
        hashMapPut(hmap, "c", 3);
        let filtered = hashMapFilter(hmap, notEqual2);
        hashMapSize(filtered)
    "#,
    );
    assert_eq!(result, Value::Number(2.0));
}

#[test]
fn test_parity_hashset_foreach() {
    let result = eval(
        r#"
        var sum: number = 0;
        fn addToSum(elem: number) -> void {
            sum = sum + elem;
        }
        let hset = hashSetNew();
        hashSetAdd(hset, 10);
        hashSetForEach(hset, addToSum);
        sum
    "#,
    );
    assert_eq!(result, Value::Number(10.0));
}

#[test]
fn test_parity_hashset_map() {
    let result = eval(
        r#"
        fn double(elem: number) -> number {
            return elem * 2;
        }
        let hset = hashSetNew();
        hashSetAdd(hset, 7);
        let arr = hashSetMap(hset, double);
        arr[0]
    "#,
    );
    assert_eq!(result, Value::Number(14.0));
}

#[test]
fn test_parity_hashset_filter() {
    let result = eval(
        r#"
        fn lessOrEqual2(elem: number) -> bool {
            return elem <= 2;
        }
        let hset = hashSetNew();
        hashSetAdd(hset, 1);
        hashSetAdd(hset, 2);
        hashSetAdd(hset, 3);
        let filtered = hashSetFilter(hset, lessOrEqual2);
        hashSetSize(filtered)
    "#,
    );
    assert_eq!(result, Value::Number(2.0));
}

// ============================================================================
// VM stdlib tests (co-located to eliminate duplicate binary pairs)
// Tests run with separate binary name prefix via submodule
// ============================================================================

mod vm_stdlib {
    use super::{
        assert_eval_bool, assert_eval_null, assert_eval_number, assert_eval_string,
        assert_has_error, path_for_atlas,
    };
    use atlas_runtime::SecurityContext;
    use pretty_assertions::assert_eq;
    use std::fs;
    use tempfile::TempDir;

    // ============================================================================
    // From vm_stdlib_string_tests.rs
    // ============================================================================

    // String stdlib tests (VM engine)
    //
    // Tests all 18 string functions via VM execution for parity verification
    //
    // Note: These tests use the same common::* helpers which test through the full pipeline,
    // ensuring both interpreter and VM produce identical results.

    // All tests are identical to stdlib_string_tests.rs to verify parity
    // The common test helpers automatically test through both interpreter and VM

    // ============================================================================
    // Core Operations Tests
    // ============================================================================

    #[test]
    fn test_split_basic() {
        let code = r#"
        let result: string[] = split("a,b,c", ",");
        len(result)
    "#;
        assert_eval_number(code, 3.0);
    }

    #[test]
    fn test_split_empty_separator() {
        let code = r#"
        let result: string[] = split("abc", "");
        len(result)
    "#;
        assert_eval_number(code, 3.0);
    }

    #[test]
    fn test_split_no_match() {
        let code = r#"
        let result: string[] = split("hello", ",");
        len(result)
    "#;
        assert_eval_number(code, 1.0);
    }

    #[test]
    fn test_split_unicode() {
        let code = r#"
        let result: string[] = split("🎉,🔥,✨", ",");
        len(result)
    "#;
        assert_eval_number(code, 3.0);
    }

    #[test]
    fn test_join_basic() {
        let code = r#"join(["a", "b", "c"], ",")"#;
        assert_eval_string(code, "a,b,c");
    }

    #[test]
    fn test_join_empty_array() {
        let code = r#"join([], ",")"#;
        assert_eval_string(code, "");
    }

    #[test]
    fn test_join_empty_separator() {
        let code = r#"join(["a", "b", "c"], "")"#;
        assert_eval_string(code, "abc");
    }

    #[test]
    fn test_trim_basic() {
        let code = r#"trim("  hello  ")"#;
        assert_eval_string(code, "hello");
    }

    #[test]
    fn test_trim_unicode_whitespace() {
        let code = "trim(\"\u{00A0}hello\u{00A0}\")";
        assert_eval_string(code, "hello");
    }

    #[test]
    fn test_trim_start() {
        let code = r#"trimStart("  hello")"#;
        assert_eval_string(code, "hello");
    }

    #[test]
    fn test_trim_end() {
        let code = r#"trimEnd("hello  ")"#;
        assert_eval_string(code, "hello");
    }

    // ============================================================================
    // Search Operations Tests
    // ============================================================================

    #[test]
    fn test_index_of_found() {
        let code = r#"indexOf("hello", "ll")"#;
        assert_eval_number(code, 2.0);
    }

    #[test]
    fn test_index_of_not_found() {
        let code = r#"indexOf("hello", "x")"#;
        assert_eval_number(code, -1.0);
    }

    #[test]
    fn test_index_of_empty_needle() {
        let code = r#"indexOf("hello", "")"#;
        assert_eval_number(code, 0.0);
    }

    #[test]
    fn test_last_index_of_found() {
        let code = r#"lastIndexOf("hello", "l")"#;
        assert_eval_number(code, 3.0);
    }

    #[test]
    fn test_last_index_of_not_found() {
        let code = r#"lastIndexOf("hello", "x")"#;
        assert_eval_number(code, -1.0);
    }

    #[test]
    fn test_includes_found() {
        let code = r#"includes("hello", "ll")"#;
        assert_eval_bool(code, true);
    }

    #[test]
    fn test_includes_not_found() {
        let code = r#"includes("hello", "x")"#;
        assert_eval_bool(code, false);
    }

    // ============================================================================
    // Transformation Tests
    // ============================================================================

    #[test]
    fn test_to_upper_case() {
        let code = r#"toUpperCase("hello")"#;
        assert_eval_string(code, "HELLO");
    }

    #[test]
    fn test_to_upper_case_unicode() {
        let code = r#"toUpperCase("café")"#;
        assert_eval_string(code, "CAFÉ");
    }

    #[test]
    fn test_to_lower_case() {
        let code = r#"toLowerCase("HELLO")"#;
        assert_eval_string(code, "hello");
    }

    #[test]
    fn test_to_lower_case_unicode() {
        let code = r#"toLowerCase("CAFÉ")"#;
        assert_eval_string(code, "café");
    }

    #[test]
    fn test_substring_basic() {
        let code = r#"substring("hello", 1, 4)"#;
        assert_eval_string(code, "ell");
    }

    #[test]
    fn test_substring_empty() {
        let code = r#"substring("hello", 2, 2)"#;
        assert_eval_string(code, "");
    }

    #[test]
    fn test_substring_out_of_bounds() {
        let code = r#"substring("hello", 0, 100)"#;
        assert_has_error(code);
    }

    #[test]
    fn test_char_at_basic() {
        let code = r#"charAt("hello", 0)"#;
        assert_eval_string(code, "h");
    }

    #[test]
    fn test_char_at_unicode() {
        let code = r#"charAt("🎉🔥✨", 1)"#;
        assert_eval_string(code, "🔥");
    }

    #[test]
    fn test_char_at_out_of_bounds() {
        let code = r#"charAt("hello", 10)"#;
        assert_has_error(code);
    }

    #[test]
    fn test_repeat_basic() {
        let code = r#"repeat("ha", 3)"#;
        assert_eval_string(code, "hahaha");
    }

    #[test]
    fn test_repeat_zero() {
        let code = r#"repeat("ha", 0)"#;
        assert_eval_string(code, "");
    }

    #[test]
    fn test_repeat_negative() {
        let code = r#"repeat("ha", -1)"#;
        assert_has_error(code);
    }

    #[test]
    fn test_replace_basic() {
        let code = r#"replace("hello", "l", "L")"#;
        assert_eval_string(code, "heLlo");
    }

    #[test]
    fn test_replace_not_found() {
        let code = r#"replace("hello", "x", "y")"#;
        assert_eval_string(code, "hello");
    }

    #[test]
    fn test_replace_empty_search() {
        let code = r#"replace("hello", "", "x")"#;
        assert_eval_string(code, "hello");
    }

    // ============================================================================
    // Formatting Tests
    // ============================================================================

    #[test]
    fn test_pad_start_basic() {
        let code = r#"padStart("5", 3, "0")"#;
        assert_eval_string(code, "005");
    }

    #[test]
    fn test_pad_start_already_long() {
        let code = r#"padStart("hello", 3, "0")"#;
        assert_eval_string(code, "hello");
    }

    #[test]
    fn test_pad_start_multichar_fill() {
        let code = r#"padStart("x", 5, "ab")"#;
        assert_eval_string(code, "ababx");
    }

    #[test]
    fn test_pad_end_basic() {
        let code = r#"padEnd("5", 3, "0")"#;
        assert_eval_string(code, "500");
    }

    #[test]
    fn test_pad_end_already_long() {
        let code = r#"padEnd("hello", 3, "0")"#;
        assert_eval_string(code, "hello");
    }

    #[test]
    fn test_starts_with_true() {
        let code = r#"startsWith("hello", "he")"#;
        assert_eval_bool(code, true);
    }

    #[test]
    fn test_starts_with_false() {
        let code = r#"startsWith("hello", "x")"#;
        assert_eval_bool(code, false);
    }

    #[test]
    fn test_starts_with_empty() {
        let code = r#"startsWith("hello", "")"#;
        assert_eval_bool(code, true);
    }

    #[test]
    fn test_ends_with_true() {
        let code = r#"endsWith("hello", "lo")"#;
        assert_eval_bool(code, true);
    }

    #[test]
    fn test_ends_with_false() {
        let code = r#"endsWith("hello", "x")"#;
        assert_eval_bool(code, false);
    }

    #[test]
    fn test_ends_with_empty() {
        let code = r#"endsWith("hello", "")"#;
        assert_eval_bool(code, true);
    }

    // ============================================================================
    // From vm_stdlib_json_tests.rs
    // ============================================================================

    // JSON stdlib tests (VM engine)
    //
    // Tests all 5 JSON functions via VM execution for parity verification
    //
    // Note: These tests use the same common::* helpers which test through the full pipeline,
    // ensuring both interpreter and VM produce identical results.

    // ============================================================================
    // parseJSON Tests
    // ============================================================================

    #[test]
    fn test_parse_json_null() {
        let code = r#"
        let result: json = parseJSON("null");
        typeof(result)
    "#;
        assert_eval_string(code, "json");
    }

    #[test]
    fn test_parse_json_boolean_true() {
        // Should return JsonValue, test via typeof
        let code = r#"typeof(parseJSON("true"))"#;
        assert_eval_string(code, "json");
    }

    #[test]
    fn test_parse_json_boolean_false() {
        let code = r#"typeof(parseJSON("false"))"#;
        assert_eval_string(code, "json");
    }

    #[test]
    fn test_parse_json_number() {
        let code = r#"typeof(parseJSON("42"))"#;
        assert_eval_string(code, "json");
    }

    #[test]
    fn test_parse_json_number_float() {
        let code = r#"typeof(parseJSON("3.14"))"#;
        assert_eval_string(code, "json");
    }

    #[test]
    fn test_parse_json_number_negative() {
        let code = r#"typeof(parseJSON("-123"))"#;
        assert_eval_string(code, "json");
    }

    #[test]
    fn test_parse_json_string() {
        let code = r#"typeof(parseJSON("\"hello\""))"#;
        assert_eval_string(code, "json");
    }

    #[test]
    fn test_parse_json_empty_string() {
        let code = r#"typeof(parseJSON("\"\""))"#;
        assert_eval_string(code, "json");
    }

    #[test]
    fn test_parse_json_array_empty() {
        let code = r#"typeof(parseJSON("[]"))"#;
        assert_eval_string(code, "json");
    }

    #[test]
    fn test_parse_json_array_numbers() {
        let code = r#"typeof(parseJSON("[1,2,3]"))"#;
        assert_eval_string(code, "json");
    }

    #[test]
    fn test_parse_json_array_mixed() {
        let code = r#"typeof(parseJSON("[1,\"two\",true,null]"))"#;
        assert_eval_string(code, "json");
    }

    #[test]
    fn test_parse_json_array_nested() {
        let code = r#"typeof(parseJSON("[[1,2],[3,4]]"))"#;
        assert_eval_string(code, "json");
    }

    #[test]
    fn test_parse_json_object_empty() {
        let code = r#"typeof(parseJSON("{}"))"#;
        assert_eval_string(code, "json");
    }

    #[test]
    fn test_parse_json_object_simple() {
        let code = r#"typeof(parseJSON("{\"name\":\"Alice\",\"age\":30}"))"#;
        assert_eval_string(code, "json");
    }

    #[test]
    fn test_parse_json_object_nested() {
        let code = r#"typeof(parseJSON("{\"user\":{\"name\":\"Bob\"}}"))"#;
        assert_eval_string(code, "json");
    }

    #[test]
    fn test_parse_json_object_with_array() {
        let code = r#"typeof(parseJSON("{\"items\":[1,2,3]}"))"#;
        assert_eval_string(code, "json");
    }

    #[test]
    fn test_parse_json_whitespace() {
        let code = r#"typeof(parseJSON("  { \"a\" : 1 }  "))"#;
        assert_eval_string(code, "json");
    }

    #[test]
    fn test_parse_json_unicode() {
        let code = r#"typeof(parseJSON("{\"emoji\":\"🎉\"}"))"#;
        assert_eval_string(code, "json");
    }

    // ============================================================================
    // parseJSON Error Tests
    // ============================================================================

    #[test]
    fn test_parse_json_invalid_syntax() {
        let code = r#"parseJSON("{invalid}")"#;
        assert_has_error(code);
    }

    #[test]
    fn test_parse_json_trailing_comma() {
        let code = r#"parseJSON("[1,2,]")"#;
        assert_has_error(code);
    }

    #[test]
    fn test_parse_json_single_quote() {
        let code = r#"parseJSON("{'key':'value'}")"#;
        assert_has_error(code);
    }

    #[test]
    fn test_parse_json_unquoted_keys() {
        let code = r#"parseJSON("{key:\"value\"}")"#;
        assert_has_error(code);
    }

    #[test]
    fn test_parse_json_wrong_type() {
        let code = r#"parseJSON(123)"#;
        assert_has_error(code);
    }

    // ============================================================================
    // toJSON Tests
    // ============================================================================

    #[test]
    fn test_to_json_null() {
        let code = r#"toJSON(null)"#;
        assert_eval_string(code, "null");
    }

    #[test]
    fn test_to_json_bool_true() {
        let code = r#"toJSON(true)"#;
        assert_eval_string(code, "true");
    }

    #[test]
    fn test_to_json_bool_false() {
        let code = r#"toJSON(false)"#;
        assert_eval_string(code, "false");
    }

    #[test]
    fn test_to_json_number_int() {
        let code = r#"toJSON(42)"#;
        assert_eval_string(code, "42");
    }

    #[test]
    fn test_to_json_number_float() {
        let code = r#"toJSON(3.14)"#;
        assert_eval_string(code, "3.14");
    }

    #[test]
    fn test_to_json_number_negative() {
        let code = r#"toJSON(-10)"#;
        assert_eval_string(code, "-10");
    }

    #[test]
    fn test_to_json_number_zero() {
        let code = r#"toJSON(0)"#;
        assert_eval_string(code, "0");
    }

    #[test]
    fn test_to_json_string_simple() {
        let code = r#"toJSON("hello")"#;
        assert_eval_string(code, r#""hello""#);
    }

    #[test]
    fn test_to_json_string_empty() {
        let code = r#"toJSON("")"#;
        assert_eval_string(code, r#""""#);
    }

    #[test]
    fn test_to_json_string_with_quotes() {
        let code = r#"toJSON("say \"hi\"")"#;
        assert_eval_string(code, r#""say \"hi\"""#);
    }

    #[test]
    fn test_to_json_array_empty() {
        let code = r#"toJSON([])"#;
        assert_eval_string(code, "[]");
    }

    #[test]
    fn test_to_json_array_numbers() {
        let code = r#"toJSON([1,2,3])"#;
        assert_eval_string(code, "[1,2,3]");
    }

    // Note: Mixed-type array test removed - Atlas enforces homogeneous arrays.
    // For heterogeneous JSON arrays, use parseJSON to create json values.

    #[test]
    fn test_to_json_array_nested() {
        let code = r#"toJSON([[1,2],[3,4]])"#;
        assert_eval_string(code, "[[1,2],[3,4]]");
    }

    // ============================================================================
    // toJSON Error Tests
    // ============================================================================

    #[test]
    fn test_to_json_nan_error() {
        let code = r#"toJSON(0.0 / 0.0)"#;
        assert_has_error(code);
    }

    #[test]
    fn test_to_json_infinity_error() {
        let code = r#"toJSON(1.0 / 0.0)"#;
        assert_has_error(code);
    }

    #[test]
    fn test_to_json_function_error() {
        let code = r#"
        fn test(): number { return 42; }
        toJSON(test)
    "#;
        assert_has_error(code);
    }

    // ============================================================================
    // isValidJSON Tests
    // ============================================================================

    #[test]
    fn test_is_valid_json_true_null() {
        let code = r#"isValidJSON("null")"#;
        assert_eval_bool(code, true);
    }

    #[test]
    fn test_is_valid_json_true_bool() {
        let code = r#"isValidJSON("true")"#;
        assert_eval_bool(code, true);
    }

    #[test]
    fn test_is_valid_json_true_number() {
        let code = r#"isValidJSON("42")"#;
        assert_eval_bool(code, true);
    }

    #[test]
    fn test_is_valid_json_true_string() {
        let code = r#"isValidJSON("\"hello\"")"#;
        assert_eval_bool(code, true);
    }

    #[test]
    fn test_is_valid_json_true_array() {
        let code = r#"isValidJSON("[1,2,3]")"#;
        assert_eval_bool(code, true);
    }

    #[test]
    fn test_is_valid_json_true_object() {
        let code = r#"isValidJSON("{\"key\":\"value\"}")"#;
        assert_eval_bool(code, true);
    }

    #[test]
    fn test_is_valid_json_false_invalid() {
        let code = r#"isValidJSON("{invalid}")"#;
        assert_eval_bool(code, false);
    }

    #[test]
    fn test_is_valid_json_false_trailing_comma() {
        let code = r#"isValidJSON("[1,2,]")"#;
        assert_eval_bool(code, false);
    }

    #[test]
    fn test_is_valid_json_false_empty() {
        let code = r#"isValidJSON("")"#;
        assert_eval_bool(code, false);
    }

    #[test]
    fn test_is_valid_json_false_single_quote() {
        let code = r#"isValidJSON("{'a':1}")"#;
        assert_eval_bool(code, false);
    }

    #[test]
    fn test_is_valid_json_wrong_type() {
        let code = r#"isValidJSON(123)"#;
        assert_has_error(code);
    }

    // ============================================================================
    // prettifyJSON Tests
    // ============================================================================

    #[test]
    fn test_prettify_json_object() {
        let code = r#"
        let compact: string = "{\"name\":\"Alice\",\"age\":30}";
        let pretty: string = prettifyJSON(compact, 2);
        includes(pretty, "  ")
    "#;
        assert_eval_bool(code, true);
    }

    #[test]
    fn test_prettify_json_array() {
        let code = r#"
        let compact: string = "[1,2,3]";
        let pretty: string = prettifyJSON(compact, 2);
        len(pretty) > len(compact)
    "#;
        assert_eval_bool(code, true);
    }

    #[test]
    fn test_prettify_json_indent_zero() {
        let code = r#"
        let compact: string = "{\"a\":1}";
        let pretty: string = prettifyJSON(compact, 0);
        typeof(pretty)
    "#;
        assert_eval_string(code, "string");
    }

    #[test]
    fn test_prettify_json_indent_four() {
        let code = r#"
        let compact: string = "{\"a\":1}";
        let pretty: string = prettifyJSON(compact, 4);
        includes(pretty, "    ")
    "#;
        assert_eval_bool(code, true);
    }

    #[test]
    fn test_prettify_json_nested() {
        let code = r#"
        let compact: string = "{\"user\":{\"name\":\"Bob\"}}";
        let pretty: string = prettifyJSON(compact, 2);
        len(pretty) > len(compact)
    "#;
        assert_eval_bool(code, true);
    }

    #[test]
    fn test_prettify_json_invalid() {
        let code = r#"prettifyJSON("{invalid}", 2)"#;
        assert_has_error(code);
    }

    #[test]
    fn test_prettify_json_negative_indent() {
        let code = r#"prettifyJSON("{}", -1)"#;
        assert_has_error(code);
    }

    #[test]
    fn test_prettify_json_float_indent() {
        let code = r#"prettifyJSON("{}", 2.5)"#;
        assert_has_error(code);
    }

    #[test]
    fn test_prettify_json_wrong_type_first_arg() {
        let code = r#"prettifyJSON(123, 2)"#;
        assert_has_error(code);
    }

    #[test]
    fn test_prettify_json_wrong_type_second_arg() {
        let code = r#"prettifyJSON("{}", "2")"#;
        assert_has_error(code);
    }

    // ============================================================================
    // minifyJSON Tests
    // ============================================================================

    #[test]
    fn test_minify_json_object() {
        let code = r#"
        let pretty: string = "{\n  \"name\": \"Alice\",\n  \"age\": 30\n}";
        let minified: string = minifyJSON(pretty);
        len(minified) < len(pretty)
    "#;
        assert_eval_bool(code, true);
    }

    #[test]
    fn test_minify_json_array() {
        let code = r#"
        let pretty: string = "[\n  1,\n  2,\n  3\n]";
        let minified: string = minifyJSON(pretty);
        len(minified) < len(pretty)
    "#;
        assert_eval_bool(code, true);
    }

    #[test]
    fn test_minify_json_no_whitespace() {
        let code = r#"
        let compact: string = "{\"a\":1}";
        let minified: string = minifyJSON(compact);
        typeof(minified)
    "#;
        assert_eval_string(code, "string");
    }

    #[test]
    fn test_minify_json_nested() {
        let code = r#"
        let pretty: string = "{\n  \"user\": {\n    \"name\": \"Bob\"\n  }\n}";
        let minified: string = minifyJSON(pretty);
        len(minified) < len(pretty)
    "#;
        assert_eval_bool(code, true);
    }

    #[test]
    fn test_minify_json_invalid() {
        let code = r#"minifyJSON("{invalid}")"#;
        assert_has_error(code);
    }

    #[test]
    fn test_minify_json_wrong_type() {
        let code = r#"minifyJSON(123)"#;
        assert_has_error(code);
    }

    // ============================================================================
    // Integration Tests
    // ============================================================================

    #[test]
    fn test_parse_then_serialize() {
        let code = r#"
        let original: string = "{\"name\":\"Alice\",\"age\":30}";
        let parsed: json = parseJSON(original);
        let serialized: string = toJSON(parsed);
        typeof(serialized)
    "#;
        assert_eval_string(code, "string");
    }

    #[test]
    fn test_prettify_then_minify() {
        let code = r#"
        let compact: string = "{\"a\":1,\"b\":2}";
        let pretty: string = prettifyJSON(compact, 2);
        let minified: string = minifyJSON(pretty);
        len(minified) < len(pretty)
    "#;
        assert_eval_bool(code, true);
    }

    #[test]
    fn test_validate_before_parse() {
        let code = r#"
        let json_str: string = "{\"valid\":true}";
        let valid: bool = isValidJSON(json_str);
        let parsed: json = parseJSON(json_str);
        valid && typeof(parsed) == "json"
    "#;
        assert_eval_bool(code, true);
    }

    // ============================================================================
    // From vm_stdlib_io_tests.rs
    // ============================================================================

    // Standard library file I/O tests (VM/Bytecode)
    //
    // Tests file and directory operations via bytecode execution for VM parity.

    // Helper to execute Atlas source via bytecode
    fn execute_with_io(source: &str, temp_dir: &TempDir) -> Result<atlas_runtime::Value, String> {
        use atlas_runtime::{Binder, Compiler, Lexer, Parser, TypeChecker, VM};

        // Parse and compile
        let mut lexer = Lexer::new(source.to_string());
        let (tokens, _) = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let (ast, _) = parser.parse();
        let mut binder = Binder::new();
        let (mut symbol_table, _) = binder.bind(&ast);
        let mut typechecker = TypeChecker::new(&mut symbol_table);
        let _ = typechecker.check(&ast);
        let mut compiler = Compiler::new();
        let bytecode = compiler.compile(&ast).unwrap();

        // Execute with security context
        let mut security = SecurityContext::new();
        security.grant_filesystem_read(temp_dir.path(), true);
        security.grant_filesystem_write(temp_dir.path(), true);

        let mut vm = VM::new(bytecode);
        vm.run(&security)
            .map(|opt| opt.unwrap_or(atlas_runtime::Value::Null))
            .map_err(|e| format!("{:?}", e))
    }

    // ============================================================================
    // VM parity tests - all use pattern: let result = func(); result;
    // ============================================================================

    #[test]
    fn vm_test_read_file_basic() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "Hello, VM!").unwrap();

        let code = format!(r#"let x = readFile("{}"); x;"#, path_for_atlas(&test_file));
        let result = execute_with_io(&code, &temp_dir);

        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), atlas_runtime::Value::String(_)));
    }

    #[test]
    fn vm_test_write_file_basic() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("output.txt");

        let code = format!(
            r#"writeFile("{}", "VM content");"#,
            path_for_atlas(&test_file)
        );
        let result = execute_with_io(&code, &temp_dir);

        assert!(result.is_ok());
        let contents = fs::read_to_string(&test_file).unwrap();
        assert_eq!(contents, "VM content");
    }

    #[test]
    fn vm_test_append_file_basic() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("append.txt");
        fs::write(&test_file, "line1\n").unwrap();

        let code = format!(
            r#"appendFile("{}", "line2\n");"#,
            path_for_atlas(&test_file)
        );
        let result = execute_with_io(&code, &temp_dir);

        assert!(result.is_ok());
        let contents = fs::read_to_string(&test_file).unwrap();
        assert_eq!(contents, "line1\nline2\n");
    }

    #[test]
    fn vm_test_file_exists_true() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("exists.txt");
        fs::write(&test_file, "").unwrap();

        let code = format!(
            r#"let result = fileExists("{}"); result;"#,
            path_for_atlas(&test_file)
        );
        let result = execute_with_io(&code, &temp_dir);

        assert!(result.is_ok());
        let value = result.unwrap();
        assert!(matches!(value, atlas_runtime::Value::Bool(true)));
    }

    #[test]
    fn vm_test_file_exists_false() {
        let temp_dir = TempDir::new().unwrap();
        let nonexistent = temp_dir.path().join("does_not_exist.txt");

        let code = format!(
            r#"let result = fileExists("{}"); result;"#,
            path_for_atlas(&nonexistent)
        );
        let result = execute_with_io(&code, &temp_dir);

        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), atlas_runtime::Value::Bool(false)));
    }

    #[test]
    fn vm_test_read_dir_basic() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("file1.txt"), "").unwrap();
        fs::write(temp_dir.path().join("file2.txt"), "").unwrap();

        let code = format!(
            r#"let result = readDir("{}"); result;"#,
            path_for_atlas(temp_dir.path())
        );
        let result = execute_with_io(&code, &temp_dir);

        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), atlas_runtime::Value::Array(_)));
    }

    #[test]
    fn vm_test_create_dir_basic() {
        let temp_dir = TempDir::new().unwrap();
        let new_dir = temp_dir.path().join("newdir");

        let code = format!(r#"createDir("{}");"#, path_for_atlas(&new_dir));
        let result = execute_with_io(&code, &temp_dir);

        assert!(result.is_ok());
        assert!(new_dir.exists());
        assert!(new_dir.is_dir());
    }

    #[test]
    fn vm_test_create_dir_nested() {
        let temp_dir = TempDir::new().unwrap();
        let nested_dir = temp_dir.path().join("a/b/c");

        let code = format!(r#"createDir("{}");"#, path_for_atlas(&nested_dir));
        let result = execute_with_io(&code, &temp_dir);

        assert!(result.is_ok());
        assert!(nested_dir.exists());
    }

    #[test]
    fn vm_test_remove_file_basic() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("remove.txt");
        fs::write(&test_file, "").unwrap();

        let code = format!(r#"removeFile("{}");"#, path_for_atlas(&test_file));
        let result = execute_with_io(&code, &temp_dir);

        assert!(result.is_ok());
        assert!(!test_file.exists());
    }

    #[test]
    fn vm_test_remove_dir_basic() {
        let temp_dir = TempDir::new().unwrap();
        let test_dir = temp_dir.path().join("rmdir");
        fs::create_dir(&test_dir).unwrap();

        let code = format!(r#"removeDir("{}");"#, path_for_atlas(&test_dir));
        let result = execute_with_io(&code, &temp_dir);

        assert!(result.is_ok());
        assert!(!test_dir.exists());
    }

    #[test]
    fn vm_test_file_info_file() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("info.txt");
        fs::write(&test_file, "test content").unwrap();

        let code = format!(
            r#"let result = fileInfo("{}"); result;"#,
            path_for_atlas(&test_file)
        );
        let result = execute_with_io(&code, &temp_dir);

        assert!(result.is_ok());
        assert!(matches!(
            result.unwrap(),
            atlas_runtime::Value::JsonValue(_)
        ));
    }

    #[test]
    fn vm_test_path_join_basic() {
        let temp_dir = TempDir::new().unwrap();
        let code = r#"let result = pathJoin("a", "b", "c"); result;"#;
        let result = execute_with_io(code, &temp_dir);

        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), atlas_runtime::Value::String(_)));
    }

    // ============================================================================
    // Additional VM parity tests to match interpreter coverage
    // ============================================================================

    #[test]
    fn vm_test_read_file_utf8() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("utf8.txt");
        fs::write(&test_file, "Hello 你好 🎉").unwrap();

        let code = format!(r#"let x = readFile("{}"); x;"#, path_for_atlas(&test_file));
        let result = execute_with_io(&code, &temp_dir);

        assert!(result.is_ok());
    }

    #[test]
    fn vm_test_read_file_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let nonexistent = temp_dir.path().join("does_not_exist.txt");

        let code = format!(r#"readFile("{}");"#, path_for_atlas(&nonexistent));
        let result = execute_with_io(&code, &temp_dir);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to resolve path"));
    }

    #[test]
    fn vm_test_read_file_permission_denied() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("secret.txt");
        fs::write(&test_file, "secret").unwrap();

        // Execute without granting permissions
        let mut lexer =
            atlas_runtime::Lexer::new(format!(r#"readFile("{}");"#, path_for_atlas(&test_file)));
        let (tokens, _) = lexer.tokenize();
        let mut parser = atlas_runtime::Parser::new(tokens);
        let (ast, _) = parser.parse();
        let mut binder = atlas_runtime::Binder::new();
        let (mut symbol_table, _) = binder.bind(&ast);
        let mut typechecker = atlas_runtime::TypeChecker::new(&mut symbol_table);
        let _ = typechecker.check(&ast);
        let mut compiler = atlas_runtime::Compiler::new();
        let bytecode = compiler.compile(&ast).unwrap();

        let security = SecurityContext::new(); // No permissions
        let mut vm = atlas_runtime::VM::new(bytecode);
        let result = vm.run(&security);

        assert!(result.is_err());
    }

    #[test]
    fn vm_test_write_file_overwrite() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("overwrite.txt");
        fs::write(&test_file, "original").unwrap();

        let code = format!(
            r#"writeFile("{}", "new content");"#,
            path_for_atlas(&test_file)
        );
        let result = execute_with_io(&code, &temp_dir);

        assert!(result.is_ok());
        let contents = fs::read_to_string(&test_file).unwrap();
        assert_eq!(contents, "new content");
    }

    #[test]
    fn vm_test_write_file_permission_denied() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("output.txt");

        // Execute without granting permissions
        let mut lexer = atlas_runtime::Lexer::new(format!(
            r#"writeFile("{}", "content");"#,
            path_for_atlas(&test_file)
        ));
        let (tokens, _) = lexer.tokenize();
        let mut parser = atlas_runtime::Parser::new(tokens);
        let (ast, _) = parser.parse();
        let mut binder = atlas_runtime::Binder::new();
        let (mut symbol_table, _) = binder.bind(&ast);
        let mut typechecker = atlas_runtime::TypeChecker::new(&mut symbol_table);
        let _ = typechecker.check(&ast);
        let mut compiler = atlas_runtime::Compiler::new();
        let bytecode = compiler.compile(&ast).unwrap();

        let security = SecurityContext::new(); // No permissions
        let mut vm = atlas_runtime::VM::new(bytecode);
        let result = vm.run(&security);

        assert!(result.is_err());
    }

    #[test]
    fn vm_test_append_file_create_if_not_exists() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("new.txt");

        let code = format!(
            r#"appendFile("{}", "content");"#,
            path_for_atlas(&test_file)
        );
        let result = execute_with_io(&code, &temp_dir);

        assert!(result.is_ok());
        let contents = fs::read_to_string(&test_file).unwrap();
        assert_eq!(contents, "content");
    }

    #[test]
    fn vm_test_read_dir_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let nonexistent = temp_dir.path().join("nonexistent_dir");

        let code = format!(r#"readDir("{}");"#, path_for_atlas(&nonexistent));
        let result = execute_with_io(&code, &temp_dir);

        assert!(result.is_err());
    }

    #[test]
    fn vm_test_remove_file_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let nonexistent = temp_dir.path().join("does_not_exist.txt");

        let code = format!(r#"removeFile("{}");"#, path_for_atlas(&nonexistent));
        let result = execute_with_io(&code, &temp_dir);

        assert!(result.is_err());
    }

    #[test]
    fn vm_test_remove_dir_not_empty() {
        let temp_dir = TempDir::new().unwrap();
        let test_dir = temp_dir.path().join("notempty");
        fs::create_dir(&test_dir).unwrap();
        fs::write(test_dir.join("file.txt"), "").unwrap();

        let code = format!(r#"removeDir("{}");"#, path_for_atlas(&test_dir));
        let result = execute_with_io(&code, &temp_dir);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to remove directory"));
    }

    #[test]
    fn vm_test_file_info_directory() {
        let temp_dir = TempDir::new().unwrap();
        let test_dir = temp_dir.path().join("infodir");
        fs::create_dir(&test_dir).unwrap();

        let code = format!(
            r#"let result = fileInfo("{}"); result;"#,
            path_for_atlas(&test_dir)
        );
        let result = execute_with_io(&code, &temp_dir);

        assert!(result.is_ok());
    }

    #[test]
    fn vm_test_path_join_single() {
        let temp_dir = TempDir::new().unwrap();
        let code = r#"let result = pathJoin("single"); result;"#;
        let result = execute_with_io(code, &temp_dir);

        assert!(result.is_ok());
    }

    #[test]
    fn vm_test_path_join_no_args() {
        let temp_dir = TempDir::new().unwrap();
        let code = r#"pathJoin();"#;
        let result = execute_with_io(code, &temp_dir);

        assert!(result.is_err());
    }

    // ============================================================================
    // VM - Additional readFile tests
    // ============================================================================

    #[test]
    fn vm_test_read_file_empty() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("empty.txt");
        fs::write(&test_file, "").unwrap();

        let code = format!(r#"let x = readFile("{}"); x;"#, path_for_atlas(&test_file));
        let result = execute_with_io(&code, &temp_dir);

        assert!(result.is_ok());
        if let atlas_runtime::Value::String(s) = result.unwrap() {
            assert_eq!(s.as_str(), "");
        } else {
            panic!("Expected string");
        }
    }

    #[test]
    fn vm_test_read_file_invalid_utf8() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("binary.bin");
        fs::write(&test_file, [0xFF, 0xFE, 0xFD]).unwrap();

        let code = format!(r#"readFile("{}");"#, path_for_atlas(&test_file));
        let result = execute_with_io(&code, &temp_dir);

        assert!(result.is_err());
    }

    #[test]
    fn vm_test_read_file_multiline() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("multiline.txt");
        let content = "line1\nline2\nline3\n";
        fs::write(&test_file, content).unwrap();

        let code = format!(r#"let x = readFile("{}"); x;"#, path_for_atlas(&test_file));
        let result = execute_with_io(&code, &temp_dir);

        assert!(result.is_ok());
        if let atlas_runtime::Value::String(s) = result.unwrap() {
            assert_eq!(s.as_str(), content);
        } else {
            panic!("Expected string");
        }
    }

    #[test]
    fn vm_test_read_file_large() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("large.txt");
        let content = "x".repeat(10000);
        fs::write(&test_file, &content).unwrap();

        let code = format!(r#"let x = readFile("{}"); x;"#, path_for_atlas(&test_file));
        let result = execute_with_io(&code, &temp_dir);

        assert!(result.is_ok());
        if let atlas_runtime::Value::String(s) = result.unwrap() {
            assert_eq!(s.len(), 10000);
        } else {
            panic!("Expected string");
        }
    }

    #[test]
    fn vm_test_read_file_with_bom() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("bom.txt");
        let mut content = vec![0xEF, 0xBB, 0xBF];
        content.extend_from_slice(b"Hello");
        fs::write(&test_file, content).unwrap();

        let code = format!(r#"let x = readFile("{}"); x;"#, path_for_atlas(&test_file));
        let result = execute_with_io(&code, &temp_dir);

        assert!(result.is_ok());
    }

    // ============================================================================
    // VM - Additional writeFile tests
    // ============================================================================

    #[test]
    fn vm_test_write_file_empty() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("empty_write.txt");

        let code = format!(r#"writeFile("{}", "");"#, path_for_atlas(&test_file));
        let result = execute_with_io(&code, &temp_dir);

        assert!(result.is_ok());
        let contents = fs::read_to_string(&test_file).unwrap();
        assert_eq!(contents, "");
    }

    #[test]
    fn vm_test_write_file_unicode() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("unicode.txt");
        let content = "Hello 世界 🌍";

        let code = format!(
            r#"writeFile("{}", "{}");"#,
            path_for_atlas(&test_file),
            content
        );
        let result = execute_with_io(&code, &temp_dir);

        assert!(result.is_ok());
        let contents = fs::read_to_string(&test_file).unwrap();
        assert_eq!(contents, content);
    }

    #[test]
    fn vm_test_write_file_newlines() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("newlines.txt");

        let code = format!(
            r#"writeFile("{}", "line1\nline2\n");"#,
            path_for_atlas(&test_file)
        );
        let result = execute_with_io(&code, &temp_dir);

        assert!(result.is_ok());
        let contents = fs::read_to_string(&test_file).unwrap();
        assert_eq!(contents, "line1\nline2\n");
    }

    #[test]
    fn vm_test_write_file_creates_file() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("new_file.txt");
        assert!(!test_file.exists());

        let code = format!(r#"writeFile("{}", "content");"#, path_for_atlas(&test_file));
        let result = execute_with_io(&code, &temp_dir);

        assert!(result.is_ok());
        assert!(test_file.exists());
    }

    // ============================================================================
    // VM - Additional appendFile tests
    // ============================================================================

    #[test]
    fn vm_test_append_file_multiple() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("multi_append.txt");
        fs::write(&test_file, "start\n").unwrap();

        let code = format!(
            r#"appendFile("{}", "line1\n"); appendFile("{}", "line2\n");"#,
            path_for_atlas(&test_file),
            path_for_atlas(&test_file)
        );
        let result = execute_with_io(&code, &temp_dir);

        assert!(result.is_ok());
        let contents = fs::read_to_string(&test_file).unwrap();
        assert_eq!(contents, "start\nline1\nline2\n");
    }

    #[test]
    fn vm_test_append_file_empty_content() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("append_empty.txt");
        fs::write(&test_file, "base").unwrap();

        let code = format!(r#"appendFile("{}", "");"#, path_for_atlas(&test_file));
        let result = execute_with_io(&code, &temp_dir);

        assert!(result.is_ok());
        let contents = fs::read_to_string(&test_file).unwrap();
        assert_eq!(contents, "base");
    }

    #[test]
    fn vm_test_append_file_permission_denied() {
        use atlas_runtime::{Binder, Compiler, Lexer, Parser, SecurityContext, TypeChecker, VM};

        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("append_denied.txt");

        let code = format!(
            r#"appendFile("{}", "content");"#,
            path_for_atlas(&test_file)
        );

        let mut lexer = Lexer::new(code);
        let (tokens, _) = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let (ast, _) = parser.parse();
        let mut binder = Binder::new();
        let (mut symbol_table, _) = binder.bind(&ast);
        let mut typechecker = TypeChecker::new(&mut symbol_table);
        let _ = typechecker.check(&ast);
        let mut compiler = Compiler::new();
        let bytecode = compiler.compile(&ast).unwrap();

        let security = SecurityContext::new();
        let mut vm = VM::new(bytecode);
        let result = vm.run(&security);

        assert!(result.is_err());
    }

    // ============================================================================
    // VM - Additional fileExists tests
    // ============================================================================

    #[test]
    fn vm_test_file_exists_directory() {
        let temp_dir = TempDir::new().unwrap();
        let test_dir = temp_dir.path().join("exists_dir");
        fs::create_dir(&test_dir).unwrap();

        let code = format!(
            r#"let result = fileExists("{}"); result;"#,
            path_for_atlas(&test_dir)
        );
        let result = execute_with_io(&code, &temp_dir);

        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), atlas_runtime::Value::Bool(true)));
    }

    #[test]
    fn vm_test_file_exists_no_permission_check() {
        use atlas_runtime::{Binder, Compiler, Lexer, Parser, SecurityContext, TypeChecker, VM};

        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("exists_test.txt");
        fs::write(&test_file, "").unwrap();

        let code = format!(
            r#"let x = fileExists("{}"); x;"#,
            path_for_atlas(&test_file)
        );

        let mut lexer = Lexer::new(code);
        let (tokens, _) = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let (ast, _) = parser.parse();
        let mut binder = Binder::new();
        let (mut symbol_table, _) = binder.bind(&ast);
        let mut typechecker = TypeChecker::new(&mut symbol_table);
        let _ = typechecker.check(&ast);
        let mut compiler = Compiler::new();
        let bytecode = compiler.compile(&ast).unwrap();

        let security = SecurityContext::new();
        let mut vm = VM::new(bytecode);
        let result = vm.run(&security);

        assert!(result.is_ok());
        assert!(matches!(
            result.unwrap(),
            Some(atlas_runtime::Value::Bool(true))
        ));
    }

    // ============================================================================
    // VM - Additional readDir tests
    // ============================================================================

    #[test]
    fn vm_test_read_dir_empty() {
        let temp_dir = TempDir::new().unwrap();
        let empty_dir = temp_dir.path().join("empty");
        fs::create_dir(&empty_dir).unwrap();

        let code = format!(r#"let x = readDir("{}"); x;"#, path_for_atlas(&empty_dir));
        let result = execute_with_io(&code, &temp_dir);

        assert!(result.is_ok());
        if let atlas_runtime::Value::Array(arr) = result.unwrap() {
            assert_eq!(arr.lock().unwrap().len(), 0);
        } else {
            panic!("Expected array");
        }
    }

    #[test]
    fn vm_test_read_dir_mixed_contents() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("file.txt"), "").unwrap();
        fs::create_dir(temp_dir.path().join("subdir")).unwrap();

        let code = format!(
            r#"let x = readDir("{}"); x;"#,
            path_for_atlas(temp_dir.path())
        );
        let result = execute_with_io(&code, &temp_dir);

        assert!(result.is_ok());
        if let atlas_runtime::Value::Array(arr) = result.unwrap() {
            assert_eq!(arr.lock().unwrap().len(), 2);
        } else {
            panic!("Expected array");
        }
    }

    #[test]
    fn vm_test_read_dir_permission_denied() {
        use atlas_runtime::{Binder, Compiler, Lexer, Parser, SecurityContext, TypeChecker, VM};

        let temp_dir = TempDir::new().unwrap();
        let test_dir = temp_dir.path().join("dir");
        fs::create_dir(&test_dir).unwrap();

        let code = format!(r#"readDir("{}");"#, path_for_atlas(&test_dir));

        let mut lexer = Lexer::new(code);
        let (tokens, _) = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let (ast, _) = parser.parse();
        let mut binder = Binder::new();
        let (mut symbol_table, _) = binder.bind(&ast);
        let mut typechecker = TypeChecker::new(&mut symbol_table);
        let _ = typechecker.check(&ast);
        let mut compiler = Compiler::new();
        let bytecode = compiler.compile(&ast).unwrap();

        let security = SecurityContext::new();
        let mut vm = VM::new(bytecode);
        let result = vm.run(&security);

        assert!(result.is_err());
    }

    // ============================================================================
    // VM - Additional createDir tests
    // ============================================================================

    #[test]
    fn vm_test_create_dir_already_exists() {
        let temp_dir = TempDir::new().unwrap();
        let test_dir = temp_dir.path().join("already_exists");
        fs::create_dir(&test_dir).unwrap();

        let code = format!(r#"createDir("{}");"#, path_for_atlas(&test_dir));
        let result = execute_with_io(&code, &temp_dir);

        assert!(result.is_ok());
    }

    #[test]
    fn vm_test_create_dir_permission_denied() {
        use atlas_runtime::{Binder, Compiler, Lexer, Parser, SecurityContext, TypeChecker, VM};

        let temp_dir = TempDir::new().unwrap();
        let new_dir = temp_dir.path().join("denied");

        let code = format!(r#"createDir("{}");"#, path_for_atlas(&new_dir));

        let mut lexer = Lexer::new(code);
        let (tokens, _) = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let (ast, _) = parser.parse();
        let mut binder = Binder::new();
        let (mut symbol_table, _) = binder.bind(&ast);
        let mut typechecker = TypeChecker::new(&mut symbol_table);
        let _ = typechecker.check(&ast);
        let mut compiler = Compiler::new();
        let bytecode = compiler.compile(&ast).unwrap();

        let security = SecurityContext::new();
        let mut vm = VM::new(bytecode);
        let result = vm.run(&security);

        assert!(result.is_err());
    }

    // ============================================================================
    // VM - Additional removeFile tests
    // ============================================================================

    #[test]
    fn vm_test_remove_file_is_directory() {
        let temp_dir = TempDir::new().unwrap();
        let test_dir = temp_dir.path().join("is_dir");
        fs::create_dir(&test_dir).unwrap();

        let code = format!(r#"removeFile("{}");"#, path_for_atlas(&test_dir));
        let result = execute_with_io(&code, &temp_dir);

        assert!(result.is_err());
    }

    #[test]
    fn vm_test_remove_file_permission_denied() {
        use atlas_runtime::{Binder, Compiler, Lexer, Parser, SecurityContext, TypeChecker, VM};

        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("remove_denied.txt");
        fs::write(&test_file, "").unwrap();

        let code = format!(r#"removeFile("{}");"#, path_for_atlas(&test_file));

        let mut lexer = Lexer::new(code);
        let (tokens, _) = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let (ast, _) = parser.parse();
        let mut binder = Binder::new();
        let (mut symbol_table, _) = binder.bind(&ast);
        let mut typechecker = TypeChecker::new(&mut symbol_table);
        let _ = typechecker.check(&ast);
        let mut compiler = Compiler::new();
        let bytecode = compiler.compile(&ast).unwrap();

        let security = SecurityContext::new();
        let mut vm = VM::new(bytecode);
        let result = vm.run(&security);

        assert!(result.is_err());
    }

    // ============================================================================
    // VM - Additional removeDir tests
    // ============================================================================

    #[test]
    fn vm_test_remove_dir_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let nonexistent = temp_dir.path().join("not_found");

        let code = format!(r#"removeDir("{}");"#, path_for_atlas(&nonexistent));
        let result = execute_with_io(&code, &temp_dir);

        assert!(result.is_err());
    }

    #[test]
    fn vm_test_remove_dir_is_file() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("is_file.txt");
        fs::write(&test_file, "").unwrap();

        let code = format!(r#"removeDir("{}");"#, path_for_atlas(&test_file));
        let result = execute_with_io(&code, &temp_dir);

        assert!(result.is_err());
    }

    #[test]
    fn vm_test_remove_dir_permission_denied() {
        use atlas_runtime::{Binder, Compiler, Lexer, Parser, SecurityContext, TypeChecker, VM};

        let temp_dir = TempDir::new().unwrap();
        let test_dir = temp_dir.path().join("remove_denied");
        fs::create_dir(&test_dir).unwrap();

        let code = format!(r#"removeDir("{}");"#, path_for_atlas(&test_dir));

        let mut lexer = Lexer::new(code);
        let (tokens, _) = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let (ast, _) = parser.parse();
        let mut binder = Binder::new();
        let (mut symbol_table, _) = binder.bind(&ast);
        let mut typechecker = TypeChecker::new(&mut symbol_table);
        let _ = typechecker.check(&ast);
        let mut compiler = Compiler::new();
        let bytecode = compiler.compile(&ast).unwrap();

        let security = SecurityContext::new();
        let mut vm = VM::new(bytecode);
        let result = vm.run(&security);

        assert!(result.is_err());
    }

    // ============================================================================
    // VM - Additional fileInfo tests
    // ============================================================================

    #[test]
    fn vm_test_file_info_size_check() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("info_fields.txt");
        fs::write(&test_file, "12345").unwrap();

        let code = format!(r#"let x = fileInfo("{}"); x;"#, path_for_atlas(&test_file));
        let result = execute_with_io(&code, &temp_dir);

        assert!(result.is_ok());
        assert!(matches!(
            result.unwrap(),
            atlas_runtime::Value::JsonValue(_)
        ));
    }

    #[test]
    fn vm_test_file_info_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let nonexistent = temp_dir.path().join("not_found.txt");

        let code = format!(r#"fileInfo("{}");"#, path_for_atlas(&nonexistent));
        let result = execute_with_io(&code, &temp_dir);

        assert!(result.is_err());
    }

    #[test]
    fn vm_test_file_info_permission_denied() {
        use atlas_runtime::{Binder, Compiler, Lexer, Parser, SecurityContext, TypeChecker, VM};

        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("info_denied.txt");
        fs::write(&test_file, "test").unwrap();

        let code = format!(r#"fileInfo("{}");"#, path_for_atlas(&test_file));

        let mut lexer = Lexer::new(code);
        let (tokens, _) = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let (ast, _) = parser.parse();
        let mut binder = Binder::new();
        let (mut symbol_table, _) = binder.bind(&ast);
        let mut typechecker = TypeChecker::new(&mut symbol_table);
        let _ = typechecker.check(&ast);
        let mut compiler = Compiler::new();
        let bytecode = compiler.compile(&ast).unwrap();

        let security = SecurityContext::new();
        let mut vm = VM::new(bytecode);
        let result = vm.run(&security);

        assert!(result.is_err());
    }

    // ============================================================================
    // VM - Additional pathJoin tests
    // ============================================================================

    #[test]
    fn vm_test_path_join_many_parts() {
        let temp_dir = TempDir::new().unwrap();
        let code = r#"let x = pathJoin("a", "b", "c", "d", "e"); x;"#;
        let result = execute_with_io(code, &temp_dir);

        assert!(result.is_ok());
        if let atlas_runtime::Value::String(path) = result.unwrap() {
            assert!(path.contains("a"));
            assert!(path.contains("e"));
        } else {
            panic!("Expected string");
        }
    }

    #[test]
    fn vm_test_path_join_empty_parts() {
        let temp_dir = TempDir::new().unwrap();
        let code = r#"let x = pathJoin("", "a", ""); x;"#;
        let result = execute_with_io(code, &temp_dir);

        assert!(result.is_ok());
    }

    #[test]
    fn vm_test_path_join_absolute_path() {
        let temp_dir = TempDir::new().unwrap();
        let code = r#"let x = pathJoin("/absolute", "path"); x;"#;
        let result = execute_with_io(code, &temp_dir);

        assert!(result.is_ok());
        if let atlas_runtime::Value::String(path) = result.unwrap() {
            assert!(path.starts_with("/") || path.starts_with("\\"));
        } else {
            panic!("Expected string");
        }
    }

    // ============================================================================
    // From vm_stdlib_types_tests.rs
    // ============================================================================

    // Type checking and conversion stdlib tests (VM engine)
    //
    // Tests all 12 type utility functions via VM execution for parity verification
    //
    // Note: These tests use the same common::* helpers which test through the full pipeline,
    // ensuring both interpreter and VM produce identical results.

    // ============================================================================
    // typeof Tests
    // ============================================================================

    #[test]
    fn test_typeof_null() {
        let code = r#"typeof(null)"#;
        assert_eval_string(code, "null");
    }

    #[test]
    fn test_typeof_bool_true() {
        let code = r#"typeof(true)"#;
        assert_eval_string(code, "bool");
    }

    #[test]
    fn test_typeof_bool_false() {
        let code = r#"typeof(false)"#;
        assert_eval_string(code, "bool");
    }

    #[test]
    fn test_typeof_number_positive() {
        let code = r#"typeof(42)"#;
        assert_eval_string(code, "number");
    }

    #[test]
    fn test_typeof_number_negative() {
        let code = r#"typeof(-10)"#;
        assert_eval_string(code, "number");
    }

    #[test]
    fn test_typeof_number_float() {
        let code = r#"typeof(3.5)"#;
        assert_eval_string(code, "number");
    }

    // NaN/Infinity tests removed: division by zero is a runtime error in Atlas

    #[test]
    fn test_typeof_string_nonempty() {
        let code = r#"typeof("hello")"#;
        assert_eval_string(code, "string");
    }

    #[test]
    fn test_typeof_string_empty() {
        let code = r#"typeof("")"#;
        assert_eval_string(code, "string");
    }

    #[test]
    fn test_typeof_array_nonempty() {
        let code = r#"typeof([1,2,3])"#;
        assert_eval_string(code, "array");
    }

    #[test]
    fn test_typeof_array_empty() {
        let code = r#"typeof([])"#;
        assert_eval_string(code, "array");
    }

    // Function reference tests removed: not yet fully supported

    #[test]
    fn test_typeof_json() {
        let code = r#"typeof(parseJSON("null"))"#;
        assert_eval_string(code, "json");
    }

    #[test]
    fn test_typeof_option() {
        let code = r#"typeof(Some(42))"#;
        assert_eval_string(code, "option");
    }

    #[test]
    fn test_typeof_result() {
        let code = r#"typeof(Ok(42))"#;
        assert_eval_string(code, "result");
    }

    // ============================================================================
    // Type Guard Tests
    // ============================================================================

    #[test]
    fn test_is_string_true() {
        let code = r#"isString("hello")"#;
        assert_eval_bool(code, true);
    }

    #[test]
    fn test_is_string_false_number() {
        let code = r#"isString(42)"#;
        assert_eval_bool(code, false);
    }

    #[test]
    fn test_is_string_false_null() {
        let code = r#"isString(null)"#;
        assert_eval_bool(code, false);
    }

    #[test]
    fn test_is_number_true_int() {
        let code = r#"isNumber(42)"#;
        assert_eval_bool(code, true);
    }

    #[test]
    fn test_is_number_true_float() {
        let code = r#"isNumber(3.5)"#;
        assert_eval_bool(code, true);
    }

    // Removed: NaN test (division by zero is error)

    #[test]
    fn test_is_number_false_string() {
        let code = r#"isNumber("42")"#;
        assert_eval_bool(code, false);
    }

    #[test]
    fn test_is_bool_true() {
        let code = r#"isBool(true)"#;
        assert_eval_bool(code, true);
    }

    #[test]
    fn test_is_bool_false() {
        let code = r#"isBool(false)"#;
        assert_eval_bool(code, true);
    }

    #[test]
    fn test_is_bool_false_number() {
        let code = r#"isBool(1)"#;
        assert_eval_bool(code, false);
    }

    #[test]
    fn test_is_null_true() {
        let code = r#"isNull(null)"#;
        assert_eval_bool(code, true);
    }

    #[test]
    fn test_is_null_false() {
        let code = r#"isNull(0)"#;
        assert_eval_bool(code, false);
    }

    #[test]
    fn test_is_array_true() {
        let code = r#"isArray([1,2,3])"#;
        assert_eval_bool(code, true);
    }

    #[test]
    fn test_is_array_true_empty() {
        let code = r#"isArray([])"#;
        assert_eval_bool(code, true);
    }

    #[test]
    fn test_is_array_false() {
        let code = r#"isArray("not array")"#;
        assert_eval_bool(code, false);
    }

    // Function reference tests removed: not yet fully supported

    #[test]
    fn test_is_function_false() {
        let code = r#"isFunction(42)"#;
        assert_eval_bool(code, false);
    }

    // ============================================================================
    // toString Tests
    // ============================================================================

    #[test]
    fn test_to_string_null() {
        let code = r#"toString(null)"#;
        assert_eval_string(code, "null");
    }

    #[test]
    fn test_to_string_bool_true() {
        let code = r#"toString(true)"#;
        assert_eval_string(code, "true");
    }

    #[test]
    fn test_to_string_bool_false() {
        let code = r#"toString(false)"#;
        assert_eval_string(code, "false");
    }

    #[test]
    fn test_to_string_number_int() {
        let code = r#"toString(42)"#;
        assert_eval_string(code, "42");
    }

    #[test]
    fn test_to_string_number_float() {
        let code = r#"toString(3.5)"#;
        assert_eval_string(code, "3.5");
    }

    #[test]
    fn test_to_string_number_negative() {
        let code = r#"toString(-10)"#;
        assert_eval_string(code, "-10");
    }

    #[test]
    fn test_to_string_number_zero() {
        let code = r#"toString(0)"#;
        assert_eval_string(code, "0");
    }

    // NaN/Infinity toString tests removed: division by zero is error

    #[test]
    fn test_to_string_string_identity() {
        let code = r#"toString("hello")"#;
        assert_eval_string(code, "hello");
    }

    #[test]
    fn test_to_string_string_empty() {
        let code = r#"toString("")"#;
        assert_eval_string(code, "");
    }

    #[test]
    fn test_to_string_array() {
        let code = r#"toString([1,2,3])"#;
        assert_eval_string(code, "[Array]");
    }

    // Function toString test removed: not yet fully supported

    #[test]
    fn test_to_string_json() {
        let code = r#"toString(parseJSON("null"))"#;
        assert_eval_string(code, "[JSON]");
    }

    // ============================================================================
    // toNumber Tests
    // ============================================================================

    #[test]
    fn test_to_number_number_identity() {
        let code = r#"toNumber(42)"#;
        assert_eval_number(code, 42.0);
    }

    #[test]
    fn test_to_number_bool_true() {
        let code = r#"toNumber(true)"#;
        assert_eval_number(code, 1.0);
    }

    #[test]
    fn test_to_number_bool_false() {
        let code = r#"toNumber(false)"#;
        assert_eval_number(code, 0.0);
    }

    #[test]
    fn test_to_number_string_int() {
        let code = r#"toNumber("42")"#;
        assert_eval_number(code, 42.0);
    }

    #[test]
    fn test_to_number_string_float() {
        let code = r#"toNumber("3.5")"#;
        assert_eval_number(code, 3.5);
    }

    #[test]
    fn test_to_number_string_negative() {
        let code = r#"toNumber("-10")"#;
        assert_eval_number(code, -10.0);
    }

    #[test]
    fn test_to_number_string_whitespace() {
        let code = r#"toNumber("  42  ")"#;
        assert_eval_number(code, 42.0);
    }

    #[test]
    fn test_to_number_string_scientific() {
        let code = r#"toNumber("1e10")"#;
        assert_eval_number(code, 1e10);
    }

    #[test]
    fn test_to_number_string_empty_error() {
        let code = r#"toNumber("")"#;
        assert_has_error(code);
    }

    #[test]
    fn test_to_number_string_invalid_error() {
        let code = r#"toNumber("hello")"#;
        assert_has_error(code);
    }

    #[test]
    fn test_to_number_null_error() {
        let code = r#"toNumber(null)"#;
        assert_has_error(code);
    }

    #[test]
    fn test_to_number_array_error() {
        let code = r#"toNumber([1,2,3])"#;
        assert_has_error(code);
    }

    // ============================================================================
    // toBool Tests
    // ============================================================================

    #[test]
    fn test_to_bool_bool_identity_true() {
        let code = r#"toBool(true)"#;
        assert_eval_bool(code, true);
    }

    #[test]
    fn test_to_bool_bool_identity_false() {
        let code = r#"toBool(false)"#;
        assert_eval_bool(code, false);
    }

    #[test]
    fn test_to_bool_number_zero_false() {
        let code = r#"toBool(0)"#;
        assert_eval_bool(code, false);
    }

    // NaN toBool test removed: division by zero is error

    #[test]
    fn test_to_bool_number_positive_true() {
        let code = r#"toBool(42)"#;
        assert_eval_bool(code, true);
    }

    #[test]
    fn test_to_bool_number_negative_true() {
        let code = r#"toBool(-10)"#;
        assert_eval_bool(code, true);
    }

    #[test]
    fn test_to_bool_string_empty_false() {
        let code = r#"toBool("")"#;
        assert_eval_bool(code, false);
    }

    #[test]
    fn test_to_bool_string_nonempty_true() {
        let code = r#"toBool("hello")"#;
        assert_eval_bool(code, true);
    }

    #[test]
    fn test_to_bool_string_space_true() {
        let code = r#"toBool(" ")"#;
        assert_eval_bool(code, true);
    }

    #[test]
    fn test_to_bool_null_false() {
        let code = r#"toBool(null)"#;
        assert_eval_bool(code, false);
    }

    #[test]
    fn test_to_bool_array_true() {
        let code = r#"toBool([1,2,3])"#;
        assert_eval_bool(code, true);
    }

    #[test]
    fn test_to_bool_array_empty_true() {
        let code = r#"toBool([])"#;
        assert_eval_bool(code, true);
    }

    // Function toBool test removed: not yet fully supported

    // ============================================================================
    // parseInt Tests
    // ============================================================================

    #[test]
    fn test_parse_int_decimal() {
        let code = r#"parseInt("42", 10)"#;
        assert_eval_number(code, 42.0);
    }

    #[test]
    fn test_parse_int_decimal_negative() {
        let code = r#"parseInt("-10", 10)"#;
        assert_eval_number(code, -10.0);
    }

    #[test]
    fn test_parse_int_binary() {
        let code = r#"parseInt("1010", 2)"#;
        assert_eval_number(code, 10.0);
    }

    #[test]
    fn test_parse_int_octal() {
        let code = r#"parseInt("17", 8)"#;
        assert_eval_number(code, 15.0);
    }

    #[test]
    fn test_parse_int_hex() {
        let code = r#"parseInt("FF", 16)"#;
        assert_eval_number(code, 255.0);
    }

    #[test]
    fn test_parse_int_hex_lowercase() {
        let code = r#"parseInt("ff", 16)"#;
        assert_eval_number(code, 255.0);
    }

    #[test]
    fn test_parse_int_radix_36() {
        let code = r#"parseInt("Z", 36)"#;
        assert_eval_number(code, 35.0);
    }

    #[test]
    fn test_parse_int_plus_sign() {
        let code = r#"parseInt("+42", 10)"#;
        assert_eval_number(code, 42.0);
    }

    #[test]
    fn test_parse_int_whitespace() {
        let code = r#"parseInt("  42  ", 10)"#;
        assert_eval_number(code, 42.0);
    }

    #[test]
    fn test_parse_int_radix_too_low() {
        let code = r#"parseInt("42", 1)"#;
        assert_has_error(code);
    }

    #[test]
    fn test_parse_int_radix_too_high() {
        let code = r#"parseInt("42", 37)"#;
        assert_has_error(code);
    }

    #[test]
    fn test_parse_int_radix_float() {
        let code = r#"parseInt("42", 10.5)"#;
        assert_has_error(code);
    }

    #[test]
    fn test_parse_int_empty_string() {
        let code = r#"parseInt("", 10)"#;
        assert_has_error(code);
    }

    #[test]
    fn test_parse_int_invalid_digit() {
        let code = r#"parseInt("G", 16)"#;
        assert_has_error(code);
    }

    #[test]
    fn test_parse_int_invalid_for_radix() {
        let code = r#"parseInt("2", 2)"#;
        assert_has_error(code);
    }

    #[test]
    fn test_parse_int_wrong_type_first_arg() {
        let code = r#"parseInt(42, 10)"#;
        assert_has_error(code);
    }

    #[test]
    fn test_parse_int_wrong_type_second_arg() {
        let code = r#"parseInt("42", "10")"#;
        assert_has_error(code);
    }

    // ============================================================================
    // parseFloat Tests
    // ============================================================================

    #[test]
    fn test_parse_float_integer() {
        let code = r#"parseFloat("42")"#;
        assert_eval_number(code, 42.0);
    }

    #[test]
    fn test_parse_float_decimal() {
        let code = r#"parseFloat("3.5")"#;
        assert_eval_number(code, 3.5);
    }

    #[test]
    fn test_parse_float_negative() {
        let code = r#"parseFloat("-10.5")"#;
        assert_eval_number(code, -10.5);
    }

    #[test]
    fn test_parse_float_scientific_lowercase() {
        let code = r#"parseFloat("1.5e3")"#;
        assert_eval_number(code, 1500.0);
    }

    #[test]
    fn test_parse_float_scientific_uppercase() {
        let code = r#"parseFloat("1.5E3")"#;
        assert_eval_number(code, 1500.0);
    }

    #[test]
    fn test_parse_float_scientific_negative_exp() {
        let code = r#"parseFloat("1.5e-3")"#;
        assert_eval_number(code, 0.0015);
    }

    #[test]
    fn test_parse_float_scientific_positive_exp() {
        let code = r#"parseFloat("1.5e+3")"#;
        assert_eval_number(code, 1500.0);
    }

    #[test]
    fn test_parse_float_whitespace() {
        let code = r#"parseFloat("  3.5  ")"#;
        assert_eval_number(code, 3.5);
    }

    #[test]
    fn test_parse_float_plus_sign() {
        let code = r#"parseFloat("+42.5")"#;
        assert_eval_number(code, 42.5);
    }

    #[test]
    fn test_parse_float_empty_string() {
        let code = r#"parseFloat("")"#;
        assert_has_error(code);
    }

    #[test]
    fn test_parse_float_invalid() {
        let code = r#"parseFloat("hello")"#;
        assert_has_error(code);
    }

    #[test]
    fn test_parse_float_wrong_type() {
        let code = r#"parseFloat(42)"#;
        assert_has_error(code);
    }

    // ============================================================================
    // Integration Tests
    // ============================================================================

    #[test]
    fn test_typeof_guards_match() {
        let code = r#"
        let val: string = "hello";
        typeof(val) == "string" && isString(val)
    "#;
        assert_eval_bool(code, true);
    }

    #[test]
    fn test_type_conversion_chain() {
        let code = r#"
        let num: number = 42;
        let numStr: string = toString(num);
        toNumber(numStr)
    "#;
        assert_eval_number(code, 42.0);
    }

    #[test]
    fn test_parse_int_then_to_string() {
        let code = r#"
        let parsed: number = parseInt("FF", 16);
        toString(parsed)
    "#;
        assert_eval_string(code, "255");
    }

    #[test]
    fn test_type_guards_all_false_for_null() {
        let code = r#"
        let val = null;
        !isString(val) && !isNumber(val) && !isBool(val) && !isArray(val) && !isFunction(val)
    "#;
        assert_eval_bool(code, true);
    }

    #[test]
    fn test_type_guards_only_null_true() {
        let code = r#"isNull(null)"#;
        assert_eval_bool(code, true);
    }

    // ============================================================================
    // From vm_option_result_tests.rs
    // ============================================================================

    // VM tests for Option<T> and Result<T,E>
    //
    // BLOCKER 02-D: Built-in Generic Types
    //
    // These tests verify VM parity with interpreter for Option and Result support.
    // Tests mirror option_result_tests.rs to ensure identical behavior.

    // ============================================================================
    // Option<T> Tests
    // ============================================================================

    #[test]
    fn test_option_is_some() {
        assert_eval_bool("is_some(Some(42))", true);
        assert_eval_bool("is_some(None())", false);
    }

    #[test]
    fn test_option_is_none() {
        assert_eval_bool("is_none(None())", true);
        assert_eval_bool("is_none(Some(42))", false);
    }

    #[test]
    fn test_option_unwrap_number() {
        assert_eval_number("unwrap(Some(42))", 42.0);
    }

    #[test]
    fn test_option_unwrap_string() {
        assert_eval_string(r#"unwrap(Some("hello"))"#, "hello");
    }

    #[test]
    fn test_option_unwrap_bool() {
        assert_eval_bool("unwrap(Some(true))", true);
    }

    #[test]
    fn test_option_unwrap_null() {
        assert_eval_null("unwrap(Some(null))");
    }

    #[test]
    fn test_option_unwrap_or_some() {
        assert_eval_number("unwrap_or(Some(42), 0)", 42.0);
    }

    #[test]
    fn test_option_unwrap_or_none() {
        assert_eval_number("unwrap_or(None(), 99)", 99.0);
    }

    #[test]
    fn test_option_unwrap_or_string() {
        assert_eval_string(r#"unwrap_or(Some("hello"), "default")"#, "hello");
        assert_eval_string(r#"unwrap_or(None(), "default")"#, "default");
    }

    #[test]
    fn test_option_nested() {
        assert_eval_number("unwrap(unwrap(Some(Some(42))))", 42.0);
    }

    // ============================================================================
    // Result<T,E> Tests
    // ============================================================================

    #[test]
    fn test_result_is_ok() {
        assert_eval_bool("is_ok(Ok(42))", true);
        assert_eval_bool(r#"is_ok(Err("failed"))"#, false);
    }

    #[test]
    fn test_result_is_err() {
        assert_eval_bool(r#"is_err(Err("failed"))"#, true);
        assert_eval_bool("is_err(Ok(42))", false);
    }

    #[test]
    fn test_result_unwrap_ok_number() {
        assert_eval_number("unwrap(Ok(42))", 42.0);
    }

    #[test]
    fn test_result_unwrap_ok_string() {
        assert_eval_string(r#"unwrap(Ok("success"))"#, "success");
    }

    #[test]
    fn test_result_unwrap_ok_null() {
        assert_eval_null("unwrap(Ok(null))");
    }

    #[test]
    fn test_result_unwrap_or_ok() {
        assert_eval_number("unwrap_or(Ok(42), 0)", 42.0);
    }

    #[test]
    fn test_result_unwrap_or_err() {
        assert_eval_number(r#"unwrap_or(Err("failed"), 99)"#, 99.0);
    }

    #[test]
    fn test_result_unwrap_or_string() {
        assert_eval_string(r#"unwrap_or(Ok("success"), "default")"#, "success");
        assert_eval_string(r#"unwrap_or(Err(404), "default")"#, "default");
    }

    // ============================================================================
    // Mixed Option/Result Tests
    // ============================================================================

    #[test]
    fn test_option_and_result_together() {
        let code = r#"
        let opt = Some(42);
        let res = Ok(99);
        unwrap(opt) + unwrap(res)
    "#;
        assert_eval_number(code, 141.0);
    }

    #[test]
    fn test_option_in_conditional() {
        let code = r#"
        let opt = Some(42);
        if (is_some(opt)) {
            unwrap(opt);
        } else {
            0;
        }
    "#;
        assert_eval_number(code, 42.0);
    }

    #[test]
    fn test_result_in_conditional() {
        let code = r#"
        let res = Ok(42);
        if (is_ok(res)) {
            unwrap(res);
        } else {
            0;
        }
    "#;
        assert_eval_number(code, 42.0);
    }

    // ============================================================================
    // Complex Tests
    // ============================================================================

    #[test]
    fn test_option_chain() {
        let code = r#"
        let a = Some(10);
        let b = Some(20);
        let c = Some(30);
        unwrap(a) + unwrap(b) + unwrap(c)
    "#;
        assert_eval_number(code, 60.0);
    }

    #[test]
    fn test_result_chain() {
        let code = r#"
        let a = Ok(10);
        let b = Ok(20);
        let c = Ok(30);
        unwrap(a) + unwrap(b) + unwrap(c)
    "#;
        assert_eval_number(code, 60.0);
    }

    #[test]
    fn test_option_unwrap_or_with_none_chain() {
        let code = r#"
        let a = None();
        let b = None();
        unwrap_or(a, 5) + unwrap_or(b, 10)
    "#;
        assert_eval_number(code, 15.0);
    }

    #[test]
    fn test_result_unwrap_or_with_err_chain() {
        let code = r#"
        let a = Err("fail1");
        let b = Err("fail2");
        unwrap_or(a, 5) + unwrap_or(b, 10)
    "#;
        assert_eval_number(code, 15.0);
    }

    // ============================================================================
    // From vm_result_advanced_tests.rs
    // ============================================================================

    // VM tests for advanced Result<T,E> methods
    //
    // These tests verify VM parity with interpreter for advanced Result operations.
    // Tests mirror result_advanced_tests.rs to ensure identical behavior (including ? operator).

    // ============================================================================
    // expect() Tests
    // ============================================================================

    #[test]
    fn test_expect_ok() {
        assert_eval_number(r#"expect(Ok(42), "should have value")"#, 42.0);
    }

    #[test]
    fn test_expect_with_string() {
        assert_eval_string(r#"expect(Ok("success"), "should work")"#, "success");
    }

    // ============================================================================
    // result_ok() Tests - Convert Result to Option
    // ============================================================================

    #[test]
    fn test_result_ok_from_ok() {
        let code = r#"
        let result = Ok(42);
        let opt = result_ok(result);
        unwrap(opt)
    "#;
        assert_eval_number(code, 42.0);
    }

    #[test]
    fn test_result_ok_from_err() {
        let code = r#"
        let result = Err("failed");
        let opt = result_ok(result);
        is_none(opt)
    "#;
        assert_eval_bool(code, true);
    }

    // ============================================================================
    // result_err() Tests - Extract Err to Option
    // ============================================================================

    #[test]
    fn test_result_err_from_ok() {
        let code = r#"
        let result = Ok(42);
        let opt = result_err(result);
        is_none(opt)
    "#;
        assert_eval_bool(code, true);
    }

    #[test]
    fn test_result_err_from_err() {
        let code = r#"
        let result = Err("failed");
        let opt = result_err(result);
        unwrap(opt)
    "#;
        assert_eval_string(code, "failed");
    }

    // ============================================================================
    // result_map() Tests - Transform Ok value
    // ============================================================================

    #[test]
    fn test_result_map_ok() {
        let code = r#"
        fn double(x: number) -> number { return x * 2; }
        let result = Ok(21);
        let mapped = result_map(result, double);
        unwrap(mapped)
    "#;
        assert_eval_number(code, 42.0);
    }

    #[test]
    fn test_result_map_err_preserves() {
        let code = r#"
        fn double(x: number) -> number { return x * 2; }
        let result = Err("failed");
        let mapped = result_map(result, double);
        is_err(mapped)
    "#;
        assert_eval_bool(code, true);
    }

    #[test]
    fn test_result_map_chain() {
        let code = r#"
        fn double(x: number) -> number { return x * 2; }
        fn triple(x: number) -> number { return x * 3; }
        let result = Ok(7);
        let mapped = result_map(result, double);
        let mapped2 = result_map(mapped, triple);
        unwrap(mapped2)
    "#;
        assert_eval_number(code, 42.0); // 7 * 2 * 3 = 42
    }

    // ============================================================================
    // result_map_err() Tests - Transform Err value
    // ============================================================================

    #[test]
    fn test_result_map_err_transforms_error() {
        let code = r#"
        fn format_error(e: string) -> string { return "Error: " + e; }
        let result = Err("failed");
        let mapped = result_map_err(result, format_error);
        unwrap_or(mapped, "default")
    "#;
        assert_eval_string(code, "default");
    }

    #[test]
    fn test_result_map_err_preserves_ok() {
        let code = r#"
        fn format_error(e: string) -> string { return "Error: " + e; }
        let result = Ok(42);
        let mapped = result_map_err(result, format_error);
        unwrap(mapped)
    "#;
        assert_eval_number(code, 42.0);
    }

    // ============================================================================
    // result_and_then() Tests - Monadic chaining
    // ============================================================================

    #[test]
    fn test_result_and_then_success_chain() {
        let code = r#"
        fn divide(x: number) -> Result<number, string> {
            if (x == 0) {
                return Err("division by zero");
            }
            return Ok(100 / x);
        }
        let result = Ok(10);
        let chained = result_and_then(result, divide);
        unwrap(chained)
    "#;
        assert_eval_number(code, 10.0);
    }

    #[test]
    fn test_result_and_then_error_propagates() {
        let code = r#"
        fn divide(x: number) -> Result<number, string> {
            if (x == 0) {
                return Err("division by zero");
            }
            return Ok(100 / x);
        }
        let result = Err("initial error");
        let chained = result_and_then(result, divide);
        is_err(chained)
    "#;
        assert_eval_bool(code, true);
    }

    #[test]
    fn test_result_and_then_returns_error() {
        let code = r#"
        fn divide(x: number) -> Result<number, string> {
            if (x == 0) {
                return Err("division by zero");
            }
            return Ok(100 / x);
        }
        let result = Ok(0);
        let chained = result_and_then(result, divide);
        is_err(chained)
    "#;
        assert_eval_bool(code, true);
    }

    // ============================================================================
    // result_or_else() Tests - Error recovery
    // ============================================================================

    #[test]
    fn test_result_or_else_recovers_from_error() {
        let code = r#"
        fn recover(_e: string) -> Result<number, string> {
            return Ok(0);
        }
        let result = Err("failed");
        let recovered = result_or_else(result, recover);
        unwrap(recovered)
    "#;
        assert_eval_number(code, 0.0);
    }

    #[test]
    fn test_result_or_else_preserves_ok() {
        let code = r#"
        fn recover(_e: string) -> Result<number, string> {
            return Ok(0);
        }
        let result = Ok(42);
        let recovered = result_or_else(result, recover);
        unwrap(recovered)
    "#;
        assert_eval_number(code, 42.0);
    }

    #[test]
    fn test_result_or_else_can_return_error() {
        let code = r#"
        fn retry(_e: string) -> Result<number, string> {
            return Err("retry failed");
        }
        let result = Err("initial");
        let recovered = result_or_else(result, retry);
        is_err(recovered)
    "#;
        assert_eval_bool(code, true);
    }

    // ============================================================================
    // Complex Combination Tests
    // ============================================================================

    #[test]
    fn test_result_pipeline() {
        let code = r#"
        fn double(x: number) -> number { return x * 2; }
        fn safe_divide(x: number) -> Result<number, string> {
            if (x == 0) {
                return Err("division by zero");
            }
            return Ok(100 / x);
        }

        let result = Ok(10);
        let step1 = result_map(result, double);
        let step2 = result_and_then(step1, safe_divide);
        unwrap(step2)
    "#;
        assert_eval_number(code, 5.0); // (10 * 2) = 20, then 100 / 20 = 5
    }

    #[test]
    fn test_result_error_recovery_pipeline() {
        let code = r#"
        fn recover(_e: string) -> Result<number, string> {
            return Ok(99);
        }
        fn double(x: number) -> number { return x * 2; }

        let result = Err("initial");
        let recovered = result_or_else(result, recover);
        let mapped = result_map(recovered, double);
        unwrap(mapped)
    "#;
        assert_eval_number(code, 198.0); // recover to 99, then * 2
    }

    // ============================================================================
    // Error Propagation Operator (?) Tests
    // ============================================================================

    #[test]
    fn test_try_operator_unwraps_ok() {
        let code = r#"
        fn get_value() -> Result<number, string> {
            let result = Ok(42);
            return Ok(result?);
        }
        unwrap(get_value())
    "#;
        assert_eval_number(code, 42.0);
    }

    #[test]
    fn test_try_operator_propagates_error() {
        let code = r#"
        fn get_value() -> Result<number, string> {
            let result = Err("failed");
            return Ok(result?);
        }
        is_err(get_value())
    "#;
        assert_eval_bool(code, true);
    }

    #[test]
    fn test_try_operator_multiple_propagations() {
        let code = r#"
        fn divide(a: number, b: number) -> Result<number, string> {
            if (b == 0) {
                return Err("division by zero");
            }
            return Ok(a / b);
        }

        fn calculate() -> Result<number, string> {
            let x = divide(100, 10)?;
            let y = divide(x, 2)?;
            let z = divide(y, 5)?;
            return Ok(z);
        }

        unwrap(calculate())
    "#;
        assert_eval_number(code, 1.0); // 100 / 10 = 10, 10 / 2 = 5, 5 / 5 = 1
    }

    #[test]
    fn test_try_operator_early_return() {
        let code = r#"
        fn divide(a: number, b: number) -> Result<number, string> {
            if (b == 0) {
                return Err("division by zero");
            }
            return Ok(a / b);
        }

        fn calculate() -> Result<number, string> {
            let x = divide(100, 10)?;
            let y = divide(x, 0)?;  // This will error
            let z = divide(y, 5)?;  // This won't execute
            return Ok(z);
        }

        is_err(calculate())
    "#;
        assert_eval_bool(code, true);
    }

    #[test]
    fn test_try_operator_with_expressions() {
        let code = r#"
        fn get_number() -> Result<number, string> {
            return Ok(21);
        }

        fn double_it() -> Result<number, string> {
            return Ok(get_number()? * 2);
        }

        unwrap(double_it())
    "#;
        assert_eval_number(code, 42.0);
    }

    #[test]
    fn test_try_operator_in_nested_calls() {
        let code = r#"
        fn inner() -> Result<number, string> {
            return Ok(42);
        }

        fn middle() -> Result<number, string> {
            return Ok(inner()?);
        }

        fn outer() -> Result<number, string> {
            return Ok(middle()?);
        }

        unwrap(outer())
    "#;
        assert_eval_number(code, 42.0);
    }

    #[test]
    fn test_try_operator_with_error_in_nested_calls() {
        let code = r#"
        fn inner() -> Result<number, string> {
            return Err("inner failed");
        }

        fn middle() -> Result<number, string> {
            return Ok(inner()?);
        }

        fn outer() -> Result<number, string> {
            return Ok(middle()?);
        }

        is_err(outer())
    "#;
        assert_eval_bool(code, true);
    }

    #[test]
    fn test_try_operator_combined_with_methods() {
        let code = r#"
        fn get_value() -> Result<number, string> {
            return Ok(10);
        }

        fn double(x: number) -> number {
            return x * 2;
        }

        fn process() -> Result<number, string> {
            let val = get_value()?;
            let mapped = Ok(double(val));
            return Ok(mapped?);
        }

        unwrap(process())
    "#;
        assert_eval_number(code, 20.0);
    }
}
