// This test ensures that the macro fails with a readable error when the
// attribute given to one variant inside the enumeration does not use one of the
// predetermined values (that would make the automatic queries fail, so the
// macros need to forbid wrong usage at compile time)
use autometrics_macros::ResultLabels;

struct Inner {}

#[derive(ResultLabels)]
enum MyError {
    Empty,
    #[label(result = "not ok")]
    ClientError {
        inner: Inner,
    },
    ServerError(u64),
}

fn main() {}
