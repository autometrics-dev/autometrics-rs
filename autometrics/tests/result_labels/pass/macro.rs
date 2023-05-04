//! This test uses interfaces not meant to be directly used.
//!
//! The goal here is to make sure that the macro has the effect we want.
//! autometrics (the library) is then responsible for orchestrating the
//! calls to `get_result_labels_for_value!` correctly when observing
//! function call results for the metrics.
use autometrics::get_result_labels_for_value;
use autometrics_macros::ResultLabels;

#[derive(Clone)]
struct Inner {}

#[derive(ResultLabels, Clone)]
enum MyEnum {
    /// When manually marked as 'error', returning this variant will
    /// _ALWAYS_ be considered as an error for Autometrics.
    /// Notably, even if you return Ok(MyEnum::Empty) from a function.
    #[label(result = "error")]
    Empty,
    /// When manually marked as 'ok', returning this variant will
    /// _ALWAYS_ be considered as a succesful result for Autometrics.
    /// Notably, even if you return Err(MyEnum::Empty) from a function.
    #[label(result = "ok")]
    ClientError { inner: Inner },
    /// Without any manual override, Autometrics will guess from the
    /// context when possible to know whether something is an issue or
    /// not. This means:
    /// - Ok(MyEnum::AmbiguousValue(_)) is a success for Autometrics
    /// - Err(MyEnum::AmbiguousValue(_)) is an error for Autometrics
    /// - Just returning MyEnum::AmbiguousValue(_) won't do anything (just like returning
    ///   a bare primitive type like usize)
    AmbiguousValue(u64),
}

fn main() {
    let is_ok = MyEnum::ClientError { inner: Inner {} };
    let labels = get_result_labels_for_value!(&is_ok);
    assert_eq!(labels.unwrap().0, "ok");

    let err = MyEnum::Empty;
    let labels = get_result_labels_for_value!(&err);
    assert_eq!(labels.unwrap().0, "error");

    let no_idea = MyEnum::AmbiguousValue(42);
    let labels = get_result_labels_for_value!(&no_idea);
    assert_eq!(labels, None);

    // Testing behaviour within an Ok() error variant
    let ok: Result<MyEnum, ()> = Ok(is_ok.clone());
    let labels = get_result_labels_for_value!(&ok);
    assert_eq!(
        labels.unwrap().0,
        "ok",
        "When wrapped as the Ok variant of a result, a manually marked 'ok' variant translates to 'ok'."
    );

    let ok: Result<MyEnum, ()> = Ok(no_idea.clone());
    let labels = get_result_labels_for_value!(&ok);
    assert_eq!(
        labels.unwrap().0,
        "ok",
        "When wrapped as the Ok variant of a result, an ambiguous variant translates to 'ok'."
    );

    let err_in_ok: Result<MyEnum, ()> = Ok(err.clone());
    let labels = get_result_labels_for_value!(&err_in_ok);
    assert_eq!(
        labels.unwrap().0,
        "error",
        "When wrapped as the Ok variant of a result, a manually marked 'error' variant translates to 'error'."
    );

    // Testing behaviour within an Err() error variant
    let ok_in_err: Result<(), MyEnum> = Err(is_ok);
    let labels = get_result_labels_for_value!(&ok_in_err);
    assert_eq!(
        labels.unwrap().0,
        "ok",
        "When wrapped as the Err variant of a result, a manually marked 'ok' variant translates to 'ok'."
    );

    let not_ok: Result<(), MyEnum> = Err(err);
    let labels = get_result_labels_for_value!(&not_ok);
    assert_eq!(
        labels.unwrap().0,
        "error",
        "When wrapped as the Err variant of a result, a manually marked 'error' variant translates to 'error'."
    );

    let ambiguous: Result<(), MyEnum> = Err(no_idea);
    let labels = get_result_labels_for_value!(&ambiguous);
    assert_eq!(
        labels.unwrap().0,
        "error",
        "When wrapped as the Err variant of a result, an ambiguous variant translates to 'error'."
    );
}
