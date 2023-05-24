//! Tracing integration for autometrics
//!
//! This module enables autometrics to use the `trace_id` field from the current span as an exemplar.
//! Exemplars are a newer Prometheus / OpenMetrics / OpenTelemetry feature that allows you to associate
//! specific traces with a given metric. This enables you to dig into the specifics that produced
//! a certain metric by looking at a detailed example.
//!
//! # Example
//! ```rust
//! use autometrics::{autometrics, integrations::tracing::AutometricsLayer};
//! use tracing::{instrument, trace};
//! use tracing_subscriber::prelude::*;
//! use uuid::Uuid;
//!
//! #[autometrics]
//! #[instrument(fields(trace_id = %Uuid::new_v4())]
//! fn my_function() {
//!     trace!("Hello world!");
//! }
//!
//! fn main() {
//!     tracing_subscriber::fmt::fmt()
//!         .finish()
//!         .with(AutometricsLayer::default())
//!         .init();
//! }
//! ```

use tracing::field::{Field, Visit};
use tracing::{span::Attributes, Id, Subscriber};
use tracing_subscriber::layer::{Context, Layer};
use tracing_subscriber::registry::{LookupSpan, Registry};

#[derive(Clone, Hash, PartialEq, Eq, Debug, Default)]
#[cfg_attr(
    feature = "prometheus-client",
    derive(prometheus_client::encoding::EncodeLabelSet)
)]
pub(crate) struct TraceLabel {
    trace_id: String,
}

impl Visit for TraceLabel {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        if field.name() == "trace_id" {
            self.trace_id = format!("{:?}", value);
        }
    }
}

/// Get the exemplar from the current tracing span
pub(crate) fn get_exemplar() -> Option<TraceLabel> {
    let span = tracing::span::Span::current();

    span.with_subscriber(|(id, sub)| {
        sub.downcast_ref::<Registry>()
            .and_then(|reg| reg.span(id))
            .and_then(|span| {
                span.scope()
                    .find_map(|span| span.extensions().get::<TraceLabel>().cloned())
            })
    })
    .flatten()
}

/// A tracing [`Layer`] that enables autometrics to use the `trace_id` field from the current span
/// as an exemplar for Prometheus metrics.
///
/// # Example
/// ```rust
/// use autometrics::integrations::tracing::AutometricsLayer;
///
/// fn main() {
///     tracing_subscriber::fmt::fmt()
///         .finish()
///         .with(AutometricsLayer::default())
///         .init();
/// }
/// ```
#[derive(Clone, Default)]
pub struct AutometricsLayer();

impl<S: Subscriber + for<'lookup> LookupSpan<'lookup>> Layer<S> for AutometricsLayer {
    fn on_new_span(&self, attrs: &Attributes<'_>, id: &Id, ctx: Context<'_, S>) {
        let mut trace_label = TraceLabel::default();
        attrs.values().record(&mut trace_label);

        if !trace_label.trace_id.is_empty() {
            if let Some(span) = ctx.span(id) {
                let mut ext = span.extensions_mut();
                ext.insert(trace_label);
            }
        }
    }
}
