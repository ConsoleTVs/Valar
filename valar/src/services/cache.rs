pub mod memory;

use std::marker::PhantomData;
// use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
pub use memory::MemoryCache;
// use serde::Deserialize;
// use serde::Serialize;
use thiserror::Error;
use tokio::time::Instant;

// use crate::services::Service;
// use crate::services::Singleton;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Cache key not found: {0}")]
    NotFound(String),

    #[error("Cache key expired: {0}")]
    Expired(String),
}

/// The default store for the cache.
pub struct Default;

pub type Cacheable<Store = Default> = dyn Cache<Store> + Send + Sync;

pub enum Insertable {}

pub enum Retreived {}

pub struct Value<T = Insertable> {
    _type: PhantomData<T>,
    value: String,
    expires_at: Option<Instant>,
}

impl<T> Value<T> {
    pub fn new(value: String) -> Self {
        Self {
            _type: PhantomData::<T>,
            value,
            expires_at: None,
        }
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn into_value(self) -> String {
        self.value
    }
}

impl Value<Insertable> {
    pub fn expires_in(mut self, duration: Duration) -> Self {
        self.expires_at = Some(Instant::now() + duration);

        self
    }

    pub fn expires_at(mut self, instant: Instant) -> Self {
        self.expires_at = Some(instant);

        self
    }
}

impl<T> From<Value<T>> for String {
    fn from(value: Value<T>) -> Self {
        value.into_value()
    }
}

#[async_trait]
pub trait Cache<Store = Default> {
    async fn get(&self, key: &str) -> Result<Value<Retreived>, Error>;
    async fn insert(&self, key: String, value: Value<Insertable>) -> Result<(), Error>;
    async fn delete(&self, key: &str) -> Result<(), Error>;
    async fn clear(&self) -> Result<(), Error>;
}

// #[derive(Error, Debug)]
// pub enum FacadeError {
//     #[error(transparent)]
//     Cache(#[from] Error),

//     #[error(transparent)]
//     Serde(#[from] serde_json::Error),
// }

// pub struct Facade<App, Store = Default> {
//     app: Arc<App>,
//     cache: Arc<Cacheable<Store>>,
// }

// impl<App, Store> Facade<App, Store> {
//     pub fn new<A, C>(app: A, cache: C) -> Self
//     where
//         A: Into<Arc<App>>,
//         C: Into<Arc<Cacheable<Store>>>,
//     {
//         Self {
//             app: app.into(),
//             cache: cache.into(),
//         }
//     }

//     pub async fn get<V>(&self, key: &str) -> Result<V,
// FacadeError>     where
//         V: for<'a> Deserialize<'a>,
//     {
//         let value = self.cache.get(key).await?;
//         let value =
// serde_json::from_str(&value.into_value())?;

//         Ok(value)
//     }

//     pub async fn insert<K, V>(&self, key: K, value: V) ->
// Result<(), FacadeError>     where
//         K: Into<String>,
//         V: Serialize,
//     {
//         let value = serde_json::to_string(&value)?;
//         self.cache.insert(key.into(),
// value.into_value()).await?;

//         Ok(())
//     }

//     pub async fn insert_for<K, V, E>(
//         &self,
//         key: K,
//         value: V,
//         expires_in: E,
//     ) -> Result<(), FacadeError>
//     where
//         K: Into<String>,
//         V: Serialize,
//         E: Into<Duration>,
//     {
//         let value = serde_json::to_string(&value)?;
//         self.0
//             .insert_for(key.into(), value,
// expires_in.into())             .await?;

//         Ok(())
//     }

//     pub async fn insert_until<K, V, E>(
//         &self,
//         key: K,
//         value: V,
//         expires_at: E,
//     ) -> Result<(), FacadeError>
//     where
//         K: Into<String>,
//         V: Serialize,
//         E: Into<Instant>,
//     {
//         let value = serde_json::to_string(&value)?;
//         self.0
//             .insert_until(key.into(), value,
// expires_at.into())             .await?;

//         Ok(())
//     }

//     pub async fn update<O, N, F>(&self, key: &str,
// callback: F) -> Result<(), FacadeError>     where
//         O: for<'a> Deserialize<'a>,
//         N: Serialize,
//         F: FnOnce(O) -> N,
//     {
//         let old: O = self.get(key).await?;
//         let new = callback(old);
//         self.insert(key, new).await?;

//         Ok(())
//     }

//     pub async fn delete(&self, key: &str) -> Result<(),
// FacadeError> {         self.0.delete(key).await?;

//         Ok(())
//     }

//     pub async fn clear(&self) -> Result<(), FacadeError>
// {         self.0.clear().await?;

//         Ok(())
//     }
// }

// impl<Store, T: Singleton<Cacheable<Store>>>
// Service<Facade<Store>> for T {     fn service(&self) ->
// Facade<Store> {         Facade::new(self,
// self.singleton())     }
// }
