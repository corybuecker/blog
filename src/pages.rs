use crate::{Page, SharedState};
use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse, Redirect, Response},
};
use bson::doc;
use mongodb::Collection;
use std::sync::Arc;

pub mod home;
pub mod sitemap;

pub async fn build_response(
    Path(slug): Path<String>,
    State(shared_state): State<Arc<SharedState>>,
) -> Response {
    let tera = &shared_state.tera;
    let database = &shared_state.mongo.database("blog");
    let mut context = tera::Context::new();

    let collection: Collection<Page> = database.collection("pages");
    let page = collection
        .find_one(doc! {"slug": slug})
        .await
        .unwrap()
        .unwrap();

    context.insert("page", &page);
    context.insert("test", &page.published_at);

    context.insert("description", &page.description);
    let mut title = page.title.to_owned();
    title.push_str(" - Cory Buecker");
    context.insert("title", &title);

    let rendered = tera.render("pages/page.html", &context).unwrap();

    Html(rendered).into_response()
}

pub async fn remove_slash(Path(slug): Path<String>) -> Redirect {
    let mut redirect = String::from("/post/");
    redirect.push_str(&slug);

    Redirect::permanent(&redirect)
}
