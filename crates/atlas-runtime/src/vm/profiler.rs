//! VM profiler — delegates to crate::profiler
//!
//! Re-exports the comprehensive `Profiler` from `crate::profiler` so that
//! `vm::Profiler` continues to work for all existing callers.

pub use crate::profiler::Profiler;

#[cfg(test)]
mod tests {
    use crate::bytecode::Opcode;
    use crate::profiler::Profiler;

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
