use autometrics::autometrics;

#[cfg(feature = "prometheus-exporter")]
#[test]
fn main() {
    #[derive(PartialEq, Debug)]
    struct Function(&'static str);

    let _ = autometrics::global_metrics_exporter();

    add(1, 2);
    other_function().unwrap();

    let result = autometrics::encode_global_metrics().unwrap();

    assert_ne!(result, "");
}

#[autometrics]
fn add(a: i32, b: i32) -> i32 {
    a + b
}

/// Example HTTP handler function
#[cfg(feature = "alerts")]
#[autometrics(
    alerts(success_rate = 99.9%, latency(99.9% < 250ms)),
)]
pub async fn get_index_handler() -> Result<String, ()> {
    Ok("Hello world!".to_string())
}

#[cfg(not(feature = "alerts"))]
#[autometrics]
pub async fn get_index_handler() -> Result<String, ()> {
    Ok("Hello world!".to_string())
}

#[autometrics(track_concurrency)]
fn other_function() -> Result<String, ()> {
    Ok("Hello world!".to_string())
}

pub struct Db {}

#[autometrics]
impl Db {
    #[skip_autometrics]
    pub fn new() -> Self {
        Db {}
    }

    pub fn get_user(&self, id: i32) -> Result<String, ()> {
        Ok(format!("User {}", id))
    }

    pub fn get_users(&self) -> Vec<String> {
        Vec::new()
    }
}

trait Foo<'a> {
    fn foo(&'a self) -> Result<String, ()>;
}

#[autometrics]
impl<'a> Foo<'a> for Db {
    fn foo(&self) -> Result<String, ()> {
        Ok("Bar".to_string())
    }
}
