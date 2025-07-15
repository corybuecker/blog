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

    let published_pages = state.published_pages.get_all()?;
    let published_page = published_pages
        .iter()
        .find(|f| f.frontmatter.slug == slug)
        .ok_or(anyhow!("could not find page"))?;

    let content = state.published_pages.read(&published_page.path).await?;
    let content = without_frontmatter(&content).await?;

    let description = published_page.frontmatter.description.clone();
    let published_at = published_page.published_at;
    let revised_at = published_page.frontmatter.revised_at;
    let title = published_page.frontmatter.title.clone();

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
    use super::{build_response, remove_slash};
    use crate::{
        SharedState,
        pages::{Frontmatter, PublicationManager, PublishedPage},
        utilities::tera::{digest_asset, embed_templates},
    };
    use anyhow::Result;
    use axum::{
        extract::{Path, State},
        http,
        response::IntoResponse,
    };
    use chrono::{DateTime, Utc};
    use std::{future::Future, pin::Pin, sync::Arc};
    use tera::Tera;

    struct MockPublishedPages {
        pages: Vec<PublishedPage>,
    }

    impl PublicationManager for MockPublishedPages {
        fn get_all(&self) -> Result<Vec<PublishedPage>> {
            Ok(self.pages.clone())
        }

        fn publish<'f>(
            &'f mut self,
        ) -> Pin<Box<dyn Future<Output = Result<usize>> + Send + Sync + 'f>> {
            Box::pin(async move { Ok(self.pages.len()) })
        }

        fn read<'f>(
            &'f self,
            _path: &'f str,
        ) -> Pin<Box<dyn Future<Output = Result<String>> + Send + Sync + 'f>> {
            Box::pin(async move { Ok("this is the page content".to_string()) })
        }
    }

    fn setup_tera() -> Tera {
        let mut tera = Tera::default();
        tera.register_function("digest_asset", digest_asset());
        embed_templates(&mut tera).unwrap();
        tera
    }

    fn create_shared_state(pages: Vec<PublishedPage>) -> Arc<SharedState> {
        let tera = setup_tera();
        let mock_pages = MockPublishedPages { pages };
        Arc::new(SharedState {
            tera,
            published_pages: Box::new(mock_pages),
        })
    }

    fn create_page(
        path: &str,
        slug: &str,
        title: &str,
        description: &str,
        revised_at: Option<DateTime<Utc>>,
    ) -> PublishedPage {
        PublishedPage {
            path: path.to_string(),
            published_at: Utc::now(),
            frontmatter: Frontmatter {
                description: description.to_string(),
                preview: format!("{title} preview"),
                published_at: Some(Utc::now()),
                revised_at,
                slug: slug.to_string(),
                title: title.to_string(),
            },
        }
    }

    async fn execute_request_and_get_body(slug: &str, state: Arc<SharedState>) -> String {
        let path = Path(slug.to_string());
        let response = build_response(path, State(state))
            .await
            .unwrap()
            .into_response();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        String::from_utf8(body.to_vec()).unwrap()
    }

    #[tokio::test]
    async fn test_valid_response() {
        let pages = vec![create_page("test", "test", "Test", "test", None)];
        let state = create_shared_state(pages);
        let body_string = execute_request_and_get_body("test", state).await;

        assert!(body_string.contains("Test"));
    }

    #[tokio::test]
    async fn test_build_response_with_revised_date() {
        let revised_date = DateTime::parse_from_rfc3339("2023-01-15T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let pages = vec![create_page(
            "test-revised",
            "revised-page",
            "Revised Page",
            "Revised page description",
            Some(revised_date),
        )];

        let state = create_shared_state(pages);
        let body_string = execute_request_and_get_body("revised-page", state).await;

        assert!(body_string.contains("Revised Page"));
        assert!(body_string.contains("Revised page description"));
    }

    #[tokio::test]
    async fn test_build_response_page_not_found() {
        let pages = vec![create_page(
            "test-existing",
            "existing",
            "Existing Page",
            "Existing page",
            None,
        )];
        let state = create_shared_state(pages);
        let path = Path("non-existent".to_string());

        let result = build_response(path, State(state)).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_build_response_title_includes_suffix() {
        let pages = vec![create_page(
            "test-title",
            "title-test",
            "My Great Article",
            "Title test page",
            None,
        )];

        let state = create_shared_state(pages);
        let body_string = execute_request_and_get_body("title-test", state).await;

        assert!(body_string.contains("My Great Article"));
    }

    #[tokio::test]
    async fn test_build_response_multiple_pages_finds_correct_one() {
        let pages = vec![
            create_page("test-page1", "page1", "Page 1", "Page 1 description", None),
            create_page("test-page2", "page2", "Page 2", "Page 2 description", None),
            create_page("test-page3", "page3", "Page 3", "Page 3 description", None),
        ];

        let state = create_shared_state(pages);
        let body_string = execute_request_and_get_body("page2", state).await;

        // Should contain the correct page content
        assert!(body_string.contains("Page 2"));
        assert!(body_string.contains("Page 2 description"));
        // Should not contain other pages
        assert!(!body_string.contains("Page 1 description"));
        assert!(!body_string.contains("Page 3 description"));
    }

    #[tokio::test]
    async fn test_build_response_empty_pages_list() {
        let state = create_shared_state(vec![]);
        let path = Path("any-slug".to_string());

        let result = build_response(path, State(state)).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_remove_slash_redirect() {
        let path = Path("my-awesome-post".to_string());
        let redirect = remove_slash(path).await;

        // Convert to response to check headers and status
        let response = redirect.into_response();

        // Check that it's a permanent redirect
        assert_eq!(response.status(), http::StatusCode::PERMANENT_REDIRECT);

        // Check the redirect location
        let location = response.headers().get("location").unwrap();
        assert_eq!(location, "/post/my-awesome-post");
    }

    #[tokio::test]
    async fn test_remove_slash_redirect_with_special_characters() {
        let path = Path("post-with-dashes-and_underscores".to_string());
        let redirect = remove_slash(path).await;

        let response = redirect.into_response();
        assert_eq!(response.status(), http::StatusCode::PERMANENT_REDIRECT);

        let location = response.headers().get("location").unwrap();
        assert_eq!(location, "/post/post-with-dashes-and_underscores");
    }

    #[tokio::test]
    async fn test_remove_slash_redirect_empty_slug() {
        let path = Path("".to_string());
        let redirect = remove_slash(path).await;

        let response = redirect.into_response();
        assert_eq!(response.status(), http::StatusCode::PERMANENT_REDIRECT);

        let location = response.headers().get("location").unwrap();
        assert_eq!(location, "/post/");
    }
}
