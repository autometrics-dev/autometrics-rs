use std::io;
use autometrics::autometrics;

// general purpose `Result`, part of the std prelude.
// notice both `Ok` and `Err` generic type arguments are explicitly provided
#[autometrics]
fn issue_121_a() -> Result<impl ToString, io::Error> {
    Ok("a")
}

// specialized `Result` which is part of std but not part of the std prelude.
// notice there is only an explicit `Ok` type in the generic args, the `Err` generic argument
// is type-defined
#[autometrics]
fn issue_121_b() -> io::Result<impl ToString> {
    Ok("b")
}

// specialized `Result` which is part of a foreign crate
// notice there is only an explicit `Ok` type in the generic args, the `Err` generic argument
// is type-defined in the foreign crate
#[autometrics]
fn issue_121_c() -> ::http::Result<impl ToString> {
    // CODE STYLE: please keep return formatted this way (with the leading `::`)
    Ok("c")
}

// Result where both `Ok` and `Error` are `impl` types
#[autometrics]
fn issue_121_d() -> Result<impl ToString, impl std::error::Error> {
    Ok("d")
}

#[test]
fn invoke_issue_121() {
    // we need to handle all three code generation cases
    issue_121_a().unwrap();
    issue_121_b().unwrap();
    issue_121_c().unwrap();
    issue_121_d().unwrap();
}
