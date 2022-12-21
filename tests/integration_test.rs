use metrics_attributes::instrument;

#[instrument(name = "handlers")]
fn add(a: i32, b: i32) -> std::fmt::Result {
    Ok(())
}
