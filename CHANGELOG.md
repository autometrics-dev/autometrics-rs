# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## Unreleased

## [3.0.0](https://github.com/autometrics-dev/autometrics-rs/releases/tag/v3.0.0) - 2025-09-15

### Breaking changes
- `metrics` has been updated to v0.24 (#183)
  New feature flag: `metrics-0_24`
  Removed feature flag: `metrics-0_21`
- `prometheus` has been updated to v0.14 (#187)
  New feature flag: `prometheus-0_14`
  Removed feature flag: `prometheus-0_13`
- No more `prometheus-exporter` with `opentelemetry`, use standard otlp exporter (see <https://github.com/open-telemetry/opentelemetry-rust/pull/2831>)
- Update `opentelemetry` to v0.30

## [2.0.0](https://github.com/autometrics-dev/autometrics-rs/releases/tag/v2.0.0) - 2024-07-25

### Breaking changes

- `opentelemetry` has been updated to v0.24 (#179)  
  New feature flags: `opentelemetry-0_24`, `exemplars-tracing-opentelemetry-0_25`  
  Removed feature flags: `opentelemetry-0_21`, `exemplars-tracing-opentelemetry-0_22`

**If you are using these metrics library separately in your application in addition 
to Autometrics, ensure that you match the version that Autometrics uses.**

## [1.0.1](https://github.com/autometrics-dev/autometrics-rs/releases/tag/v1.0.1) - 2024-02-12

- Update `http` to `1.0`. This fixes compatibility with `axum 0.7` (#167)
- Explicitly set default timeout and period for the OTEL push exporter (#168)
- Fix a regression that made all compilation errors in instrumented code appear as located in
  the `#[autometrics]` annotation instead of the original location (#170)
- Add missing feature flag for otel-push-exporter (#172)

## [1.0.0](https://github.com/autometrics-dev/autometrics-rs/releases/tag/v1.0.0) - 2023-12-01

### Breaking changes

- `opentelemetry` has been updated to v0.21 (#159)  
  New feature flags: `opentelemetry-0_21`, `exemplars-tracing-opentelemetry-0_22`  
  Removed feature flags: `opentelemetry-0_20`, `exemplars-tracing-opentelemetry-0_20`, `exemplars-tracing-opentelemetry-0_21`
- `prometheus-client` has been updated to v0.22 (#160)  
  New feature flag: `prometheus-client-0_22`  
  Removed feature flag: `prometheus-client-0_21`

**If you are using these metrics separately in your application in addition to Autometrics,
ensure that you match the version that Autometrics uses.**

### General changes

- New exporter, `otel_push_exporter` is now available in addition to the existing
  `prometheus_exporter`. It can be used to push metrics in the OTEL format via
  HTTP or gRPC to a OTEL-collector-(compatible) server.
- Update to `syn` v2 (#145)
- Add support for tracing-opentelemetry v0.21 (#147)
- Error messages when using `#[autometrics]` together with `#[async_trait]`
  has been improved (#149)
- Fixed missing feature flag for `opentelemetry-otlp` when autometrics feature
  `otel-push-exporter` is enabled
- Fixed incorrect duration being recorded when using `#[async_trait]` together with `#[autometrics]` (#161)   
  **Please note that the `#[autometrics]` macro needs to be defined BEFORE `#[async_trait]`.**
- Fixed value of the `result` label being empty when the function is annotated with `#[async_trait]` (#161)
- Fixed `#[async_trait]` not being correctly detected if its re-exported in another crate (#164)

### Autometrics 1.0 compliance

The [Autometrics specification v1.0.0](https://github.com/autometrics-dev/autometrics-shared/blob/main/specs/autometrics_v1.0.0.md) has been released.
The following changes are made in order to bring the release in compliance with the spec:

- Added `repository.url` and `repository.provider` as `build_info` labels. They can be
  configured either through environment variables or automatically based on your `Cargo.toml`
  `package.repository` value (only GitHub, GitLab and BitBucket) (#152)
- Added `autometrics.version` as `build_info` label, specifying the autometrics spec
  version that we are targeting. It is currently hardcoded to `1.0.0` (#154)
- Environment variables for `repository.url` and `repository.provider` will now be read at 
  both compile time and runtime (#156)

## [0.6.0](https://github.com/autometrics-dev/autometrics-rs/releases/tag/v0.5.0) - 2023-08-08

### Added

- Autometrics settings can now be configured via `settings::AutometricsSettings::builder()`
- A custom `Registry` can be used to collect metrics. This may be used to add
  custom metrics alongside those produced by Autometrics, as well as to export
  the metrics without using the provided `prometheus_exporter`
- All metrics now have a `service.name` label attached. This is set via the settings,
  via runtime environment variable (`AUTOMETRICS_SERVICE_NAME` or `OTEL_SERVICE_NAME`),
  or it falls back to the crate name defined in the `Cargo.toml`
- Function counters are initialized to zero in debug builds. This exposes details of
  instrumented functions to Prometheus and visualization tools built on top of it,
  before the functions have been called.
- Basic benchmarking

### Changed

- Renamed the `function.calls.count` metric to `function.calls` (which is exported
  to Prometheus as `function_calls_total`) to be in line with OpenTelemetry and
  OpenMetrics naming conventions. **Dashboards and alerting rules must be updated.**
- When the `function.calls.duration` histogram is exported to Prometheus, it now
  includes the units (`function_calls_duration_seconds`) to be in line with
  Prometheus/OpenMetrics naming conventions. **Dashboards and alerting rules must be updated.**
- Struct methods now have the struct name as part of the `function` label on the metrics
  (for example, `MyStruct::my_method`)
- The `caller` label on the `function.calls` metric was replaced with `caller.function`
  and `caller.module`
- The `custom-objective-latency` feature can now be used with the `prometheus-exporter`, as well
  as with the `prometheus` and `prometheus-client` crates, because the histogram buckets can now
  be configured via the settings
- Upgraded `opentelemetry` to v0.20

### Deprecated

- `metrics` feature flag (replaced with `metrics-0_21`)
- `opentelemetry` feature flag (replaced with `opentelemetry-0_20`)
- `prometheus` feature flag (replaced with `prometheus-0_13`)
- `prometheus-client` feature flag (replaced with `prometheus-client-0_21`)
- `exemplars-tracing-opentelemetry` feature flag (replaced with `exemplars-tracing-opentelemetry-0_20`)

### Removed

- `encode_global_metrics` was removed and replaced by `prometheus_exporter::encode_to_string`
- `global_metrics_exporter` was removed and replaced by `prometheus_exporter::init`
- `backends::prometheus_client::REGISTRY` was removed. The `Registry` used with the `prometheus-client`
  backend can now be accessed via `AutometricsSettings::prometheus_client_registry`

### Fixed

- Return types on functions annotated with `#[autometrics]` containing generic
  `impl` types in their type arguments (`fn() -> Result<impl ToString, impl std::error::Error>`)
  no longer fail to compile.

## [0.5.0](https://github.com/autometrics-dev/autometrics-rs/releases/tag/v0.5.0) - 2023-06-02

### Added

- Support the official `prometheus-client` crate for producing metrics
- Support exemplars when using the feature flags `exemplars-tracing` or `exemplars-tracing-opentelemetry`.
  Autometrics can now extract fields from the current span and attach them as exemplars on the
  counter and histogram metrics
- `ResultLabels` derive macro allows to specify on an enum whether variants should
  always be "ok", or "error" for the success rate metrics of functions using them. (#61)
- The `prometheus_exporter` module contains all functions related to the `prometheus-exporter` feature
- `prometheus_exporter::encode_http_response` function returns an `http::Response` with the metrics.
  This is especially recommended when using exemplars, because it automatically uses the OpenMetrics
  `Content-Type` header, which is required for Prometheus to scrape metrics with exemplars
- `AUTOMETRICS_DISABLE_DOCS` environment variable can be set to disable doc comment generation
  (this is mainly for use with editor extensions that generate doc comments themselves)

### Changed

- Users must configure the metrics library they want to use autometrics with, unless the
  `prometheus-exporter` feature is enabled. In that case, the official `prometheus-client` will be used
- Metrics library feature flags are now mutually exclusive (previously, `autometrics` would only
  produce metrics using a single metrics library if multiple feature flags were enabled, using
  a prioritization order defined internally)
- `GetLabels` trait (publicly exported but meant for internal use) changed the signature
  of its function to accomodate the new `ResultLabels` macro. This change is not significant
  if you never imported `autometrics::__private` manually (#61)
- When using the `opentelemetry` together with the `prometheus-exporter`, it will no longer
  use the default registry provided by the `prometheus` crate. It will instead use a new registry
- Updated `opentelemetry` dependencies to v0.19. This means that users using autometrics
  with `opentelemetry` but not using the `prometheus-exporter` must update the `opentelemetry`
  to use v0.19.

### Deprecated
- `global_metrics_exporter` and `encode_global_metrics` have been deprecated and replaced by
  `prometheus_exporter::init` and `prometheus_exporter::encode_to_string`, respectively

### Removed

- `opentelemetry` is no longer used by default
- `GetLabelsForResult` trait (publicly exported but meant for internal use) was removed
  to accomodate the new `ResultLabels` macro. This change is not significant
  if you never imported `autometrics::__private` manually (#61)

### Fixed

- `#[autometrics]` now works on functions that use type inference in their return statement
  (#74, #61)

## [0.4.1](https://github.com/autometrics-dev/autometrics-rs/releases/tag/v0.4.1) - 2023-05-05

### Changed

- Overhaul documentation

### Fixed

- Generated latency query needs additional labels to show lines for 95th and 99th percentiles

## [0.4.0](https://github.com/autometrics-dev/autometrics-rs/releases/tag/v0.4.0) - 2023-04-26

### Added

- `build_info` metric tracks software version and commit
- Queries now use `build_info` metric to correlate version info with problems

### Fixed

- Prometheus rules handle when the counter metric name has a `_total` suffix

## [0.3.3](https://github.com/autometrics-dev/autometrics-rs/releases/tag/v0.3.3) - 2023-04-14

### Added

- Alerts have minimum traffic threshold of 1 request / minute

### Fixed

- Latency SLO total query in alerting rules

## [0.3.1](https://github.com/autometrics-dev/autometrics-rs/releases/tag/v0.3.1) - 2023-03-21

### Added

- `custom-objective-latency` and `custom-objective-percentile` feature flags

### Changed

- Use the OpenTelemetry default histogram buckets

### Removed

- Remove the latency objective values for 150, 200, and 350 milliseconds
- `custom-objectives` feature flag

## [0.3.0](https://github.com/autometrics-dev/autometrics-rs/releases/tag/v0.3.0) - 2023-03-14

### Added

- Support defining Service-Level Objectives (SLOs) in code
- CLI to generate Sloth file, which is then used to generate Prometheus alerting rules
- `#[skip_autometrics]` annotation when applying autometrics to an `impl` block
- `ok_if` and `error_if` autometrics parameters

## [0.2.4](https://github.com/autometrics-dev/autometrics-rs/releases/tag/v0.2.4) - 2023-02-08

### Fixed

- Histogram buckets


## [0.2.3](https://github.com/autometrics-dev/autometrics-rs/releases/tag/v0.2.3) - 2023-01-31

### Fixed

- Building of documentation on docs.rs

## [0.2.0](https://github.com/autometrics-dev/autometrics-rs/releases/tag/v0.2.0) - 2023-01-31

### Added

- Support `opentelemetry` and `metrics` crates for tracking metrics
- Support applying autometrics to an `impl` block

### Changed

- Tracking function concurrency is optional

## [0.1.1](https://github.com/autometrics-dev/autometrics-rs/releases/tag/v0.1.1) - 2023-01-27

### Added

- Track concurrent requests
- Add return types as labels
- Separate function call counter
- `caller` label

### Changed

- Use OpenTelemetry metric naming conventions
