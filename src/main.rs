use anyhow::{Result, anyhow};
use axum::{Router, routing::get};
use mongodb::{Client, bson::doc};
use pages::{
    home::{self},
    sitemap,
};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::env;
use tera::Tera;
use tokio::{
    select,
    signal::unix::{SignalKind, signal},
    spawn,
};
use tower_http::services::ServeDir;
use tracing::Level;
use utils::tera::{digest_asset, embed_templates};

mod admin;
mod pages;
mod utils;

#[derive(Clone, Debug)]
pub struct SharedState {
    tera: Tera,
    mongo: Client,
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug)]
struct Page {
    _id: mongodb::bson::oid::ObjectId,
    content: String,
    created_at: mongodb::bson::DateTime,
    description: String,
    id: Option<String>,
    markdown: String,
    preview: String,

    #[serde_as(as = "Option<bson::DateTime>")]
    published_at: Option<chrono::DateTime<chrono::Utc>>,

    #[serde_as(as = "Option<bson::DateTime>")]
    revised_at: Option<chrono::DateTime<chrono::Utc>>,
    slug: String,
    title: String,
    updated_at: mongodb::bson::DateTime,
}

async fn signal_listener() {
    let mut signal = signal(SignalKind::terminate()).unwrap();
    signal.recv().await;
}

async fn run_listener(app: Router) {
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing::subscriber::set_global_default(
        tracing_subscriber::FmtSubscriber::builder()
            .with_max_level(Level::DEBUG)
            .finish(),
    )?;

    let mongo = Client::with_uri_str(env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();

    let mut tera = Tera::default();
    tera.register_function("digest_asset", digest_asset());
    embed_templates(&mut tera).map_err(|e| anyhow!("Failed to embed templates: {}", e))?;

    let shared_state = SharedState { tera, mongo };

    let app = Router::new()
        .route("/", get(home::build_response))
        .route("/post/{slug}/", get(pages::remove_slash))
        .route("/post/{slug}", get(pages::build_response))
        .route("/sitemap.xml", get(sitemap::build_response))
        .nest_service("/assets", ServeDir::new("static"))
        .nest_service("/images", ServeDir::new("static/images"))
        .nest("/admin", admin::admin_routes(shared_state.clone()))
        .with_state(shared_state);

    select! {
      _ = spawn(async { run_listener(app).await }) => {},
      _ = spawn(async { signal_listener().await }) => {}
    }

    Ok(())
}
