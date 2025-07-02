use crate::{
    types::SharedState,
    utilities::tera::{digest_asset, embed_templates},
};
use anyhow::Result;
use rand::{Rng, distr::Alphanumeric};
use std::iter;
use std::sync::Arc;
use tera::Tera;
use tokio::sync::RwLock;
use tokio_postgres::{Client, NoTls, Socket, connect, tls::NoTlsStream};

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

/// Creates a connection to the test database.
///
/// This function connects to a PostgreSQL database using the TEST_DATABASE_URL
/// environment variable. It's designed for test environments and will panic
/// if the environment variable is not set.
///
/// # Returns
///
/// A tuple containing the PostgreSQL client and the connection object.
/// The connection should be spawned in a separate task.
///
/// # Example
///
/// ```
/// let (client, connection) = create_test_db_connection().await?;
/// tokio::spawn(connection);
/// ```
#[allow(dead_code)]
pub async fn create_test_db_connection()
-> Result<(Client, tokio_postgres::Connection<Socket, NoTlsStream>)> {
    let database_url =
        std::env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL must be set for tests");

    let (client, connection) = connect(&database_url, NoTls).await?;

    Ok((client, connection))
}

/// Creates a TestSharedState instance for testing.
///
/// This function creates a TestSharedState with a connection to the test database
/// and a simple Tera instance. It's useful for testing components that
/// require a SharedState-like object.
///
/// # Returns
///
/// An Arc-wrapped TestSharedState instance ready for use in tests.
///
/// # Example
///
/// ```
/// let shared_state = create_test_shared_state().await?;
/// // Use shared_state in your tests
/// ```
///
#[allow(dead_code)]
pub async fn create_test_shared_state() -> Result<Arc<SharedState>> {
    let (client, connection) = create_test_db_connection().await?;

    // Spawn the connection in a background task
    tokio::spawn(connection);

    // Create a simple Tera instance for testing
    let mut tera = Tera::default();
    tera.register_function("digest_asset", digest_asset());
    embed_templates(&mut tera)?;

    Ok(Arc::new(SharedState {
        tera,
        client: RwLock::new(client),
    }))
}

/// Cleans up test data from the database.
///
/// This function can be used to clean up any test data created during tests.
/// It should be called in test teardown to ensure a clean state for the next test.
///
/// # Arguments
///
/// * `client` - A reference to the PostgreSQL client.
///
/// # Example
///
/// ```
/// // After your test
/// cleanup_test_data(&shared_state.client).await?;
/// ```
#[allow(dead_code)]
pub async fn cleanup_test_data(_client: &Client) -> Result<()> {
    // Add specific cleanup queries here
    // For example:
    // client.execute("DELETE FROM pages WHERE title LIKE 'Test%'", &[]).await?;

    Ok(())
}

/// Sets up test data in the database.
///
/// This function can be used to set up any test data needed for tests.
/// It should be called in test setup to ensure the necessary data is available.
///
/// # Arguments
///
/// * `client` - A reference to the PostgreSQL client.
///
/// # Example
///
/// ```
/// // Before your test
/// setup_test_data(&shared_state.client).await?;
/// ```
#[allow(dead_code)]
pub async fn setup_test_data(_client: &Client) -> Result<()> {
    // Add specific setup queries here
    // For example:
    // client.execute(
    //     "INSERT INTO pages (title, slug, content, markdown, description, preview, created_at, updated_at)
    //      VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
    //     &[&"Test Page", &"test-page", &"<p>Test content</p>", &"Test content", &"Test description", &"Test preview", &Utc::now(), &Utc::now()]
    // ).await?;

    Ok(())
}
