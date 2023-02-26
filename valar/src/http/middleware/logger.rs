use std::sync::Arc;

use async_trait::async_trait;

use crate::http::Request;
use crate::http::Response;
use crate::routing::Middleware;
use crate::Application;

pub struct Logger;

#[async_trait]
impl<App: Application> Middleware<App> for Logger {
    async fn before(&self, _app: Arc<App>, request: &mut Request) -> Option<Response> {
        println!("{request}");

        None
    }

    async fn after(&self, _app: Arc<App>, response: &mut Response) {
        println!("{response}");
    }
}
