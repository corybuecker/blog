use axum::{
    Router,
    extract::Request,
    http::StatusCode,
    middleware::{Next, from_fn},
    response::IntoResponse,
    routing::get,
};
use rust_web_common::telemetry::TelemetryBuilder;
use std::sync::Arc;
use tera::Tera;
use tokio::{select, signal::unix::SignalKind};
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing::info;
use types::SharedState;
use utilities::tera::{digest_asset, embed_templates};

mod pages;
mod types;
mod utilities;

async fn shutdown_handler() {
    let mut signal = tokio::signal::unix::signal(SignalKind::terminate())
        .expect("failed to install SIGTERM handler");

    signal.recv().await;
}

async fn server_handler(state: Arc<SharedState>) {
    let app = Router::new()
        .route("/", get(pages::home::build_response))
        .route("/post/{slug}/", get(pages::remove_slash))
        .route("/post/{slug}", get(pages::build_response))
        .route("/sitemap.xml", get(pages::sitemap::build_response))
        .nest_service("/assets", ServeDir::new("static"))
        .nest_service("/images", ServeDir::new("static/images"))
        .with_state(state.clone())
        .layer(TraceLayer::new_for_http())
        .layer(from_fn(metrics))
        .route("/healthcheck", get(StatusCode::OK));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();

    axum::serve(listener, app)
        .await
        .expect("failed to start server");
}

async fn metrics(request: Request, next: Next) -> impl IntoResponse {
    let start = std::time::Instant::now();
    let path = request.uri().to_string();
    let method = request.method().to_string();

    let response = next.run(request).await;

    if response.status().is_success() {
        info!(
            histogram.latency = start.elapsed().as_millis() as f64,
            method = method,
            path = path,
        );
    }

    response
}

#[tokio::main]
async fn main() {
    let _telemetry_providers = TelemetryBuilder::new("blog".to_string())
        .build()
        .expect("failed to initialize telemetry");

    let mut tera = Tera::default();

    tera.register_function("digest_asset", digest_asset());
    embed_templates(&mut tera)
        .map_err(|e| anyhow::anyhow!("Failed to embed templates: {}", e))
        .unwrap();

    let shared_state = Arc::new(SharedState { tera });

    select! {
        _ = shutdown_handler() => {}
        _ = server_handler(shared_state.clone()) => {}
    }
}
