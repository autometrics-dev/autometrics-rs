use metrics_attributes::instrument;

#[instrument(name = "handlers")]
fn add(a: i32, b: i32) -> Result<i32, ()> {
    Ok(a + b)
}

#[instrument(name = "other")]
fn add2(a: i32, b: i32) -> std::fmt::Result {
    Ok(())
}
