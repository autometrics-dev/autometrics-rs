//! Tests relying on macros or compiler diagnostics

#[test]
fn harness() {
    let t = trybuild::TestCases::new();

    // Test the ResultLabels macro
    t.pass("tests/compilation/result_labels/pass/*.rs");
    t.compile_fail("tests/compilation/result_labels/fail/*.rs");

    // Test that compiler reports errors in the correct location
    t.compile_fail("tests/compilation/error_locus/fail/*.rs");
}
