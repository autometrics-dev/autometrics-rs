// This test ensures that when an instrumented function has a compilation error,
// then the error is reported at the correct line in the original code.
use autometrics::autometrics;

#[autometrics]
fn bad_function() {
    // This vec is not mut
    let contents: Vec<u32> = Vec::new();

    contents.push(2);
}

fn main() {
    bad_function();
}
