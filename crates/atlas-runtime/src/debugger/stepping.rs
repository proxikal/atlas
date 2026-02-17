//! Source-line-aware stepping for the Atlas debugger.
//!
//! Extends the basic instruction-level stepping with line-level awareness,
//! ensuring step-over/step-into/step-out operate on source lines rather
//! than individual bytecode instructions.

use crate::debugger::protocol::{PauseReason, SourceLocation};
use crate::debugger::source_map::SourceMap;

// ── StepRequest ──────────────────────────────────────────────────────────────

/// A step operation requested by the debugger client.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepRequest {
    /// Step into: pause at the next instruction (descend into calls).
    Into,
    /// Step over: pause at the next source line at the same or shallower depth.
    Over,
    /// Step out: pause when the current function returns.
    Out,
    /// Run to a specific source line.
    RunToLine { file: String, line: u32 },
    /// Run to a specific instruction offset.
    RunToOffset(usize),
}

// ── StepTracker ──────────────────────────────────────────────────────────────

/// Tracks step state for source-line-aware stepping.
///
/// The VM calls `should_pause` before each instruction. The tracker determines
/// whether execution should stop based on the current step request, frame depth,
/// and source line changes.
#[derive(Debug)]
pub struct StepTracker {
    /// The active step request (None = free running).
    active_request: Option<StepRequest>,
    /// Frame depth when the step was initiated.
    start_frame_depth: usize,
    /// Source line at the start of the step (for line-level step-over).
    start_line: Option<u32>,
    /// Source file at the start of the step.
    start_file: Option<String>,
    /// Number of instructions executed since the step started.
    instructions_since_step: u64,
    /// Maximum instructions before force-pausing (safety limit).
    max_instructions: u64,
}

impl StepTracker {
    /// Create a new step tracker with no active step request.
    pub fn new() -> Self {
        Self {
            active_request: None,
            start_frame_depth: 0,
            start_line: None,
            start_file: None,
            instructions_since_step: 0,
            max_instructions: 1_000_000,
        }
    }

    /// Start a step operation.
    ///
    /// `frame_depth`: current call frame depth.
    /// `current_location`: the source location where the step starts.
    pub fn begin_step(
        &mut self,
        request: StepRequest,
        frame_depth: usize,
        current_location: Option<&SourceLocation>,
    ) {
        self.start_frame_depth = frame_depth;
        self.start_line = current_location.map(|l| l.line);
        self.start_file = current_location.map(|l| l.file.clone());
        self.instructions_since_step = 0;
        self.active_request = Some(request);
    }

    /// Cancel the active step request.
    pub fn cancel(&mut self) {
        self.active_request = None;
        self.instructions_since_step = 0;
    }

    /// Returns `true` if a step operation is active.
    pub fn is_stepping(&self) -> bool {
        self.active_request.is_some()
    }

    /// Get the active step request.
    pub fn active_request(&self) -> Option<&StepRequest> {
        self.active_request.as_ref()
    }

    /// Get the frame depth when the step started.
    pub fn start_depth(&self) -> usize {
        self.start_frame_depth
    }

    /// Get instructions executed since step started.
    pub fn instructions_executed(&self) -> u64 {
        self.instructions_since_step
    }

    /// Set the maximum instruction limit for safety.
    pub fn set_max_instructions(&mut self, max: u64) {
        self.max_instructions = max;
    }

    /// Determine whether execution should pause at this instruction.
    ///
    /// Returns `Some(PauseReason)` if the step condition is met, `None` otherwise.
    ///
    /// Parameters:
    /// - `ip`: current instruction pointer
    /// - `frame_depth`: current call frame depth
    /// - `source_map`: for resolving IP to source location
    pub fn should_pause(
        &mut self,
        ip: usize,
        frame_depth: usize,
        source_map: &SourceMap,
    ) -> Option<PauseReason> {
        let request = match &self.active_request {
            Some(r) => r.clone(),
            None => return None,
        };

        self.instructions_since_step += 1;

        // Safety limit: force pause after too many instructions
        if self.instructions_since_step > self.max_instructions {
            self.active_request = None;
            return Some(PauseReason::Step);
        }

        let current_location = source_map.location_for_offset(ip);
        let current_line = current_location.map(|l| l.line);
        let current_file = current_location.map(|l| l.file.as_str());

        match request {
            StepRequest::Into => {
                // Step-into: pause at the first instruction on a different source line,
                // OR immediately if we have no line info.
                if self.start_line.is_none() || current_line.is_none() {
                    self.active_request = None;
                    return Some(PauseReason::Step);
                }
                let on_new_line = current_line != self.start_line
                    || current_file.map(|f| f.to_string()) != self.start_file;
                if on_new_line {
                    self.active_request = None;
                    return Some(PauseReason::Step);
                }
                None
            }
            StepRequest::Over => {
                // Step-over: pause at next source line at same or shallower depth.
                if frame_depth > self.start_frame_depth {
                    return None; // Still inside a call — keep running
                }
                // At same or shallower depth — check for line change
                if self.start_line.is_none() || current_line.is_none() {
                    self.active_request = None;
                    return Some(PauseReason::Step);
                }
                let on_new_line = current_line != self.start_line
                    || current_file.map(|f| f.to_string()) != self.start_file;
                if on_new_line {
                    self.active_request = None;
                    return Some(PauseReason::Step);
                }
                None
            }
            StepRequest::Out => {
                // Step-out: pause when frame depth decreases below start.
                if frame_depth < self.start_frame_depth {
                    self.active_request = None;
                    return Some(PauseReason::Step);
                }
                None
            }
            StepRequest::RunToLine { ref file, line } => {
                if let Some(loc) = current_location {
                    if loc.line == line && loc.file == *file {
                        self.active_request = None;
                        return Some(PauseReason::Step);
                    }
                }
                None
            }
            StepRequest::RunToOffset(target) => {
                if ip == target {
                    self.active_request = None;
                    return Some(PauseReason::Step);
                }
                None
            }
        }
    }
}

impl Default for StepTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_source_map() -> SourceMap {
        let mut map = SourceMap::new();
        // Simulate a 3-line program with 2 instructions per line
        map.insert(0, SourceLocation::new("test.atlas", 1, 1));
        map.insert(1, SourceLocation::new("test.atlas", 1, 5));
        map.insert(2, SourceLocation::new("test.atlas", 2, 1));
        map.insert(3, SourceLocation::new("test.atlas", 2, 5));
        map.insert(4, SourceLocation::new("test.atlas", 3, 1));
        map.insert(5, SourceLocation::new("test.atlas", 3, 5));
        map
    }

    #[test]
    fn test_tracker_initial_state() {
        let tracker = StepTracker::new();
        assert!(!tracker.is_stepping());
        assert!(tracker.active_request().is_none());
    }

    #[test]
    fn test_step_into_pauses_on_new_line() {
        let map = make_source_map();
        let mut tracker = StepTracker::new();
        let start_loc = SourceLocation::new("test.atlas", 1, 1);
        tracker.begin_step(StepRequest::Into, 1, Some(&start_loc));
        assert!(tracker.is_stepping());

        // Same line (offset 1) — should not pause
        assert!(tracker.should_pause(1, 1, &map).is_none());
        // New line (offset 2) — should pause
        assert!(tracker.should_pause(2, 1, &map).is_some());
        assert!(!tracker.is_stepping());
    }

    #[test]
    fn test_step_over_skips_deeper_frames() {
        let map = make_source_map();
        let mut tracker = StepTracker::new();
        let start_loc = SourceLocation::new("test.atlas", 1, 1);
        tracker.begin_step(StepRequest::Over, 1, Some(&start_loc));

        // Deeper frame (depth 2) — should not pause even on new line
        assert!(tracker.should_pause(2, 2, &map).is_none());
        // Back to same depth, new line — should pause
        assert!(tracker.should_pause(2, 1, &map).is_some());
    }

    #[test]
    fn test_step_over_pauses_at_shallower_depth() {
        let map = make_source_map();
        let mut tracker = StepTracker::new();
        let start_loc = SourceLocation::new("test.atlas", 2, 1);
        tracker.begin_step(StepRequest::Over, 2, Some(&start_loc));

        // Shallower depth (returned from function), different line — pause
        assert!(tracker.should_pause(0, 1, &map).is_some());
    }

    #[test]
    fn test_step_out_pauses_on_return() {
        let map = make_source_map();
        let mut tracker = StepTracker::new();
        let start_loc = SourceLocation::new("test.atlas", 2, 1);
        tracker.begin_step(StepRequest::Out, 2, Some(&start_loc));

        // Same depth — don't pause
        assert!(tracker.should_pause(0, 2, &map).is_none());
        // Shallower depth — pause
        assert!(tracker.should_pause(0, 1, &map).is_some());
    }

    #[test]
    fn test_step_out_doesnt_pause_at_deeper() {
        let map = make_source_map();
        let mut tracker = StepTracker::new();
        tracker.begin_step(StepRequest::Out, 2, None);

        // Deeper — don't pause
        assert!(tracker.should_pause(0, 3, &map).is_none());
    }

    #[test]
    fn test_run_to_line() {
        let map = make_source_map();
        let mut tracker = StepTracker::new();
        tracker.begin_step(
            StepRequest::RunToLine {
                file: "test.atlas".into(),
                line: 3,
            },
            1,
            None,
        );

        assert!(tracker.should_pause(0, 1, &map).is_none()); // line 1
        assert!(tracker.should_pause(2, 1, &map).is_none()); // line 2
        assert!(tracker.should_pause(4, 1, &map).is_some()); // line 3
    }

    #[test]
    fn test_run_to_offset() {
        let map = make_source_map();
        let mut tracker = StepTracker::new();
        tracker.begin_step(StepRequest::RunToOffset(3), 1, None);

        assert!(tracker.should_pause(0, 1, &map).is_none());
        assert!(tracker.should_pause(1, 1, &map).is_none());
        assert!(tracker.should_pause(3, 1, &map).is_some());
    }

    #[test]
    fn test_cancel_step() {
        let mut tracker = StepTracker::new();
        tracker.begin_step(StepRequest::Into, 1, None);
        assert!(tracker.is_stepping());
        tracker.cancel();
        assert!(!tracker.is_stepping());
    }

    #[test]
    fn test_safety_limit() {
        let map = make_source_map();
        let mut tracker = StepTracker::new();
        tracker.set_max_instructions(5);
        let start_loc = SourceLocation::new("test.atlas", 1, 1);
        tracker.begin_step(StepRequest::Over, 1, Some(&start_loc));

        // All on same line — normally wouldn't pause, but safety limit kicks in
        for _ in 0..5 {
            assert!(tracker.should_pause(0, 2, &map).is_none()); // deeper frame, same line
        }
        // 6th call exceeds limit
        assert!(tracker.should_pause(0, 2, &map).is_some());
    }

    #[test]
    fn test_step_into_no_line_info_pauses_immediately() {
        let map = SourceMap::new(); // empty — no line info
        let mut tracker = StepTracker::new();
        tracker.begin_step(StepRequest::Into, 1, None);
        assert!(tracker.should_pause(0, 1, &map).is_some());
    }

    #[test]
    fn test_instructions_executed_counter() {
        let map = make_source_map();
        let mut tracker = StepTracker::new();
        let start_loc = SourceLocation::new("test.atlas", 1, 1);
        tracker.begin_step(StepRequest::Over, 1, Some(&start_loc));

        tracker.should_pause(0, 2, &map); // deeper frame
        tracker.should_pause(1, 2, &map);
        assert_eq!(tracker.instructions_executed(), 2);
    }

    #[test]
    fn test_start_depth() {
        let mut tracker = StepTracker::new();
        tracker.begin_step(StepRequest::Out, 3, None);
        assert_eq!(tracker.start_depth(), 3);
    }
}
