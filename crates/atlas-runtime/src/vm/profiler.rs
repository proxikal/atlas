//! VM profiling and instrumentation
//!
//! Provides optional profiling capabilities for performance analysis.
//! Profiling is disabled by default and has zero overhead when not enabled.

use crate::bytecode::Opcode;
use std::collections::HashMap;

/// VM profiler for performance analysis
///
/// Tracks instruction execution counts and provides hooks for
/// custom profiling callbacks. Disabled by default for production use.
#[derive(Debug, Clone)]
pub struct Profiler {
    /// Whether profiling is enabled
    enabled: bool,
    /// Total instructions executed
    total_instructions: u64,
    /// Instructions executed per opcode
    instruction_counts: HashMap<u8, u64>,
}

impl Profiler {
    /// Create a new profiler (disabled by default)
    pub fn new() -> Self {
        Self {
            enabled: false,
            total_instructions: 0,
            instruction_counts: HashMap::new(),
        }
    }

    /// Create a new profiler with profiling enabled
    pub fn enabled() -> Self {
        Self {
            enabled: true,
            total_instructions: 0,
            instruction_counts: HashMap::new(),
        }
    }

    /// Enable profiling
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Disable profiling
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Check if profiling is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Reset profiling statistics
    pub fn reset(&mut self) {
        self.total_instructions = 0;
        self.instruction_counts.clear();
    }

    /// Record an instruction execution
    ///
    /// This is called by the VM for each instruction when profiling is enabled.
    /// Has zero overhead when profiling is disabled (checked at VM level).
    pub fn record_instruction(&mut self, opcode: Opcode) {
        if !self.enabled {
            return;
        }

        self.total_instructions += 1;
        let byte = opcode as u8;
        *self.instruction_counts.entry(byte).or_insert(0) += 1;
    }

    /// Get total instructions executed
    pub fn total_instructions(&self) -> u64 {
        self.total_instructions
    }

    /// Get instruction count for a specific opcode
    pub fn instruction_count(&self, opcode: Opcode) -> u64 {
        let byte = opcode as u8;
        self.instruction_counts.get(&byte).copied().unwrap_or(0)
    }

    /// Get all instruction counts
    ///
    /// Returns a map of opcode byte values to execution counts.
    pub fn instruction_counts(&self) -> &HashMap<u8, u64> {
        &self.instruction_counts
    }

    /// Generate a profiling report
    ///
    /// Returns a formatted string with execution statistics.
    pub fn report(&self) -> String {
        if !self.enabled {
            return "Profiling not enabled".to_string();
        }

        let mut report = String::new();
        report.push_str(&format!(
            "Total instructions executed: {}\n\n",
            self.total_instructions
        ));

        if self.instruction_counts.is_empty() {
            report.push_str("No instructions recorded\n");
            return report;
        }

        report.push_str("Instruction counts by opcode:\n");

        // Sort by count (descending)
        let mut counts: Vec<_> = self.instruction_counts.iter().collect();
        counts.sort_by(|a, b| b.1.cmp(a.1));

        for (byte, count) in counts {
            let opcode = Opcode::try_from(*byte);
            let opcode_name = match opcode {
                Ok(op) => format!("{:?}", op),
                Err(_) => format!("Unknown({:#04x})", byte),
            };

            let percentage = (*count as f64 / self.total_instructions as f64) * 100.0;
            report.push_str(&format!(
                "  {:<20} {:>10} ({:>6.2}%)\n",
                opcode_name, count, percentage
            ));
        }

        report
    }
}

impl Default for Profiler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profiler_new() {
        let profiler = Profiler::new();
        assert!(!profiler.is_enabled());
        assert_eq!(profiler.total_instructions(), 0);
    }

    #[test]
    fn test_profiler_enabled() {
        let profiler = Profiler::enabled();
        assert!(profiler.is_enabled());
        assert_eq!(profiler.total_instructions(), 0);
    }

    #[test]
    fn test_enable_disable() {
        let mut profiler = Profiler::new();
        assert!(!profiler.is_enabled());

        profiler.enable();
        assert!(profiler.is_enabled());

        profiler.disable();
        assert!(!profiler.is_enabled());
    }

    #[test]
    fn test_record_instruction() {
        let mut profiler = Profiler::enabled();

        profiler.record_instruction(Opcode::Add);
        profiler.record_instruction(Opcode::Add);
        profiler.record_instruction(Opcode::Mul);

        assert_eq!(profiler.total_instructions(), 3);
        assert_eq!(profiler.instruction_count(Opcode::Add), 2);
        assert_eq!(profiler.instruction_count(Opcode::Mul), 1);
        assert_eq!(profiler.instruction_count(Opcode::Sub), 0);
    }

    #[test]
    fn test_record_instruction_when_disabled() {
        let mut profiler = Profiler::new(); // disabled by default

        profiler.record_instruction(Opcode::Add);
        profiler.record_instruction(Opcode::Mul);

        // Should not record when disabled
        assert_eq!(profiler.total_instructions(), 0);
        assert_eq!(profiler.instruction_count(Opcode::Add), 0);
    }

    #[test]
    fn test_reset() {
        let mut profiler = Profiler::enabled();

        profiler.record_instruction(Opcode::Add);
        profiler.record_instruction(Opcode::Mul);
        assert_eq!(profiler.total_instructions(), 2);

        profiler.reset();
        assert_eq!(profiler.total_instructions(), 0);
        assert_eq!(profiler.instruction_count(Opcode::Add), 0);
    }

    #[test]
    fn test_instruction_counts() {
        let mut profiler = Profiler::enabled();

        profiler.record_instruction(Opcode::Add);
        profiler.record_instruction(Opcode::Add);
        profiler.record_instruction(Opcode::Mul);

        let counts = profiler.instruction_counts();
        assert_eq!(counts.len(), 2); // Add and Mul
        assert_eq!(counts.get(&(Opcode::Add as u8)), Some(&2));
        assert_eq!(counts.get(&(Opcode::Mul as u8)), Some(&1));
    }

    #[test]
    fn test_report_when_disabled() {
        let profiler = Profiler::new();
        let report = profiler.report();
        assert!(report.contains("not enabled"));
    }

    #[test]
    fn test_report_with_data() {
        let mut profiler = Profiler::enabled();

        profiler.record_instruction(Opcode::Add);
        profiler.record_instruction(Opcode::Add);
        profiler.record_instruction(Opcode::Mul);

        let report = profiler.report();
        assert!(report.contains("Total instructions executed: 3"));
        assert!(report.contains("Add"));
        assert!(report.contains("Mul"));
        assert!(report.contains("66.67%")); // Add: 2/3
        assert!(report.contains("33.33%")); // Mul: 1/3
    }

    #[test]
    fn test_report_empty() {
        let profiler = Profiler::enabled();
        let report = profiler.report();
        assert!(report.contains("No instructions recorded"));
    }

    #[test]
    fn test_default() {
        let profiler = Profiler::default();
        assert!(!profiler.is_enabled());
    }
}
