use super::without_frontmatter;
use crate::{AppError, SharedState};
use anyhow::anyhow;
use axum::{extract::State, response::Html};
use serde::Serialize;
use std::{collections::VecDeque, sync::Arc};

#[derive(Serialize, Debug)]
struct Link {
    title: String,
    slug: String,
}

pub async fn build_response(
    State(shared_state): State<Arc<SharedState>>,
) -> Result<Html<String>, AppError> {
    let tera = &shared_state.tera;
    let mut context = tera::Context::new();

    let published_pages = shared_state.published_pages.fetch().await?;
    let published_page = published_pages
        .first()
        .ok_or(anyhow!("could not get homepage"))?;

    let content = shared_state
        .published_pages
        .read_content(&published_page.path.to_string())
        .await?;
    let content = without_frontmatter(&content).await?;

    let description = published_page.frontmatter.description.clone();
    let published_at = published_page.published_at;
    let title = published_page.frontmatter.title.clone();
    let revised_at = published_page.frontmatter.revised_at;

    let mut pages: VecDeque<Link> = published_pages
        .into_iter()
        .map(|row| Link {
            title: row.frontmatter.title,
            slug: row.frontmatter.slug,
        })
        .collect();

    pages.pop_front();

    context.insert("pages", &pages);
    context.insert("content", &content);
    context.insert("description", &description);
    context.insert("title", &title);
    context.insert("published_at", &published_at);
    context.insert("revised_at", &revised_at);

    let rendered = tera
        .render("pages/home.html", &context)
        .map_err(|e| anyhow!("could not render template: {e}"))?;

    Ok(Html(rendered))
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
    use axum::{extract::State, response::IntoResponse};
    use chrono::{DateTime, Utc};
    use std::{future::Future, pin::Pin, sync::Arc};
    use tera::Tera;

    struct MockPublishedPages {
        pages: Vec<PublishedPage>,
    }

    impl PublishedPagesBuilder for MockPublishedPages {
        fn fetch<'f>(
            &'f self,
        ) -> Pin<Box<dyn Future<Output = Result<Vec<PublishedPage>>> + Send + Sync + 'f>> {
            let pages = self.pages.clone();
            Box::pin(async move { Ok(pages) })
        }

        fn read_content<'f>(
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

    async fn execute_request_and_get_body(state: Arc<SharedState>) -> String {
        let response = build_response(State(state)).await.unwrap().into_response();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        String::from_utf8(body.to_vec()).unwrap()
    }

    #[tokio::test]
    async fn test_build_response_success() {
        let pages = vec![
            create_page("test-home", "home", "Home", "Home page description", None),
            create_page("test-page1", "page1", "Page 1", "Page 1 description", None),
            create_page("test-page2", "page2", "Page 2", "Page 2 description", None),
        ];

        let state = create_shared_state(pages);
        let body_string = execute_request_and_get_body(state).await;

        // Check that the home page title is rendered with the suffix
        assert!(body_string.contains("Home"));

        // Check that the home page description is included
        assert!(body_string.contains("Home page description"));

        // Check that other pages are listed (first page is excluded from the list)
        assert!(body_string.contains("Page 1"));
        assert!(body_string.contains("Page 2"));
    }

    #[tokio::test]
    async fn test_build_response_with_revised_at() {
        let revised_date = Utc::now();
        let pages = vec![create_page(
            "test-revised",
            "revised",
            "Revised Page",
            "Revised page description",
            Some(revised_date),
        )];

        let state = create_shared_state(pages);
        let body_string = execute_request_and_get_body(state).await;

        assert!(body_string.contains("Revised Page"));
    }

    #[tokio::test]
    async fn test_build_response_no_pages_error() {
        let state = create_shared_state(vec![]);
        let result = build_response(State(state)).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_build_response_single_page() {
        let pages = vec![create_page(
            "test-single",
            "single",
            "Single Page",
            "Single page description",
            None,
        )];

        let state = create_shared_state(pages);
        let body_string = execute_request_and_get_body(state).await;

        // With only one page, the pages list should be empty after pop_front()
        assert!(body_string.contains("Single Page"));
    }
}
