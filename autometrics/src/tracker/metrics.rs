use crate::{constants::*, labels::Label, tracker::TrackMetrics};
use metrics::{
    describe_counter, describe_gauge, describe_histogram, register_counter, register_gauge,
    register_histogram, Gauge,
};
use std::{sync::Once, time::Instant};

static ONCE: Once = Once::new();

fn describe_metrics() {
    ONCE.call_once(|| {
        describe_counter!(COUNTER_NAME, COUNTER_DESCRIPTION);
        describe_histogram!(HISTOGRAM_NAME, HISTOGRAM_DESCRIPTION);
        describe_gauge!(GAUGE_NAME, GAUGE_DESCRIPTION);
    });
}

pub struct MetricsTracker {
    module: &'static str,
    function: &'static str,
    gauge: Option<Gauge>,
    start: Instant,
}

impl TrackMetrics for MetricsTracker {
    fn function(&self) -> &'static str {
        self.function
    }

    fn module(&self) -> &'static str {
        self.module
    }

    fn start(function: &'static str, module: &'static str, track_concurrency: bool) -> Self {
        describe_metrics();

        let gauge = if track_concurrency {
            let gauge = register_gauge!(GAUGE_NAME, "function" => function, "module" => module);
            gauge.increment(1.0);
            Some(gauge)
        } else {
            None
        };

        Self {
            module,
            function,
            gauge,
            start: Instant::now(),
        }
    }

    fn finish<'a>(self, counter_labels: &[Label]) {
        let duration = self.start.elapsed().as_secs_f64();
        register_counter!(COUNTER_NAME, counter_labels).increment(1);
        register_histogram!(HISTOGRAM_NAME, "function" => self.function, "module" => self.module)
            .record(duration);
        if let Some(gauge) = self.gauge {
            gauge.decrement(1.0);
        }
    }
}
