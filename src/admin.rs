use crate::types::SharedState;
use axum::{
    Router,
    extract::{Request, State},
    http::StatusCode,
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
};
use std::{env, sync::Arc};
use tower_cookies::{CookieManagerLayer, Cookies, Key};

mod authentication;
mod pages;
mod types;

async fn require_authentication(
    State(state): State<Arc<SharedState>>,
    cookies: Cookies,
    request: Request,
    next: Next,
) -> Response {
    let email = cookies
        .signed(&Key::from(env::var("SECRET_KEY").unwrap().as_bytes()))
        .get("email");

    if email.is_some() {
        let email = email.unwrap().value().to_owned();
        let client = state.client.read().await;
        let result = client
            .query_one(
                "SELECT true AS exists FROM users WHERE email = $1 LIMIT 1",
                &[&email],
            )
            .await;

        match result {
            Ok(result) => match result.try_get("exists") {
                Ok(true) => next.run(request).await,
                Ok(_) => StatusCode::FORBIDDEN.into_response(),
                Err(_) => StatusCode::FORBIDDEN.into_response(),
            },
            Err(_) => StatusCode::FORBIDDEN.into_response(),
        }
    } else {
        StatusCode::FORBIDDEN.into_response()
    }
}

pub fn admin_routes(state: Arc<SharedState>) -> Router<Arc<SharedState>> {
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
