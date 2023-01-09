use metrics_attributes::instrument;

#[instrument(infallible, name = "handlers")]
fn add(a: i32, b: i32) -> i32 {
    a + b
}

/// The add two function
#[instrument]
fn add2(a: i32, b: i32) -> std::fmt::Result {
    Ok(())
}
