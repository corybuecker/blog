use crate::types::{AppError, Page, SharedState};
use anyhow::Context;
use axum::{extract::State, response::Html};
use serde::Serialize;
use std::{collections::VecDeque, sync::Arc};

#[derive(Serialize, Debug)]
struct Link {
    title: String,
    slug: String,
}

pub async fn build_response(
    State(shared_state): State<Arc<SharedState>>,
) -> Result<Html<String>, AppError> {
    let tera = &shared_state.tera;
    let client = &shared_state.client;
    let pages = client
        .query(
            "SELECT title, slug FROM pages WHERE published_at IS NOT NULL ORDER BY published_at DESC",
            &[],
        )
        .await
        .context("could not fetch pages")?;
    let homepage: Page = client
        .query_one(
            "SELECT * FROM pages WHERE published_at IS NOT NULL ORDER BY published_at DESC LIMIT 1",
            &[],
        )
        .await
        .context("could not fetch homepage")?
        .try_into()?;

    let mut context = tera::Context::new();
    let mut pages: VecDeque<Link> = pages
        .into_iter()
        .map(|row| Link {
            title: row.get("title"),
            slug: row.get("slug"),
        })
        .collect();

    pages.pop_front();

    context.insert("pages", &pages);
    context.insert("homepage", &homepage);

    context.insert("description", &homepage.description);
    let mut title = homepage.title.to_owned();
    title.push_str(" - Cory Buecker");
    context.insert("title", &title);

    let rendered = tera
        .render("pages/home.html", &context)
        .context("could not render template")?;

    Ok(Html(rendered))
}
