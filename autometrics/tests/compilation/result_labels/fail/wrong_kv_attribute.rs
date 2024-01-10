// This test ensures that the macro fails with a readable
// error when the attribute given to one variant inside the
// enumeration is not in the correct form.
use autometrics_macros::ResultLabels;

struct Inner {}

#[derive(ResultLabels)]
enum MyError {
    Empty,
    #[label = "error"]
    ClientError {
        inner: Inner,
    },
    ServerError(u64),
}

fn main() {}
