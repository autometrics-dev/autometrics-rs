// This is a convoluted way to figure out if the return type resolves to a Result
// or not. We cannot simply parse the code using syn to figure out if it's a Result
// because syn doesn't do type resolution and thus would count any renamed version
// of Result as a different type. Instead, we define two traits with intentionally
// conflicting method names and use a trick based on the order in which Rust resolves
// method names to return a different value based on whether the return value is
// a Result or anything else.
// This approach is based on dtolnay's answer to this question:
// https://users.rust-lang.org/t/how-to-check-types-within-macro/33803/5
// and this answer explains why it works:
// https://users.rust-lang.org/t/how-to-check-types-within-macro/33803/8

pub trait GetLabelsFromResult {
    fn __metrics_attributes_get_labels(&self) -> &'static [(&'static str, &'static str)];
}
impl<T, E> GetLabelsFromResult for Result<T, E> {
    fn __metrics_attributes_get_labels(&self) -> &'static [(&'static str, &'static str)] {
        match self {
            Ok(_) => &[("result", "ok")],
            Err(_) => &[("result", "err")],
        }
    }
}
pub trait GetLabels {
    fn __metrics_attributes_get_labels(&self) -> &'static [(&'static str, &'static str)] {
        &[]
    }
}
impl<T> GetLabels for &T {}
