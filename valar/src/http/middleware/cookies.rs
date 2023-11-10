use async_trait::async_trait;

use crate::http::Cookie;
use crate::http::Request;
use crate::http::Response;
use crate::http::Result as HttpResult;
use crate::routing::middleware::Handler;
use crate::routing::middleware::Middleware;

pub struct QueueableCookies;

#[async_trait]
impl<App: Send + Sync + 'static> Middleware<App> for QueueableCookies {
    async fn handle(&self, next: Handler<App>, request: Request<App>) -> HttpResult {
        let context = request.context().clone();
        let cookies: Vec<Cookie<Response>> = Vec::new();
        context.insert("response:queued_cookies", cookies).await;

        let mut response = next(request).await;

        let cookies: Vec<Cookie<Response>> = context
            .remove("response:queued_cookies")
            .await
            .expect("The queued cookies key should exist in the context.");

        let raw_response = match &mut response {
            Ok(response) => response,
            Err(response) => response,
        };

        for cookie in cookies {
            raw_response.headers_mut().set_cookie(cookie);
        }

        response
    }
}
