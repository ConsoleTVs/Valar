use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use serde::Deserialize;
use serde::Serialize;
use tokio::time::interval;
use tokio::time::Instant;

use crate::drivers::cache::Error;
use crate::drivers::Cache;
use crate::State;

/// A memory cache implementation that has a passive and
/// active expiration policy for each entry.
pub struct MemoryCache {
    state: Arc<State<HashMap<String, String>>>,
    expirations: Arc<State<HashMap<String, Instant>>>,
}

impl MemoryCache {
    pub fn with_purge_interval(purge_interval: Duration) -> Self {
        let memory = Self {
            state: Arc::default(),
            expirations: Arc::default(),
        };
        let state = memory.state.clone();
        let expirations = memory.expirations.clone();

        // Passive elimination of expired entries.
        tokio::spawn(async move {
            let mut interval = interval(purge_interval);

            loop {
                interval.tick().await;

                let mut expirations = expirations.get().await;

                let expired_keys: Vec<String> = expirations
                    .iter()
                    .filter(|(_, expiration)| Instant::now() > **expiration)
                    .map(|(key, _)| key.clone())
                    .collect();

                let mut state = state.get().await;

                for key in expired_keys {
                    state.remove(&key);
                    expirations.remove(&key);
                }

                drop(state);
                drop(expirations);
            }
        });

        memory
    }
}

#[async_trait]
impl Cache for MemoryCache {
    async fn get<V>(&self, key: &str) -> Result<V, Error>
    where
        V: for<'de> Deserialize<'de> + Send,
    {
        let mut state = self.state.get().await;

        let value = state
            .get(key)
            .cloned()
            .ok_or(Error::NotFound(key.to_string()))?;

        let mut expirations = self.expirations.get().await;

        if let Some(expiration) = expirations.get(key) {
            if Instant::now() > *expiration {
                state.remove(key);
                expirations.remove(key);
                return Err(Error::NotFound(key.to_string()));
            }
        }

        drop(state);
        drop(expirations);

        serde_json::from_str(&value).map_err(Error::Deserialize)
    }

    async fn insert<K, V>(&self, key: K, value: V) -> Result<&Self, Error>
    where
        K: Into<String> + Send,
        V: Serialize + Send,
    {
        let value = serde_json::to_string(&value).map_err(Error::Serialize)?;
        let mut state = self.state.get().await;

        state.insert(key.into(), value);

        Ok(self)
    }

    async fn insert_expirable<K, V>(
        &self,
        key: K,
        value: V,
        expires_in: Duration,
    ) -> Result<&Self, Error>
    where
        K: Into<String> + Send,
        V: Serialize + Send,
    {
        let key: String = key.into();
        self.insert(key.clone(), value).await?;

        let mut expirations = self.expirations.get().await;
        expirations.insert(key, Instant::now() + expires_in);

        Ok(self)
    }

    async fn delete(&self, key: &str) {
        let mut state = self.state.get().await;

        state.remove(key);
    }

    async fn clear(&self) {
        let mut state = self.state.get().await;

        state.clear();
    }
}
