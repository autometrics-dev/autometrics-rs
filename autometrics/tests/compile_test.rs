use autometrics::autometrics;

#[autometrics]
fn issue_121() -> Result<impl ToString, std::io::Error> {
    Ok("")
}

#[test]
fn invoke_issue_121() {
    issue_121().unwrap();
}
