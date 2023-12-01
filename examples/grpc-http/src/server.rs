use autometrics::autometrics;
use job::job_runner_server::JobRunner;
use job::{Empty, JobList, JobReply, JobRequest};
use tonic::{Request, Response, Status};
use crate::db_manager::DBManager;

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


#[tonic::async_trait]
impl JobRunner for MyJobRunner {
    #[autometrics]
    async fn send_job(&self, request: Request<JobRequest>) -> Result<Response<JobReply>, Status> {
        println!("Got a request: {:?}", request);

        // Write into the mock database
        self.db_manager.write_into_table().await.expect("Failed to query database");

        let reply = job::JobReply {
            message: format!("Hello {}!", request.into_inner().name).into(),
        };

        Ok(Response::new(reply))
    }

    #[autometrics]
    async fn list_jobs(&self, request: Request<Empty>) -> Result<Response<JobList>, Status> {
        println!("Got a request: {:?}", request);

        // Query the mock database
        self.db_manager.query_table().await.expect("Failed to query database");

        let reply = job::JobList {
            job: vec![job::Job {
                id: 1,
                name: "test".into(),
            }],
        };

        Ok(Response::new(reply))
    }
}
