use super::*;
use metrics::{Counter, Gauge, Histogram, Key, KeyName, Recorder, SharedString, Unit};
use metrics_util::registry::{AtomicStorage, Registry};
use std::{collections::HashMap, fmt::Result, sync::Mutex};

struct TestRecorder(Registry<Key, AtomicStorage>);

impl Recorder for TestRecorder {
    fn describe_counter(&self, key: KeyName, unit: Option<Unit>, description: SharedString) {
        todo!()
    }

    fn describe_gauge(&self, key: KeyName, _unit: Option<Unit>, _description: SharedString) {
        todo!()
    }

    fn describe_histogram(&self, key: KeyName, unit: Option<Unit>, description: SharedString) {
        todo!()
    }

    fn register_counter(&self, key: &Key) -> Counter {
        Counter::from_arc(self.0.get_or_create_counter(key, |inner| inner.clone()))
    }

    fn register_gauge(&self, key: &Key) -> Gauge {
        Gauge::from_arc(self.0.get_or_create_gauge(key, |inner| inner.clone()))
    }

    fn register_histogram(&self, key: &Key) -> Histogram {
        Histogram::from_arc(self.0.get_or_create_histogram(key, |inner| inner.clone()))
    }
}

#[instrument]
fn add(a: i32, b: i32) -> std::fmt::Result {
    Ok(())
}

#[test]
fn simple_function() {}
