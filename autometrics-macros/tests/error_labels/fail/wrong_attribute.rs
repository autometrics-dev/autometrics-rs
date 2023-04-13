// This test ensures that the macro fails with a readable
// error when the attribute given to one variant inside the
// enumeration is not in the correct form.
use autometrics_macros::ErrorLabels;

struct Inner {}

#[derive(ErrorLabels)]
enum MyError {
    Empty,
    #[label]
    ClientError {
        inner: Inner,
    },
    ServerError(u64),
}

fn main() {}
