pub mod client;
pub mod cookie;
pub mod error;
pub mod headers;
pub mod middleware;
pub mod request;
pub mod response;
pub mod server;
pub mod session;

use std::future::Future;
use std::pin::Pin;
use std::result::Result as BaseResult;
use std::sync::Arc;

pub use client::Client;
pub use cookie::Cookie;
pub use cookie::HasCookies;
pub use cookie::RequestCookie;
pub use cookie::ResponseCookie;
pub use error::ErrorResponse;
pub use headers::HasHeaders;
pub use headers::Headers;
pub use http::Method;
pub use http::StatusCode;
pub use http::Uri;
pub use http::Version;
pub use request::FakeRequest;
pub use request::Request;
pub use response::FakeResponse;
pub use response::Response;
pub use server::Server;

use crate::Error;

/// Determines the result type of an http handler.
pub type Result = BaseResult<Response, Error>;

/// A route handler is an async function that takes
/// a request and returns a response.
pub type Handler<App> = Arc<
    dyn Fn(Arc<App>, Request) -> Pin<Box<dyn Future<Output = Result> + Send + 'static>>
        + Send
        + Sync
        + 'static,
>;
