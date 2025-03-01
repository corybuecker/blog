use crate::types::SharedState;
use axum::{
    Router,
    extract::{Request, State},
    http::StatusCode,
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
};
use mongodb::bson::doc;
use std::env;
use tower_cookies::{CookieManagerLayer, Cookies, Key};
use types::User;

mod authentication;
mod pages;
mod types;

async fn require_authentication(
    State(state): State<SharedState>,
    cookies: Cookies,
    request: Request,
    next: Next,
) -> Response {
    let email = cookies
        .signed(&Key::from(env::var("SECRET_KEY").unwrap().as_bytes()))
        .get("email");

    if email.is_some() {
        let mongo = state.mongo.database("blog").collection::<User>("users");

        let result = mongo
            .find_one(doc! {"email":  email.unwrap().value().to_string() })
            .await;

        match result {
            Ok(None) => StatusCode::FORBIDDEN.into_response(),
            Ok(_) => next.run(request).await,
            Err(_) => StatusCode::FORBIDDEN.into_response(),
        }
    } else {
        StatusCode::FORBIDDEN.into_response()
    }
}

pub fn admin_routes(state: SharedState) -> Router<SharedState> {
    let pages = Router::new()
        .route("/", get(pages::index).post(pages::create))
        .route("/new", get(pages::new))
        .route("/{id}", get(pages::edit).post(pages::update))
        .route_layer(middleware::from_fn_with_state(
            state,
            require_authentication,
        ))
        .layer(CookieManagerLayer::new());

    Router::new()
        .route("/login", get(authentication::login))
        .route("/login/callback", get(authentication::callback))
        .nest("/pages", pages)
        .layer(CookieManagerLayer::new())
}
