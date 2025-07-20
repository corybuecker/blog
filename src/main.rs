use axum::{
    Router,
    extract::Request,
    http::StatusCode,
    middleware::{Next, from_fn},
    response::IntoResponse,
    routing::get,
};
use pages::{PublicationManager, PublishedPages};
use rust_web_common::telemetry::TelemetryBuilder;
use std::sync::Arc;
use tera::Tera;
use tokio::{join, process::Command, select, signal::unix::SignalKind, spawn};
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing::{Instrument, debug, error, info, info_span, instrument};
use utilities::tera::{digest_asset, embed_templates};

mod pages;
mod utilities;

#[derive(Debug)]
pub struct AppError(anyhow::Error);

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError(err)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        error!("{}", self.0);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Something has gone wrong.",
        )
            .into_response()
    }
}

pub struct SharedState {
    pub tera: Tera,
    pub published_pages: Box<dyn PublicationManager>,
}

async fn shutdown_handler() {
    let mut signal = tokio::signal::unix::signal(SignalKind::terminate())
        .expect("failed to install SIGTERM handler");

    signal.recv().await;
}

async fn server_handler(state: Arc<SharedState>) {
    let app = Router::new()
        .route("/", get(pages::home::build_response))
        .route("/post/{slug}/", get(pages::page::remove_slash))
        .route("/post/{slug}", get(pages::page::build_response))
        .route("/sitemap.xml", get(pages::sitemap::build_response))
        .nest_service("/assets", ServeDir::new("static").precompressed_gzip())
        .nest_service(
            "/images",
            ServeDir::new("static/images").precompressed_gzip(),
        )
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
    let mut telemetry = TelemetryBuilder::new("blog".to_string());
    telemetry.init().expect("could not initialize subscriber");

    spawn(compile_assets());

    let mut tera = Tera::default();

    tera.register_function("digest_asset", digest_asset());
    embed_templates(&mut tera)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to embed templates: {}", e))
        .unwrap();

    let mut published_pages = PublishedPages::default();
    published_pages
        .publish()
        .await
        .expect("Failed to publish pages during application startup");

    let shared_state = Arc::new(SharedState {
        tera,
        published_pages: Box::new(published_pages),
    });

    select! {
        _ = shutdown_handler() => {}
        _ = server_handler(shared_state.clone()) => {}
    }
}

#[instrument]
async fn compile_assets() {
    let css_command = Command::new("npx")
        .arg("tailwindcss")
        .arg("--input")
        .arg("css/app.css")
        .arg("--output")
        .arg("static/app.css")
        .output()
        .instrument(info_span!("compile css"));

    let js_command = Command::new("npx")
        .arg("esbuild")
        .arg("--bundle")
        .arg("--outdir=static")
        .arg("--sourcemap")
        .arg("--format=esm")
        .arg("js/app.ts")
        .output()
        .instrument(info_span!("compile js"));

    let (output_1, output_2) = join!(css_command, js_command);

    debug!("{:#?}", output_1);
    debug!("{:#?}", output_2);
}
