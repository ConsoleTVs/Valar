pub mod client;
pub mod cookies;
pub mod error;
pub mod headers;
pub mod request;
pub mod response;
pub mod server;
pub mod session;

use std::future::Future;
use std::pin::Pin;
use std::result::Result as BaseResult;
use std::sync::Arc;

pub use client::Client;
pub use cookies::Cookie;
pub use cookies::HasCookies;
pub use cookies::RequestCookie;
pub use cookies::ResponseCookie;
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
pub type Handler<App> = Box<
    dyn Fn(
            Arc<App>,
            Request,
        ) -> Pin<Box<dyn Future<Output = Result> + Send + 'static>>
        + Send
        + Sync
        + 'static,
>;
