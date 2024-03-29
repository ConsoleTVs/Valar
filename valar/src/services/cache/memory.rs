use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use tokio::spawn;
use tokio::time::interval;
use tokio::time::Instant;

use crate::services::cache::Error;
use crate::services::cache::Insertable;
use crate::services::cache::Retreived;
use crate::services::cache::Value;
use crate::services::Cache;
use crate::State;

/// A memory cache implementation that has a passive and
/// active expiration policy for each entry.
pub struct MemoryCache {
    state: Arc<State<HashMap<String, String>>>,
    expirations: Arc<State<HashMap<String, Instant>>>,
}

impl MemoryCache {
    pub fn new(purge_interval: Duration) -> Self {
        let memory = Self {
            state: Arc::default(),
            expirations: Arc::default(),
        };
        let state = memory.state.clone();
        let expirations = memory.expirations.clone();

        // Passive elimination of expired entries.
        spawn(async move {
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
impl<App> Cache<App> for MemoryCache {
    async fn get(&self, key: &str) -> Result<Value<Retreived>, Error> {
        let mut state = self.state.get().await;

        let value = state
            .get(key)
            .cloned()
            .ok_or_else(|| Error::NotFound(key.to_string()))?;

        let mut expirations = self.expirations.get().await;

        if let Some(expiration) = expirations.get(key) {
            if Instant::now() > *expiration {
                state.remove(key);
                expirations.remove(key);
                return Err(Error::Expired(key.to_string()));
            }
        }

        Ok(Value::new(value))
    }

    async fn insert(&self, key: String, value: Value<Insertable>) -> Result<(), Error> {
        let mut state = self.state.get().await;
        state.insert(key, value.into_value());

        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<(), Error> {
        let mut state = self.state.get().await;

        state.remove(key);

        Ok(())
    }

    async fn clear(&self) -> Result<(), Error> {
        let mut state = self.state.get().await;

        state.clear();

        Ok(())
    }
}
