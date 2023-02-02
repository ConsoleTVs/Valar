use http::Response as BaseResponse;
use http::Result as HttpResult;
use hyper::Body;
use serde::Serialize;
#[cfg(feature = "json")]
use serde_json::Error as JsonError;
#[cfg(feature = "json")]
use serde_json::Result as JsonResult;

use crate::http::cookies::HasCookies;
use crate::http::headers::HasHeaders;
use crate::http::Headers;
use crate::http::ResponseCookie;
use crate::http::StatusCode;
use crate::http::Version;

/// A response is used to send a response back
/// to the client.
pub struct Response {
    /// The response's status
    status: StatusCode,

    /// The response's version
    version: Version,

    /// The response's headers
    headers: Headers,

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

    /// Returns a response builder with a created status
    /// code.
    pub fn created() -> ResponseBuilder {
        Self::builder().created()
    }

    /// Returns a response builder with a no content status
    /// code.
    pub fn no_content() -> ResponseBuilder {
        Self::builder().no_content()
    }

    /// Returns a response builder with a not found status
    /// code.
    pub fn not_found() -> ResponseBuilder {
        Self::builder().not_found()
    }

    /// Returns the response status code.
    pub fn status(&self) -> &StatusCode {
        &self.status
    }

    /// Returns the response's HTTP version.
    pub fn version(&self) -> &Version {
        &self.version
    }

    /// Returns the response's body.
    pub fn body(&self) -> &str {
        &self.body
    }

    /// Determines if the given header exists.
    pub fn has_header(&self, key: &str) -> bool {
        self.headers().has(key)
    }

    /// Determines if the response contains a JSON value
    /// based on the Content-Type header.
    pub fn is_json(&self) -> bool {
        self.headers().contains("Content-Type", "application/json")
    }

    /// Transforms the response to a hyper Response.
    pub(crate) fn into_base_response(self) -> HttpResult<BaseResponse<Body>> {
        let mut builder = BaseResponse::builder();

        for (header, value) in self.headers {
            builder = builder.header(header, value);
        }

        builder
            .status(self.status)
            .version(self.version)
            .body(Body::from(self.body))
    }
}

impl HasHeaders for Response {
    /// Returns the response's headers.
    fn headers(&self) -> &Headers {
        &self.headers
    }
}

impl HasCookies for Response {
    type Item = ResponseCookie;
}

pub struct ResponseBuilder {
    /// The response's status
    status: StatusCode,

    /// The response's version
    version: Version,

    /// The response's headers
    headers: Headers,

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

    /// Sets a header to the response.
    pub fn header<V>(mut self, header: &str, value: V) -> Self
    where
        V: Into<String>,
    {
        self.headers.insert(header, value.into());

        self
    }

    /// Appends a header to the response.
    pub fn append_header<V>(mut self, header: &str, value: V) -> Self
    where
        V: Into<String>,
    {
        self.headers.append(header, value.into());

        self
    }

    /// Add a cookie to the response.
    pub fn cookie<C>(mut self, cookie: C) -> Self
    where
        C: Into<ResponseCookie>,
    {
        let cookie: ResponseCookie = cookie.into();
        self.headers.append("Set-Header", cookie.to_string());

        self
    }

    /// Set the headers of the request.
    pub fn headers<H>(mut self, headers: H) -> Self
    where
        H: Into<Headers>,
    {
        let headers: Headers = headers.into();
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

    /// Sets the status code to OK.
    pub fn ok(self) -> Self {
        self.status(StatusCode::OK)
    }

    /// Sets the status code to CREATED.
    pub fn created(self) -> Self {
        self.status(StatusCode::CREATED)
    }

    /// Sets the status code to NO CONTENT.
    pub fn no_content(self) -> Self {
        self.status(StatusCode::NO_CONTENT)
    }

    /// Sets the status code to UNAUTHORIZED.
    pub fn unauthorized(mut self, challenges: &str) -> Self {
        self.headers.insert("WWW-Authenticate", challenges);
        self.status = StatusCode::UNAUTHORIZED;

        self
    }

    /// Sets the status code to NOT FOUND.
    pub fn not_found(mut self) -> Self {
        self.status = StatusCode::NOT_FOUND;

        self
    }

    /// Sets the status code to METHOD NOT ALLOWED.
    pub fn method_not_allowed(mut self) -> Self {
        self.status = StatusCode::METHOD_NOT_ALLOWED;

        self
    }

    /// Sets the status code to INTERNAL SERVER ERROR.
    pub fn internal_server_error(mut self) -> Self {
        self.status = StatusCode::INTERNAL_SERVER_ERROR;

        self
    }

    /// Sets the apropiate body and headers for a HTML
    /// response.
    pub fn html<H>(mut self, html: H) -> Self
    where
        H: Into<String>,
    {
        self.headers.insert("Content-Type", "text/html");
        self.body = html.into();

        self
    }

    /// Sets the apropiate headers for a text response.
    pub fn text<T>(mut self, text: T) -> Self
    where
        T: Into<String>,
    {
        self.headers.insert("Content-Type", "text/plain");
        self.body = text.into();

        self
    }

    /// Sets the apropiate body and headers for a JSON
    /// response.
    #[cfg(feature = "json")]
    pub fn json<J>(mut self, json: &J) -> JsonResult<Self>
    where
        J: Serialize,
    {
        self.headers.insert("Content-Type", "application/json");
        self.body = serde_json::to_string(json)?;

        Ok(self)
    }

    #[cfg(feature = "json")]
    pub fn json_or<J>(mut self, json: &J, default: String) -> Self
    where
        J: Serialize,
    {
        self.headers.insert("Content-Type", "application/json");
        self.body = serde_json::to_string(json).unwrap_or(default);

        self
    }

    #[cfg(feature = "json")]
    pub fn json_or_else<J, D>(mut self, json: &J, default: D) -> Self
    where
        J: Serialize,
        D: Fn(JsonError) -> String,
    {
        self.headers.insert("Content-Type", "application/json");
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
            headers: Headers::new(),
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
        assert!(self.0.headers().is(key, value));

        self
    }

    pub fn assert_header_contains(&self, key: &str, value: &str) -> &Self {
        assert!(self.0.headers().contains(key, value));

        self
    }

    pub fn assert_is_json(&self) -> &Self {
        assert!(self.0.is_json());

        self
    }
}
