//! OpenTelemetry Integration
//!
//! This module provides optional integration with OpenTelemetry for distributed tracing
//! and metrics collection. It is enabled via the `telemetry` feature flag.

#[cfg(feature = "telemetry")]
use opentelemetry::{global, KeyValue};
#[cfg(feature = "telemetry")]
use opentelemetry_sdk::{trace as sdktrace, Resource};
#[cfg(feature = "telemetry")]
use tracing_subscriber::{layer::SubscriberExt, Registry};

/// Initialize telemetry if the feature is enabled
pub fn init_telemetry() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    #[cfg(feature = "telemetry")]
    {
        init_otlp_tracing()?;
        init_otlp_metrics()?;
    }
    Ok(())
}

#[cfg(feature = "telemetry")]
fn init_otlp_tracing() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let service_name = std::env::var("OTEL_SERVICE_NAME").unwrap_or_else(|_| "vaughan-wallet".to_string());
    
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
        )
        .with_trace_config(
            sdktrace::config().with_resource(Resource::new(vec![
                KeyValue::new("service.name", service_name),
                KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
            ])),
        )
        .install_batch(opentelemetry_sdk::runtime::Tokio)?;

    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    // Initialize the global subscriber
    // Note: In a real app, you'd likely compose this with fmt::layer()
    use tracing_subscriber::util::SubscriberInitExt;
    Registry::default().with(telemetry).init();
    
    Ok(())
}

#[cfg(feature = "telemetry")]
fn init_otlp_metrics() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let service_name = std::env::var("OTEL_SERVICE_NAME").unwrap_or_else(|_| "vaughan-wallet".to_string());

    let meter_provider = opentelemetry_otlp::new_pipeline()
        .metrics(opentelemetry_sdk::runtime::Tokio)
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
        )
        .with_resource(Resource::new(vec![
            KeyValue::new("service.name", service_name),
            KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
        ]))
        .build()?;

    global::set_meter_provider(meter_provider);
    Ok(())
}
