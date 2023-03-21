use async_trait::async_trait;
use uuid::Uuid;

use crate::http::Cookie;
use crate::http::Request;
use crate::http::Result;
use crate::routing::middleware::Handler;
use crate::routing::middleware::Middleware;

pub struct Session;

#[async_trait]
impl Middleware for Session {
    async fn handle(&self, next: Handler, mut request: Request) -> Result {
        if request.headers().has_cookie("session_uuid") {
            return next(request).await;
        }

        let uuid = Uuid::now_v7();
        let cookie =
            Cookie::<Request>::builder("session_uuid", uuid.as_hyphenated().to_string()).build();

        request.headers_mut().set_cookie(cookie);

        let mut response = next(request).await;

        let cookie = Cookie::builder("session_uuid", uuid.as_hyphenated().to_string())
            .http_only(true)
            .build();

        let raw_response = match &mut response {
            Ok(response) => response,
            Err(response) => response,
        };

        raw_response
            .headers_mut()
            .append("Set-Cookie", cookie.to_string());

        Ok(response?)
    }
}
