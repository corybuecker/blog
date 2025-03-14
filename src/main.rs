use axum::{Router, routing::get};
use std::sync::Arc;
use tera::Tera;
use tokio::signal::unix::SignalKind;
use tokio_postgres::{NoTls, connect};
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing::{Level, info};
use types::SharedState;
use utils::tera::{digest_asset, embed_templates};

mod admin;
mod pages;
mod types;
mod utils;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing::subscriber::set_global_default(
        tracing_subscriber::FmtSubscriber::builder()
            .with_max_level(Level::DEBUG)
            .finish(),
    )?;

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
        .layer(TraceLayer::new_for_http());

    let shutdown_signal = async {
        let mut signal = tokio::signal::unix::signal(SignalKind::terminate())
            .expect("failed to install SIGTERM handler");
        signal.recv().await;
        info!("Received shutdown signal, gracefully shutting down");
    };

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await?;

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal)
        .await?;

    Ok(())
}
