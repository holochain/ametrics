#![deny(missing_docs)]
#![deny(unsafe_code)]
#![deny(warnings)]
//! Promethus handlers for ametrics::register_global_metrics_handler.

use std::sync::Arc;

/// types
mod types {
    pub struct PrometheusCounterU64(pub prometheus::IntCounter);

    impl ametrics::types::CounterU64 for PrometheusCounterU64 {
        fn add(&self, count: u64) {
            self.0.inc_by(count);
        }
    }
}

/// A Metrics handler using the prometheus default global registry.
pub struct PrometheusDefaultMetricsHandler;

impl ametrics::types::MetricsHandler for PrometheusDefaultMetricsHandler {
    fn register_counter_u64(
        &self,
        name: &'static str,
        desc: Option<&'static str>,
        _unit: Option<&'static str>,
    ) -> Arc<dyn ametrics::types::CounterU64> {
        let raw = prometheus::IntCounter::new(name, desc.unwrap_or("")).unwrap();
        prometheus::default_registry().register(Box::new(raw.clone())).unwrap();
        Arc::new(types::PrometheusCounterU64(raw))
    }

    fn report(&self) -> Vec<ametrics::types::Metric> {
        prometheus::default_registry().gather().into_iter().map(|mut fam| {
            let name = fam.take_name();
            let desc = if fam.has_help() {
                Some(fam.take_help())
            } else {
                None
            };
            let value = match fam.take_metric().pop() {
                Some(mut metric) => {
                    if metric.has_counter() {
                        let counter = metric.take_counter();
                        if counter.has_value() {
                            let value = counter.get_value();
                            if ((value as u64) as f64) == value {
                                ametrics::types::MetricValue::CounterU64(value as u64)
                            } else {
                                ametrics::types::MetricValue::CounterF64(value)
                            }
                        } else {
                            ametrics::types::MetricValue::None
                        }
                    } else {
                        ametrics::types::MetricValue::None
                    }
                }
                None => ametrics::types::MetricValue::None,
            };
            ametrics::types::Metric {
                name,
                desc,
                unit: None,
                value,
            }
        }).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static COUNT_1: ametrics::CounterU64 = ametrics::CounterU64::new(
        "count1",
        Some("the first counter"),
        None,
    );
    static COUNT_2: ametrics::CounterU64 = ametrics::CounterU64::new(
        "count2",
        Some("the second counter"),
        None,
    );

    #[test]
    fn default_registry() {
        ametrics::register_global_metrics_handler(PrometheusDefaultMetricsHandler);
        COUNT_1.add(42);
        COUNT_2.add(1);
        COUNT_2.add(1);
        COUNT_2.add(1);

        let report = ametrics::global_metrics_report();
        println!("{report:#?}");

        for m in report.unwrap() {
            if m.name == "count1" {
                assert_eq!(m.desc.unwrap(), "the first counter");
                assert!(matches!(m.value, ametrics::types::MetricValue::CounterU64(42)));
            } else if m.name == "count2" {
                assert_eq!(m.desc.unwrap(), "the second counter");
                assert!(matches!(m.value, ametrics::types::MetricValue::CounterU64(3)));
            } else {
                panic!("unexpected name: {}", m.name);
            }
        }
    }
}
