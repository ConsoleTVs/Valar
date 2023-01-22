use crate::http::ErrorResponse;
use crate::http::FakeResponse;
use crate::http::Method;
use crate::http::StatusCode;
use crate::http::Uri;
use crate::http::Version;
use crate::Application;
use crate::Error;
use crate::FakeApplication;
use http::Request as BaseRequest;
use http::Result as HttpResult;
use hyper::Body;
use serde::Deserialize;
use std::collections::HashMap;
use std::str::FromStr;

#[cfg(feature = "json")]
use serde_json::Result as JsonResult;

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
    pub(crate) method: Method,

    /// The request's URI
    pub(crate) uri: Uri,

    /// The request's version
    pub(crate) version: Version,

    /// The request's headers
    pub(crate) headers: HashMap<String, String>,

    /// The request's body
    pub(crate) body: String,

    /// The URL parameters.
    pub(crate) route_parameters: HashMap<String, String>,

    /// The URI query parameters.
    pub(crate) query_parameters: HashMap<String, String>,
}

impl Request {
    /// Creates a new request using the builder pattern.
    /// This is the preferred way to create a new request.
    ///
    /// # Example
    ///
    /// ```
    /// use valar::http::Request;
    /// use valar::http::Method;
    /// use valar::http::Uri;
    ///
    /// let uri = Uri::from_static("http://localhost:3000/?id=1&name=John");
    ///
    /// let request = Request::builder()
    ///     .method(Method::GET)
    ///     .uri(uri)
    ///     .build();
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
    /// ```
    /// use valar::http::Request;
    /// use valar::http::Method;
    ///
    /// let request = Request::builder()
    ///     .method(Method::GET)
    ///     .build();
    ///
    /// assert_eq!(request.method(), &Method::GET);
    /// ```
    ///
    pub fn method(&self) -> &Method {
        &self.method
    }

    /// Returns the URI of the HTTP request.
    /// The URI contains the path and the query string.
    ///
    /// # Example
    ///
    /// ```
    /// use valar::http::Request;
    /// use valar::http::Method;
    /// use valar::http::Uri;
    ///
    /// let uri = Uri::from_static("http://localhost:3000/foo");
    ///
    /// let request = Request::builder()
    ///     .method(Method::GET)
    ///     .uri(uri)
    ///     .build();
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
    /// ```
    /// use valar::http::Request;
    /// use valar::http::Version;
    ///
    /// let request = Request::builder()
    ///     .version(Version::HTTP_11)
    ///     .build();
    ///
    /// assert_eq!(request.version(), &Version::HTTP_11);
    /// ```
    pub fn version(&self) -> &Version {
        &self.version
    }

    /// Returns the headers of the HTTP request as a HashMap.
    /// The keys are the header names and the values are the header values.
    ///
    /// # Example
    ///
    /// ```
    /// use valar::http::Request;
    /// use std::collections::HashMap;
    ///
    /// let headers = HashMap::from([
    ///     ("Content-Type".to_string(), "application/json".to_string())
    /// ]);
    ///
    /// let request = Request::builder()
    ///     .headers(headers)
    ///     .build();
    ///
    /// assert_eq!(request.headers().get("Content-Type").unwrap(), &"application/json".to_string());
    /// ```
    pub fn headers(&self) -> &HashMap<String, String> {
        &self.headers
    }

    /// Returns the body of the HTTP request.
    ///
    /// # Example
    /// ```
    /// use valar::http::Request;
    ///
    /// let request = Request::builder()
    ///     .body("Hello World!")
    ///     .build();
    ///
    /// assert_eq!(request.body(), "Hello World!");
    /// ```
    pub fn body(&self) -> &String {
        &self.body
    }

    /// Determines if the request has the given header.
    ///
    /// # Example
    ///
    /// ```
    /// use valar::http::Request;
    /// use std::collections::HashMap;
    ///
    /// let headers = HashMap::from([
    ///     ("Content-Type".to_string(), "application/json".to_string())
    /// ]);
    ///
    /// let request = Request::builder()
    ///     .headers(headers)
    ///     .build();
    ///
    /// assert_eq!(request.has_header("Content-Type"), true);
    /// ```
    pub fn has_header(&self, key: &str) -> bool {
        self.headers().contains_key(key)
    }

    /// Returns true if the request has the value in the given header.
    ///
    /// # Example
    ///
    /// ```
    /// use valar::http::Request;
    /// use std::collections::HashMap;
    ///
    /// let headers = HashMap::from([
    ///     ("Content-Type".to_string(), "application/json".to_string())
    /// ]);
    ///
    /// let request = Request::builder()
    ///     .headers(headers)
    ///     .build();
    ///
    /// assert_eq!(request.header_is("Content-Type", "application/json"), true);
    /// ```
    pub fn header_is(&self, key: &str, value: &str) -> bool {
        self.headers().get(key).map_or(false, |key| key == value)
    }

    /// Returns true if the request contains the value in the given header.
    /// This is useful for checking if the header contains a specific
    /// subset of a string.
    /// For example, if you want to check if the header contains the
    /// character set "utf-8".
    ///
    /// # Example
    /// ```
    /// use valar::http::Request;
    /// use std::collections::HashMap;
    ///
    /// let headers = HashMap::from([
    ///     ("Content-Type".to_string(), "application/json; charset=utf-8".to_string())
    /// ]);
    ///
    /// let request = Request::builder()
    ///     .headers(headers)
    ///     .build();
    ///
    /// assert_eq!(request.header_contains("Content-Type", "charset=utf-8"), true);
    /// ```
    pub fn header_contains(&self, key: &str, value: &str) -> bool {
        self.headers()
            .get(key)
            .map_or(false, |key| key.contains(value))
    }

    /// Returns true if the request is considered to have a JSON body.
    /// This is determined by the "Content-Type" header.
    ///
    /// If the header is not present, this will return false.
    ///
    /// # Example
    /// ```
    /// use valar::http::Request;
    /// use std::collections::HashMap;
    ///
    /// let headers = HashMap::from([
    ///     ("Content-Type".to_string(), "application/json".to_string())
    /// ]);
    ///
    /// let request = Request::builder()
    ///     .headers(headers)
    ///     .body(r#"{"name": "John"}"#.to_string())
    ///     .build();
    ///
    /// assert_eq!(request.is_json(), true);
    /// ```
    pub fn is_json(&self) -> bool {
        self.header_contains("Content-Type", "application/json")
    }

    /// Returns true if the request is considered to want a JSON response.
    /// This is determined by the "Accept" header.
    ///
    /// If the header is not present, this will return false.
    ///
    /// # Example
    /// ```
    /// use valar::http::Request;
    /// use std::collections::HashMap;
    ///
    /// let headers = HashMap::from([
    ///     ("Accept".to_string(), "application/json".to_string())
    /// ]);
    ///
    /// let request = Request::builder()
    ///     .headers(headers)
    ///     .build();
    ///
    /// assert_eq!(request.wants_json(), true);
    /// ```
    pub fn wants_json(&self) -> bool {
        self.header_contains("Accept", "application/json")
    }

    /// Returns true is the route parameter is found in the request.
    ///
    /// # Example
    /// ```
    /// use valar::http::Request;
    /// use std::collections::HashMap;
    ///
    /// let parameters = HashMap::from([
    ///     ("id".to_string(), "1".to_string())
    /// ]);
    ///
    /// let request = Request::builder()
    ///     .route_parameters(parameters)
    ///     .build();
    ///
    /// assert_eq!(request.has_parameter("id"), true);
    /// assert_eq!(request.has_parameter("name"), false);
    /// ```
    pub fn has_parameter(&self, name: &str) -> bool {
        self.route_parameters.contains_key(name)
    }

    /// Gets the given route parameter from the current route.
    ///
    /// # Example
    /// ```
    /// use valar::http::Request;
    /// use std::collections::HashMap;
    ///
    /// let parameters = HashMap::from([
    ///     ("id".to_string(), "1".to_string())
    /// ]);
    ///
    /// let request = Request::builder()
    ///     .route_parameters(parameters)
    ///     .build();
    ///
    /// assert_eq!(request.maybe_parameter("id").unwrap(), &"1".to_string());
    /// assert_eq!(request.maybe_parameter("name"), None);
    /// ```
    pub fn maybe_parameter(&self, name: &str) -> Option<&String> {
        self.route_parameters.get(name)
    }

    /// Gets the given route parameter from
    /// the current route or returns an error.
    ///
    /// # Example
    /// ```
    /// use valar::http::Request;
    /// use std::collections::HashMap;
    ///
    /// let parameters = HashMap::from([
    ///     ("id".to_string(), "1".to_string())
    /// ]);
    ///
    /// let request = Request::builder()
    ///     .route_parameters(parameters)
    ///     .build();
    ///
    /// assert_eq!(request.route_parameter("id").unwrap(), &"1".to_string());
    /// assert!(request.route_parameter("name").is_err());
    /// ```
    pub fn route_parameter(&self, name: &str) -> Result<&String, ErrorResponse> {
        self.maybe_parameter(name).ok_or_else(|| {
            ErrorResponse::new()
                .message(format!("Unknown route parameter: `{}`", name))
                .status(StatusCode::INTERNAL_SERVER_ERROR)
        })
    }

    /// Gets the given route parameter from the current route
    /// and parses it to the given type.
    /// This is useful for parsing route parameters to integers
    /// or other types.
    ///
    /// # Example
    /// ```
    /// use valar::http::Request;
    /// use std::collections::HashMap;
    ///
    /// let parameters = HashMap::from([
    ///     ("id".to_string(), "1".to_string())
    /// ]);
    ///
    /// let request = Request::builder()
    ///     .route_parameters(parameters)
    ///     .build();
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
    /// ```
    /// use valar::http::Request;
    /// use valar::http::Uri;
    ///
    /// let uri = Uri::from_static("http://localhost:3000/?id=1&name=John");
    ///
    /// let request = Request::builder()
    ///     .uri(uri)
    ///     .build();
    ///
    /// assert_eq!(request.query_parameters().get("id").unwrap(), "1");
    /// assert_eq!(request.query_parameters().get("name").unwrap(), "John");
    /// ```
    pub fn query_parameters(&self) -> &HashMap<String, String> {
        &self.query_parameters
    }

    /// Creaates the query parameters from the request's URI.
    /// This is used internally to create the query parameters
    /// from the request's URI.
    pub(crate) fn query_parameters_from(value: &Uri) -> HashMap<String, String> {
        match value.path_and_query() {
            Some(query) => query
                .as_str()
                .trim_start_matches('/')
                .trim_start_matches('?')
                .split('&')
                .map(|pair| {
                    let mut pair = pair.split('=');
                    let key = pair.next().unwrap_or_default().to_string();
                    let value = pair.next().unwrap_or_default().to_string();

                    (key, value)
                })
                .collect(),
            None => HashMap::default(),
        }
    }

    /// Checks if the request has the given query parameter.
    ///
    /// # Example
    /// ```
    /// use valar::http::Request;
    /// use valar::http::Uri;
    ///
    /// let uri = Uri::from_static("http://localhost:3000/?id=1&name=John");
    ///
    /// let request = Request::builder()
    ///     .uri(uri)
    ///     .build();
    ///
    /// assert!(request.has_query("id"));
    /// assert!(request.has_query("name"));
    /// assert!(!request.has_query("age"));
    /// ```
    pub fn has_query(&self, name: &str) -> bool {
        self.query_parameters.contains_key(name)
    }

    /// Gets the given query parameter from the request's URI.
    /// If the query parameter does not exist, `None` is returned.
    ///
    /// # Example
    /// ```
    /// use valar::http::Request;
    /// use valar::http::Uri;
    ///
    /// let uri = Uri::from_static("http://localhost:3000/?id=1&name=John");
    ///
    /// let request = Request::builder()
    ///     .uri(uri)
    ///     .build();
    ///
    /// assert_eq!(request.maybe_query("id").unwrap(), "1");
    /// assert_eq!(request.maybe_query("name").unwrap(), "John");
    /// assert!(request.maybe_query("age").is_none());
    /// ```
    pub fn maybe_query(&self, name: &str) -> Option<&String> {
        self.query_parameters.get(name)
    }

    /// Gets the given query parameter from the request's URI.
    /// If the query parameter does not exist, an error is returned.
    ///
    /// # Example
    /// ```
    /// use valar::http::Request;
    /// use valar::http::Uri;
    ///
    /// let uri = Uri::from_static("http://localhost:3000/?id=1&name=John");
    ///
    /// let request = Request::builder()
    ///     .uri(uri)
    ///     .build();
    ///
    /// assert_eq!(request.query_parameter("id").unwrap(), "1");
    /// assert_eq!(request.query_parameter("name").unwrap(), "John");
    /// assert!(request.query_parameter("age").is_err());
    /// ```
    pub fn query_parameter(&self, name: &str) -> Result<&String, ErrorResponse> {
        self.maybe_query(name).ok_or_else(|| {
            ErrorResponse::new()
                .message(format!("Unknown query parameter: `{}`", name))
                .status(StatusCode::INTERNAL_SERVER_ERROR)
        })
    }

    /// Gets the given query parameter from the request's URI.
    /// If the query parameter does not exist, an error is returned.
    /// The query parameter is then parsed to the given type.
    /// If the query parameter cannot be parsed, an error is returned.
    /// This method is a shorthand for `query_parameter` and `parse`.
    ///
    /// # Example
    /// ```
    /// use valar::http::Request;
    /// use valar::http::Uri;
    ///
    /// let uri = Uri::from_static("http://localhost:3000/?id=1&name=John");
    ///
    /// let request = Request::builder()
    ///     .uri(uri)
    ///     .build();
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
    /// ```
    /// use valar::http::Request;
    /// use serde::Deserialize;
    ///
    /// #[derive(Deserialize)]
    /// struct User {
    ///     name: String
    /// }
    ///
    /// let request = Request::builder()
    ///     .body(r#"{"name": "John"}"#)
    ///     .build();
    ///
    /// let user: User = request.json().unwrap();
    ///
    /// assert_eq!(user.name, "John");
    /// ```
    #[cfg(feature = "json")]
    pub fn json<'a, T>(&'a self) -> JsonResult<T>
    where
        T: Deserialize<'a>,
    {
        serde_json::from_str(&self.body)
    }
}

#[derive(Default)]
pub struct RequestBuilder {
    method: Method,
    uri: Uri,
    version: Version,
    headers: HashMap<String, String>,
    body: String,
    route_parameters: HashMap<String, String>,
}

impl RequestBuilder {
    pub fn new() -> RequestBuilder {
        RequestBuilder::default()
    }

    pub fn method(mut self, method: Method) -> Self {
        self.method = method;

        self
    }

    pub fn uri(mut self, uri: Uri) -> Self {
        self.uri = uri;

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

    pub fn header(mut self, key: &str, value: &str) -> Self {
        self.headers.insert(key.to_string(), value.to_string());

        self
    }

    pub fn headers(mut self, headers: HashMap<String, String>) -> Self {
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

    pub fn route_parameters(mut self, parameters: HashMap<String, String>) -> Self {
        self.route_parameters = parameters;

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
        }
    }
}

pub struct FakeRequest<'a, App: Application> {
    app: &'a FakeApplication<App>,
    method: Method,
    uri: Uri,
    version: Version,
    headers: HashMap<String, String>,
    body: String,
}

impl<'a, App: Application> FakeRequest<'a, App> {
    pub fn new(app: &'a FakeApplication<App>) -> FakeRequest<'a, App> {
        FakeRequest {
            app,
            method: Method::default(),
            uri: Uri::default(),
            version: Version::default(),
            headers: HashMap::default(),
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

    pub fn headers(mut self, headers: HashMap<String, String>) -> Self {
        self.headers = headers;

        self
    }

    pub fn header<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.headers.insert(key.into(), value.into());

        self
    }

    pub fn body<V>(mut self, value: V) -> Self
    where
        V: Into<String>,
    {
        self.body = value.into();

        self
    }

    fn to_base_request(&self) -> HttpResult<BaseRequest<Body>> {
        let mut request = BaseRequest::builder()
            .method(self.method.clone())
            .uri(self.uri.clone());

        for (key, value) in &self.headers {
            request = request.header(key, value);
        }

        request.body(Body::from(self.body.clone()))
    }

    pub async fn send(&self) -> FakeResponse {
        let request = self.to_base_request().unwrap_or_default();
        let response = self.app.matcher().handle(self.app.app_arc(), request).await;

        FakeResponse::new(response)
    }
}
