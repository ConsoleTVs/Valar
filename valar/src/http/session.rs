use std::marker::PhantomData;
use std::str::FromStr;
use std::time::Duration;

use thiserror::Error;
use uuid::Uuid;

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

pub struct Session(Uuid);

pub struct Insert {
    key: String,
    value: String,
    expires_at: Option<Duration>,
}

impl Session {}

impl From<Uuid> for Session {
    fn from(session: Uuid) -> Self {
        Self(session)
    }
}

impl TryFrom<&Request> for Session {
    type Error = Error;

    fn try_from(request: &Request) -> Result<Self, Self::Error> {
        let cookie = request.cookie("session_uuid").ok_or(Error::NoSessionUuid)?;
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
