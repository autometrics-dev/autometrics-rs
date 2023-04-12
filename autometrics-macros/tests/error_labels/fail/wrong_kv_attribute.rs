use autometrics_macros::ErrorLabels;

struct Inner {}

#[derive(ErrorLabels)]
enum MyError {
    Empty,
    #[label = "error"]
    ClientError {
        inner: Inner,
    },
    ServerError(u64),
}

fn main() {}
