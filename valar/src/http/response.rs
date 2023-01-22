use crate::http::StatusCode;
use crate::http::Version;
use http::Response as BaseResponse;
use http::Result as HttpResult;
use hyper::Body;
use serde::Serialize;
use std::collections::HashMap;

#[cfg(feature = "json")]
use serde_json::Error as JsonError;

#[cfg(feature = "json")]
use serde_json::Result as JsonResult;

/// A response is used to send a response back
/// to the client.
pub struct Response {
    /// The response's status
    status: StatusCode,

    /// The response's version
    version: Version,

    /// The response's headers
    headers: HashMap<String, String>,

    /// The body of the response.
    body: String,
}

impl Response {
    /// Returns a response builder.
    pub fn builder() -> ResponseBuilder {
        ResponseBuilder::new()
    }

    /// Returns a response builder with an ok status code.
    pub fn ok() -> ResponseBuilder {
        Self::builder().ok()
    }

    /// Returns a response builder with a created status code.
    pub fn created() -> ResponseBuilder {
        Self::builder().created()
    }

    /// Returns a response builder with a no content status code.
    pub fn no_content() -> ResponseBuilder {
        Self::builder().no_content()
    }

    /// Returns a response builder with a not found status code.
    pub fn not_found() -> ResponseBuilder {
        Self::builder().not_found()
    }

    pub fn status(&self) -> &StatusCode {
        &self.status
    }

    pub fn version(&self) -> &Version {
        &self.version
    }

    pub fn headers(&self) -> &HashMap<String, String> {
        &self.headers
    }

    pub fn has_header(&self, key: &str) -> bool {
        self.headers().contains_key(key)
    }

    pub fn header_is(&self, key: &str, value: &str) -> bool {
        self.headers()
            .get(key)
            .map(|key| key == value)
            .unwrap_or(false)
    }

    pub fn header_contains(&self, key: &str, value: &str) -> bool {
        self.headers()
            .get(key)
            .map(|key| key.contains(value))
            .unwrap_or(false)
    }

    pub fn is_json(&self) -> bool {
        self.header_contains("Content-Type", "application/json")
    }

    pub fn body(&self) -> &String {
        &self.body
    }

    pub(crate) fn into_base_response(self) -> HttpResult<BaseResponse<Body>> {
        let mut builder = BaseResponse::builder();

        for (key, value) in &self.headers {
            builder = builder.header(key, value);
        }

        builder
            .status(self.status)
            .version(self.version)
            .body(Body::from(self.body))
    }
}

pub struct ResponseBuilder {
    /// The response's status
    status: StatusCode,

    /// The response's version
    version: Version,

    /// The response's headers
    headers: HashMap<String, String>,

    /// The body of the response.
    body: String,
}

impl ResponseBuilder {
    /// Create a new response builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the status code of the response.
    pub fn status(mut self, status: StatusCode) -> Self {
        self.status = status;

        self
    }

    /// Add a header to the response.
    pub fn header(mut self, key: String, value: String) -> Self {
        self.headers.insert(key, value);

        self
    }

    /// Set the headers of the request.
    pub fn headers(mut self, headers: HashMap<String, String>) -> Self {
        self.headers = headers;

        self
    }

    /// Set the body of the response.
    pub fn body<B>(mut self, body: B) -> Self
    where
        B: Into<String>,
    {
        self.body = body.into();

        self
    }

    pub fn ok(self) -> Self {
        self.status(StatusCode::OK)
    }

    pub fn created(self) -> Self {
        self.status(StatusCode::CREATED)
    }

    pub fn no_content(self) -> Self {
        self.status(StatusCode::NO_CONTENT)
    }

    pub fn unauthorized(mut self, challenges: String) -> Self {
        self.headers
            .insert("WWW-Authenticate".to_string(), challenges);
        self.status = StatusCode::UNAUTHORIZED;

        self
    }

    pub fn not_found(mut self) -> Self {
        self.status = StatusCode::NOT_FOUND;

        self
    }

    pub fn method_not_allowed(mut self) -> Self {
        self.status = StatusCode::METHOD_NOT_ALLOWED;

        self
    }

    pub fn internal_server_error(mut self) -> Self {
        self.status = StatusCode::INTERNAL_SERVER_ERROR;

        self
    }

    /// Sets the apropiate body and headers for a HTML response.
    pub fn html<H>(mut self, html: H) -> Self
    where
        H: Into<String>,
    {
        self.headers
            .insert("Content-Type".to_string(), "text/html".to_string());
        self.body = html.into();

        self
    }

    /// Sets the apropiate headers for a text response.
    pub fn text<T>(mut self, text: T) -> Self
    where
        T: Into<String>,
    {
        self.headers
            .insert("Content-Type".to_string(), "text/plain".to_string());
        self.body = text.into();

        self
    }

    /// Sets the apropiate body and headers for a JSON response.
    #[cfg(feature = "json")]
    pub fn json<J>(mut self, json: &J) -> JsonResult<Self>
    where
        J: Serialize,
    {
        self.headers
            .insert("Content-Type".to_string(), "application/json".to_string());
        self.body = serde_json::to_string(json)?;

        Ok(self)
    }

    #[cfg(feature = "json")]
    pub fn json_or<J>(mut self, json: &J, default: String) -> Self
    where
        J: Serialize,
    {
        self.headers
            .insert("Content-Type".to_string(), "application/json".to_string());
        self.body = serde_json::to_string(json).unwrap_or(default);

        self
    }

    #[cfg(feature = "json")]
    pub fn json_or_else<J, D>(mut self, json: &J, default: D) -> Self
    where
        J: Serialize,
        D: Fn(JsonError) -> String,
    {
        self.headers
            .insert("Content-Type".to_string(), "application/json".to_string());
        self.body = serde_json::to_string(json).unwrap_or_else(default);

        self
    }

    /// Builds the HTTP response.
    pub fn build(self) -> Response {
        Response {
            status: self.status,
            version: self.version,
            headers: self.headers,
            body: self.body,
        }
    }

    /// Produces a handler response from the builder.
    pub fn produce(self) -> Result<Response, anyhow::Error> {
        Ok(self.build())
    }
}

impl From<ResponseBuilder> for Response {
    /// Transforms the builder into a response.
    fn from(builder: ResponseBuilder) -> Self {
        builder.build()
    }
}

impl Default for ResponseBuilder {
    fn default() -> Self {
        Self {
            status: StatusCode::OK,
            version: Version::HTTP_11,
            headers: HashMap::new(),
            body: String::new(),
        }
    }
}

pub struct FakeResponse(Response);

impl FakeResponse {
    pub fn new(response: Response) -> Self {
        Self(response)
    }

    pub fn assert_status(&self, status: &StatusCode) -> &Self {
        assert_eq!(*self.0.status(), *status);

        self
    }

    pub fn assert_ok(&self) -> &Self {
        assert_eq!(*self.0.status(), StatusCode::OK);

        self
    }

    pub fn assert_created(&self) -> &Self {
        assert_eq!(*self.0.status(), StatusCode::CREATED);

        self
    }

    pub fn assert_no_content(&self) -> &Self {
        assert_eq!(*self.0.status(), StatusCode::NO_CONTENT);

        self
    }

    pub fn assert_not_found(&self) -> &Self {
        assert_eq!(*self.0.status(), StatusCode::NOT_FOUND);

        self
    }

    pub fn assert_version(&self, version: &Version) -> &Self {
        assert_eq!(*self.0.version(), *version);

        self
    }

    pub fn assert_has_header(&self, key: &str) -> &Self {
        assert!(self.0.has_header(key));

        self
    }

    pub fn assert_header_is(&self, key: &str, value: &str) -> &Self {
        assert!(self.0.header_is(key, value));

        self
    }

    pub fn assert_header_contains(&self, key: &str, value: &str) -> &Self {
        assert!(self.0.header_contains(key, value));

        self
    }

    pub fn assert_is_json(&self) -> &Self {
        assert!(self.0.is_json());

        self
    }
}
