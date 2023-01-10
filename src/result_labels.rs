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
    fn __metrics_attributes_get_result_label(&self) -> Option<&'static str>;
}
impl<T, E> GetLabelsFromResult for Result<T, E> {
    fn __metrics_attributes_get_result_label(&self) -> Option<&'static str> {
        match self {
            Ok(_) => Some("ok"),
            Err(_) => Some("err"),
        }
    }
}
pub trait GetLabels {
    fn __metrics_attributes_get_result_label(&self) -> Option<&'static str> {
        None
    }
}
impl<T> GetLabels for &T {}

// Implement for primitives
impl GetLabels for i8 {}
impl GetLabels for i16 {}
impl GetLabels for i32 {}
impl GetLabels for i64 {}
impl GetLabels for i128 {}
impl GetLabels for isize {}
impl GetLabels for u8 {}
impl GetLabels for u16 {}
impl GetLabels for u32 {}
impl GetLabels for u64 {}
impl GetLabels for u128 {}
impl GetLabels for usize {}
impl GetLabels for f32 {}
impl GetLabels for f64 {}
impl GetLabels for char {}
impl GetLabels for bool {}
impl GetLabels for () {}
