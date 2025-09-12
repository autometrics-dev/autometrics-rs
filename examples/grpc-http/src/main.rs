use std::net::SocketAddr;
use tonic::transport::Server as TonicServer;
use warp::http::StatusCode;
use warp::Filter;

use autometrics::prometheus_exporter;
use server::MyJobRunner;

use crate::server::job::job_runner_server::JobRunnerServer;

mod db_manager;
mod server;
mod shutdown;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up prometheus metrics exporter
    prometheus_exporter::init();

    // Set up two different ports for gRPC and HTTP
    let grpc_addr = "127.0.0.1:50051"
        .parse()
        .expect("Failed to parse gRPC address");
    let web_addr: SocketAddr = "127.0.0.1:8080"
        .parse()
        .expect("Failed to parse web address");

    // Build new DBManager that connects to the database
    let dbm = db_manager::DBManager::new();
    // Connect to the database
    dbm.connect_to_db()
        .await
        .expect("Failed to connect to database");

    // gRPC server with DBManager
    let grpc_svc = JobRunnerServer::new(MyJobRunner::new(dbm));

    // Sigint signal handler that closes the DB connection upon shutdown
    let signal = shutdown::grpc_sigint(dbm.clone());

    // Construct health service for gRPC server
    let (mut health_reporter, health_svc) = tonic_health::server::health_reporter();
    health_reporter
        .set_serving::<JobRunnerServer<MyJobRunner>>()
        .await;

    // Build gRPC server with health service and signal sigint handler
    let grpc_server = TonicServer::builder()
        .add_service(grpc_svc)
        .add_service(health_svc)
        .serve_with_shutdown(grpc_addr, signal);

    // Build http /metrics endpoint
    let routes = warp::get().and(warp::path("metrics")).map(|| {
        prometheus_exporter::encode_to_string().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    });

    // Build http web server
    let (_, web_server) =
        warp::serve(routes).bind_with_graceful_shutdown(web_addr, shutdown::http_sigint());

    // Create handler for each server
    //  https://github.com/hyperium/tonic/discussions/740
    let grpc_handle = tokio::spawn(grpc_server);
    let grpc_web_handle = tokio::spawn(web_server);

    // Join all servers together and start the the main loop
    print_start(&web_addr, &grpc_addr);
    let _ = tokio::try_join!(grpc_handle, grpc_web_handle)
        .expect("Failed to start gRPC and http server");

    Ok(())
}

fn print_start(web_addr: &SocketAddr, grpc_addr: &SocketAddr) {
    println!();
    println!("Started gRPC server on port {:?}", grpc_addr.port());
    println!("Started metrics on port {:?}", web_addr.port());
    println!("Stop service with Ctrl+C");
    println!();
    println!("Explore autometrics at http://127.0.0.1:6789");
    println!();
}
