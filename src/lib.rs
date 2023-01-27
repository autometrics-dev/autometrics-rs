mod labels;
#[cfg(feature = "prometheus-exporter")]
mod prometheus;
mod task_local;
mod tracker;

#[cfg(feature = "prometheus-exporter")]
#[cfg_attr(docsrs, doc(cfg(feature = "prometheus-exporter")))]
pub use self::prometheus::*;
pub use autometrics_macros::autometrics;

// Not public API.
#[doc(hidden)]
pub mod __private {
    use crate::task_local::LocalKey;
    use std::{cell::RefCell, thread_local};

    pub use crate::labels::{GetLabels, GetLabelsFromResult};
    pub use crate::tracker::AutometricsTracker;
    pub use const_format::str_replace;

    /// Task-local value used for tracking which function called the current function
    pub static CALLER: LocalKey<&'static str> = {
        // This does the same thing as the tokio::thread_local macro with the exception that
        // it initializes the value with the empty string.
        // The tokio macro does not allow you to get the value before setting it.
        // However, in our case, we want it to simply return the empty string rather than panicking.
        thread_local! {
            static CALLER_KEY: RefCell<Option<&'static str>> = const { RefCell::new(Some("")) };
        }

        LocalKey { inner: CALLER_KEY }
    };
}
