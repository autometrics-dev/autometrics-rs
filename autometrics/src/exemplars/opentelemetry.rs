use super::TraceLabels;
use opentelemetry_api::{trace::TraceContextExt, Context};
use std::iter::FromIterator;

pub fn get_exemplar() -> Option<TraceLabels> {
    let context = Context::current();
    let span = context.span();
    let span_context = span.span_context();

    if span_context.is_valid() {
        Some(TraceLabels::from_iter([
            ("trace_id", span_context.trace_id().to_string()),
            ("span_id", span_context.span_id().to_string()),
        ]))
    } else {
        None
    }
}
