use std::collections::HashMap;
use std::fmt::Display;
use std::ops::Deref;
use std::str::FromStr;

use colored::Colorize;
use http::Request as BaseRequest;
use http::Result as HttpResult;
use hyper::Body;
use serde::Deserialize;
use serde_json::Result as JsonResult;

use crate::http::cookie::HasCookies;
use crate::http::headers::HasHeaders;
use crate::http::ErrorResponse;
use crate::http::FakeResponse;
use crate::http::Headers;
use crate::http::Method;
use crate::http::RequestCookie;
use crate::http::StatusCode;
use crate::http::Uri;
use crate::http::Version;
use crate::routing::Route;
use crate::Application;
use crate::Error;
use crate::FakeApplication;

/// A request is used to store information about
/// the incoming request.
///
/// Requests are usually found in the handler functions.
///
/// You should usually not build a request manually.
/// Althought it is possible using the provided builder.
pub struct Request {
    /// Stores the full request URI.
    /// The request's method
    method: Method,

    /// The request's URI
    uri: Uri,

    /// The request's version
    version: Version,

    /// The request's headers
    headers: Headers,

    /// The request's body
    body: String,

    /// The URL parameters.
    route_parameters: HashMap<String, String>,

    /// The URI query parameters.
    query_parameters: HashMap<String, String>,

    /// Stores additional request metadata.
    metadata: HashMap<String, String>,
}

impl Request {
    /// Creates a new request using the builder pattern.
    /// This is the preferred way to create a new request.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use valar::http::Method;
    /// use valar::http::Request;
    /// use valar::http::Uri;
    ///
    /// let uri = Uri::from_static("http://localhost:3000/?id=1&name=John");
    ///
    /// let request = Request::builder().method(Method::GET).uri(uri).build();
    ///
    /// assert_eq!(request.method(), &Method::GET);
    /// assert_eq!(request.uri().path(), "/");
    /// assert!(request.has_query("id"));
    /// assert!(request.has_query("name"));
    /// ```
    pub fn builder() -> RequestBuilder {
        RequestBuilder::new()
    }

    /// Returns the method of the HTTP request.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use valar::http::Method;
    /// use valar::http::Request;
    ///
    /// let request = Request::builder().method(Method::GET).build();
    ///
    /// assert_eq!(request.method(), &Method::GET);
    /// ```
    pub fn method(&self) -> &Method {
        &self.method
    }

    /// Returns the URI of the HTTP request.
    /// The URI contains the path and the query string.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use valar::http::Method;
    /// use valar::http::Request;
    /// use valar::http::Uri;
    ///
    /// let uri = Uri::from_static("http://localhost:3000/foo");
    ///
    /// let request = Request::builder().method(Method::GET).uri(uri).build();
    ///
    /// assert_eq!(request.uri().path(), "/foo");
    /// ```
    pub fn uri(&self) -> &Uri {
        &self.uri
    }

    /// Returns the HTTP Version of the HTTP request.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use valar::http::Request;
    /// use valar::http::Version;
    ///
    /// let request = Request::builder().version(Version::HTTP_11).build();
    ///
    /// assert_eq!(request.version(), &Version::HTTP_11);
    /// ```
    pub fn version(&self) -> &Version {
        &self.version
    }

    /// Returns the body of the HTTP request.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use valar::http::Request;
    ///
    /// let request = Request::builder().body("Hello World!").build();
    ///
    /// assert_eq!(request.body(), "Hello World!");
    /// ```
    pub fn body(&self) -> &str {
        &self.body
    }

    pub fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }

    pub fn metadata_mut(&mut self) -> &mut HashMap<String, String> {
        &mut self.metadata
    }

    /// Returns true if the request is considered to have a
    /// JSON body. This is determined by the
    /// "Content-Type" header.
    ///
    /// If the header is not present, this will return
    /// false.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::collections::HashMap;
    ///
    /// use valar::http::Request;
    ///
    /// let request = Request::builder()
    ///     .headers([("Content-Type", "application/json")])
    ///     .body(r#"{"name": "John"}"#.to_string())
    ///     .build();
    ///
    /// assert_eq!(request.is_json(), true);
    /// ```
    pub fn is_json(&self) -> bool {
        self.headers().contains("Content-Type", "application/json")
    }

    /// Returns true if the request is considered to want a
    /// JSON response. This is determined by the
    /// "Accept" header.
    ///
    /// If the header is not present, this will return
    /// false.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::collections::HashMap;
    ///
    /// use valar::http::Request;
    ///
    /// let request = Request::builder()
    ///     .headers([("Content-Type", "application/json")])
    ///     .build();
    ///
    /// assert_eq!(request.wants_json(), true);
    /// ```
    pub fn wants_json(&self) -> bool {
        self.headers().contains("Accept", "application/json")
    }

    /// Returns true is the route parameter is found in the
    /// request.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::collections::HashMap;
    ///
    /// use valar::http::Request;
    ///
    /// let request = Request::builder().route_parameters([("id", "1")]).build();
    ///
    /// assert_eq!(request.has_parameter("id"), true);
    /// assert_eq!(request.has_parameter("name"), false);
    /// ```
    pub fn has_parameter(&self, name: &str) -> bool {
        self.route_parameters.contains_key(name)
    }

    /// Gets the given route parameter from the current
    /// route.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::collections::HashMap;
    ///
    /// use valar::http::Request;
    ///
    /// let request = Request::builder().route_parameters([("id", "1")]).build();
    ///
    /// assert_eq!(request.maybe_parameter("id").unwrap(), "1");
    /// assert_eq!(request.maybe_parameter("name"), None);
    /// ```
    pub fn maybe_parameter(&self, name: &str) -> Option<&str> {
        self.route_parameters.get(name).map(|name| name.deref())
    }

    /// Gets the given route parameter from
    /// the current route or returns an error.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::collections::HashMap;
    ///
    /// use valar::http::Request;
    ///
    /// let request = Request::builder().route_parameters([("id", "1")]).build();
    ///
    /// assert_eq!(request.route_parameter("id").unwrap(), "1");
    /// assert!(request.route_parameter("name").is_err());
    /// ```
    pub fn route_parameter(&self, name: &str) -> Result<&str, ErrorResponse> {
        self.maybe_parameter(name).ok_or_else(|| {
            ErrorResponse::new()
                .message(format!("Unknown route parameter: `{}`", name))
                .status(StatusCode::INTERNAL_SERVER_ERROR)
        })
    }

    /// Gets the given route parameter from the current
    /// route and parses it to the given type.
    /// This is useful for parsing route parameters to
    /// integers or other types.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::collections::HashMap;
    ///
    /// use valar::http::Request;
    ///
    /// let request = Request::builder().route_parameters([("id", "1")]).build();
    ///
    /// let id: u32 = request.parameter("id").unwrap();
    ///
    /// assert_eq!(id, 1);
    /// assert!(request.parameter::<u32>("name").is_err());
    /// ```
    pub fn parameter<T>(&self, name: &str) -> Result<T, Error>
    where
        T: FromStr,
        T::Err: std::error::Error + Sync + Send + 'static,
    {
        let param: T = self.route_parameter(name)?.parse()?;

        Ok(param)
    }

    /// Returns the query parameters from the request.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use valar::http::Request;
    /// use valar::http::Uri;
    ///
    /// let uri = Uri::from_static("http://localhost:3000/?id=1&name=John");
    ///
    /// let request = Request::builder().uri(uri).build();
    ///
    /// assert_eq!(request.query_parameters().get("id").unwrap(), "1");
    /// assert_eq!(request.query_parameters().get("name").unwrap(), "John");
    /// ```
    pub fn query_parameters(&self) -> &HashMap<String, String> {
        &self.query_parameters
    }

    /// Creaates the query parameters from the request's
    /// URI. This is used internally to create the query
    /// parameters from the request's URI.
    pub(crate) fn query_parameters_from(value: &Uri) -> HashMap<String, String> {
        match value.path_and_query() {
            Some(query) => query
                .as_str()
                .trim_start_matches('/')
                .trim_start_matches('?')
                .split('&')
                .filter_map(|pair| {
                    let mut pair = pair.split('=');

                    let key = pair.next()?;
                    let value = pair.next()?;

                    Some((key.to_string(), value.to_string()))
                })
                .collect(),
            None => HashMap::default(),
        }
    }

    /// Checks if the request has the given query parameter.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use valar::http::Request;
    /// use valar::http::Uri;
    ///
    /// let uri = Uri::from_static("http://localhost:3000/?id=1&name=John");
    ///
    /// let request = Request::builder().uri(uri).build();
    ///
    /// assert!(request.has_query("id"));
    /// assert!(request.has_query("name"));
    /// assert!(!request.has_query("age"));
    /// ```
    pub fn has_query(&self, name: &str) -> bool {
        self.query_parameters.contains_key(name)
    }

    /// Gets the given query parameter from the request's
    /// URI. If the query parameter does not exist,
    /// `None` is returned.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use valar::http::Request;
    /// use valar::http::Uri;
    ///
    /// let uri = Uri::from_static("http://localhost:3000/?id=1&name=John");
    ///
    /// let request = Request::builder().uri(uri).build();
    ///
    /// assert_eq!(request.maybe_query("id").unwrap(), "1");
    /// assert_eq!(request.maybe_query("name").unwrap(), "John");
    /// assert!(request.maybe_query("age").is_none());
    /// ```
    pub fn maybe_query(&self, name: &str) -> Option<&str> {
        self.query_parameters.get(name).map(|query| query.deref())
    }

    /// Gets the given query parameter from the request's
    /// URI. If the query parameter does not exist, an
    /// error is returned.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use valar::http::Request;
    /// use valar::http::Uri;
    ///
    /// let uri = Uri::from_static("http://localhost:3000/?id=1&name=John");
    ///
    /// let request = Request::builder().uri(uri).build();
    ///
    /// assert_eq!(request.query_parameter("id").unwrap(), "1");
    /// assert_eq!(request.query_parameter("name").unwrap(), "John");
    /// assert!(request.query_parameter("age").is_err());
    /// ```
    pub fn query_parameter(&self, name: &str) -> Result<&str, ErrorResponse> {
        self.maybe_query(name).ok_or_else(|| {
            ErrorResponse::new()
                .message(format!("Unknown query parameter: `{}`", name))
                .status(StatusCode::INTERNAL_SERVER_ERROR)
        })
    }

    /// Gets the given query parameter from the request's
    /// URI. If the query parameter does not exist, an
    /// error is returned. The query parameter is then
    /// parsed to the given type. If the query parameter
    /// cannot be parsed, an error is returned.
    /// This method is a shorthand for `query_parameter` and
    /// `parse`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use valar::http::Request;
    /// use valar::http::Uri;
    ///
    /// let uri = Uri::from_static("http://localhost:3000/?id=1&name=John");
    ///
    /// let request = Request::builder().uri(uri).build();
    ///
    /// let id: u32 = request.query("id").unwrap();
    /// let name: String = request.query("name").unwrap();
    ///
    /// assert_eq!(id, 1);
    /// assert_eq!(name, "John");
    /// ```
    pub fn query<T>(&self, name: &str) -> Result<T, Error>
    where
        T: FromStr,
        T::Err: std::error::Error + Sync + Send + 'static,
    {
        let result: T = self.query_parameter(name)?.parse()?;

        Ok(result)
    }

    /// Transforms the body of the request to the given
    /// deserializable type.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use serde::Deserialize;
    /// use valar::http::Request;
    ///
    /// #[derive(Deserialize)]
    /// struct User {
    ///     name: String,
    /// }
    ///
    /// let request = Request::builder().body(r#"{"name": "John"}"#).build();
    ///
    /// let user: User = request.json().unwrap();
    ///
    /// assert_eq!(user.name, "John");
    /// ```
    pub fn json<'a, T>(&'a self) -> JsonResult<T>
    where
        T: Deserialize<'a>,
    {
        serde_json::from_str(&self.body)
    }

    pub fn parematrized<App: Application>(mut self, route: &Route<App>) -> Self {
        self.route_parameters = route.parameters(self.uri());

        self
    }
}

impl Display for Request {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let method = self.method().as_str();
        let path = self.uri().path().bold();
        let query = self.uri().query().unwrap_or_default();

        write!(f, "⮕  {method} {path} {query}")
    }
}

impl HasHeaders for Request {
    /// Returns the headers of the HTTP request as a
    /// HashMap. The keys are the header names and the
    /// values are the header values.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::collections::HashMap;
    ///
    /// use valar::http::HasHeaders;
    /// use valar::http::Request;
    ///
    /// let request = Request::builder()
    ///     .headers([("Content-Type", "application/json")])
    ///     .build();
    ///
    /// assert!(request.headers().is("Content-Type", "application/json"));
    /// ```
    fn headers(&self) -> &Headers {
        &self.headers
    }
}

impl HasCookies for Request {
    type Item = RequestCookie;
}

#[derive(Default)]
pub struct RequestBuilder {
    method: Method,
    uri: Uri,
    version: Version,
    headers: Headers,
    body: String,
    route_parameters: HashMap<String, String>,
    metadata: HashMap<String, String>,
}

impl RequestBuilder {
    pub fn new() -> RequestBuilder {
        RequestBuilder::default()
    }

    pub fn method(mut self, method: Method) -> Self {
        self.method = method;

        self
    }

    pub fn cookie<C>(self, cookie: C) -> Self
    where
        C: Into<RequestCookie>,
    {
        let cookie: RequestCookie = cookie.into();

        self.header("Cookie", cookie.to_string())
    }

    pub fn uri(mut self, uri: Uri) -> Self {
        self.uri = uri;

        self
    }

    pub fn metadata<M>(mut self, metadata: M) -> Self
    where
        M: Into<HashMap<String, String>>,
    {
        self.metadata = metadata.into();

        self
    }

    pub fn meta(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());

        self
    }

    pub fn uri_str(mut self, uri: &str) -> Result<Self, <http::Uri as FromStr>::Err> {
        self.uri = Uri::from_str(uri)?;

        Ok(self)
    }

    pub fn version(mut self, version: Version) -> Self {
        self.version = version;

        self
    }

    pub fn header<H, V>(mut self, header: H, value: V) -> Self
    where
        H: Into<String>,
        V: Into<String>,
    {
        self.headers.append(header, value);

        self
    }

    pub fn headers<H>(mut self, headers: H) -> Self
    where
        H: Into<Headers>,
    {
        let headers: Headers = headers.into();
        self.headers = headers;

        self
    }

    pub fn body<T>(mut self, body: T) -> Self
    where
        T: Into<String>,
    {
        self.body = body.into();

        self
    }

    pub fn route_parameters<P, T>(mut self, parameters: P) -> Self
    where
        P: Into<HashMap<T, T>>,
        T: Into<String>,
    {
        let parameters: HashMap<T, T> = parameters.into();

        self.route_parameters = parameters
            .into_iter()
            .map(|(key, value)| (key.into(), value.into()))
            .collect();

        self
    }

    pub fn build(self) -> Request {
        Request {
            route_parameters: self.route_parameters,
            query_parameters: Request::query_parameters_from(&self.uri),
            method: self.method,
            uri: self.uri,
            version: self.version,
            headers: self.headers,
            body: self.body,
            metadata: self.metadata,
        }
    }
}

pub struct FakeRequest<'a, App: Application + Send + Sync + 'static> {
    app: &'a FakeApplication<App>,
    method: Method,
    uri: Uri,
    version: Version,
    headers: Headers,
    body: String,
}

impl<'a, App: Application + Send + Sync + 'static> FakeRequest<'a, App> {
    pub fn new(app: &'a FakeApplication<App>) -> FakeRequest<'a, App> {
        FakeRequest {
            app,
            method: Method::default(),
            uri: Uri::default(),
            version: Version::default(),
            headers: Headers::default(),
            body: String::default(),
        }
    }

    pub fn method(mut self, method: Method) -> Self {
        self.method = method;

        self
    }

    pub fn uri(mut self, uri: Uri) -> Self {
        self.uri = uri;

        self
    }

    pub fn path(mut self, path: &str) -> Self {
        self.uri = Uri::builder()
            .path_and_query(path)
            .build()
            .unwrap_or_default();

        self
    }

    pub fn version(mut self, version: Version) -> Self {
        self.version = version;

        self
    }

    pub fn headers<H>(mut self, headers: H) -> Self
    where
        H: Into<Headers>,
    {
        self.headers = headers.into();

        self
    }

    pub fn header<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.headers.append(key.into(), value.into());

        self
    }

    pub fn body<V>(mut self, value: V) -> Self
    where
        V: Into<String>,
    {
        self.body = value.into();

        self
    }

    fn into_base_request(self) -> HttpResult<BaseRequest<Body>> {
        let mut request = BaseRequest::builder()
            .method(self.method.clone())
            .uri(self.uri.clone());

        for (key, value) in self.headers {
            request = request.header(key, value);
        }

        request.body(Body::from(self.body.clone()))
    }

    pub async fn send(self) -> FakeResponse {
        let router = self.app.router();
        let app = self.app.app_arc();
        let request = self.into_base_request().unwrap_or_default();
        let response = router.handle(app, request).await;

        FakeResponse::new(response)
    }
}
