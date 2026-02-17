//! Profile data collector
//!
//! Records execution statistics during VM runs: instruction counts,
//! per-location hotness, stack depth extremes, and function call counts.

use crate::bytecode::Opcode;
use std::collections::HashMap;

/// Execution statistics collected during a VM run
#[derive(Debug, Clone)]
pub struct ProfileCollector {
    /// Total instructions executed
    total_instructions: u64,
    /// Count per opcode (opcode byte → count)
    instruction_counts: HashMap<u8, u64>,
    /// Count per instruction location (IP → count)
    location_counts: HashMap<usize, u64>,
    /// Opcode recorded at each IP (for hotspot labelling)
    location_opcodes: HashMap<usize, u8>,
    /// Maximum call stack depth observed
    max_stack_depth: usize,
    /// Maximum value stack depth observed
    max_value_stack_depth: usize,
    /// Total function calls recorded
    function_calls: u64,
    /// Calls per named function
    function_call_counts: HashMap<String, u64>,
}

impl ProfileCollector {
    /// Create a new, empty collector
    pub fn new() -> Self {
        Self {
            total_instructions: 0,
            instruction_counts: HashMap::new(),
            location_counts: HashMap::new(),
            location_opcodes: HashMap::new(),
            max_stack_depth: 0,
            max_value_stack_depth: 0,
            function_calls: 0,
            function_call_counts: HashMap::new(),
        }
    }

    /// Record an instruction execution at a specific IP
    pub fn record_instruction(&mut self, opcode: Opcode, ip: usize) {
        self.total_instructions += 1;
        let byte = opcode as u8;
        *self.instruction_counts.entry(byte).or_insert(0) += 1;
        *self.location_counts.entry(ip).or_insert(0) += 1;
        self.location_opcodes.entry(ip).or_insert(byte);
    }

    /// Record an instruction without a specific location (backward compat)
    pub fn record_instruction_opcode(&mut self, opcode: Opcode) {
        self.total_instructions += 1;
        let byte = opcode as u8;
        *self.instruction_counts.entry(byte).or_insert(0) += 1;
    }

    /// Update the observed call stack depth
    pub fn update_frame_depth(&mut self, depth: usize) {
        if depth > self.max_stack_depth {
            self.max_stack_depth = depth;
        }
    }

    /// Update the observed value stack depth
    pub fn update_value_stack_depth(&mut self, depth: usize) {
        if depth > self.max_value_stack_depth {
            self.max_value_stack_depth = depth;
        }
    }

    /// Record a named function call
    pub fn record_function_call(&mut self, name: &str) {
        self.function_calls += 1;
        *self
            .function_call_counts
            .entry(name.to_string())
            .or_insert(0) += 1;
    }

    /// Reset all counters
    pub fn reset(&mut self) {
        self.total_instructions = 0;
        self.instruction_counts.clear();
        self.location_counts.clear();
        self.location_opcodes.clear();
        self.max_stack_depth = 0;
        self.max_value_stack_depth = 0;
        self.function_calls = 0;
        self.function_call_counts.clear();
    }

    // --- Accessors ---

    /// Total instructions executed
    pub fn total_instructions(&self) -> u64 {
        self.total_instructions
    }

    /// Count for a specific opcode
    pub fn instruction_count(&self, opcode: Opcode) -> u64 {
        self.instruction_counts
            .get(&(opcode as u8))
            .copied()
            .unwrap_or(0)
    }

    /// All per-opcode counts (opcode byte → count)
    pub fn instruction_counts(&self) -> &HashMap<u8, u64> {
        &self.instruction_counts
    }

    /// All per-location counts (IP → count)
    pub fn location_counts(&self) -> &HashMap<usize, u64> {
        &self.location_counts
    }

    /// The opcode recorded at a specific IP (if any)
    pub fn opcode_at(&self, ip: usize) -> Option<Opcode> {
        self.location_opcodes
            .get(&ip)
            .and_then(|&b| Opcode::try_from(b).ok())
    }

    /// Maximum call frame stack depth
    pub fn max_stack_depth(&self) -> usize {
        self.max_stack_depth
    }

    /// Maximum value stack depth
    pub fn max_value_stack_depth(&self) -> usize {
        self.max_value_stack_depth
    }

    /// Total named function calls
    pub fn function_calls(&self) -> u64 {
        self.function_calls
    }

    /// Per-function call counts
    pub fn function_call_counts(&self) -> &HashMap<String, u64> {
        &self.function_call_counts
    }

    /// Top N opcodes by execution count (sorted descending)
    pub fn top_opcodes(&self, n: usize) -> Vec<(Opcode, u64)> {
        let mut pairs: Vec<(Opcode, u64)> = self
            .instruction_counts
            .iter()
            .filter_map(|(&byte, &count)| Opcode::try_from(byte).ok().map(|op| (op, count)))
            .collect();
        pairs.sort_by(|a, b| b.1.cmp(&a.1));
        pairs.truncate(n);
        pairs
    }

    /// Top N hot locations by execution count (sorted descending)
    pub fn top_locations(&self, n: usize) -> Vec<(usize, u64)> {
        let mut pairs: Vec<(usize, u64)> = self
            .location_counts
            .iter()
            .map(|(&ip, &c)| (ip, c))
            .collect();
        pairs.sort_by(|a, b| b.1.cmp(&a.1));
        pairs.truncate(n);
        pairs
    }
}

impl Default for ProfileCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_collector_is_empty() {
        let c = ProfileCollector::new();
        assert_eq!(c.total_instructions(), 0);
        assert_eq!(c.max_stack_depth(), 0);
        assert_eq!(c.function_calls(), 0);
    }

    #[test]
    fn test_record_instruction_increments_total() {
        let mut c = ProfileCollector::new();
        c.record_instruction(Opcode::Add, 0);
        c.record_instruction(Opcode::Add, 3);
        assert_eq!(c.total_instructions(), 2);
    }

    #[test]
    fn test_record_instruction_per_opcode() {
        let mut c = ProfileCollector::new();
        c.record_instruction(Opcode::Add, 0);
        c.record_instruction(Opcode::Add, 3);
        c.record_instruction(Opcode::Mul, 6);
        assert_eq!(c.instruction_count(Opcode::Add), 2);
        assert_eq!(c.instruction_count(Opcode::Mul), 1);
        assert_eq!(c.instruction_count(Opcode::Sub), 0);
    }

    #[test]
    fn test_record_instruction_per_location() {
        let mut c = ProfileCollector::new();
        c.record_instruction(Opcode::Add, 10);
        c.record_instruction(Opcode::Add, 10);
        c.record_instruction(Opcode::Mul, 20);
        assert_eq!(c.location_counts()[&10], 2);
        assert_eq!(c.location_counts()[&20], 1);
    }

    #[test]
    fn test_opcode_at_ip() {
        let mut c = ProfileCollector::new();
        c.record_instruction(Opcode::Add, 10);
        assert_eq!(c.opcode_at(10), Some(Opcode::Add));
        assert_eq!(c.opcode_at(99), None);
    }

    #[test]
    fn test_update_frame_depth() {
        let mut c = ProfileCollector::new();
        c.update_frame_depth(2);
        c.update_frame_depth(5);
        c.update_frame_depth(3);
        assert_eq!(c.max_stack_depth(), 5);
    }

    #[test]
    fn test_update_value_stack_depth() {
        let mut c = ProfileCollector::new();
        c.update_value_stack_depth(8);
        c.update_value_stack_depth(4);
        assert_eq!(c.max_value_stack_depth(), 8);
    }

    #[test]
    fn test_record_function_call() {
        let mut c = ProfileCollector::new();
        c.record_function_call("foo");
        c.record_function_call("foo");
        c.record_function_call("bar");
        assert_eq!(c.function_calls(), 3);
        assert_eq!(c.function_call_counts()["foo"], 2);
        assert_eq!(c.function_call_counts()["bar"], 1);
    }

    #[test]
    fn test_reset_clears_all() {
        let mut c = ProfileCollector::new();
        c.record_instruction(Opcode::Add, 0);
        c.update_frame_depth(5);
        c.record_function_call("foo");
        c.reset();
        assert_eq!(c.total_instructions(), 0);
        assert_eq!(c.max_stack_depth(), 0);
        assert_eq!(c.function_calls(), 0);
        assert!(c.instruction_counts().is_empty());
    }

    #[test]
    fn test_top_opcodes() {
        let mut c = ProfileCollector::new();
        for _ in 0..10 {
            c.record_instruction(Opcode::Add, 0);
        }
        for _ in 0..5 {
            c.record_instruction(Opcode::Mul, 3);
        }
        for _ in 0..3 {
            c.record_instruction(Opcode::Sub, 6);
        }
        let top = c.top_opcodes(2);
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].0, Opcode::Add);
        assert_eq!(top[0].1, 10);
        assert_eq!(top[1].0, Opcode::Mul);
    }

    #[test]
    fn test_top_locations() {
        let mut c = ProfileCollector::new();
        for _ in 0..20 {
            c.record_instruction(Opcode::Loop, 100);
        }
        for _ in 0..5 {
            c.record_instruction(Opcode::Add, 50);
        }
        let top = c.top_locations(1);
        assert_eq!(top[0].0, 100);
        assert_eq!(top[0].1, 20);
    }

    #[test]
    fn test_record_instruction_opcode_no_location() {
        let mut c = ProfileCollector::new();
        c.record_instruction_opcode(Opcode::Add);
        assert_eq!(c.total_instructions(), 1);
        assert!(c.location_counts().is_empty());
    }
}
