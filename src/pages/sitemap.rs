use crate::types::{AppError, Page, SharedState};
use anyhow::Context;
use axum::{body::Body, extract::State};
use std::sync::Arc;
use xml_builder::{XMLBuilder, XMLElement, XMLVersion};

pub async fn build_response(
    State(shared_state): State<Arc<SharedState>>,
) -> Result<Body, AppError> {
    let client = &shared_state.client;

    let pages = Page::published(client).await?;

    let mut xml = XMLBuilder::new()
        .version(XMLVersion::XML1_1)
        .encoding("UTF-8".into())
        .build();

    let mut urlset = XMLElement::new("urlset");
    urlset.add_attribute("xmlns", "http://www.sitemaps.org/schemas/sitemap/0.9");

    for (current_index, page) in pages.into_iter().enumerate() {
        let mut url = XMLElement::new("url");
        if current_index == 0 {
            let mut loc = XMLElement::new("loc");
            let _ = loc.add_text("https://corybuecker.com".to_string());
            let _ = url.add_child(loc);
        } else {
            let mut loc = XMLElement::new("loc");
            let _ = loc.add_text(format!("https://corybuecker.com/post/{}", page.slug));
            let _ = url.add_child(loc);
        }

        let lastmodts = page
            .revised_at
            .or(page.published_at)
            .or(Some(page.created_at));

        if let Some(lastmodts) = lastmodts {
            let mut lastmod = XMLElement::new("lastmod");
            let _ = lastmod.add_text(lastmodts.to_rfc3339());
            let _ = url.add_child(lastmod);
        }

        let _ = urlset.add_child(url);
    }

    xml.set_root_element(urlset);

    let mut output = Vec::<u8>::new();
    let _ = xml.generate(&mut output);

    Ok(Body::from(
        String::from_utf8(output).context("could not render XML")?,
    ))
}
