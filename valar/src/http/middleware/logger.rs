use async_trait::async_trait;
use colored::Colorize;

use crate::http::Request;
use crate::http::Result as HttpResult;
use crate::routing::middleware::Handler;
use crate::routing::middleware::Middleware;

pub struct Logger;

#[async_trait]
impl<App: Send + Sync + 'static> Middleware<App> for Logger {
    async fn handle(&self, next: Handler<App>, request: Request<App>) -> HttpResult {
        let request_str = request.to_fixed_string();
        let response = next(request).await;

        #[inline(always)]
        fn print(prefix: String, sufix: String) {
            println!("{} {} {}", prefix, "•".dimmed(), sufix)
        }

        match &response {
            Ok(response) => print(request_str, response.to_fixed_string()),
            Err(response) => print(request_str, response.to_fixed_string()),
        };

        Ok(response?)
    }
}
