pub mod client;
pub mod context;
pub mod cookie;
pub mod headers;
pub mod middleware;
pub mod request;
pub mod response;
pub mod server;

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

pub use client::Client;
pub use cookie::Cookie;
pub use headers::Headers;
pub use http::Method;
pub use http::StatusCode;
pub use http::Uri;
pub use http::Version;
pub use request::Request;
pub use response::Response;
pub use server::Server;

/// Determines the result type of an http handler.
pub type Result = std::result::Result<Response, Response>;

/// A route handler is an async function that takes
/// a request and returns a response.
pub type Handler<App> = Arc<
    dyn Fn(Request<App>) -> Pin<Box<dyn Future<Output = Result> + Send + 'static>>
        + Send
        + Sync
        + 'static,
>;
