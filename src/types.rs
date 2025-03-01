use mongodb::Client;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use tera::Tera;

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
