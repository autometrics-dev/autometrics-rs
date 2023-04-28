//! This test uses interfaces not meant to be directly used.
//!
//! The goal here is to make sure that the macro has the effect we want.
//! autometrics (the library) is then responsible for orchestrating the
//! calls to `__autometrics_get_error_label` correctly when observing
//! function call results for the metrics.
use autometrics::__private::GetResultLabelFromEnum;
use autometrics_macros::ResultLabels;

struct Inner {}

#[derive(ResultLabels)]
enum MyError {
    #[label(result = "error")]
    Empty,
    #[label(result = "ok")]
    ClientError {
        inner: Inner,
    },
    ServerError(u64),
}

fn main() {
    let err = MyError::ClientError { inner: Inner {} };
    assert_eq!(err.__autometrics_get_result_label(), "ok");

    let err = MyError::Empty;
    assert_eq!(err.__autometrics_get_result_label(), "error");

    let err = MyError::ServerError(502);
    assert_eq!(err.__autometrics_get_result_label(), "error");
}
