//! Call frame implementation for function calls

/// Call frame for function calls
///
/// Each function call creates a new frame that tracks:
/// - Where to return to (return_ip)
/// - Where the function's local variables start on the stack (stack_base)
/// - How many locals the function has (local_count)
///
/// The main/top-level code also has a frame ("<main>") with stack_base = 0.
///
/// ## Stack Layout Example
///
/// ```text
/// Stack with two frames (main called function "add"):
///
/// [global1][global2] | [func_ptr][arg1][arg2][local1]
///  ^                  ^
///  main frame base    add frame base
/// ```
///
/// Local variable access is frame-relative:
/// - GetLocal 0 in main -> stack[0]
/// - GetLocal 0 in add -> stack[stack_base + 0]
#[derive(Debug, Clone)]
pub struct CallFrame {
    /// Function name (for debugging and error messages)
    pub function_name: String,
    /// Instruction pointer to return to after function completes
    pub return_ip: usize,
    /// Stack index where this frame's local variables begin
    ///
    /// Local variable N is at stack[stack_base + N]
    pub stack_base: usize,
    /// Number of local variables in this frame
    pub local_count: usize,
}
