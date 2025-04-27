use crate::types::{AppError, Page, SharedState};
use anyhow::{Context, anyhow};
use axum::http::{StatusCode, header};
use axum::{body::Body, extract::State, http::HeaderValue, response::IntoResponse};
use std::sync::Arc;
use xml_builder::{XMLBuilder, XMLElement, XMLVersion};

pub async fn build_response(
    State(shared_state): State<Arc<SharedState>>,
) -> Result<impl IntoResponse, AppError> {
    let client = shared_state.client.read().await;

    let pages = Page::published(&client).await?;

    let mut xml = XMLBuilder::new()
        .version(XMLVersion::XML1_1)
        .encoding("UTF-8".into())
        .build();

    let mut urlset = XMLElement::new("urlset");
    urlset.add_attribute("xmlns", "http://www.sitemaps.org/schemas/sitemap/0.9");

    for (current_index, page) in pages.into_iter().enumerate() {
        let mut url = XMLElement::new("url");
        let mut loc = XMLElement::new("loc");

        // First page is treated as the homepage
        if current_index == 0 {
            loc.add_text("https://corybuecker.com".to_string())
                .map_err(|e| anyhow!("Failed to add homepage URL: {}", e))?;
        } else {
            loc.add_text(format!("https://corybuecker.com/post/{}", page.slug))
                .map_err(|e| anyhow!("Failed to add page URL: {}", e))?;
        }

        url.add_child(loc)
            .map_err(|e| anyhow!("Failed to add location to URL: {}", e))?;

        // Use the most recent timestamp available
        let last_modified = page
            .revised_at
            .or(page.published_at)
            .unwrap_or(page.created_at);

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
