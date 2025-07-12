use super::without_frontmatter;
use crate::{AppError, SharedState};
use anyhow::anyhow;
use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse, Redirect},
};
use std::sync::Arc;

pub async fn build_response(
    Path(slug): Path<String>,
    State(state): State<Arc<SharedState>>,
) -> Result<impl IntoResponse, AppError> {
    let tera = &state.tera;
    let mut context = tera::Context::new();

    let published_pages = state.published_pages.fetch().await?;
    let published_page = published_pages
        .iter()
        .find(|f| f.frontmatter.slug == slug)
        .ok_or(anyhow!("could not find page"))?;

    let content = without_frontmatter(&published_page.path).await?;
    let description = published_page.frontmatter.description.clone();
    let published_at = published_page.published_at;
    let revised_at = published_page.frontmatter.revised_at;
    let mut title = published_page.frontmatter.title.clone();
    title.push_str(" - Cory Buecker");

    context.insert("content", &content);
    context.insert("description", &description);
    context.insert("title", &title);
    context.insert("published_at", &published_at);
    context.insert("revised_at", &revised_at);

    let rendered = tera
        .render("pages/page.html", &context)
        .map_err(|e| anyhow!("could not render template: {e}"))?;

    Ok(Html(rendered))
}

pub async fn remove_slash(Path(path_slug): Path<String>) -> Redirect {
    let mut redirect = String::from("/post/");
    redirect.push_str(&path_slug);

    Redirect::permanent(&redirect)
}
