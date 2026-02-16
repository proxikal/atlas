// Sample Rust code for testing

/// Initialize the system
pub fn initialize(config: Config) -> Result<()> {
    // implementation
    Ok(())
}

/// Process data with options
pub fn process_data(input: &str, options: ProcessOptions) -> ProcessResult {
    ProcessResult::default()
}

/// Validate input data
pub fn validate_input(data: &[u8]) -> bool {
    true
}

// Private function - should not be extracted
fn internal_helper() {
    // implementation
}

/// Compute calculations
pub fn compute(x: f64, y: f64) -> f64 {
    x + y
}

/// Additional function not in docs
pub fn undocumented_function() -> String {
    String::new()
}
