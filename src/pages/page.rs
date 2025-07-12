use super::without_frontmatter;
use crate::{AppError, SharedState};
use anyhow::anyhow;
use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse, Redirect},
};
use std::sync::Arc;

pub async fn build_response(
    Path(slug): Path<String>,
    State(state): State<Arc<SharedState>>,
) -> Result<impl IntoResponse, AppError> {
    let tera = &state.tera;
    let mut context = tera::Context::new();

    let published_pages = state.published_pages.fetch().await?;
    let published_page = published_pages
        .iter()
        .find(|f| f.frontmatter.slug == slug)
        .ok_or(anyhow!("could not find page"))?;

    let content = without_frontmatter(&published_page.path).await?;
    let description = published_page.frontmatter.description.clone();
    let published_at = published_page.published_at;
    let revised_at = published_page.frontmatter.revised_at;
    let mut title = published_page.frontmatter.title.clone();
    title.push_str(" - Cory Buecker");

    context.insert("content", &content);
    context.insert("description", &description);
    context.insert("title", &title);
    context.insert("published_at", &published_at);
    context.insert("revised_at", &revised_at);

    let rendered = tera
        .render("pages/page.html", &context)
        .map_err(|e| anyhow!("could not render template: {e}"))?;

    Ok(Html(rendered))
}

pub async fn remove_slash(Path(path_slug): Path<String>) -> Redirect {
    let mut redirect = String::from("/post/");
    redirect.push_str(&path_slug);

    Redirect::permanent(&redirect)
}

#[cfg(test)]
mod tests {
    use super::build_response;
    use crate::{
        SharedState,
        pages::{Frontmatter, PublishedPage, PublishedPagesBuilder},
        utilities::tera::{digest_asset, embed_templates},
    };
    use anyhow::Result;
    use axum::{
        extract::{Path, State},
        response::IntoResponse,
    };
    use chrono::Utc;
    use std::{pin::Pin, sync::Arc};
    use tera::Tera;

    struct PublishedPages;

    impl PublishedPagesBuilder for PublishedPages {
        fn fetch<'f>(
            &'f self,
        ) -> Pin<Box<dyn Future<Output = Result<Vec<PublishedPage>>> + Send + Sync + 'f>> {
            Box::pin(async move {
                Ok(vec![PublishedPage {
                    path: "test".to_string(),
                    published_at: Utc::now(),
                    frontmatter: Frontmatter {
                        description: "test".to_string(),
                        preview: "test".to_string(),
                        published_at: Some(Utc::now()),
                        revised_at: None,
                        slug: "test".to_string(),
                        title: "Test".to_string(),
                    },
                }])
            })
        }
    }

    #[tokio::test]
    async fn test_valid_response() {
        let mut tera = Tera::default();

        tera.register_function("digest_asset", digest_asset());
        embed_templates(&mut tera).unwrap();

        let state: SharedState = SharedState {
            tera,
            published_pages: Box::new(PublishedPages),
        };
        let state = Arc::new(state);
        let state = State(state);
        let path = Path("test".to_string());

        let response = build_response(path, state).await.unwrap().into_response();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_string = String::from_utf8(body.to_vec()).unwrap();

        assert!(body_string.contains("Test"));
    }
}
