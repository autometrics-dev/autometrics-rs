//! Tests for the ResultLabels macro

#[test]
fn error_locus_reporting() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/error_locus/fail/*.rs")
}
