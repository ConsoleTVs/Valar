use async_trait::async_trait;
use colored::Colorize;

use crate::http::Request;
use crate::http::Response;
use crate::http::Result as HttpResult;
use crate::routing::middleware::Handler;
use crate::routing::middleware::Middleware;

pub struct Logger;

#[async_trait]
impl Middleware for Logger {
    async fn handle(&self, next: Handler, request: Request) -> HttpResult {
        let request_str = request.to_fixed_string();
        let response = next(request).await;

        #[inline(always)]
        fn print(prefix: String, sufix: String) {
            println!("{} {} {}", prefix, "•".dimmed(), sufix)
        }

        match &response {
            Ok(response) => print(request_str, response.to_fixed_string()),
            Err(error) => match error.downcast_ref::<Response>() {
                Some(response) => print(request_str, response.to_fixed_string()),
                None => print(request_str, format!("{}: {}", "Error".red(), error)),
            },
        };

        Ok(response?)
    }
}
