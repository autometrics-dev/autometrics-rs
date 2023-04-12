//! Tests for the ErrorLabels macro

#[test]
fn harness() {
    let t = trybuild::TestCases::new();
    t.pass("tests/error_labels/pass/*.rs");
    t.compile_fail("tests/error_labels/fail/*.rs")
}
