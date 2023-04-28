fn main() {
    vergen::EmitBuilder::builder()
        .git_sha(true) // short commit hash
        .emit()
        .expect("Unable to generate build info");
}
