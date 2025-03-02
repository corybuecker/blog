use axum::{http::StatusCode, response::IntoResponse};
use mongodb::Client;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use tera::Tera;
use tracing::error;

pub struct AppError(anyhow::Error);

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError(err)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        error!("{}", self.0);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Something has gone wrong.",
        )
            .into_response()
    }
}

#[derive(Clone, Debug)]
pub struct SharedState {
    pub tera: Tera,
    pub mongo: Client,
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug)]
pub struct Page {
    pub _id: mongodb::bson::oid::ObjectId,
    pub content: String,
    pub created_at: mongodb::bson::DateTime,
    pub description: String,
    pub id: Option<String>,
    pub markdown: String,
    pub preview: String,

    #[serde_as(as = "Option<bson::DateTime>")]
    pub published_at: Option<chrono::DateTime<chrono::Utc>>,

    #[serde_as(as = "Option<bson::DateTime>")]
    pub revised_at: Option<chrono::DateTime<chrono::Utc>>,
    pub slug: String,
    pub title: String,
    pub updated_at: mongodb::bson::DateTime,
}
