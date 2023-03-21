// Use the unstable `doc_cfg` feature when docs.rs is building the documentation
// https://stackoverflow.com/questions/61417452/how-to-get-a-feature-requirement-tag-in-the-documentation-generated-by-cargo-do/61417700#61417700
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![cfg_attr(docsrs, feature(doc_cfg_hide))]
#![cfg_attr(docsrs, doc(cfg_hide(doc)))]
#![doc = include_str!("../README.md")]

mod constants;
mod labels;
pub mod objectives;
#[cfg(feature = "prometheus-exporter")]
mod prometheus_exporter;
mod task_local;
mod tracker;

pub extern crate linkme;

pub use labels::GetLabel;

pub extern crate autometrics_macros;
pub use autometrics_macros::{autometrics, AutometricsLabel};

// Optional exports
#[cfg(feature = "prometheus-exporter")]
pub use self::prometheus_exporter::*;

// Not public API
// Note that this needs to be publicly exported (despite being called private)
// because it is used by code generated by the autometrics macro.
// We could move more or all of the code into the macro itself.
// However, the compiler would need to compile a lot of duplicate code in every
// instrumented function. It's also harder to develop and maintain macros with
// too much generated code, because rust-analyzer treats the macro code as a kind of string
// so you don't get any autocompletion or type checking.
#[doc(hidden)]
pub mod __private {
    use crate::task_local::LocalKey;
    use std::{cell::RefCell, thread_local};

    pub use linkme::distributed_slice;
    pub use crate::labels::*;
    pub use crate::tracker::{AutometricsTracker, TrackMetrics};

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
