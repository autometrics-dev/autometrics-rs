use metrics_attributes::instrument;

#[instrument]
fn add(a: i32, b: i32) -> std::fmt::Result {
    Ok(())
}
