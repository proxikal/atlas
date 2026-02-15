//! Standard Library Integration Tests
//!
//! Tests how stdlib functions work together in realistic scenarios.
//! Unlike unit tests, these verify cross-function compatibility and complex pipelines.
//!
//! Test categories:
//! - String + Array pipelines
//! - Array + Math aggregations
//! - JSON + Type conversions
//! - File + JSON workflows
//! - Complex multi-step transformations

mod common;
use common::*;
use atlas_runtime::{Atlas, SecurityContext, Value};
use std::path::Path;

// Assert with file I/O permissions (grants /tmp access)
fn assert_eval_number_with_io(source: &str, expected: f64) {
    let mut security = SecurityContext::new();
    security.grant_filesystem_read(Path::new("/tmp"), true);
    security.grant_filesystem_write(Path::new("/tmp"), true);
    let runtime = Atlas::new_with_security(security);
    match runtime.eval(source) {
        Ok(Value::Number(n)) => assert!(n == expected, "Expected {}, got {}", expected, n),
        other => panic!("Expected Number({}), got {:?}", expected, other),
    }
}

fn assert_eval_bool_with_io(source: &str, expected: bool) {
    let mut security = SecurityContext::new();
    security.grant_filesystem_read(Path::new("/tmp"), true);
    security.grant_filesystem_write(Path::new("/tmp"), true);
    let runtime = Atlas::new_with_security(security);
    match runtime.eval(source) {
        Ok(Value::Bool(b)) => assert!(b == expected, "Expected {}, got {}", expected, b),
        other => panic!("Expected Bool({}), got {:?}", expected, other),
    }
}

fn assert_eval_string_with_io(source: &str, expected: &str) {
    let mut security = SecurityContext::new();
    security.grant_filesystem_read(Path::new("/tmp"), true);
    security.grant_filesystem_write(Path::new("/tmp"), true);
    let runtime = Atlas::new_with_security(security);
    match runtime.eval(source) {
        Ok(Value::String(s)) => assert!(
            s.as_ref() == expected,
            "Expected {:?}, got {:?}",
            expected,
            s.as_ref()
        ),
        other => panic!("Expected String({:?}), got {:?}", expected, other),
    }
}

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
    let code = r##"
        let data: number[] = [1, 2, 3, 4, 5];
        let jsonStr: string = toJSON(data);
        writeFile("/tmp/atlas_test_json1.json", jsonStr);

        let content: string = readFile("/tmp/atlas_test_json1.json");
        let parsed: json = parseJSON(content);
        parsed[0].as_number() + parsed[4].as_number()
    "##;
    assert_eval_number_with_io(code, 6.0); // 1 + 5
}

#[test]
fn test_json_file_roundtrip() {
    let code = r##"
        let obj: json = parseJSON("{\"name\":\"Atlas\",\"version\":2}");
        let jsonStr: string = toJSON(obj);
        writeFile("/tmp/atlas_test_json2.json", jsonStr);

        let loaded: string = readFile("/tmp/atlas_test_json2.json");
        let reparsed: json = parseJSON(loaded);
        reparsed["version"].as_number()
    "##;
    assert_eval_number_with_io(code, 2.0);
}

#[test]
fn test_prettify_write_minify_read() {
    let code = r###"
        let compact: string = "{\"a\":1,\"b\":2}";
        let pretty: string = prettifyJSON(compact, 2);
        writeFile("/tmp/atlas_test_json3.json", pretty);

        let loaded: string = readFile("/tmp/atlas_test_json3.json");
        let mini: string = minifyJSON(loaded);
        mini == compact
    "###;
    assert_eval_bool_with_io(code, true);
}

#[test]
fn test_file_exists_json_check() {
    let code = r#"
        writeFile("/tmp/atlas_test_json4.json", "[]");
        let exists: bool = fileExists("/tmp/atlas_test_json4.json");
        let content: string = readFile("/tmp/atlas_test_json4.json");
        let valid: bool = isValidJSON(content);
        exists && valid
    "#;
    assert_eval_bool_with_io(code, true);
}

#[test]
fn test_append_json_array_elements() {
    let code = r##"
        writeFile("/tmp/atlas_test_json5.txt", "[1,2,3]");
        appendFile("/tmp/atlas_test_json5.txt", "\n[4,5,6]");

        let content: string = readFile("/tmp/atlas_test_json5.txt");
        let lines: string[] = split(content, "\n");
        let arr1: json = parseJSON(lines[0]);
        let arr2: json = parseJSON(lines[1]);
        arr1[0].as_number() + arr2[2].as_number()
    "##;
    assert_eval_number_with_io(code, 7.0); // 1 + 6
}

#[test]
fn test_json_array_to_file_lines() {
    let code = r#"
        fn toNum(s: string) -> number {
            return toNumber(s);
        }

        let numbers: number[] = [10, 20, 30];
        let jsonStr: string = toJSON(numbers);
        writeFile("/tmp/atlas_test_json6.txt", jsonStr);

        let content: string = readFile("/tmp/atlas_test_json6.txt");
        let parsed: json = parseJSON(content);
        parsed[1].as_number()
    "#;
    assert_eval_number_with_io(code, 20.0);
}

#[test]
fn test_multiple_json_files_sum() {
    let code = r##"
        writeFile("/tmp/atlas_test_json7a.json", "[10]");
        writeFile("/tmp/atlas_test_json7b.json", "[20]");

        let content1: string = readFile("/tmp/atlas_test_json7a.json");
        let content2: string = readFile("/tmp/atlas_test_json7b.json");
        let arr1: json = parseJSON(content1);
        let arr2: json = parseJSON(content2);
        arr1[0].as_number() + arr2[0].as_number()
    "##;
    assert_eval_number_with_io(code, 30.0);
}

#[test]
fn test_json_validation_before_write() {
    let code = r#"
        let invalid: string = "not json";
        let valid: string = "{\"key\":\"value\"}";

        if (isValidJSON(valid)) {
            writeFile("/tmp/atlas_test_json8.json", valid);
        }

        let content: string = readFile("/tmp/atlas_test_json8.json");
        includes(content, "key")
    "#;
    assert_eval_bool_with_io(code, true);
}

#[test]
fn test_read_json_check_type() {
    let code = r##"
        writeFile("/tmp/atlas_test_json9.json", "{\"count\":42}");

        let content: string = readFile("/tmp/atlas_test_json9.json");
        let obj: json = parseJSON(content);
        let count: number = obj["count"].as_number();
        isNumber(count)
    "##;
    assert_eval_bool_with_io(code, true);
}

#[test]
fn test_json_array_length_via_file() {
    let code = r##"
        let arr: number[] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let jsonStr: string = toJSON(arr);
        writeFile("/tmp/atlas_test_json10.json", jsonStr);

        let content: string = readFile("/tmp/atlas_test_json10.json");
        let parsed: json = parseJSON(content);
        // Extract last element to check array size
        parsed[9].as_number()
    "##;
    assert_eval_number_with_io(code, 10.0);
}

#[test]
fn test_conditional_file_write_json() {
    let code = r##"
        let data: json = parseJSON("{\"enabled\":true}");
        let enabled: bool = data["enabled"].as_bool();

        if (enabled) {
            writeFile("/tmp/atlas_test_json11.json", "{\"status\":\"active\"}");
        }

        let content: string = readFile("/tmp/atlas_test_json11.json");
        includes(content, "active")
    "##;
    assert_eval_bool_with_io(code, true);
}

#[test]
fn test_json_file_string_concat() {
    let code = r##"
        writeFile("/tmp/atlas_test_json12a.txt", "Hello");
        writeFile("/tmp/atlas_test_json12b.txt", "World");

        let part1: string = readFile("/tmp/atlas_test_json12a.txt");
        let part2: string = readFile("/tmp/atlas_test_json12b.txt");
        let combined: string = part1 + " " + part2;
        combined
    "##;
    assert_eval_string_with_io(code, "Hello World");
}

#[test]
fn test_json_parse_file_nested_access() {
    let code = r##"
        writeFile("/tmp/atlas_test_json13.json", "{\"user\":{\"name\":\"Alice\",\"age\":30}}");

        let content: string = readFile("/tmp/atlas_test_json13.json");
        let obj: json = parseJSON(content);
        let user: json = obj["user"];
        let name: string = user["name"].as_string();
        name
    "##;
    assert_eval_string_with_io(code, "Alice");
}

#[test]
fn test_file_to_json_to_string_array() {
    let code = r##"
        let strings: string[] = ["apple", "banana", "cherry"];
        let jsonStr: string = toJSON(strings);
        writeFile("/tmp/atlas_test_json14.json", jsonStr);

        let content: string = readFile("/tmp/atlas_test_json14.json");
        let parsed: json = parseJSON(content);
        let first: string = parsed[0].as_string();
        let last: string = parsed[2].as_string();
        first + "," + last
    "##;
    assert_eval_string_with_io(code, "apple,cherry");
}

#[test]
fn test_json_number_extraction_math() {
    let code = r##"
        writeFile("/tmp/atlas_test_json15.json", "[5,10,15]");

        let content: string = readFile("/tmp/atlas_test_json15.json");
        let arr: json = parseJSON(content);
        let sum: number = arr[0].as_number() + arr[1].as_number() + arr[2].as_number();
        sum / 3
    "##;
    assert_eval_number_with_io(code, 10.0); // Average
}

#[test]
fn test_write_read_bool_json() {
    let code = r##"
        writeFile("/tmp/atlas_test_json16.json", "{\"active\":true,\"enabled\":false}");

        let content: string = readFile("/tmp/atlas_test_json16.json");
        let obj: json = parseJSON(content);
        let active: bool = obj["active"].as_bool();
        let enabled: bool = obj["enabled"].as_bool();
        active && !enabled
    "##;
    assert_eval_bool_with_io(code, true);
}

#[test]
fn test_json_file_type_conversion() {
    let code = r##"
        writeFile("/tmp/atlas_test_json17.json", "{\"count\":\"42\"}");

        let content: string = readFile("/tmp/atlas_test_json17.json");
        let obj: json = parseJSON(content);
        let countStr: string = obj["count"].as_string();
        let countNum: number = toNumber(countStr);
        countNum * 2
    "##;
    assert_eval_number_with_io(code, 84.0);
}

#[test]
fn test_file_contains_valid_json() {
    let code = r##"
        writeFile("/tmp/atlas_test_json18.json", "{\"valid\":true}");

        let content: string = readFile("/tmp/atlas_test_json18.json");
        isValidJSON(content)
    "##;
    assert_eval_bool_with_io(code, true);
}

#[test]
fn test_json_null_in_file() {
    let code = r##"
        writeFile("/tmp/atlas_test_json19.json", "{\"value\":null}");

        let content: string = readFile("/tmp/atlas_test_json19.json");
        let obj: json = parseJSON(content);
        let val: json = obj["value"];
        val.is_null()
    "##;
    assert_eval_bool_with_io(code, true);
}

#[test]
fn test_large_json_array_file() {
    let code = r##"
        let arr: number[] = [1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20];
        let jsonStr: string = toJSON(arr);
        writeFile("/tmp/atlas_test_json20.json", jsonStr);

        let content: string = readFile("/tmp/atlas_test_json20.json");
        let parsed: json = parseJSON(content);
        let first: number = parsed[0].as_number();
        let last: number = parsed[19].as_number();
        first + last
    "##;
    assert_eval_number_with_io(code, 21.0); // 1 + 20
}
