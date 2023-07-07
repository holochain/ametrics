#![deny(missing_docs)]
#![deny(unsafe_code)]
#![deny(warnings)]
//! A metrics abstraction, currently supporting concrete implementations
//! via prometheus or opentelemetry.
//!
//! This is the core crate that should be used in libraries to create
//! and update metrics. In the binary, please see ametrics-prometheus,
//! or ametrics-opentelemetry for the concrete implementation to
//! use for actually handling and reporting the metrics.

use once_cell::sync::OnceCell;
use std::sync::Arc;

/// types
pub mod types {
    use super::*;

    /// MetricValue
    #[derive(Debug)]
    pub enum MetricValue {
        /// an empty metric.
        None,
        /// u64 counter type.
        CounterU64(u64),
        /// f64 counter type.
        CounterF64(f64),
    }

    /// Metric
    #[derive(Debug)]
    pub struct Metric {
        /// Name of metric.
        pub name: String,

        /// Description / help text of metric.
        pub desc: Option<String>,

        /// Unit of metric.
        pub unit: Option<String>,

        /// Type + Value of metric.
        pub value: MetricValue,
    }

    /// Represents an individual counter u64 metric instance.
    pub trait CounterU64: 'static + Send + Sync {
        /// Add to this counter metric.
        fn add(&self, count: u64);
    }

    /// Represents a metrics handler implementation.
    pub trait MetricsHandler: 'static + Send + Sync {
        /// Register a new counter u64 metric.
        fn register_counter_u64(
            &self,
            name: &'static str,
            desc: Option<&'static str>,
            unit: Option<&'static str>,
        ) -> Arc<dyn CounterU64>;

        /// Gather a report of all metrics current values.
        fn report(&self) -> Vec<Metric>;
    }
}

/// Global metric handler.
static GLOBAL_METRICS_HANDLER: OnceCell<Arc<dyn types::MetricsHandler>> = OnceCell::new();

/// Register the global MetricsHandler instance.
/// Call this once from a binary crate to register the concrete handler.
/// Return true only if this was the first sucessful call.
pub fn register_global_metrics_handler<H: types::MetricsHandler>(handler: H) -> bool {
    GLOBAL_METRICS_HANDLER.set(Arc::new(handler)).is_ok()
}

/// Gather a report of all metrics current values.
/// Will return None if no global metrics handler has been registered.
pub fn global_metrics_report() -> Option<Vec<types::Metric>> {
    GLOBAL_METRICS_HANDLER.get().map(|handler| handler.report())
}

/// A u64 based counter metric that can only increase.
pub struct CounterU64 {
    name: &'static str,
    desc: Option<&'static str>,
    unit: Option<&'static str>,
    metric: OnceCell<Arc<dyn types::CounterU64>>,
}

impl CounterU64 {
    /// Construct a new CounterU64 metric.
    pub const fn new(name: &'static str, desc: Option<&'static str>, unit: Option<&'static str>) -> Self {
        Self {
            name,
            desc,
            unit,
            metric: OnceCell::new(),
        }
    }

    /// Add to this metric instance count.
    pub fn add(&self, count: u64) {
        if let Ok(metric) = self.metric.get_or_try_init(|| {
            if let Some(handler) = GLOBAL_METRICS_HANDLER.get() {
                Ok(handler.register_counter_u64(self.name, self.desc, self.unit))
            } else {
                Err(())
            }
        }) {
            metric.add(count);
        }
    }
}
