use crate::db_manager::DBManager;
use autometrics::autometrics;
use job::job_runner_server::JobRunner;
use job::{Empty, JobList, JobReply, JobRequest};
use tonic::{Request, Response, Status};

use autometrics::objectives::{Objective, ObjectiveLatency, ObjectivePercentile};

// Add autometrics Service-Level Objectives (SLOs)
// https://docs.autometrics.dev/rust/adding-alerts-and-slos
const API_SLO: Objective = Objective::new("job_runner_api")
    // We expect 99.9% of all requests to succeed.
    .success_rate(ObjectivePercentile::P99_9)
    // We expect 99% of all latencies to be below 250ms.
    .latency(ObjectiveLatency::Ms250, ObjectivePercentile::P99);
// Autometrics raises an alert whenever any of the SLO objectives fail.

pub mod job {
    tonic::include_proto!("job");
}

#[derive(Debug, Default)]
pub struct MyJobRunner {
    db_manager: DBManager,
}

impl MyJobRunner {
    pub fn new(db_manager: DBManager) -> Self {
        Self { db_manager }
    }
}

// Instrument all API functions of the implementation of JobRunner via macro.
// https://docs.autometrics.dev/rust/quickstart
// Attach the SLO to each API function.
//
// Notice, all API functions are instrumented with the same SLO.
// If you want to have different SLOs for different API functions,
// You have to create a separate SLO for each API function and instrument
// each API function individually instead of using the macro on trait level.
// Docs https://docs.autometrics.dev/rust/adding-alerts-and-slos
#[tonic::async_trait]
#[autometrics(objective = API_SLO)]
impl JobRunner for MyJobRunner {
    async fn send_job(&self, request: Request<JobRequest>) -> Result<Response<JobReply>, Status> {
        println!("Got a request: {:?}", request);

        // Write into the mock database
        self.db_manager
            .write_into_table()
            .await
            .expect("Failed to query database");

        let reply = job::JobReply {
            message: format!("Hello {}!", request.into_inner().name).into(),
        };

        Ok(Response::new(reply))
    }

    async fn list_jobs(&self, request: Request<Empty>) -> Result<Response<JobList>, Status> {
        println!("Got a request: {:?}", request);

        // Query the mock database
        self.db_manager
            .query_table()
            .await
            .expect("Failed to query database");

        let reply = job::JobList {
            job: vec![job::Job {
                id: 1,
                name: "test".into(),
            }],
        };

        Ok(Response::new(reply))
    }
}
