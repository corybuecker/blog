use crate::{Page, SharedState};
use axum::{
    extract::State,
    response::{Html, IntoResponse, Response},
};
use bson::doc;
use mongodb::options::FindOptions;
use std::collections::VecDeque;

pub async fn build_response(State(shared_state): State<SharedState>) -> Response {
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
        .unwrap();

    let mut pages: VecDeque<Page> = VecDeque::new();

    while cur.advance().await.unwrap() {
        let page = cur.deserialize_current().unwrap();
        pages.push_back(page)
    }

    let homepage = pages.pop_front().unwrap();

    context.insert("pages", &pages);
    context.insert("homepage", &homepage);

    context.insert("description", &homepage.description);
    let mut title = homepage.title.to_owned();
    title.push_str(" - Cory Buecker");
    context.insert("title", &title);

    let rendered = tera.render("pages/home.html", &context).unwrap();

    Html(rendered).into_response()
}
