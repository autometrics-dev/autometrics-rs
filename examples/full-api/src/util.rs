use crate::routes::CreateUser;
use rand::{thread_rng, Rng};
use std::time::Duration;
use tokio::time::sleep;

/// Make some random API calls to generate data that we can see in the graphs
pub async fn generate_random_traffic() {
    let client = reqwest::Client::new();
    loop {
        let request_type = thread_rng().gen_range(0..3);
        let sleep_duration = Duration::from_millis(thread_rng().gen_range(10..50));
        match request_type {
            0 => {
                let _ = client.get("http://localhost:3000").send().await;
            }
            1 => {
                let _ = client
                    .post("http://localhost:3000/users")
                    .json(&CreateUser {
                        username: "test".to_string(),
                    })
                    .send()
                    .await;
            }
            2 => {
                let _ = reqwest::get("http://localhost:3000/random-error").await;
            }
            _ => unreachable!(),
        }
        sleep(sleep_duration).await
    }
}
