use autometrics::{autometrics, generate_alerts};

/// Example HTTP handler function
#[autometrics(
    alerts(success_rate = 99.9%, latency(99.9% <= 200ms)),
)]
pub async fn get_index_handler() -> Result<String, ()> {
    Ok("Hello world!".to_string())
}

pub fn main() {
    println!("{}\n", generate_alerts())
}
