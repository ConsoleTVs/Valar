pub mod memory;

use std::time::Duration;

use async_trait::async_trait;
pub use memory::MemoryCache;
use serde::Deserialize;
use serde::Serialize;
use thiserror::Error;
use tokio::time::Instant;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Cache key not found: {0}")]
    NotFound(String),

    #[error(transparent)]
    Deserialize(serde_json::Error),

    #[error(transparent)]
    Serialize(serde_json::Error),
}

#[async_trait]
pub trait Cache {
    async fn get<V>(&self, key: &str) -> Result<V, Error>
    where
        V: for<'de> Deserialize<'de> + Send;

    async fn has(&self, key: &str) -> bool {
        self.get::<()>(key).await.is_ok()
    }

    async fn get_or<V>(&self, key: &str, default: V) -> Result<V, Error>
    where
        V: for<'de> Deserialize<'de> + Send,
    {
        let value = self.get(key).await;

        match value {
            Ok(value) => Ok(value),
            Err(Error::NotFound(_)) => Ok(default),
            Err(error) => Err(error),
        }
    }

    async fn get_or_else<V, F>(&self, key: &str, callback: F) -> Result<V, Error>
    where
        V: for<'de> Deserialize<'de> + Send,
        F: FnOnce() -> V + Send,
    {
        let value = self.get(key).await;

        match value {
            Ok(value) => Ok(value),
            Err(Error::NotFound(_)) => Ok(callback()),
            Err(error) => Err(error),
        }
    }

    async fn get_or_default<V>(&self, key: &str) -> Result<V, Error>
    where
        V: for<'de> Deserialize<'de> + Send,
        V: Default,
    {
        let value = self.get(key).await;

        match value {
            Ok(value) => Ok(value),
            Err(Error::NotFound(_)) => Ok(V::default()),
            Err(error) => Err(error),
        }
    }

    async fn insert<K, V>(&self, key: K, value: V) -> Result<(), Error>
    where
        K: Into<String> + Send,
        V: Serialize + Send;

    async fn insert_for<K, V, D>(&self, key: K, value: V, expires_in: D) -> Result<(), Error>
    where
        K: Into<String> + Send,
        V: Serialize + Send,
        D: Into<Duration> + Send;

    async fn insert_until<K, V, I>(&self, key: K, value: V, expires_at: I) -> Result<(), Error>
    where
        K: Into<String> + Send,
        V: Serialize + Send,
        I: Into<Instant> + Send;

    async fn map<K, V, SV, F>(&self, key: K, callback: F) -> Result<(), Error>
    where
        K: Into<String> + Send,
        V: Serialize + Send,
        SV: for<'de> Deserialize<'de> + Send,
        F: FnOnce(SV) -> V + Send,
    {
        let key: String = key.into();
        let old = self.get(&key).await?;
        let new = callback(old);

        self.insert(key, new).await?;

        Ok(())
    }

    async fn map_or<K, V, SV, F>(&self, default: SV, key: K, callback: F) -> Result<(), Error>
    where
        K: Into<String> + Send,
        V: Serialize + Send,
        SV: for<'de> Deserialize<'de> + Send,
        F: FnOnce(SV) -> V + Send,
    {
        let key: String = key.into();
        let old = self.get_or(&key, default).await?;
        let new = callback(old);

        self.insert(key, new).await?;

        Ok(())
    }

    async fn map_or_else<K, V, SV, SF, F>(
        &self,
        default: SF,
        key: K,
        callback: F,
    ) -> Result<(), Error>
    where
        K: Into<String> + Send,
        V: Serialize + Send,
        SV: for<'de> Deserialize<'de> + Send,
        SF: FnOnce() -> SV + Send,
        F: FnOnce(SV) -> V + Send,
    {
        let key: String = key.into();
        let old = self.get_or_else(&key, default).await?;
        let new = callback(old);

        self.insert(key, new).await?;

        Ok(())
    }

    async fn map_or_default<K, V, SV, F>(&self, key: K, callback: F) -> Result<(), Error>
    where
        K: Into<String> + Send,
        V: Serialize + Send,
        SV: Default,
        SV: for<'de> Deserialize<'de> + Send,
        F: FnOnce(SV) -> V + Send,
    {
        let key: String = key.into();
        let old = self.get_or_default(&key).await?;
        let new = callback(old);

        self.insert(key, new).await?;

        Ok(())
    }

    async fn delete(&self, key: &str);

    async fn clear(&self);
}
