use crate::utils::from_env::{
    FromEnv,
    FromEnvErr,
    FromEnvVar,
};
use metrics_exporter_prometheus::PrometheusBuilder;

use super::from_env::EnvItemInfo;

/// Metrics port env var
const TRACING_METRICS_PORT: &str = "TRACING_METRICS_PORT";

/// Prometheus metrics configuration struct.
///
/// Uses the following environment variables:
/// - `TRACING_METRICS_PORT` - optional. Defaults to 9000 if missing or unparseable.
///   The port to bind the metrics server to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
#[non_exhaustive]
#[serde(from = "Option<u16>")]
pub struct MetricsConfig {
    /// `TRACING_METRICS_PORT` - The port on which to bind the metrics server. Defaults
    /// to `9000` if missing or unparseable.
    pub port: u16,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self { port: 9000 }
    }
}

impl From<Option<u16>> for MetricsConfig {
    fn from(port: Option<u16>) -> Self {
        Self {
            port: port.unwrap_or(9000),
        }
    }
}

impl From<u16> for MetricsConfig {
    fn from(port: u16) -> Self {
        Self { port }
    }
}

impl FromEnv for MetricsConfig {
    type Error = std::num::ParseIntError;

    fn inventory() -> Vec<&'static EnvItemInfo> {
        vec![&EnvItemInfo {
            var: TRACING_METRICS_PORT,
            description: "Port on which to serve metrics, u16, defaults to 9000",
            optional: true,
        }]
    }

    fn from_env() -> Result<Self, FromEnvErr<Self::Error>> {
        match u16::from_env_var(TRACING_METRICS_PORT).map(Self::from) {
            Ok(cfg) => Ok(cfg),
            Err(_) => Ok(Self::default()),
        }
    }
}

/// Initialize a [`metrics_exporter_prometheus`] exporter.
///
/// Reads the `TRACING_METRICS_PORT` environment variable to determine the port to bind
/// the exporter to. If the variable is missing or unparseable, it defaults to
/// 9000.
///
/// See [`MetricsConfig`] for more information.
///
/// # Panics
///
/// This function will panic if the exporter fails to install, e.g. if the port
/// is in use.
pub fn init_metrics() {
    let cfg = MetricsConfig::from_env().unwrap();

    PrometheusBuilder::new()
        .with_http_listener(([0, 0, 0, 0], cfg.port))
        .install()
        .expect("failed to install prometheus exporter");
}
