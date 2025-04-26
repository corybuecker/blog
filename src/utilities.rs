use anyhow::Result;
use opentelemetry::trace::TracerProvider;
use opentelemetry_otlp::WithExportConfig;
use std::fs::DirEntry;
use std::{fs::read_dir, path::Path};
use tracing_opentelemetry::MetricsLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{Layer, Registry};

pub mod tera;

pub enum Provider {
    MeterProvider(opentelemetry_sdk::metrics::SdkMeterProvider),
    TracerProvider(opentelemetry_sdk::trace::SdkTracerProvider),
}

pub fn initialize_tracing() -> anyhow::Result<Vec<Provider>> {
    let target = tracing_subscriber::filter::Targets::new()
        .with_default(tracing::level_filters::LevelFilter::DEBUG);

    let fmt: Box<dyn Layer<Registry> + Send + Sync> = tracing_subscriber::fmt::layer()
        .with_level(true)
        .with_filter(target)
        .boxed();

    let mut layers: Vec<Box<dyn Layer<Registry> + Send + Sync>> = vec![fmt];

    #[allow(unused_assignments)]
    let mut meter_provider: Option<opentelemetry_sdk::metrics::SdkMeterProvider> = None;
    #[allow(unused_assignments)]
    let mut tracer_provider: Option<opentelemetry_sdk::trace::SdkTracerProvider> = None;

    if let Ok(metrics_endpoint) = std::env::var("METRICS_ENDPOINT") {
        let metrics_exporter = opentelemetry_otlp::MetricExporter::builder()
            .with_http()
            .with_protocol(opentelemetry_otlp::Protocol::HttpBinary)
            .with_endpoint(metrics_endpoint)
            .build()
            .expect("Failed to create metric exporter");

        let inner_meter_provider = opentelemetry_sdk::metrics::SdkMeterProvider::builder()
            .with_periodic_exporter(metrics_exporter)
            .with_resource(
                opentelemetry_sdk::Resource::builder()
                    .with_service_name("blog")
                    .build(),
            )
            .build();

        let layer: Box<dyn Layer<Registry> + Send + Sync> =
            MetricsLayer::new(inner_meter_provider.clone()).boxed();

        layers.push(layer);

        meter_provider = Some(inner_meter_provider);
    }

    if let Ok(tracing_endpoint) = std::env::var("TRACING_ENDPOINT") {
        let exporter = opentelemetry_otlp::SpanExporter::builder()
            .with_http()
            .with_protocol(opentelemetry_otlp::Protocol::HttpBinary)
            .with_endpoint(tracing_endpoint)
            .build()
            .expect("Failed to create span exporter");

        let inner_tracer_provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
            .with_batch_exporter(exporter)
            .with_resource(
                opentelemetry_sdk::Resource::builder()
                    .with_service_name("blog")
                    .build(),
            )
            .build();

        let tracer = inner_tracer_provider.tracer("blog");

        let layer: Box<dyn Layer<Registry> + Send + Sync> =
            tracing_opentelemetry::layer().with_tracer(tracer).boxed();

        tracer_provider = Some(inner_tracer_provider);

        layers.push(layer);
    }

    tracing_subscriber::registry().with(layers).init();

    let mut providers = Vec::new();

    if let Some(meter_provider) = meter_provider {
        providers.push(Provider::MeterProvider(meter_provider));
    }

    if let Some(tracer_provider) = tracer_provider {
        providers.push(Provider::TracerProvider(tracer_provider));
    }

    Ok(providers)
}

#[allow(dead_code)]
pub fn read_all_files(path: &Path) -> Result<Vec<DirEntry>> {
    let mut files = Vec::new();

    for file in read_dir(path)? {
        let file = file?;
        if Path::new(&file.path()).is_dir() {
            files.append(&mut read_all_files(&file.path())?);
        } else {
            files.push(file);
        }
    }

    files.sort_unstable_by_key(|a| a.path());

    Ok(files)
}

pub mod tests {
    pub mod helpers;
}
