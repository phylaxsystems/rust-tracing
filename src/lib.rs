//! # `rust-tracing`

#![warn(
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    unreachable_pub,
    clippy::missing_const_for_fn,
    rustdoc::all
)]
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![deny(unused_must_use, rust_2018_idioms)]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

/// Signet utilities.
pub mod utils {
    /// Prometheus metrics utilities.
    pub mod metrics;

    /// OpenTelemetry utilities.
    pub mod otlp;

    /// [`FromEnv`], [`FromEnvVar`] traits and related utilities.
    ///
    /// [`FromEnv`]: from_env::FromEnv
    /// [`FromEnvVar`]: from_env::FromEnvVar
    pub mod from_env;

    /// Tracing utilities.
    pub mod tracing;
}

/// Re-exports of common dependencies.
pub mod deps {
    pub use metrics;
    pub use opentelemetry;
    pub use opentelemetry_otlp;
    pub use opentelemetry_sdk;
    pub use tracing;
    pub use tracing_core;
    pub use tracing_opentelemetry;
    pub use tracing_subscriber;
}

/// Init metrics and tracing, including OTLP if enabled.
///
/// This will perform the following:
/// - Read environment configuration for tracing
/// - Determine whether to enable OTLP
/// - Install a global tracing subscriber, using the OTLP provider if enabled
/// - Read environment configuration for metrics
/// - Install a global metrics recorder and serve it over HTTP on 0.0.0.0
///
/// See [`init_tracing`] and [`init_metrics`] for more
/// details on specific actions taken and env vars read.
///
/// # Returns
///
/// The OpenTelemetry guard, if OTLP is enabled. This guard should be kept alive
/// for the lifetime of the program to ensure the exporter continues to send
/// data to the remote API.
///
/// [`init_tracing`]: utils::tracing::init_tracing
/// [`init_metrics`]: utils::metrics::init_metrics
pub fn trace() -> Option<utils::otlp::OtelGuard> {
    let guard = utils::tracing::init_tracing();
    utils::metrics::init_metrics();
    guard
}

/// Init metrics and tracing with journald, including OTLP if enabled.
///
/// This will perform the following:
/// - Read environment configuration for tracing
/// - Determine whether to enable OTLP
/// - Install a global tracing subscriber, using the OTLP provider if enabled
/// - Read environment configuration for metrics
/// - Install a global metrics recorder and serve it over HTTP on 0.0.0.0
///
/// See [`init_tracing_with_journald`] and [`init_metrics`] for more
/// details on specific actions taken and env vars read.
///
/// # Returns
///
/// The OpenTelemetry guard, if OTLP is enabled. This guard should be kept alive
/// for the lifetime of the program to ensure the exporter continues to send
/// data to the remote API.
///
/// [`init_tracing_with_journald`]: utils::tracing::init_tracing_with_journald
/// [`init_metrics`]: utils::metrics::init_metrics
#[cfg(feature = "journald")]
pub fn trace_with_journald() -> Option<utils::otlp::OtelGuard> {
    let guard = utils::tracing::init_tracing_with_journald();
    utils::metrics::init_metrics();
    guard
}

/// Init tracing only, including OTLP if enabled.
///
/// This will perform the following:
/// - Read environment configuration for tracing
/// - Determine whether to enable OTLP
/// - Install a global tracing subscriber, using the OTLP provider if enabled
///
/// See [`init_tracing`] for more
/// details on specific actions taken and env vars read.
///
/// # Returns
///
/// The OpenTelemetry guard, if OTLP is enabled. This guard should be kept alive
/// for the lifetime of the program to ensure the exporter continues to send
/// data to the remote API.
///
/// [`init_tracing`]: utils::tracing::init_tracing
/// [`init_metrics`]: utils::metrics::init_metrics
pub fn trace_only() -> Option<utils::otlp::OtelGuard> {
    utils::tracing::init_tracing()
}
