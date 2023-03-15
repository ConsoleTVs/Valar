use async_trait::async_trait;
use uuid::Uuid;

use crate::http::HasCookies;
use crate::http::HasHeaders;
use crate::http::Request;
use crate::http::RequestCookie;
use crate::http::Result;
use crate::routing::middleware::Handler;
use crate::routing::middleware::Middleware;

pub struct Session;

#[async_trait]
impl Middleware for Session {
    async fn handle(&self, next: Handler, mut request: Request) -> Result {
        if request.has_cookie("session_uuid") {
            return next(request).await;
        }

        let uuid = Uuid::now_v7();
        let cookie =
            RequestCookie::new("session_uuid", uuid.as_hyphenated().to_string()).to_string();

        request.headers_mut().insert("Cookie", cookie);

        let response = next(request).await;

        //

        Ok(response?)
    }
}
