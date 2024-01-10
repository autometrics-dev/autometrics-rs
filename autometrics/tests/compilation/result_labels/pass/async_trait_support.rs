use async_trait::async_trait;
use autometrics::autometrics;

// https://github.com/autometrics-dev/autometrics-rs/issues/141

#[async_trait]
trait TestTrait {
    async fn method() -> bool;
    async fn self_method(&self) -> bool;
}

#[derive(Default)]
struct TestStruct;

#[autometrics]
#[async_trait]
impl TestTrait for TestStruct {
    async fn method() -> bool {
        true
    }

    async fn self_method(&self) -> bool {
        true
    }
}

fn main() {
    let ts = TestStruct::default();

    let _ = async move {
        <TestStruct as TestTrait>::method().await;
        ts.self_method().await;
    };
}
