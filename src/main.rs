use axum::{
    Router,
    extract::Request,
    middleware::{Next, from_fn},
    response::IntoResponse,
    routing::get,
};
use opentelemetry::{KeyValue, global};
use std::sync::Arc;
use tera::Tera;
use tokio::signal::unix::SignalKind;
use tokio_postgres::{NoTls, connect};
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing::info;
use types::SharedState;
use utils::{
    initialize_tracing,
    tera::{digest_asset, embed_templates},
};

mod admin;
mod pages;
mod types;
mod utils;

async fn metrics(request: Request, next: Next) -> impl IntoResponse {
    let start = std::time::Instant::now();
    let path = request.uri().to_string();

    let response = next.run(request).await;

    let meter = global::meter("blog");
    meter.f64_counter("latency").build().add(
        start.elapsed().as_secs_f64(),
        &[KeyValue::new("path", path)],
    );

    response
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let metric_provider = initialize_tracing()?;
    let mut tera = Tera::default();

    tera.register_function("digest_asset", digest_asset());
    embed_templates(&mut tera).map_err(|e| anyhow::anyhow!("Failed to embed templates: {}", e))?;

    let database_url = std::env::var("DATABASE_URL")?;
    let (client, connection) = connect(&database_url, NoTls).await?;

    tokio::spawn(connection);

    let shared_state = Arc::new(SharedState { tera, client });
    let app = Router::new()
        .route("/", get(pages::home::build_response))
        .route("/post/{slug}/", get(pages::remove_slash))
        .route("/post/{slug}", get(pages::build_response))
        .route("/sitemap.xml", get(pages::sitemap::build_response))
        .nest_service("/assets", ServeDir::new("static"))
        .nest_service("/images", ServeDir::new("static/images"))
        .nest("/admin", admin::admin_routes(shared_state.clone()))
        .with_state(shared_state)
        .layer(TraceLayer::new_for_http())
        .layer(from_fn(metrics));

    let shutdown_signal = async move {
        let mut signal = tokio::signal::unix::signal(SignalKind::terminate())
            .expect("failed to install SIGTERM handler");
        signal.recv().await;
        metric_provider.shutdown().unwrap();
        info!("Received shutdown signal, gracefully shutting down");
    };

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await?;

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal)
        .await?;

    Ok(())
}
