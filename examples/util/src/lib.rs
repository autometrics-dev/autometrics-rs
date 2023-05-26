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

pub fn run_prometheus(enable_exemplars: bool) -> ChildGuard {
    let mut args = vec!["--config.file", PROMETHEUS_CONFIG_PATH];
    if enable_exemplars {
        args.push("--enable-feature=exemplar-storage");
    }

    match Command::new("prometheus")
        .args(&args)
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
                "Running Prometheus on port 9090 (using config file: {PROMETHEUS_CONFIG_PATH})",
            );
            if enable_exemplars {
                eprintln!(
                    "Exemplars are enabled (using the flag: --enable-feature=exemplar-storage)"
                );
            }
            ChildGuard(child)
        }
    }
}

pub async fn sleep_random_duration() {
    let sleep_duration = Duration::from_millis(thread_rng().gen_range(0..300));
    sleep(sleep_duration).await;
}
