use crate::{
    types::SharedState,
    utilities::tera::{digest_asset, embed_templates},
};
use anyhow::Result;
use rand::{Rng, distr::Alphanumeric};
use std::iter;
use std::sync::Arc;
use tera::Tera;

// Generate a random slug (lowercase with hyphens)
#[allow(dead_code)]
pub fn random_slug(prefix: &str) -> String {
    let random_part = random_string(24).to_lowercase();
    format!("{prefix}-{random_part}")
}

// Generate a random string of specified length
#[allow(dead_code)]
fn random_string(length: usize) -> String {
    let mut rng = rand::rng();
    iter::repeat_with(|| rng.sample(Alphanumeric))
        .map(char::from)
        .take(length)
        .collect()
}

#[allow(dead_code)]
pub async fn create_test_shared_state() -> Result<Arc<SharedState>> {
    // Create a simple Tera instance for testing
    let mut tera = Tera::default();
    tera.register_function("digest_asset", digest_asset());
    embed_templates(&mut tera)?;

    Ok(Arc::new(SharedState { tera }))
}
