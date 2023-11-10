use std::error::Error;
use std::fmt::Display;

use colored::Colorize;
use http::Response as BaseResponse;
use http::Result as BaseHttpResult;
use serde::Serialize;
use serde_json::Error as JsonError;
use serde_json::Result as JsonResult;

use crate::http::Cookie;
use crate::http::Headers;
use crate::http::Request;
use crate::http::Result as HttpResult;
use crate::http::StatusCode;
use crate::http::Version;
use crate::utils::TruncatableToFit;

/// A response is used to send a response back
/// to the client.
#[derive(Debug)]
pub struct Response {
    status: StatusCode,
    version: Version,
    headers: Headers<Self>,
    body: String,
}

impl Response {
    pub fn to_fixed_string(&self) -> String {
        let code = self.status().as_u16();

        let code_str = self
            .status()
            .canonical_reason()
            .unwrap_or("Unknown")
            .trim()
            .truncate_to_fit(11)
            .bold();

        let code = match code {
            100..=199 => code.to_string().truncate_to_fit(3).cyan(),
            200..=299 => code.to_string().truncate_to_fit(3).green(),
            300..=399 => code.to_string().truncate_to_fit(3).yellow(),
            400..=599 => code.to_string().truncate_to_fit(3).red(),
            _ => code.to_string().truncate_to_fit(3).white(),
        };

        format!("{code:.<3} {code_str:.<11}")
    }

    /// Returns a response builder.
    pub fn builder() -> ResponseBuilder {
        ResponseBuilder::new()
    }

    /// Returns a response builder with an ok status code.
    pub fn ok() -> ResponseBuilder {
        Self::builder().ok()
    }

    pub fn redirect<P>(location: P) -> ResponseBuilder
    where
        P: Into<String>,
    {
        Self::builder().redirect(location)
    }

    pub fn temporary_redirect<P>(location: P) -> ResponseBuilder
    where
        P: Into<String>,
    {
        Self::builder().temporary_redirect(location)
    }

    pub fn permanent_redirect<P>(location: P) -> ResponseBuilder
    where
        P: Into<String>,
    {
        Self::builder().permanent_redirect(location)
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

    /// Returns a response builder with a not found status
    /// code.
    pub fn internal_server_error() -> ResponseBuilder {
        Self::builder().internal_server_error()
    }

    pub fn payload_too_large() -> ResponseBuilder {
        Self::builder().payload_too_large()
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

    /// Returns the headers of the request.
    pub fn headers(&self) -> &Headers<Self> {
        &self.headers
    }

    /// Returns a mutable reference to the headers of the
    /// request.
    pub fn headers_mut(&mut self) -> &mut Headers<Self> {
        &mut self.headers
    }

    /// Determines if the response contains a JSON value
    /// based on the Content-Type header.
    pub fn is_json(&self) -> bool {
        self.headers().contains("Content-Type", "application/json")
    }

    pub fn assert_status(&self, status: &StatusCode) -> &Self {
        assert_eq!(*self.status(), *status);

        self
    }

    pub fn assert_ok(&self) -> &Self {
        assert_eq!(*self.status(), StatusCode::OK);

        self
    }

    pub fn assert_created(&self) -> &Self {
        assert_eq!(*self.status(), StatusCode::CREATED);

        self
    }

    pub fn assert_no_content(&self) -> &Self {
        assert_eq!(*self.status(), StatusCode::NO_CONTENT);

        self
    }

    pub fn assert_not_found(&self) -> &Self {
        assert_eq!(*self.status(), StatusCode::NOT_FOUND);

        self
    }

    pub fn assert_version(&self, version: &Version) -> &Self {
        assert_eq!(*self.version(), *version);

        self
    }

    pub fn assert_has_header(&self, key: &str) -> &Self {
        assert!(self.headers().has(key));

        self
    }

    pub fn assert_header_is(&self, key: &str, value: &str) -> &Self {
        assert!(self.headers().is(key, value));

        self
    }

    pub fn assert_header_contains(&self, key: &str, value: &str) -> &Self {
        assert!(self.headers().contains(key, value));

        self
    }

    pub fn assert_is_json(&self) -> &Self {
        assert!(self.is_json());

        self
    }

    /// Transforms the response to a hyper Response.
    pub(crate) fn into_base_response(self) -> BaseHttpResult<BaseResponse<Body>> {
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

// impl Error for Response {}

impl<E> From<E> for Response
where
    E: Error + Send + Sync + 'static,
{
    fn from(err: E) -> Self {
        Self::internal_server_error().body(err.to_string()).build()
    }
}

impl Display for Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let code = self.status().as_u16();

        let code_str = self
            .status()
            .canonical_reason()
            .unwrap_or("Unknown")
            .trim()
            .bold();

        let code = match code {
            100..=199 => code.to_string().cyan(),
            200..=299 => code.to_string().green(),
            300..=399 => code.to_string().yellow(),
            400..=599 => code.to_string().red(),
            _ => code.to_string().white(),
        };

        write!(f, "{code} {code_str}")
    }
}

pub enum ResponseMessage {
    Text(String),
    Canonical,
}

pub struct ResponseBuilder {
    status: StatusCode,
    version: Version,
    headers: Headers<Response>,
    body: Option<String>,
    message: Option<ResponseMessage>,
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
    pub fn header<H, V>(mut self, header: H, value: V) -> Self
    where
        H: Into<String>,
        V: Into<String>,
    {
        self.headers.insert(header, value.into());

        self
    }

    /// Appends a header to the response.
    pub fn append_header<H, V>(mut self, header: H, value: V) -> Self
    where
        H: Into<String>,
        V: Into<String>,
    {
        self.headers.append(header, value.into());

        self
    }

    /// Add a cookie to the response.
    pub fn cookie<C>(mut self, cookie: C) -> Self
    where
        C: Into<Cookie<Response>>,
    {
        let cookie: Cookie<Response> = cookie.into();
        self.headers.set_cookie(cookie);

        self
    }

    /// Set the headers of the request.
    pub fn headers<H>(mut self, headers: H) -> Self
    where
        H: Into<Headers<Response>>,
    {
        self.headers = headers.into();

        self
    }

    pub fn headers_iter<H, N, V>(mut self, headers: H) -> Self
    where
        H: IntoIterator<Item = (N, V)>,
        N: Into<String>,
        V: Into<String>,
    {
        self.headers = Headers::from_iter(headers);

        self
    }

    /// Set the body of the response.
    pub fn body<B>(mut self, body: B) -> Self
    where
        B: Into<String>,
    {
        self.body = Some(body.into());

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

    /// Sets the status code to INTERNAL SERVER ERROR.
    pub fn payload_too_large(mut self) -> Self {
        self.status = StatusCode::PAYLOAD_TOO_LARGE;

        self
    }

    pub fn see_other<L>(mut self, location: L) -> Self
    where
        L: Into<String>,
    {
        self.headers.insert("Location", location);
        self.status = StatusCode::SEE_OTHER;

        self
    }

    /// Alias to see_other. Use when the redirect is a
    /// direct response to a POST / PUT action where the
    /// request is intended to go.
    pub fn redirect<L>(self, location: L) -> Self
    where
        L: Into<String>,
    {
        self.see_other(location)
    }

    /// Temporary redirect. Use when the resource is now
    /// located at a different URI temporarily.
    pub fn temporary_redirect<L>(mut self, location: L) -> Self
    where
        L: Into<String>,
    {
        self.headers.insert("Location", location);
        self.status = StatusCode::TEMPORARY_REDIRECT;

        self
    }

    /// Permanent redirect. Use when the resource is now
    /// located at a different URI permanently.
    pub fn permanent_redirect<L>(mut self, location: L) -> Self
    where
        L: Into<String>,
    {
        self.headers.insert("Location", location);
        self.status = StatusCode::PERMANENT_REDIRECT;

        self
    }

    pub fn message<M>(mut self, message: M) -> Self
    where
        M: Into<String>,
    {
        self.message = Some(ResponseMessage::Text(message.into()));

        self
    }

    pub fn with_canonical_message(mut self) -> Self {
        self.message = Some(ResponseMessage::Canonical);

        self
    }

    /// Sets the apropiate body and headers for a HTML
    /// response.
    pub fn html<H>(mut self, html: H) -> Self
    where
        H: Into<String>,
    {
        self.headers.insert("Content-Type", "text/html");

        self.body(html.into())
    }

    /// Sets the apropiate headers for a text response.
    pub fn text<T>(mut self, text: T) -> Self
    where
        T: Into<String>,
    {
        self.headers.insert("Content-Type", "text/plain");

        self.body(text.into())
    }

    /// Sets the apropiate body and headers for a JSON
    /// response.
    pub fn json<J>(mut self, json: &J) -> JsonResult<Self>
    where
        J: Serialize,
    {
        self.headers.insert("Content-Type", "application/json");
        self = self.body(serde_json::to_string(json)?);

        Ok(self)
    }

    pub fn json_or<J>(mut self, json: &J, default: String) -> Self
    where
        J: Serialize,
    {
        self.headers.insert("Content-Type", "application/json");

        self.body(serde_json::to_string(json).unwrap_or(default))
    }

    pub fn json_or_else<J, D>(mut self, json: &J, default: D) -> Self
    where
        J: Serialize,
        D: Fn(JsonError) -> String,
    {
        self.headers.insert("Content-Type", "application/json");

        self.body(serde_json::to_string(json).unwrap_or_else(default))
    }

    pub fn content_type<V>(self, value: V) -> Self
    where
        V: Into<String>,
    {
        self.header("Content-Type", value)
    }

    pub fn json_content_type(self) -> Self {
        self.content_type("application/json")
    }

    pub fn match_content_type<App: Send + Sync + 'static>(self, request: &Request<App>) -> Self {
        match request.headers().first("Accept") {
            Some(header) => self.content_type(header),
            None => self,
        }
    }

    /// Builds the HTTP response.
    pub fn build(self) -> Response {
        let body = match (self.body, self.message) {
            (Some(body), _) => body,
            (None, None) => String::new(),
            (None, Some(message)) => {
                let message = match message {
                    ResponseMessage::Text(message) => message,
                    ResponseMessage::Canonical => self
                        .status
                        .canonical_reason()
                        .unwrap_or("An unknown error occurred")
                        .trim()
                        .to_string(),
                };

                message

                // TODO: Make this based on content type?

                // match self.headers.contains("
                // Content-Type", "application/json") {
                //     true => format!(r#"{{ "message":
                // "{message}" }}"#),
                //     false => message,
                // }
            }
        };

        Response {
            status: self.status,
            version: self.version,
            headers: self.headers,
            body,
        }
    }

    /// Produces a handler response from the builder.
    pub fn into_ok(self) -> HttpResult {
        Ok(self.build())
    }

    /// Produces a handler response from the builder.
    pub fn into_err(self) -> HttpResult {
        Err(self.build())
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
            headers: Headers::default(),
            body: None,
            message: None,
        }
    }
}
