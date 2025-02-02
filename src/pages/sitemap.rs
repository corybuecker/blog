use crate::{Page, SharedState};
use axum::{
    body::Body,
    extract::State,
    response::{IntoResponse, Response},
};
use bson::doc;
use mongodb::options::FindOptions;
use std::sync::Arc;
use xml_builder::{XMLBuilder, XMLElement, XMLVersion};

pub async fn build_response(State(shared_state): State<Arc<SharedState>>) -> Response {
    let database = &shared_state.mongo.database("blog");
    let find_options = FindOptions::builder();
    let find_options = find_options.sort(doc! {"published_at": -1});
    let collection = database
        .collection::<Page>("pages")
        .find(doc! {"published_at": doc!{"$exists": true}})
        .with_options(find_options.build())
        .await;

    let mut xml = XMLBuilder::new()
        .version(XMLVersion::XML1_1)
        .encoding("UTF-8".into())
        .build();

    let mut urlset = XMLElement::new("urlset");
    urlset.add_attribute("xmlns", "http://www.sitemaps.org/schemas/sitemap/0.9");
    let mut current_index = 0;
    if let Ok(mut cursor) = collection {
        while cursor.advance().await.unwrap() {
            let page = cursor.deserialize_current().unwrap();
            if current_index == 0 {
                current_index += 1;
                continue;
            }
            let mut url = XMLElement::new("url");
            let lastmodts = page
                .revised_at
                .or(page.published_at)
                .or(Some(page.created_at.into()));

            if let Some(lastmodts) = lastmodts {
                let mut lastmod = XMLElement::new("lastmod");
                let _ = lastmod.add_text(lastmodts.to_rfc3339());
                let _ = url.add_child(lastmod);
            }

            let mut loc = XMLElement::new("loc");
            let _ = loc.add_text(format!("https://corybuecker.com/post/{}", page.slug));
            let _ = url.add_child(loc);

            let _ = urlset.add_child(url);
        }
    }

    xml.set_root_element(urlset);

    let mut output = Vec::<u8>::new();
    let _ = xml.generate(&mut output);
    Body::from(String::from_utf8(output).unwrap()).into_response()
}
