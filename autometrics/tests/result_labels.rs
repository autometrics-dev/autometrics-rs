//! Tests for the ResultLabels macro

#[test]
fn harness() {
    let t = trybuild::TestCases::new();
    t.pass("tests/result_labels/pass/*.rs");
    t.compile_fail("tests/result_labels/fail/*.rs")
}
