mod result_labels;
#[cfg(test)]
mod tests;

pub use autometrics_macros::autometrics;

// Not public API.
#[doc(hidden)]
pub mod __private {
    pub use crate::result_labels::*;
    pub use const_format::str_replace;
}
