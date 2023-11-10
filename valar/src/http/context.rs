use std::any::Any;
use std::collections::HashMap;

use thiserror::Error;
use tokio::sync::Mutex;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Key {0} not found")]
    KeyNotFound(String),

    #[error("Type missmatch")]
    TypeMismatch,
}

#[derive(Default, Debug)]
pub struct Context(Mutex<HashMap<String, Box<dyn Any + Send + Sync>>>);

impl Context {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn insert<K, V>(&self, key: K, value: V)
    where
        K: Into<String>,
        V: Any + Send + Sync,
    {
        let mut state = self.0.lock().await;

        state.insert(key.into(), Box::new(value));
    }

    pub async fn cloned<T>(&self, key: &str) -> Result<T, Error>
    where
        T: Clone + Any + Send + Sync,
    {
        let state = self.0.lock().await;

        let value = state
            .get(key)
            .ok_or_else(|| Error::KeyNotFound(key.to_string()))?;

        let value = value.downcast_ref::<T>().ok_or(Error::TypeMismatch)?;

        Ok(value.clone())
    }

    pub async fn has(&self, key: &str) -> bool {
        let state = self.0.lock().await;

        state.contains_key(key)
    }

    pub async fn update<F, T>(&self, key: &str, callback: F) -> Result<(), Error>
    where
        T: Any,
        F: FnOnce(&mut T),
    {
        let mut state = self.0.lock().await;

        let value = state
            .get_mut(key)
            .ok_or_else(|| Error::KeyNotFound(key.to_string()))?;

        let value = value.downcast_mut::<T>().ok_or(Error::TypeMismatch)?;

        callback(value);

        Ok(())
    }

    pub async fn remove<T: Any>(&self, key: &str) -> Result<T, Error> {
        let mut state = self.0.lock().await;

        let value = state
            .remove(key)
            .ok_or_else(|| Error::KeyNotFound(key.to_string()))?;

        let value = value.downcast::<T>().map_err(|_| Error::TypeMismatch)?;

        Ok(*value)
    }
}
