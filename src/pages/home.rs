use crate::types::{AppError, Page, SharedState};
use anyhow::Context;
use axum::{extract::State, response::Html};
use bson::doc;
use mongodb::options::FindOptions;
use std::collections::VecDeque;

pub async fn build_response(
    State(shared_state): State<SharedState>,
) -> Result<Html<String>, AppError> {
    let tera = &shared_state.tera;
    let mongo = shared_state
        .mongo
        .database("blog")
        .collection::<Page>("pages");

    let mut context = tera::Context::new();
    let find_options = FindOptions::builder()
        .sort(doc! {"published_at": -1})
        .build();

    let mut cur = mongo
        .find(doc! {"published_at": doc!{"$lte": mongodb::bson::DateTime::now()}})
        .with_options(find_options)
        .await
        .context("database error")?;

    let mut pages: VecDeque<Page> = VecDeque::new();

    while cur.advance().await.context("database error")? {
        let page = cur.deserialize_current().context("database error")?;
        pages.push_back(page)
    }

    let homepage = pages
        .pop_front()
        .context("cannot render a homepage without any pages")?;

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
