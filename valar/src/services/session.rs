use std::time::Duration;

use async_trait::async_trait;
use thiserror::Error;
use tokio::time::Instant;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Session key not found: {0}")]
    NotFound(String),

    #[error("Session key expired: {0}")]
    Expired(String),
}

#[async_trait]
pub trait Session {
    async fn get(&self, key: &str) -> Result<String, Error>;

    async fn insert(&self, key: String, value: String) -> Result<(), Error>;

    async fn insert_for(
        &self,
        key: String,
        value: String,
        expires_in: Duration,
    ) -> Result<(), Error>;

    async fn insert_until(
        &self,
        key: String,
        value: String,
        expires_at: Instant,
    ) -> Result<(), Error>;

    async fn delete(&self, key: &str) -> Result<(), Error>;

    async fn clear(&self) -> Result<(), Error>;
}
