// This test ensures that the macro fails with a readable
// error when the attribute given to one variant inside the
// enumeration does not use the correct key for the label.
use autometrics_macros::ResultLabels;

struct Inner {}

#[derive(ResultLabels)]
enum MyError {
    Empty,
    #[label(unknown = "ok")]
    ClientError {
        inner: Inner,
    },
    ServerError(u64),
}

fn main() {}
