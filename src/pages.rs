use crate::types::{AppError, Page, SharedState};
use anyhow::{Context, anyhow};
use axum::{
    extract::{Path, State},
    response::{Html, Redirect},
};
use std::sync::Arc;

pub mod home;
pub mod sitemap;

pub async fn build_response(
    Path(slug): Path<String>,
    State(shared_state): State<Arc<SharedState>>,
) -> Result<Html<String>, AppError> {
    let tera = &shared_state.tera;
    let mut context = tera::Context::new();
    let client = &shared_state.client;

    let page: Page = client
        .query_one(
            "SELECT * FROM pages WHERE published_at IS NOT NULL AND slug = $1",
            &[&slug],
        )
        .await
        .context("could not find page")?
        .try_into()?;

    context.insert("page", &page);
    context.insert("description", &page.description);
    let mut title = page.title.to_owned();
    title.push_str(" - Cory Buecker");
    context.insert("title", &());

    let rendered = tera
        .render("pages/page.html", &context)
        .map_err(|e| anyhow!("could not render template: {}", e))?;

    Ok(Html(rendered))
}

pub async fn remove_slash(Path(path_slug): Path<String>) -> Redirect {
    let mut redirect = String::from("/post/");
    redirect.push_str(&path_slug);

    Redirect::permanent(&redirect)
}
