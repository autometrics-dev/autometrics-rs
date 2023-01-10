use metrics_attributes::instrument;

#[instrument(infallible, name = "util_function_call")]
fn add(a: i32, b: i32) -> i32 {
    a + b
}

/// "HTTP" handler function
#[instrument]
fn get_index_handler() -> Result<String, ()> {
    Ok("Hello world!".to_string())
}
