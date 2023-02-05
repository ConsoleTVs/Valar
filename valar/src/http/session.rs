use std::str::FromStr;

use serde::Deserialize;
use serde::Serialize;
use thiserror::Error;
use uuid::Uuid;

use crate::drivers::cache;
use crate::drivers::Cache;
use crate::http::Cookie;
use crate::http::HasCookies;
use crate::http::Request;

#[derive(Error, Debug)]
pub enum Error {
    #[error("No `session_uuid` cookie found")]
    NoSessionUuid,

    #[error(transparent)]
    UuidParse(#[from] uuid::Error),
}

pub struct Session<'a, C: Cache + Sync> {
    cache: &'a C,
    uuid: Uuid,
}

impl<'a, C: Cache + Sync> Session<'a, C> {
    fn key_of(&self, key: &str) -> String {
        format!("session:{}:{}", self.uuid, key)
    }

    pub async fn get<V>(&self, key: &str) -> Result<V, cache::Error>
    where
        V: for<'de> Deserialize<'de> + Send,
    {
        self.cache.get(&self.key_of(key)).await
    }

    pub async fn has(&self, key: &str) -> bool {
        self.cache.has(&self.key_of(key)).await
    }

    pub async fn insert<K, V>(
        &self,
        key: K,
        value: V,
    ) -> Result<&Session<'a, C>, cache::Error>
    where
        K: Into<String> + Send,
        V: Serialize + Send,
    {
        self.cache.insert(&self.key_of(&key.into()), value).await?;

        Ok(self)
    }
}

impl Request {
    pub fn session<'a, C>(&'a self, cache: &'a C) -> Result<Session<'a, C>, Error>
    where
        C: Cache + Sync,
    {
        let cookie = self.cookie("session_uuid").ok_or(Error::NoSessionUuid)?;
        let uuid = Uuid::from_str(cookie.value())?;
        let session = Session { uuid, cache };

        Ok(session)
    }
}
