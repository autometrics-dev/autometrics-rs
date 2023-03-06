use rand::{thread_rng, Rng};
use std::process::{Child, Command, Stdio};
use std::{io::ErrorKind, time::Duration};
use tokio::time::sleep;

const PROMETHEUS_CONFIG_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/prometheus.yml");

pub struct ChildGuard(Child);

impl Drop for ChildGuard {
    fn drop(&mut self) {
        match self.0.kill() {
            Ok(_) => eprintln!("Stopped Prometheus server"),
            Err(_) => eprintln!("Failed to stop Prometheus server"),
        }
    }
}

pub fn run_prometheus() -> ChildGuard {
    match Command::new("prometheus")
        .args(["--config.file", PROMETHEUS_CONFIG_PATH])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        Err(err) if err.kind() == ErrorKind::NotFound => {
            panic!("Failed to start prometheus (do you have the prometheus binary installed and in your path?)");
        }
        Err(err) => {
            panic!("Failed to start prometheus: {}", err);
        }
        Ok(child) => {
            eprintln!(
                "Running Prometheus on port 9090 (using config file: {})\n",
                PROMETHEUS_CONFIG_PATH
            );
            ChildGuard(child)
        }
    }
}

pub async fn sleep_random_duration() {
    let sleep_duration = Duration::from_millis(thread_rng().gen_range(0..300));
    sleep(sleep_duration).await;
}
