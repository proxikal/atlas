//! VM profiler — comprehensive performance analysis
//!
//! Provides instruction counting, timing, hotspot detection, and formatted
//! reports. The `Profiler` struct is the primary public interface; it wraps
//! the lower-level [`ProfileCollector`], [`HotspotDetector`], and
//! [`ProfileReport`] types.
//!
//! # Quick start
//!
//! ```no_run
//! use atlas_runtime::profiler::Profiler;
//! use atlas_runtime::bytecode::Opcode;
//!
//! let mut p = Profiler::enabled();
//! p.start_timing();
//! p.record_instruction_at(Opcode::Add, 0);
//! p.record_instruction_at(Opcode::Add, 0);
//! p.record_instruction_at(Opcode::Mul, 3);
//! p.stop_timing();
//! let report = p.generate_report(1.0);
//! println!("{}", report.format_detailed());
//! ```

pub mod collector;
pub mod hotspots;
pub mod report;

pub use collector::ProfileCollector;
pub use hotspots::{HotOpcode, Hotspot, HotspotDetector};
pub use report::ProfileReport;

use crate::bytecode::Opcode;
use std::collections::HashMap;
use std::time::Instant;

/// Complete VM profiler
///
/// Wraps a `ProfileCollector` with optional timing and a report builder.
/// When `enabled` is false all methods are no-ops, so there is zero
/// overhead in production runs.
#[derive(Debug)]
pub struct Profiler {
    /// Whether profiling is active
    enabled: bool,
    /// Data collector
    collector: ProfileCollector,
    /// When timing started
    start_time: Option<Instant>,
    /// Captured elapsed duration in seconds
    elapsed_secs: Option<f64>,
}

impl Profiler {
    /// Create a disabled profiler (zero overhead)
    pub fn new() -> Self {
        Self {
            enabled: false,
            collector: ProfileCollector::new(),
            start_time: None,
            elapsed_secs: None,
        }
    }

    /// Create a profiler that is ready to collect
    pub fn enabled() -> Self {
        Self {
            enabled: true,
            collector: ProfileCollector::new(),
            start_time: None,
            elapsed_secs: None,
        }
    }

    /// Enable collection
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Disable collection (data already collected is preserved)
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Whether profiling is currently enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Start the wall-clock timer
    pub fn start_timing(&mut self) {
        if self.enabled {
            self.start_time = Some(Instant::now());
        }
    }

    /// Stop the wall-clock timer and record elapsed time
    pub fn stop_timing(&mut self) {
        if let Some(start) = self.start_time.take() {
            self.elapsed_secs = Some(start.elapsed().as_secs_f64());
        }
    }

    /// Elapsed time in seconds (None if timing was not started/stopped)
    pub fn elapsed_secs(&self) -> Option<f64> {
        self.elapsed_secs
    }

    /// Reset all collected data and timing
    pub fn reset(&mut self) {
        self.collector.reset();
        self.start_time = None;
        self.elapsed_secs = None;
    }

    // --- Instruction recording ---

    /// Record an instruction execution (with IP for location tracking)
    ///
    /// This is the preferred method; it enables hotspot detection.
    pub fn record_instruction_at(&mut self, opcode: Opcode, ip: usize) {
        if self.enabled {
            self.collector.record_instruction(opcode, ip);
        }
    }

    /// Record an instruction without location info (backward compat)
    ///
    /// Used by existing VM code that does not yet pass the IP.
    pub fn record_instruction(&mut self, opcode: Opcode) {
        if self.enabled {
            self.collector.record_instruction_opcode(opcode);
        }
    }

    // --- Stack / call tracking ---

    /// Update observed call frame stack depth
    pub fn update_frame_depth(&mut self, depth: usize) {
        if self.enabled {
            self.collector.update_frame_depth(depth);
        }
    }

    /// Update observed value stack depth
    pub fn update_value_stack_depth(&mut self, depth: usize) {
        if self.enabled {
            self.collector.update_value_stack_depth(depth);
        }
    }

    /// Record a named function call
    pub fn record_function_call(&mut self, name: &str) {
        if self.enabled {
            self.collector.record_function_call(name);
        }
    }

    // --- Basic accessors (backward compat with vm/profiler.rs API) ---

    /// Total instructions executed
    pub fn total_instructions(&self) -> u64 {
        self.collector.total_instructions()
    }

    /// Count for a specific opcode
    pub fn instruction_count(&self, opcode: Opcode) -> u64 {
        self.collector.instruction_count(opcode)
    }

    /// All per-opcode counts (opcode byte → count)
    pub fn instruction_counts(&self) -> &HashMap<u8, u64> {
        self.collector.instruction_counts()
    }

    /// Maximum call frame stack depth observed
    pub fn max_stack_depth(&self) -> usize {
        self.collector.max_stack_depth()
    }

    /// Total named function calls recorded
    pub fn function_calls(&self) -> u64 {
        self.collector.function_calls()
    }

    /// Read-only access to the collector for advanced queries
    pub fn collector(&self) -> &ProfileCollector {
        &self.collector
    }

    // --- Reports ---

    /// Basic text report (backward compat with vm/profiler.rs)
    pub fn report(&self) -> String {
        if !self.enabled {
            return "Profiling not enabled".to_string();
        }

        let total = self.collector.total_instructions();
        let mut out = format!("Total instructions executed: {}\n\n", total);

        if self.collector.instruction_counts().is_empty() {
            out.push_str("No instructions recorded\n");
            return out;
        }

        out.push_str("Instruction counts by opcode:\n");

        let mut counts: Vec<_> = self.collector.instruction_counts().iter().collect();
        counts.sort_by(|a, b| b.1.cmp(a.1));

        for (byte, count) in counts {
            let opcode_name = Opcode::try_from(*byte)
                .map(|op| format!("{:?}", op))
                .unwrap_or_else(|_| format!("Unknown({:#04x})", byte));
            let pct = (*count as f64 / total as f64) * 100.0;
            out.push_str(&format!(
                "  {:<20} {:>10} ({:>6.2}%)\n",
                opcode_name, count, pct
            ));
        }

        out
    }

    /// Generate a comprehensive `ProfileReport`
    ///
    /// `hotspot_threshold` is the minimum percentage (0–100) for a location
    /// to be included in the hotspots section.
    pub fn generate_report(&self, hotspot_threshold: f64) -> ProfileReport {
        let total = self.collector.total_instructions();

        let ips = self.elapsed_secs.and_then(|secs| {
            if secs > 0.0 {
                Some(total as f64 / secs)
            } else {
                None
            }
        });

        let detector = HotspotDetector::with_threshold(hotspot_threshold);
        let hotspots = detector.detect(&self.collector);
        let top_opcodes: Vec<(String, u64, f64)> = detector
            .top_opcodes(&self.collector, 20)
            .into_iter()
            .map(|h| (format!("{:?}", h.opcode), h.count, h.percentage))
            .collect();

        ProfileReport {
            total_instructions: total,
            elapsed_secs: self.elapsed_secs,
            ips,
            max_stack_depth: self.collector.max_stack_depth(),
            max_value_stack_depth: self.collector.max_value_stack_depth(),
            function_calls: self.collector.function_calls(),
            top_opcodes,
            hotspots,
        }
    }

    /// Shorthand: hotspots at default 1% threshold
    pub fn hotspots(&self) -> Vec<Hotspot> {
        HotspotDetector::new().detect(&self.collector)
    }

    /// Shorthand: top N opcodes
    pub fn top_opcodes(&self, n: usize) -> Vec<HotOpcode> {
        HotspotDetector::new().top_opcodes(&self.collector, n)
    }
}

impl Default for Profiler {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for Profiler {
    fn clone(&self) -> Self {
        Self {
            enabled: self.enabled,
            collector: self.collector.clone(),
            start_time: None, // Instant is not Clone-able in a meaningful way
            elapsed_secs: self.elapsed_secs,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bytecode::Opcode;

    // --- Basic enabled/disabled ---

    #[test]
    fn test_new_is_disabled() {
        let p = Profiler::new();
        assert!(!p.is_enabled());
        assert_eq!(p.total_instructions(), 0);
    }

    #[test]
    fn test_enabled_constructor() {
        let p = Profiler::enabled();
        assert!(p.is_enabled());
    }

    #[test]
    fn test_enable_disable() {
        let mut p = Profiler::new();
        p.enable();
        assert!(p.is_enabled());
        p.disable();
        assert!(!p.is_enabled());
    }

    #[test]
    fn test_record_instruction_when_disabled() {
        let mut p = Profiler::new();
        p.record_instruction(Opcode::Add);
        assert_eq!(p.total_instructions(), 0);
    }

    #[test]
    fn test_record_instruction_when_enabled() {
        let mut p = Profiler::enabled();
        p.record_instruction(Opcode::Add);
        p.record_instruction(Opcode::Add);
        p.record_instruction(Opcode::Mul);
        assert_eq!(p.total_instructions(), 3);
        assert_eq!(p.instruction_count(Opcode::Add), 2);
        assert_eq!(p.instruction_count(Opcode::Mul), 1);
        assert_eq!(p.instruction_count(Opcode::Sub), 0);
    }

    #[test]
    fn test_record_instruction_at() {
        let mut p = Profiler::enabled();
        p.record_instruction_at(Opcode::Loop, 100);
        p.record_instruction_at(Opcode::Loop, 100);
        assert_eq!(p.total_instructions(), 2);
        assert_eq!(p.collector().location_counts()[&100], 2);
    }

    // --- Stack / call depth ---

    #[test]
    fn test_update_frame_depth() {
        let mut p = Profiler::enabled();
        p.update_frame_depth(3);
        p.update_frame_depth(7);
        p.update_frame_depth(1);
        assert_eq!(p.max_stack_depth(), 7);
    }

    #[test]
    fn test_update_frame_depth_disabled() {
        let mut p = Profiler::new();
        p.update_frame_depth(10);
        assert_eq!(p.max_stack_depth(), 0);
    }

    #[test]
    fn test_record_function_call() {
        let mut p = Profiler::enabled();
        p.record_function_call("main");
        p.record_function_call("foo");
        p.record_function_call("foo");
        assert_eq!(p.function_calls(), 3);
    }

    #[test]
    fn test_record_function_call_disabled() {
        let mut p = Profiler::new();
        p.record_function_call("main");
        assert_eq!(p.function_calls(), 0);
    }

    // --- Timing ---

    #[test]
    fn test_timing_records_elapsed() {
        let mut p = Profiler::enabled();
        p.start_timing();
        // Do a small amount of work
        for i in 0..100 {
            p.record_instruction_at(Opcode::Add, i);
        }
        p.stop_timing();
        assert!(p.elapsed_secs().is_some());
        assert!(p.elapsed_secs().unwrap() >= 0.0);
    }

    #[test]
    fn test_timing_disabled_no_elapsed() {
        let mut p = Profiler::new();
        p.start_timing();
        p.stop_timing();
        assert!(p.elapsed_secs().is_none());
    }

    // --- Reset ---

    #[test]
    fn test_reset_clears_data() {
        let mut p = Profiler::enabled();
        p.start_timing();
        p.record_instruction(Opcode::Add);
        p.stop_timing();
        p.reset();
        assert_eq!(p.total_instructions(), 0);
        assert!(p.elapsed_secs().is_none());
    }

    // --- Basic report (backward compat) ---

    #[test]
    fn test_report_disabled() {
        let p = Profiler::new();
        assert!(p.report().contains("not enabled"));
    }

    #[test]
    fn test_report_empty() {
        let p = Profiler::enabled();
        assert!(p.report().contains("No instructions recorded"));
    }

    #[test]
    fn test_report_with_data() {
        let mut p = Profiler::enabled();
        p.record_instruction(Opcode::Add);
        p.record_instruction(Opcode::Add);
        p.record_instruction(Opcode::Mul);
        let r = p.report();
        assert!(r.contains("Total instructions executed: 3"));
        assert!(r.contains("Add"));
        assert!(r.contains("Mul"));
        assert!(r.contains("66.67%"));
    }

    // --- generate_report ---

    #[test]
    fn test_generate_report_basic() {
        let mut p = Profiler::enabled();
        for i in 0..100usize {
            p.record_instruction_at(Opcode::Add, i % 10);
        }
        p.update_frame_depth(5);
        p.record_function_call("foo");
        let r = p.generate_report(1.0);
        assert_eq!(r.total_instructions, 100);
        assert_eq!(r.max_stack_depth, 5);
        assert_eq!(r.function_calls, 1);
        assert!(!r.top_opcodes.is_empty());
    }

    #[test]
    fn test_generate_report_hotspots() {
        let mut p = Profiler::enabled();
        // 50 executions at IP 0 (50% of total 100) → hotspot
        for _ in 0..50 {
            p.record_instruction_at(Opcode::Loop, 0);
        }
        for i in 1..=50usize {
            p.record_instruction_at(Opcode::Add, i);
        }
        let r = p.generate_report(1.0);
        // IP 0 is hot (50%)
        assert!(r.hotspots.iter().any(|h| h.ip == 0));
    }

    #[test]
    fn test_generate_report_ips_calculation() {
        let mut p = Profiler::enabled();
        p.start_timing();
        for i in 0..1000usize {
            p.record_instruction_at(Opcode::Add, i % 100);
        }
        p.stop_timing();
        let r = p.generate_report(1.0);
        assert!(r.ips.is_some());
        assert!(r.ips.unwrap() > 0.0);
    }

    #[test]
    fn test_hotspots_shorthand() {
        let mut p = Profiler::enabled();
        for _ in 0..100 {
            p.record_instruction_at(Opcode::Loop, 0);
        }
        assert!(!p.hotspots().is_empty());
    }

    #[test]
    fn test_top_opcodes_shorthand() {
        let mut p = Profiler::enabled();
        for _ in 0..50 {
            p.record_instruction(Opcode::Add);
        }
        for _ in 0..30 {
            p.record_instruction(Opcode::Mul);
        }
        let top = p.top_opcodes(5);
        assert!(!top.is_empty());
        assert_eq!(top[0].opcode, Opcode::Add);
    }

    #[test]
    fn test_default_is_disabled() {
        let p = Profiler::default();
        assert!(!p.is_enabled());
    }

    #[test]
    fn test_clone_preserves_data() {
        let mut p = Profiler::enabled();
        p.record_instruction(Opcode::Add);
        let c = p.clone();
        assert_eq!(c.total_instructions(), 1);
        assert!(c.is_enabled());
    }
}
