use std::{env, sync::Arc};
mod admin;
use axum::{
    body::Body,
    extract::{MatchedPath, Path, State},
    http::Request,
    response::{Html, IntoResponse, Redirect, Response},
    routing::get,
    Router,
};
use futures::{StreamExt, TryStreamExt};
use mongodb::{bson::doc, options::FindOptions, Client, Collection};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::collections::VecDeque;
use tera::Tera;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tracing::info_span;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use xml_builder::{XMLBuilder, XMLElement, XMLVersion};
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

async fn home(State(shared_state): State<Arc<SharedState>>) -> Response {
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

    while let Some(page) = cur.try_next().await.unwrap() {
        pages.push_back(page)
    }

    let homepage = pages.pop_front().unwrap();

    context.insert("pages", &pages);
    context.insert("homepage", &homepage);

    context.insert("description", &homepage.description);
    let mut title = homepage.title.to_owned();
    title.push_str(" - Cory Buecker");
    context.insert("title", &title);

    let rendered = tera.render("home.html", &context).unwrap();

    Html(rendered).into_response()
}

async fn page(Path(slug): Path<String>, State(shared_state): State<Arc<SharedState>>) -> Response {
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

    let rendered = tera.render("page.html", &context).unwrap();

    Html(rendered).into_response()
}
async fn remove_slash(Path(slug): Path<String>) -> Redirect {
    let mut redirect = String::from("/post/");
    redirect.push_str(&slug);
    return Redirect::permanent(&redirect);
}

async fn sitemap(State(shared_state): State<Arc<SharedState>>) -> Response {
    let database = &shared_state.mongo.database("blog");
    let mut find_options = FindOptions::builder();
    let find_options = find_options.sort(doc! {"published_at": -1});
    let collection = database
        .collection::<Page>("pages")
        .find(doc! {"published_at": doc!{"$exists": true}})
        .with_options(find_options.build())
        .await;

    let mut xml = XMLBuilder::new()
        .version(XMLVersion::XML1_1)
        .encoding("UTF-8".into())
        .build();

    let mut urlset = XMLElement::new("urlset");
    urlset.add_attribute("xmlns", "http://www.sitemaps.org/schemas/sitemap/0.9");
    let mut current_index = 0;
    if let Ok(mut cursor) = collection {
        while let Some(Ok(page)) = cursor.next().await {
            if current_index == 0 {
                current_index = current_index + 1;
                continue;
            }
            let mut url = XMLElement::new("url");
            let lastmodts = page
                .revised_at
                .or(page.published_at)
                .or(Some(page.created_at.into()));

            if let Some(lastmodts) = lastmodts {
                let mut lastmod = XMLElement::new("lastmod");
                let _ = lastmod.add_text(lastmodts.to_rfc3339());
                let _ = url.add_child(lastmod);
            }

            let mut loc = XMLElement::new("loc");
            let _ = loc.add_text(format!("https://corybuecker.com/post/{}", page.slug));
            let _ = url.add_child(loc);

            let _ = urlset.add_child(url);
        }
    }

    xml.set_root_element(urlset);

    let mut output = Vec::<u8>::new();
    let _ = xml.generate(&mut output);
    Body::from(String::from_utf8(output).unwrap()).into_response()
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "blog=debug,tower_http=debug,axum::rejection=trace".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let mongo = Client::with_uri_str(env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();
    let tera = Tera::new("templates/**/*.html").unwrap();

    let shared_state = Arc::new(SharedState { tera, mongo });

    let app = Router::new()
        .route("/", get(home))
        .route("/post/{slug}/", get(remove_slash))
        .route("/post/{slug}", get(page))
        .route("/sitemap.xml", get(sitemap))
        .nest_service("/assets", ServeDir::new("static"))
        .nest_service("/images", ServeDir::new("static/images"))
        .nest("/admin", admin::admin_routes(shared_state.clone()))
        .layer(
            TraceLayer::new_for_http().make_span_with(|request: &Request<_>| {
                let matched_path = request
                    .extensions()
                    .get::<MatchedPath>()
                    .map(MatchedPath::as_str);

                info_span!(
                    "http_request",
                    method = ?request.method(),
                    matched_path,
                    some_other_field = tracing::field::Empty,
                )
            }),
        )
        .with_state(shared_state.clone());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();

    axum::serve(listener, app).await.unwrap();
}
