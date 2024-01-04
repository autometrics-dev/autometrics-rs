//! Tests for the ResultLabels macro

#[test]
fn harness() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/error_locus/fail/*.rs")
}
