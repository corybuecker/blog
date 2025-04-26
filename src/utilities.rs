use anyhow::Result;
use opentelemetry::global;
use opentelemetry_otlp::{MetricExporter, Protocol, WithExportConfig};
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::metrics::SdkMeterProvider;
use std::fs::DirEntry;
use std::{fs::read_dir, path::Path};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::Layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

pub mod tera;

pub fn initialize_tracing() -> Result<SdkMeterProvider> {
    let fmt = tracing_subscriber::fmt::layer()
        .with_thread_names(true)
        .with_level(true)
        .with_filter(LevelFilter::DEBUG);

    tracing_subscriber::registry().with(fmt).init();

    let metrics_endpoint = std::env::var("METRICS_ENDPOINT")?;
    let exporter = MetricExporter::builder()
        .with_http()
        .with_protocol(Protocol::HttpBinary)
        .with_endpoint(metrics_endpoint)
        .build()
        .expect("Failed to create metric exporter");

    let resource = Resource::builder().build();
    let meter_provider = SdkMeterProvider::builder()
        .with_periodic_exporter(exporter)
        .with_resource(resource)
        .build();

    global::set_meter_provider(meter_provider.clone());
    Ok(meter_provider)
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
