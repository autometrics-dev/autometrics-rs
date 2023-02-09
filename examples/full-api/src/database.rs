use crate::error::ApiError;
use autometrics::autometrics;
use autometrics_example_util::sleep_random_duration;
use rand::random;

#[derive(Clone)]
pub struct Database;

// You can instrument a whole impl block like this:
#[autometrics]
impl Database {
    #[skip_autometrics]
    pub fn new() -> Self {
        Self
    }

    /// An internal function that is also instrumented with autometrics
    pub async fn load_details(&self) -> Result<(), ApiError> {
        let should_error = random::<bool>();

        sleep_random_duration().await;

        if should_error {
            Err(ApiError::Internal)
        } else {
            Ok(())
        }
    }
}
