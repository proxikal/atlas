use atlas_runtime::{Atlas, SecurityContext, Value};

fn main() {
    // Allow file I/O and module loading for the demo.
    let runtime = Atlas::new_with_security(SecurityContext::allow_all());

    match runtime.eval_file("demo/main.atl") {
        Ok(Value::Null) => {}
        Ok(value) => println!("{}", value),
        Err(diags) => {
            for diag in diags {
                eprintln!("{:?}", diag);
            }
            std::process::exit(1);
        }
    }
}
