use axum::{
    Router,
    extract::Request,
    middleware::{Next, from_fn},
    response::IntoResponse,
    routing::get,
};
use std::{process::exit, sync::Arc};
use tera::Tera;
use tokio::{signal::unix::SignalKind, spawn};
use tokio_postgres::{NoTls, connect};
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing::info;
use types::SharedState;
use utilities::{
    initialize_tracing,
    tera::{digest_asset, embed_templates},
};

mod admin;
mod pages;
mod types;
mod utilities;

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
    let providers = initialize_tracing().expect("could not initialize logging/tracing");
    let mut tera = Tera::default();

    tera.register_function("digest_asset", digest_asset());
    embed_templates(&mut tera)
        .map_err(|e| anyhow::anyhow!("Failed to embed templates: {}", e))
        .unwrap();

    let database_url = std::env::var("DATABASE_URL").unwrap();
    let (client, connection) = connect(&database_url, NoTls).await.unwrap();

    spawn(async move {
        if let Err(e) = connection.await {
            tracing::error!("Postgres connection error: {}", e);
            exit(1);
        }
    });

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

        for provider in providers {
            match provider {
                utilities::Provider::MeterProvider(provider) => {
                    provider.shutdown().unwrap();
                }
                utilities::Provider::TracerProvider(tracer_provider) => {
                    tracer_provider.shutdown().unwrap();
                }
            }
        }
    };

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal)
        .await
        .expect("failed to start server");
}
