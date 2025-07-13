use crate::{AppError, SharedState};
use anyhow::{Context, anyhow};
use axum::extract::State;
use axum::http::{StatusCode, header};
use axum::{body::Body, http::HeaderValue, response::IntoResponse};
use chrono::Utc;
use std::sync::Arc;
use xml_builder::{XMLBuilder, XMLElement, XMLVersion};

pub async fn build_response(
    State(state): State<Arc<SharedState>>,
) -> Result<impl IntoResponse, AppError> {
    let published_pages = state.published_pages.fetch().await?;

    let mut xml = XMLBuilder::new()
        .version(XMLVersion::XML1_1)
        .encoding("UTF-8".into())
        .build();

    let mut urlset = XMLElement::new("urlset");
    urlset.add_attribute("xmlns", "http://www.sitemaps.org/schemas/sitemap/0.9");

    for (current_index, page) in published_pages.into_iter().enumerate() {
        let mut url = XMLElement::new("url");
        let mut loc = XMLElement::new("loc");

        // First page is treated as the homepage
        if current_index == 0 {
            loc.add_text("https://corybuecker.com".to_string())
                .map_err(|e| anyhow!("Failed to add homepage URL: {}", e))?;
        } else {
            loc.add_text(format!(
                "https://corybuecker.com/post/{}",
                page.frontmatter.slug
            ))
            .map_err(|e| anyhow!("Failed to add page URL: {}", e))?;
        }

        url.add_child(loc)
            .map_err(|e| anyhow!("Failed to add location to URL: {}", e))?;

        // Use the most recent timestamp available
        let last_modified = page
            .frontmatter
            .revised_at
            .or(page.frontmatter.published_at)
            .unwrap_or(Utc::now());

        let mut lastmod = XMLElement::new("lastmod");
        lastmod
            .add_text(last_modified.to_rfc3339())
            .map_err(|e| anyhow!("Failed to add lastmod text: {}", e))?;

        url.add_child(lastmod)
            .map_err(|e| anyhow!("Failed to add lastmod to URL: {}", e))?;

        urlset
            .add_child(url)
            .map_err(|e| anyhow!("Failed to add URL to urlset: {}", e))?;
    }

    xml.set_root_element(urlset);

    let mut output = Vec::<u8>::new();
    xml.generate(&mut output)
        .map_err(|e| anyhow!("Failed to generate XML: {}", e))?;

    let xml_string = String::from_utf8(output).context("could not render XML")?;

    // Create a response with the correct content type
    let response = (
        StatusCode::OK,
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/xml"),
        )],
        Body::from(xml_string),
    )
        .into_response();

    Ok(response)
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
    use axum::{extract::State, http::StatusCode, response::IntoResponse};
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
        published_at: DateTime<Utc>,
        revised_at: Option<DateTime<Utc>>,
    ) -> PublishedPage {
        PublishedPage {
            path: path.to_string(),
            published_at,
            frontmatter: Frontmatter {
                description: description.to_string(),
                preview: format!("{title} preview"),
                published_at: Some(published_at),
                revised_at,
                slug: slug.to_string(),
                title: title.to_string(),
            },
        }
    }

    async fn execute_request_and_get_body(state: Arc<SharedState>) -> (String, StatusCode) {
        let response = build_response(State(state)).await.unwrap().into_response();
        let status = response.status();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_string = String::from_utf8(body.to_vec()).unwrap();
        (body_string, status)
    }

    #[tokio::test]
    async fn test_build_response_success() {
        let published_date = DateTime::parse_from_rfc3339("2023-01-01T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let revised_date = DateTime::parse_from_rfc3339("2023-01-15T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc);

        let pages = vec![
            create_page(
                "test-home",
                "home",
                "Home",
                "Home page description",
                published_date,
                None,
            ),
            create_page(
                "test-page1",
                "page1",
                "Page 1",
                "Page 1 description",
                published_date,
                Some(revised_date),
            ),
        ];

        let state = create_shared_state(pages);
        let (body_string, status) = execute_request_and_get_body(state).await;

        // Check status
        assert_eq!(status, StatusCode::OK);

        // Check that it's valid XML
        assert!(body_string.starts_with("<?xml"));
        assert!(body_string.contains("urlset"));
        assert!(body_string.contains("http://www.sitemaps.org/schemas/sitemap/0.9"));

        // Check that homepage URL is included
        assert!(body_string.contains("https://corybuecker.com</loc>"));

        // Check that other pages are included with correct URLs
        assert!(body_string.contains("https://corybuecker.com/post/page1"));

        // Check that lastmod dates are included
        assert!(body_string.contains("2023-01-15T00:00:00+00:00")); // revised date
        assert!(body_string.contains("2023-01-01T00:00:00+00:00")); // published date
    }

    #[tokio::test]
    async fn test_build_response_uses_revised_date_when_available() {
        let published_date = DateTime::parse_from_rfc3339("2023-01-01T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let revised_date = DateTime::parse_from_rfc3339("2023-01-15T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc);

        let pages = vec![create_page(
            "test-revised",
            "revised",
            "Revised Page",
            "Revised page description",
            published_date,
            Some(revised_date),
        )];

        let state = create_shared_state(pages);
        let (body_string, _) = execute_request_and_get_body(state).await;

        // Should use the revised date, not the published date
        assert!(body_string.contains("2023-01-15T00:00:00+00:00"));
        assert!(!body_string.contains("2023-01-01T00:00:00+00:00"));
    }

    #[tokio::test]
    async fn test_build_response_uses_published_date_when_no_revised() {
        let published_date = DateTime::parse_from_rfc3339("2023-01-01T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc);

        let pages = vec![create_page(
            "test-published",
            "published",
            "Published Page",
            "Published page description",
            published_date,
            None,
        )];

        let state = create_shared_state(pages);
        let (body_string, _) = execute_request_and_get_body(state).await;

        // Should use the published date
        assert!(body_string.contains("2023-01-01T00:00:00+00:00"));
    }

    #[tokio::test]
    async fn test_build_response_no_dates_uses_current_time() {
        let pages = vec![PublishedPage {
            path: "test-no-dates".to_string(),
            published_at: Utc::now(),
            frontmatter: Frontmatter {
                description: "No dates page description".to_string(),
                preview: "No dates preview".to_string(),
                published_at: None,
                revised_at: None,
                slug: "no-dates".to_string(),
                title: "No Dates Page".to_string(),
            },
        }];

        let state = create_shared_state(pages);
        let (body_string, _) = execute_request_and_get_body(state).await;

        // Should contain some valid RFC3339 timestamp (current time)
        assert!(body_string.contains("T"));
        assert!(body_string.contains("+00:00"));
    }

    #[tokio::test]
    async fn test_build_response_empty_pages() {
        let state = create_shared_state(vec![]);
        let (body_string, _) = execute_request_and_get_body(state).await;

        // Should still be valid XML with empty urlset
        assert_eq!(body_string.matches("<url>").count(), 0);
    }

    #[tokio::test]
    async fn test_build_response_multiple_pages() {
        let published_date = DateTime::parse_from_rfc3339("2023-01-01T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc);

        let pages = vec![
            create_page(
                "test-home",
                "home",
                "Home",
                "Home page",
                published_date,
                None,
            ),
            create_page(
                "test-page1",
                "page1",
                "Page 1",
                "Page 1",
                published_date,
                None,
            ),
            create_page(
                "test-page2",
                "page2",
                "Page 2",
                "Page 2",
                published_date,
                None,
            ),
        ];

        let state = create_shared_state(pages);
        let (body_string, _) = execute_request_and_get_body(state).await;

        // First page should be homepage
        assert!(body_string.contains("https://corybuecker.com</loc>"));

        // Other pages should have /post/ prefix
        assert!(body_string.contains("https://corybuecker.com/post/page1"));
        assert!(body_string.contains("https://corybuecker.com/post/page2"));

        // Should contain 3 URL entries total
        assert_eq!(body_string.matches("<url>").count(), 3);
        assert_eq!(body_string.matches("</url>").count(), 3);
    }
}
