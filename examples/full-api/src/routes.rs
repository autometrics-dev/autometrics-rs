use autometrics::{
    autometrics, encode_global_metrics, Objective, ObjectivePercentage, TargetLatency,
};
use autometrics_example_util::sleep_random_duration;
use axum::{extract::State, http::StatusCode, Json};
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};

use crate::database::Database;
use crate::error::ApiError;

/// This is a Service-Level Objective (SLO) we're defining for our API.
const API_SLO: Objective = Objective::new("api")
    .success_rate(ObjectivePercentage::P99_9)
    .latency(TargetLatency::Ms200, ObjectivePercentage::P99);

// Starting simple, hover over the function name to see the Autometrics graph links in the Rust Docs!
/// This is a simple endpoint that never errors
#[autometrics]
pub async fn get_index() -> &'static str {
    "Hello, World!"
}

/// This is a function that returns an error ~25% of the time
/// The call counter metric generated by autometrics will have a label
/// `result` = `ok` or `error`, depending on what the function returned
#[autometrics(objective = API_SLO)]
pub async fn get_random_error() -> Result<(), ApiError> {
    let error = thread_rng().gen_range(0..4);

    sleep_random_duration().await;

    match error {
        0 => Err(ApiError::NotFound),
        1 => Err(ApiError::BadRequest),
        2 => Err(ApiError::Internal),
        _ => Ok(()),
    }
}

// This handler calls another internal function that is also instrumented with autometrics.
//
// Unlike other instrumentation libraries, autometrics is designed to give you more
// granular metrics that allow you to dig into the internals of your application
// before even reaching for logs or traces.
//
// Try hovering over the function name to see the graph links and pay special attention
// to the links for the functions _called by this function_.
// You can also hover over the load_details_from_database function to see the graph links for that function.
#[autometrics(objective = API_SLO)]
pub async fn create_user(
    State(database): State<Database>,
    Json(payload): Json<CreateUser>,
) -> Result<Json<User>, ApiError> {
    let user = User {
        id: 1337,
        username: payload.username,
    };

    database.load_details().await?;

    sleep_random_duration().await;

    Ok(Json(user))
}

// The input to our `create_user` handler
#[derive(Serialize, Deserialize)]
pub struct CreateUser {
    pub username: String,
}

// The output to our `create_user` handler
#[derive(Serialize)]
pub struct User {
    pub id: u64,
    pub username: String,
}

/// This handler serializes the metrics into a string for Prometheus to scrape
pub async fn get_metrics() -> (StatusCode, String) {
    match encode_global_metrics() {
        Ok(metrics) => (StatusCode::OK, metrics),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, format!("{:?}", err)),
    }
}
