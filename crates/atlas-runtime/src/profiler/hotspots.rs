//! Hotspot detection
//!
//! Analyses profiler data to find instruction locations that account for
//! a disproportionate share of total execution.

use crate::bytecode::Opcode;
use crate::profiler::collector::ProfileCollector;

/// A single hotspot — an instruction location above the detection threshold
#[derive(Debug, Clone)]
pub struct Hotspot {
    /// Instruction pointer of the hot location
    pub ip: usize,
    /// Times this location was executed
    pub count: u64,
    /// Percentage of total instructions (0–100)
    pub percentage: f64,
    /// The opcode at this location (None if unknown)
    pub opcode: Option<Opcode>,
}

/// A hot opcode summary
#[derive(Debug, Clone)]
pub struct HotOpcode {
    pub opcode: Opcode,
    pub count: u64,
    pub percentage: f64,
}

/// Detects hotspots in profiler data
#[derive(Debug, Clone)]
pub struct HotspotDetector {
    /// Minimum percentage to qualify as a hotspot (default 1.0 = 1%)
    threshold_pct: f64,
    /// Maximum number of hotspots to return
    max_hotspots: usize,
}

impl HotspotDetector {
    /// Create a detector with default settings (1% threshold, 20 hotspots)
    pub fn new() -> Self {
        Self {
            threshold_pct: 1.0,
            max_hotspots: 20,
        }
    }

    /// Create a detector with a custom threshold percentage (0–100)
    pub fn with_threshold(threshold_pct: f64) -> Self {
        Self {
            threshold_pct: threshold_pct.clamp(0.0, 100.0),
            max_hotspots: 20,
        }
    }

    /// Set the maximum number of hotspots to return
    pub fn with_max_hotspots(mut self, n: usize) -> Self {
        self.max_hotspots = n;
        self
    }

    /// Get the threshold in use
    pub fn threshold(&self) -> f64 {
        self.threshold_pct
    }

    /// Detect hotspot locations from collector data
    ///
    /// Returns hotspots sorted by execution count (highest first),
    /// limited to those above `threshold_pct` of total instructions.
    pub fn detect(&self, collector: &ProfileCollector) -> Vec<Hotspot> {
        let total = collector.total_instructions();
        if total == 0 {
            return Vec::new();
        }

        let mut hotspots: Vec<Hotspot> = collector
            .location_counts()
            .iter()
            .filter_map(|(&ip, &count)| {
                let pct = (count as f64 / total as f64) * 100.0;
                if pct >= self.threshold_pct {
                    Some(Hotspot {
                        ip,
                        count,
                        percentage: pct,
                        opcode: collector.opcode_at(ip),
                    })
                } else {
                    None
                }
            })
            .collect();

        hotspots.sort_by(|a, b| b.count.cmp(&a.count));
        hotspots.truncate(self.max_hotspots);
        hotspots
    }

    /// Identify the top N opcodes by execution frequency
    pub fn top_opcodes(&self, collector: &ProfileCollector, n: usize) -> Vec<HotOpcode> {
        let total = collector.total_instructions();
        if total == 0 {
            return Vec::new();
        }

        collector
            .top_opcodes(n)
            .into_iter()
            .map(|(opcode, count)| HotOpcode {
                opcode,
                count,
                percentage: (count as f64 / total as f64) * 100.0,
            })
            .collect()
    }

    /// Check whether a specific location qualifies as a hotspot
    pub fn is_hotspot(&self, collector: &ProfileCollector, ip: usize) -> bool {
        let total = collector.total_instructions();
        if total == 0 {
            return false;
        }
        let count = collector.location_counts().get(&ip).copied().unwrap_or(0);
        (count as f64 / total as f64) * 100.0 >= self.threshold_pct
    }
}

impl Default for HotspotDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_collector(instructions: &[(Opcode, usize, u32)]) -> ProfileCollector {
        let mut c = ProfileCollector::new();
        for &(opcode, ip, count) in instructions {
            for _ in 0..count {
                c.record_instruction(opcode, ip);
            }
        }
        c
    }

    #[test]
    fn test_detect_empty_collector() {
        let c = ProfileCollector::new();
        let detector = HotspotDetector::new();
        assert!(detector.detect(&c).is_empty());
    }

    #[test]
    fn test_detect_single_hotspot() {
        // 100 instructions all at the same IP → 100% hotspot
        let c = make_collector(&[(Opcode::Loop, 10, 100)]);
        let detector = HotspotDetector::new();
        let hotspots = detector.detect(&c);
        assert_eq!(hotspots.len(), 1);
        assert_eq!(hotspots[0].ip, 10);
        assert_eq!(hotspots[0].count, 100);
        assert!((hotspots[0].percentage - 100.0).abs() < 0.01);
        assert_eq!(hotspots[0].opcode, Some(Opcode::Loop));
    }

    #[test]
    fn test_detect_above_threshold() {
        // Loop 50%, Add 30%, Mul 20% → all above 1% threshold
        let c = make_collector(&[
            (Opcode::Loop, 100, 50),
            (Opcode::Add, 200, 30),
            (Opcode::Mul, 300, 20),
        ]);
        let detector = HotspotDetector::new();
        let hotspots = detector.detect(&c);
        assert_eq!(hotspots.len(), 3);
        // Sorted descending by count
        assert_eq!(hotspots[0].ip, 100);
    }

    #[test]
    fn test_detect_below_threshold_excluded() {
        // Only 1 instruction out of 1000 → 0.1%, below default 1% threshold
        let mut c = make_collector(&[(Opcode::Loop, 100, 999)]);
        c.record_instruction(Opcode::Add, 200);
        let detector = HotspotDetector::new();
        let hotspots = detector.detect(&c);
        // ip=200 should not be a hotspot (0.1% < 1%)
        assert!(!hotspots.iter().any(|h| h.ip == 200));
    }

    #[test]
    fn test_custom_threshold() {
        let c = make_collector(&[(Opcode::Loop, 100, 95), (Opcode::Add, 200, 5)]);
        // 5% threshold → Add (5%) is on the border, Loop (95%) passes
        let detector = HotspotDetector::with_threshold(5.0);
        let hotspots = detector.detect(&c);
        assert!(hotspots.iter().any(|h| h.ip == 100));
        assert!(hotspots.iter().any(|h| h.ip == 200));
    }

    #[test]
    fn test_max_hotspots_limit() {
        // Create 10 different locations each at 10%
        let mut c = ProfileCollector::new();
        for i in 0..10usize {
            for _ in 0..10u32 {
                c.record_instruction(Opcode::Add, i * 10);
            }
        }
        let detector = HotspotDetector::new().with_max_hotspots(3);
        let hotspots = detector.detect(&c);
        assert_eq!(hotspots.len(), 3);
    }

    #[test]
    fn test_hotspots_sorted_descending() {
        let c = make_collector(&[
            (Opcode::Add, 10, 30),
            (Opcode::Mul, 20, 50),
            (Opcode::Sub, 30, 20),
        ]);
        let detector = HotspotDetector::new();
        let hotspots = detector.detect(&c);
        // Should be sorted: 50, 30, 20
        assert_eq!(hotspots[0].count, 50);
        assert_eq!(hotspots[1].count, 30);
        assert_eq!(hotspots[2].count, 20);
    }

    #[test]
    fn test_top_opcodes_empty() {
        let c = ProfileCollector::new();
        let detector = HotspotDetector::new();
        assert!(detector.top_opcodes(&c, 5).is_empty());
    }

    #[test]
    fn test_top_opcodes_ranking() {
        let c = make_collector(&[
            (Opcode::Add, 0, 100),
            (Opcode::Mul, 3, 50),
            (Opcode::Sub, 6, 25),
        ]);
        let detector = HotspotDetector::new();
        let top = detector.top_opcodes(&c, 2);
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].opcode, Opcode::Add);
        assert_eq!(top[1].opcode, Opcode::Mul);
    }

    #[test]
    fn test_top_opcodes_percentage() {
        let c = make_collector(&[(Opcode::Add, 0, 200), (Opcode::Mul, 3, 200)]);
        let detector = HotspotDetector::new();
        let top = detector.top_opcodes(&c, 2);
        for entry in &top {
            assert!((entry.percentage - 50.0).abs() < 0.01);
        }
    }

    #[test]
    fn test_is_hotspot_true() {
        let c = make_collector(&[(Opcode::Loop, 0, 100)]);
        let detector = HotspotDetector::new();
        assert!(detector.is_hotspot(&c, 0));
    }

    #[test]
    fn test_is_hotspot_false_zero_total() {
        let c = ProfileCollector::new();
        let detector = HotspotDetector::new();
        assert!(!detector.is_hotspot(&c, 0));
    }

    #[test]
    fn test_is_hotspot_missing_ip() {
        let c = make_collector(&[(Opcode::Add, 0, 100)]);
        let detector = HotspotDetector::new();
        assert!(!detector.is_hotspot(&c, 999));
    }

    #[test]
    fn test_zero_threshold_all_hotspots() {
        let c = make_collector(&[(Opcode::Add, 0, 1), (Opcode::Mul, 3, 1)]);
        let detector = HotspotDetector::with_threshold(0.0);
        let hotspots = detector.detect(&c);
        assert_eq!(hotspots.len(), 2);
    }
}
