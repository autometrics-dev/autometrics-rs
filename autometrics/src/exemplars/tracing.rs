//! Extract fields from [`tracing::Span`]s as exemplars.
//!
//! This module enables autometrics to use fields from the current [`Span`] as exemplar labels.
//!
//! # Example
//!
//! ```rust
//! use autometrics::autometrics;
//! use autometrics::exemplars::tracing::AutometricsExemplarExtractor;
//! use tracing::{instrument, trace};
//! use tracing_subscriber::prelude::*;
//! use uuid::Uuid;
//!
//! #[autometrics]
//! #[instrument(fields(trace_id = %Uuid::new_v4()))]
//! fn my_function() {
//!     trace!("Hello world!");
//! }
//!
//! fn main() {
//!     tracing_subscriber::fmt::fmt()
//!         .finish()
//!         .with(AutometricsExemplarExtractor::from_fields(&["trace_id"]))
//!         .init();
//! }
//! ```
//!
//! [`Span`]: tracing::Span

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

/// A [`tracing_subscriber::Layer`] that enables autometrics to use fields from the current span as exemplars for
/// the metrics it produces.
///
/// # Example
/// ```rust
/// use autometrics::exemplars::tracing::AutometricsExemplarExtractor;
/// use tracing_subscriber::prelude::*;
///
/// fn main() {
///     tracing_subscriber::fmt::fmt()
///         .finish()
///         .with(AutometricsExemplarExtractor::from_fields(&["trace_id"]))
///         .init();
/// }
/// ```
#[derive(Clone)]
pub struct AutometricsExemplarExtractor {
    fields: &'static [&'static str],
}

impl AutometricsExemplarExtractor {
    /// Create a new [`AutometricsExemplarExtractor`] that will extract the given fields from the current [`Span`] scope
    /// to use as the labels for the exemplars.
    ///
    /// [`Span`]: tracing::Span
    pub fn from_fields(fields: &'static [&'static str]) -> Self {
        Self { fields }
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

    fn record_str(&mut self, field: &Field, value: &str) {
        if self.fields.contains(&field.name()) && !self.labels.contains_key(field.name()) {
            self.labels.insert(field.name(), value.to_string());
        }
    }
}
