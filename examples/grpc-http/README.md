# gRPC service built with Tonic, http server build with warp, and Instrumented with Autometrics

 This code example has been adapted and modified from a Blog post by Mies Hernandez van Leuffen: [Adding observability to Rust gRPC services using Tonic and Autometrics](https://autometrics.dev/blog/adding-observability-to-rust-grpc-services-using-tonic-and-autometrics). 

## Overview 

This example shows how to: 
* Add observability to a gRPC services
* Add a http service 
* Start both, the grpc and http server 
* Add a graceful shutdown to both, grpc and http server
* Closes a DB connection during graceful shutdown

### Install the protobuf compiler

The protobuf compiler (protoc) compiles protocol buffers into Rust code.
Cargo will call protoc automatically during the build process, but you will
get an error when protoc is not installed. Therefore, ensure protoc is installed.

The recommended installation for macOS is via [Homebrew](https://brew.sh/):

```bash
brew install protobuf
```
Check if the installation worked correctly:

```bash
protoc --version
```

## Local Observability Development

The easiest way to get up and running with this application is to clone the repo and get a local Prometheus setup using the [Autometrics CLI](https://github.com/autometrics-dev/am).

Read more about Autometrics in Rust [here](https://github.com/autometrics-dev/autometrics-rs) and general docs [here](https://docs.autometrics.dev/). 


### Install the Autometrics CLI

The recommended installation for macOS is via [Homebrew](https://brew.sh/):

```
brew install autometrics-dev/tap/am
```

Alternatively, you can download the latest version from the [releases page](https://github.com/autometrics-dev/am/releases)

Spin up local Prometheus and start scraping your application that listens on port :8080.

```
am start :8080
```

If you now inspect the Autometrics explorer on `http://localhost:6789` you will see your metrics. However, upon first start, all matrics are
empty because no request has been sent yet. 

Now you can test your endpoints and generate some traffic and refresh the autometrics explorer to see you metrics. 

### Starting the Service

```bash
cargo run
```

Expected output:

```
Started gRPC server on port 50051
Started metrics on port 8080
Explore autometrics at http://127.0.0.1:6789
```

### Stopping the Service 

You can stop the service either via ctrl-c ore by sending a SIGTERM signal to kill the process. This has been implemented for Windows, Linux, Mac, and should also work on Docker and Kubernetes. 

On Windows, Linux, or Mac, just hit Ctrl-C 

Alternatively, you can send a SIGTERM signal from another process
using the kill command on Linux or Mac. 

In a second terminal, run

```bash
 ps | grep grpc-http
```

Sample output:

```
73014 ttys002    0:00.25 /Users/.../autometrics-rs/target/debug/grpc-http
```

In this example, the service runs on PID 73014. Let's send a sigterm signal to shutdown the service. On you system, a different PID will be returned so please use that one instead. 

```bash
kill 73014
```

Expected output:

```
Received SIGTERM
DB connection closed
gRPC shutdown complete
http shutdown complete
```


## Testing the GRPC endpoints

Easiest way to test the endpoints is with `grpcurl` (`brew install grpcurl`).

```bash
grpcurl -plaintext -import-path ./proto -proto job.proto -d '{"name": "Tonic"}' 'localhost:50051' job.JobRunner.SendJob
```

returns

```
{
  "message": "Hello Tonic!"
}
```

Getting the list of jobs (currently hardcoded to return one job)

```bash
grpcurl -plaintext -import-path ./proto -proto job.proto -d '{}' 'localhost:50051' job.JobRunner.ListJobs
```

returns:

```
{
  "job": [
    {
      "id": 1,
      "name": "test"
    }
  ]
}
```

## Viewing the metrics

When you inspect the Autometrics explorer on `http://localhost:6789` you will see your metrics and SLOs. The explorer shows four tabs:

1) Dashboard: Aggregated overview of all metrics
2) Functions: Detailed metrics for each instrumented API function
3) SLO's: Service Level Agreements for each instrumented API function
4) Alerts: Notifications of violated SLO's or any other anomaly. 

