use axum::{
    Router,
    extract::Request,
    http::{HeaderValue, StatusCode, header::CONTENT_SECURITY_POLICY},
    middleware::{Next, from_fn},
    response::{Html, IntoResponse},
    routing::get,
};
use pages::{PublicationManager, PublishedPages};
use rust_web_common::{
    telemetry::TelemetryBuilder,
    templating::{Renderer, RendererError},
};
use std::sync::Arc;
use tokio::{join, process::Command, select, signal::unix::SignalKind, spawn};
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing::{Instrument, debug, error, info, info_span, instrument};

mod pages;

const CROSS_ORIGIN_OPENER_POLICY: &str = "Cross-Origin-Opener-Policy";

#[derive(Debug)]
pub enum AppError {
    PageNotFound,
    Unknown(anyhow::Error),
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::Unknown(err)
    }
}

impl From<RendererError> for AppError {
    fn from(err: RendererError) -> Self {
        error!("{:?}", err);
        AppError::Unknown(err.into())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        match self {
            AppError::PageNotFound => match Renderer::new("templates".to_string()) {
                Ok(renderer) => match renderer.render("errors/404") {
                    Ok(response) => (StatusCode::NOT_FOUND, Html(response)).into_response(),
                    Err(_err) => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Something has gone wrong.",
                    )
                        .into_response(),
                },
                Err(_err) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Something has gone wrong.",
                )
                    .into_response(),
            },
            AppError::Unknown(err) => {
                error!("Unknown error: {}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Something has gone wrong.",
                )
                    .into_response()
            }
        }
    }
}

pub struct SharedState {
    pub renderer: Renderer,
    pub published_pages: Box<dyn PublicationManager>,
}

async fn shutdown_handler() {
    let mut signal = tokio::signal::unix::signal(SignalKind::terminate())
        .expect("failed to install SIGTERM handler");

    signal.recv().await;
}

async fn secure_headers(request: Request, next: Next) -> impl IntoResponse {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();

    headers.remove(CROSS_ORIGIN_OPENER_POLICY);
    headers.insert(
        CROSS_ORIGIN_OPENER_POLICY,
        HeaderValue::from_static("same-origin"),
    );

    headers.remove(CONTENT_SECURITY_POLICY);
    headers.insert(
        CONTENT_SECURITY_POLICY,
        HeaderValue::from_static(
            "default-src 'none'; style-src 'self'; script-src 'self'; img-src 'self'; frame-ancestors 'none'",
        ),
    );

    response
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
        .fallback(|| async { Err::<StatusCode, AppError>(AppError::PageNotFound) })
        .layer(from_fn(secure_headers))
        .layer(from_fn(metrics))
        .layer(TraceLayer::new_for_http())
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

    let renderer = Renderer::new("templates".to_string()).unwrap();

    let mut published_pages = PublishedPages::default();
    published_pages
        .publish()
        .await
        .expect("Failed to publish pages during application startup");

    let shared_state = Arc::new(SharedState {
        renderer,
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
        .arg("--map")
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
