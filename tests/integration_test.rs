use autometrics::autometrics;

#[autometrics(infallible, name = "util_function_call")]
fn add(a: i32, b: i32) -> i32 {
    a + b
}

/// "HTTP" handler function
#[autometrics]
fn get_index_handler() -> Result<String, ()> {
    Ok("Hello world!".to_string())
}
