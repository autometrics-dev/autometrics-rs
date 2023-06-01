use cfg_aliases::cfg_aliases;

pub fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    cfg_aliases! {
      // Backends
      metrics: { feature = "metrics" },
      opentelemetry: { feature = "opentelemetry" },
      prometheus: { feature = "prometheus" },
      prometheus_client_feature: { feature = "prometheus-client" },
      default_backend: { all(
        prometheus_exporter,
        not(any(metrics, opentelemetry, prometheus, prometheus_client_feature))
      ) },
      prometheus_client: { any(prometheus_client_feature, default_backend) },

      // Misc
      prometheus_exporter: { feature = "prometheus-exporter" },

      // Exemplars
      exemplars: { any(exemplars_tracing, exemplars_tracing_opentelemetry) },
      exemplars_tracing: { feature = "exemplars-tracing" },
      exemplars_tracing_opentelemetry: { feature = "exemplars-tracing-opentelemetry" },

      // Custom objectives
      custom_objective_percentile: { feature = "custom-objective-percentile" },
      custom_objective_latency: { feature = "custom-objective-latency" },
    }
}
