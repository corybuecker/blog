use crate::types::{AppError, SharedState};
use anyhow::anyhow;
use axum::{extract::State, response::Html};
use serde::Serialize;
use std::{collections::VecDeque, sync::Arc};

use super::{published_pages, without_frontmatter};

#[allow(dead_code)]
#[derive(Serialize, Debug)]
struct Link {
    title: String,
    slug: String,
}

pub async fn build_response(
    State(shared_state): State<Arc<SharedState>>,
) -> Result<Html<String>, AppError> {
    let tera = &shared_state.tera;
    let mut context = tera::Context::new();

    let published_pages = published_pages().await?;
    let published_page = published_pages
        .first()
        .ok_or(anyhow!("could not get homepage"))?;

    let content = without_frontmatter(&published_page.path).await?;
    let description = published_page.frontmatter.description.clone();
    let published_at = published_page.published_at;
    let mut title = published_page.frontmatter.title.clone();
    title.push_str(" - Cory Buecker");
    let revised_at = published_page.frontmatter.revised_at;

    let mut pages: VecDeque<Link> = published_pages
        .into_iter()
        .map(|row| Link {
            title: row.frontmatter.title,
            slug: row.frontmatter.slug,
        })
        .collect();

    pages.pop_front();

    context.insert("pages", &pages);
    context.insert("content", &content);
    context.insert("description", &description);
    context.insert("title", &title);
    context.insert("published_at", &published_at);
    context.insert("revised_at", &revised_at);

    let rendered = tera
        .render("pages/home.html", &context)
        .map_err(|e| anyhow!("could not render template: {e}"))?;

    Ok(Html(rendered))
}
