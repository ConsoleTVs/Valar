use crate::core::Application;
use crate::http::ErrorResponse;
use crate::http::Method;
use crate::http::StatusCode;
use crate::http::Uri;
use crate::http::Version;
use hyper::Body;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;

#[cfg(feature = "json")]
use serde_json::Result as JsonResult;

/// A request is used to store information about
/// the incoming request.
pub struct Request<Context: Sync + Send + 'static> {
    /// Stores the application.
    pub(crate) application: Arc<Application>,

    /// Stores the application's shared state.
    pub(crate) context: Arc<Context>,

    /// Stores the full request URI.
    /// The request's method
    pub(crate) method: Method,

    /// The request's URI
    pub(crate) uri: Uri,

    /// The request's version
    pub(crate) version: Version,

    /// The request's headers
    pub(crate) headers: HashMap<String, String>,

    /// The request's body
    pub(crate) body: Body,

    /// The URL parameters.
    pub(crate) route_parameters: HashMap<String, String>,
}

impl<Context: Sync + Send + 'static> Request<Context> {
    /// Gets the application shared state.
    pub fn application(&self) -> &Application {
        self.application.as_ref()
    }

    /// Gets the application shared state.
    pub fn context(&self) -> &Context {
        self.context.as_ref()
    }

    pub fn method(&self) -> &Method {
        &self.method
    }

    pub fn uri(&self) -> &Uri {
        &self.uri
    }

    pub fn version(&self) -> &Version {
        &self.version
    }

    pub fn headers(&self) -> &HashMap<String, String> {
        &self.headers
    }

    pub fn body(&self) -> &Body {
        &self.body
    }

    /// Gets the given route parameter from
    /// the current route.
    pub fn maybe_parameter(&self, name: &str) -> Option<&String> {
        self.route_parameters.get(name)
    }

    /// Gets the given route parameter from
    /// the current route or returns an error.
    pub fn parameter(&self, name: &str) -> Result<&String, ErrorResponse> {
        self.maybe_parameter(name).ok_or_else(|| {
            ErrorResponse::new()
                .message(format!("Unknown route parameter: `{}`", name))
                .status(StatusCode::INTERNAL_SERVER_ERROR)
        })
    }

    #[cfg(feature = "json")]
    pub fn is_json(&self) -> bool {
        match self.headers.get("Content-Type") {
            Some(content_type) => content_type == "application/json",
            None => false,
        }
    }

    #[cfg(feature = "json")]
    pub fn wants_json(&self) -> bool {
        match self.headers.get("Accept") {
            Some(accept) => accept == "application/json",
            None => false,
        }
    }

    /// Transforms the body of the request to the given
    /// deserializable type.
    #[cfg(feature = "json")]
    pub fn json<'a, T: Deserialize<'a>>(&'a self) -> JsonResult<T> {
        // serde_json::from_str(&self.body)
        todo!()
    }
}
