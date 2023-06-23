use crate::constants::*;
use crate::labels::{BuildInfoLabels, CounterLabels, GaugeLabels, HistogramLabels};
use crate::tracker::TrackMetrics;
use metrics::{
    describe_counter, describe_gauge, describe_histogram, register_counter, register_gauge,
    register_histogram, Gauge, Unit,
};
use std::{sync::Once, time::Instant};

static DESCRIBE_METRICS: Once = Once::new();
static SET_BUILD_INFO: Once = Once::new();

fn describe_metrics() {
    DESCRIBE_METRICS.call_once(|| {
        describe_counter!(COUNTER_NAME_PROMETHEUS, COUNTER_DESCRIPTION);
        describe_histogram!(
            HISTOGRAM_NAME_PROMETHEUS,
            Unit::Seconds,
            HISTOGRAM_DESCRIPTION
        );
        describe_gauge!(GAUGE_NAME_PROMETHEUS, GAUGE_DESCRIPTION);
        describe_gauge!(BUILD_INFO_NAME, BUILD_INFO_DESCRIPTION);
    });
}

pub struct MetricsTracker {
    gauge: Option<Gauge>,
    start: Instant,
}

impl TrackMetrics for MetricsTracker {
    fn start(gauge_labels: Option<&GaugeLabels>) -> Self {
        describe_metrics();

        let gauge = if let Some(gauge_labels) = gauge_labels {
            let gauge = register_gauge!(GAUGE_NAME, &gauge_labels.to_array());
            gauge.increment(1.0);
            Some(gauge)
        } else {
            None
        };

        Self {
            gauge,
            start: Instant::now(),
        }
    }

    fn finish(self, counter_labels: &CounterLabels, histogram_labels: &HistogramLabels) {
        let duration = self.start.elapsed().as_secs_f64();
        register_counter!(COUNTER_NAME_PROMETHEUS, &counter_labels.to_vec()).increment(1);
        register_histogram!(HISTOGRAM_NAME_PROMETHEUS, &histogram_labels.to_vec()).record(duration);
        if let Some(gauge) = self.gauge {
            gauge.decrement(1.0);
        }
    }

    fn set_build_info(build_info_labels: &BuildInfoLabels) {
        SET_BUILD_INFO.call_once(|| {
            register_gauge!(BUILD_INFO_NAME, &build_info_labels.to_vec()).set(1.0);
        });
    }
}
