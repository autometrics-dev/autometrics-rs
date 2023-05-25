//! Attach exemplars to the generated metrics (used for connecting metrics to traces)
//!
//! Autometrics integrates with tracing libraries to extract details from the
//! current span context and attach them as exemplars to the metrics it generates.
//!
//! See each of the submodules for details on how to integrate with each tracing library.
//!
//! **Note:** This is currently only supported with the `prometheus-client` metrics library,
//! because that is the only one that currently supports producing metrics with exemplars.

#[cfg(feature = "exemplars-tracing")]
pub mod tracing;
