//! Functionality specific to one of the libraries used to produce metrics.

/// Access the [`Registry`] used to collect metrics when the `prometheus-client` feature is enabled
///
/// [`Registry`]: ::prometheus_client::registry::Registry
#[cfg(prometheus_client)]
pub mod prometheus_client {
    pub use crate::tracker::prometheus_client::REGISTRY;
}
