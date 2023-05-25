//! Tracing integration for autometrics
//!
//! This module enables autometrics to use the `trace_id` field from the current span as an exemplar.
//! Exemplars are a newer Prometheus / OpenMetrics / OpenTelemetry feature that allows you to associate
//! specific traces with a given metric. This enables you to dig into the specifics that produced
//! a certain metric by looking at a detailed example.
//!
//! # Example
//! ```rust
//! use autometrics::{autometrics, integrations::tracing::AutometricsExemplarExtractor};
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
//!         .with(AutometricsExemplarExtractor::from_field("trace_id"))
//!         .init();
//! }
//! ```

use std::collections::HashMap;
use tracing::field::{Field, Visit};
use tracing::{span::Attributes, Id, Subscriber};
use tracing_subscriber::layer::{Context, Layer};
use tracing_subscriber::registry::{LookupSpan, Registry};

pub(crate) type TraceLabels = HashMap<&'static str, String>;

/// Get the exemplar from the current tracing span
pub(crate) fn get_exemplar() -> Option<TraceLabels> {
    let span = tracing::span::Span::current();

    span.with_subscriber(|(id, sub)| {
        sub.downcast_ref::<Registry>()
            .and_then(|reg| reg.span(id))
            .and_then(|span| {
                span.scope()
                    .find_map(|span| span.extensions().get::<TraceLabels>().cloned())
            })
    })
    .flatten()
}

/// A tracing [`Layer`] that enables autometrics to use fields from the current span as exemplars for
/// the metrics it produces.
///
/// By default, it will look for a field called `trace_id` in the current span scope and use that
/// as the exemplar. You can customize this by using [`AutometricsExemplarExtractor::from_field`]
/// or [`AutometricsExemplarExtractor::from_fields`].
///
/// # Example
/// ```rust
/// use autometrics::integrations::tracing::AutometricsExemplarExtractor;
///
/// fn main() {
///     tracing_subscriber::fmt::fmt()
///         .finish()
///         .with(AutometricsExemplarExtractor::default())
///         .init();
/// }
/// ```
#[derive(Clone)]
pub struct AutometricsExemplarExtractor {
    fields: &'static [&'static str],
}

impl AutometricsExemplarExtractor {
    /// Create a new [`AutometricsExemplarExtractor`] that will extract the given field from the current [`Span`] scope
    /// to use as the labels for the exemplars.
    pub const fn from_field(field: &'static str) -> Self {
        Self { fields: &[field] }
    }

    /// Create a new [`AutometricsExemplarExtractor`] that will extract the given fields from the current [`Span`] scope
    /// to use as the labels for the exemplars.
    pub const fn from_fields(fields: &'static [&'static str]) -> Self {
        Self { fields }
    }
}

impl Default for AutometricsExemplarExtractor {
    fn default() -> Self {
        Self {
            fields: &["trace_id"],
        }
    }
}

impl<S: Subscriber + for<'lookup> LookupSpan<'lookup>> Layer<S> for AutometricsExemplarExtractor {
    fn on_new_span(&self, attrs: &Attributes<'_>, id: &Id, ctx: Context<'_, S>) {
        let mut visitor = TraceLabelVisitor::new(self.fields);
        attrs.values().record(&mut visitor);

        if !visitor.labels.is_empty() {
            if let Some(span) = ctx.span(id) {
                let mut ext = span.extensions_mut();
                ext.insert(visitor.labels);
            }
        }
    }
}

struct TraceLabelVisitor {
    fields: &'static [&'static str],
    labels: TraceLabels,
}

impl TraceLabelVisitor {
    fn new(fields: &'static [&'static str]) -> Self {
        Self {
            fields,
            labels: HashMap::with_capacity(fields.len()),
        }
    }
}

impl Visit for TraceLabelVisitor {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        if self.fields.contains(&field.name()) && !self.labels.contains_key(field.name()) {
            self.labels.insert(field.name(), format!("{:?}", value));
        }
    }
}
