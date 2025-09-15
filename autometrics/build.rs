use cfg_aliases::cfg_aliases;

pub fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    #[cfg(feature = "metrics")]
    println!("cargo:warning=The `metrics` feature is deprecated and will be removed in the next version. Please use `metrics-0_24` instead.");
    #[cfg(feature = "opentelemetry")]
    println!("cargo:warning=The `opentelemetry` feature is deprecated and will be removed in the next version. Please use `opentelemetry-0_30` instead.");
    #[cfg(feature = "prometheus")]
    println!("cargo:warning=The `prometheus` feature is deprecated and will be removed in the next version. Please use `prometheus-0_14` instead.");
    #[cfg(feature = "prometheus-client")]
    println!("cargo:warning=The `prometheus-client` feature is deprecated and will be removed in the next version. Please use `prometheus-client-0_22` instead.");
    #[cfg(feature = "exemplars-tracing-opentelemetry")]
    println!("cargo:warning=The `exemplars-tracing-opentelemetry` feature is deprecated and will be removed in the next version. Please use `exemplars-tracing-opentelemetry-0_31` instead.");

    cfg_aliases! {
      // Backends
      metrics: { any(feature = "metrics", feature = "metrics-0_24") },
      opentelemetry: { any(feature = "opentelemetry", feature = "opentelemetry-0_30") },
      prometheus: { any(feature = "prometheus", feature = "prometheus-0_14") },
      prometheus_client_feature: { any(feature = "prometheus-client", feature = "prometheus-client-0_22") },
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
      exemplars_tracing_opentelemetry: { any(feature = "exemplars-tracing-opentelemetry-0_31", feature = "exemplars-tracing-opentelemetry") },

      // Custom objectives
      custom_objective_percentile: { feature = "custom-objective-percentile" },
      custom_objective_latency: { feature = "custom-objective-latency" },
    }
}
