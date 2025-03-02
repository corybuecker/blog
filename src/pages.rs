use crate::types::{AppError, Page, SharedState};
use anyhow::Context;
use axum::{
    extract::{Path, State},
    response::{Html, Redirect},
};
use bson::doc;
use mongodb::Collection;

pub mod home;
pub mod sitemap;

pub async fn build_response(
    Path(slug): Path<String>,
    State(shared_state): State<SharedState>,
) -> Result<Html<String>, AppError> {
    let tera = &shared_state.tera;
    let database = &shared_state.mongo.database("blog");
    let mut context = tera::Context::new();

    let collection: Collection<Page> = database.collection("pages");
    let page = collection
        .find_one(doc! {"slug": slug})
        .await
        .context("database error")?
        .context("no page found")?;

    context.insert("page", &page);
    context.insert("test", &page.published_at);

    context.insert("description", &page.description);
    let mut title = page.title.to_owned();
    title.push_str(" - Cory Buecker");
    context.insert("title", &title);

    let rendered = tera
        .render("pages/page.html", &context)
        .context("could not render template")?;

    Ok(Html(rendered))
}

pub async fn remove_slash(Path(slug): Path<String>) -> Redirect {
    let mut redirect = String::from("/post/");
    redirect.push_str(&slug);

    Redirect::permanent(&redirect)
}
