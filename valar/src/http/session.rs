use std::str::FromStr;
use std::time::Duration;

use thiserror::Error;
use tokio::time::Instant;
use uuid::Uuid;

use crate::drivers::cache;
use crate::drivers::Cache;
use crate::http::Request;

#[derive(Error, Debug)]
pub enum Error {
    #[error("No `session_uuid` cookie found. Make sure the session middleware is in place.")]
    NoSessionUuid,

    #[error(transparent)]
    UuidParse(#[from] uuid::Error),
}

#[derive(Clone, Copy)]
pub struct Session(Uuid);

pub struct Insert {
    session: Session,
    key: String,
    value: String,
    expires_at: Option<Instant>,
}

pub struct Obtain {
    session: Session,
    key: String,
}

impl Insert {
    pub fn session<S>(mut self, session: S) -> Self
    where
        S: Into<Session>,
    {
        self.session = session.into();

        self
    }

    pub fn expires_at(mut self, expires_at: Instant) -> Self {
        self.expires_at = Some(expires_at);

        self
    }

    pub fn expires_in(mut self, expires_in: Duration) -> Self {
        self.expires_at = Some(Instant::now() + expires_in);

        self
    }

    pub async fn on<C>(self, cache: &C) -> Result<(), cache::Error>
    where
        C: Cache,
    {
        let key = format!("session:{}:{}", self.session.0.as_hyphenated(), self.key);
        let expires_at = self
            .expires_at
            .unwrap_or_else(|| Instant::now() + Duration::from_secs(60 * 60 * 24));

        cache.insert_until(key, self.value, expires_at).await
    }
}

impl Obtain {
    pub async fn on<C>(self, cache: &C) -> Result<String, cache::Error>
    where
        C: Cache,
    {
        let key = format!("session:{}:{}", self.session.0.as_hyphenated(), self.key);

        cache.get(&key).await
    }
}

impl Session {
    pub fn set<K, V>(&self, key: K, value: V) -> Insert
    where
        K: Into<String>,
        V: Into<String>,
    {
        Insert {
            session: *self,
            key: key.into(),
            value: value.into(),
            expires_at: None,
        }
    }

    pub fn get<K>(&self, key: K) -> Obtain
    where
        K: Into<String>,
    {
        Obtain {
            session: *self,
            key: key.into(),
        }
    }
}

impl From<Uuid> for Session {
    fn from(session: Uuid) -> Self {
        Self(session)
    }
}

impl TryFrom<&Request> for Session {
    type Error = Error;

    fn try_from(request: &Request) -> Result<Self, Self::Error> {
        let cookie = request
            .headers()
            .cookie("session_uuid")
            .ok_or(Error::NoSessionUuid)?;
        let uuid = Uuid::from_str(cookie.value())?;
        let session = Session::from(uuid);

        Ok(session)
    }
}

impl Request {
    pub fn session(&self) -> Result<Session, Error> {
        Session::try_from(self)
    }
}
