#![deny(missing_docs)]
#![deny(unsafe_code)]
#![deny(warnings)]
//! Opentelemetry handlers for ametrics::register_global_metrics_handler.

use std::sync::Arc;

/// types
mod types {
    pub struct OTelCounterU64(pub opentelemetry::metrics::Counter<u64>);

    impl ametrics::types::CounterU64 for OTelCounterU64 {
        fn add(&self, count: u64) {
            self.0.add(&opentelemetry::Context::current(), count, &[]);
        }
    }
}

struct Bob;

impl opentelemetry::sdk::metrics::reader::MetricReader for Bob {
    fn register_pipeline(&self, _pipeline: std::sync::Weak<opentelemetry::sdk::metrics::Pipeline>) {
        todo!()
    }

    fn register_producer(
        &self,
        _producer: Box<dyn opentelemetry::sdk::metrics::reader::MetricProducer, std::alloc::Global>,
    ) {
        todo!()
    }

    fn collect(
        &self,
        _rm: &mut opentelemetry::sdk::metrics::data::ResourceMetrics,
    ) -> Result<(), opentelemetry::metrics::MetricsError> {
        todo!()
    }

    fn force_flush(
        &self,
        _cx: &opentelemetry::Context,
    ) -> Result<(), opentelemetry::metrics::MetricsError> {
        todo!()
    }

    fn shutdown(&self) -> Result<(), opentelemetry::metrics::MetricsError> {
        todo!()
    }
}

/// A Metrics handler using the anonymous opentelemetry metric space.
pub struct OTelMetricsHandler(opentelemetry::metrics::Meter);

impl OTelMetricsHandler {
    /// Construct a new anonymous opentelemetry metric handler.
    pub fn new() -> Self {
        let bob = Bob;
        let provider = opentelemetry::sdk::metrics::MeterProvider::builder()
            .with_reader(bob)
            .build();
        opentelemetry::global::set_meter_provider(provider);
        Self(opentelemetry::global::meter(""))
    }
}

impl ametrics::types::MetricsHandler for OTelMetricsHandler {
    fn register_counter_u64(
        &self,
        name: &'static str,
        desc: Option<&'static str>,
        unit: Option<&'static str>,
    ) -> Arc<dyn ametrics::types::CounterU64> {
        let builder = self.0.u64_counter(name);
        let builder = if let Some(desc) = desc {
            builder.with_description(desc)
        } else {
            builder
        };
        let builder = if let Some(unit) = unit {
            builder.with_unit(opentelemetry::metrics::Unit::new(unit))
        } else {
            builder
        };
        let metric = builder.init();
        Arc::new(types::OTelCounterU64(metric))
    }

    fn report(&self) -> Vec<ametrics::types::Metric> {
        let _reader = opentelemetry::sdk::metrics::ManualReader::builder().build();
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static COUNT_1: ametrics::CounterU64 =
        ametrics::CounterU64::new("count1", Some("the first counter"), None);
    static COUNT_2: ametrics::CounterU64 =
        ametrics::CounterU64::new("count2", Some("the second counter"), None);

    #[test]
    fn default_registry() {
        ametrics::register_global_metrics_handler(OTelMetricsHandler::new());
        COUNT_1.add(42);
        COUNT_2.add(1);
        COUNT_2.add(1);
        COUNT_2.add(1);

        let report = ametrics::global_metrics_report();
        println!("{report:#?}");
    }
}
