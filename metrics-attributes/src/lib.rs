pub use metrics::{histogram, increment_counter, increment_gauge};
pub use metrics_attributes_macros::instrument;
#[cfg(test)]
mod tests;
