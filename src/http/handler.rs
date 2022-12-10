use crate::http::Request;
use crate::http::Response;
use anyhow::Error;
use std::future::Future;
use std::pin::Pin;
use std::result::Result as BaseResult;

/// Determines the result type of an http handler.
pub type Result = BaseResult<Response, Error>;

/// A route handler is an async function that takes
/// a request and returns a response.
pub type Handler<Context> = Box<
    dyn Fn(Request<Context>) -> Pin<Box<dyn Future<Output = Result> + Send + 'static>>
        + Send
        + Sync
        + 'static,
>;
