use metrics_attributes::instrument;

mod hello {
    use super::*;

    #[instrument]
    fn add(a: i32, b: i32) -> std::fmt::Result {
        Ok(())
    }
}
