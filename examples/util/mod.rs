use rand::{thread_rng, Rng};
use std::process::{Command, Stdio};
use std::{io::ErrorKind, time::Duration};
use tokio::time::sleep;

const PROMETHEUS_CONFIG_PATH: &str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/examples/prometheus.yml");

pub fn run_prometheus() {
    match Command::new("prometheus")
        .args(["--config.file", PROMETHEUS_CONFIG_PATH])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        Err(err) if err.kind() == ErrorKind::NotFound => {
            eprintln!("Failed to start prometheus (do you have the prometheus binary installed and in your path?)");
        }
        Err(err) => {
            eprintln!("Failed to start prometheus: {}", err);
        }
        Ok(_) => {}
    }
}

pub async fn sleep_random_duration() {
    let sleep_duration = Duration::from_millis(thread_rng().gen_range(0..300));
    sleep(sleep_duration).await;
}
