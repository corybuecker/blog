use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct User {
    #[allow(dead_code)]
    pub email: String,
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum AuthenticationError {
    NoEmail,
}

#[derive(Deserialize, Serialize)]
pub struct PostForm {
    pub content: String,
    pub description: String,
    pub preview: String,
    pub published_at: Option<String>,
    pub revised_at: Option<String>,
    pub slug: String,
    pub title: String,
}
