use std::collections::HashMap;
use std::sync::Arc;

use http::Request as BaseRequest;
use http::StatusCode;
use hyper::body::to_bytes;
use hyper::body::HttpBody;
use hyper::Body;
use regex::Error as RegexError;
use regex::Regex;

use crate::http::headers::Headers;
use crate::http::ErrorResponse;
use crate::http::Handler;
use crate::http::Method;
use crate::http::Request;
use crate::http::Result as HttpResult;
use crate::http::Uri;
use crate::Application;
use crate::Error;

/// Routes are used to match requests to handlers.
/// They store information about the path, the HTTP method
/// and the handler function.
pub struct Route<App: Application> {
    /// The path where the route will match on.
    path: String,

    /// The HTTP method that the route will match on.
    method: Method,

    /// The handler function that will be called when
    /// the route matches.
    handler: Handler<App>,
}

impl<'a, App: Application> Route<App> {
    /// Creates a new route.
    pub fn new<P>(path: P, method: Method, handler: Handler<App>) -> Self
    where
        P: Into<String>,
    {
        Self {
            path: path.into(),
            method,
            handler,
        }
    }

    /// Returns the path of the route.
    pub fn path(&self) -> &String {
        &self.path
    }

    /// Returns the HTTP method of the route.
    pub fn method(&self) -> &Method {
        &self.method
    }

    /// Handles the route with the given app and request.
    pub async fn handle(&self, app: Arc<App>, request: Request) -> HttpResult {
        (self.handler)(app, request).await
    }

    /// Returns the regex string literal for the given
    /// route.
    fn to_regex_string(&self) -> String {
        let regex_path = self
            .path
            .trim_matches('/')
            .split('/')
            .map(|segment| match segment.starts_with(':') {
                true => "[a-zA-Z0-9-_]+",
                false => segment,
            })
            .collect::<Vec<_>>()
            .join("/");

        match regex_path.is_empty() {
            true => "^/$".to_string(),
            false => format!("^/{regex_path}/?$"),
        }
    }

    /// Returns the regex of the route.
    /// This generates a new regex every time it is called.
    pub(crate) fn to_regex(&self) -> Result<Regex, RegexError> {
        Regex::new(&self.to_regex_string())
    }

    /// Get the parameters of the route given a path.
    pub(crate) fn parameters(&self, uri: &Uri) -> HashMap<String, String> {
        self.path
            .trim_matches('/')
            .split('/')
            .zip(uri.path().trim_matches('/').split('/'))
            .filter_map(|(route_segment, path_segment)| {
                route_segment.starts_with(':').then(|| {
                    let parameter =
                        route_segment.trim_start_matches(':').to_string();
                    let value = path_segment.to_string();

                    (parameter, value)
                })
            })
            .collect()
    }

    /// Turns a request into a base `Request` object.
    pub(crate) async fn to_request(
        &self,
        base: &mut BaseRequest<Body>,
    ) -> Result<Request, Error> {
        // TODO: Allow this to be dynamic. Current hardcoded 2MB
        // limit.
        const MAX_ALLOWED_RESPONSE_SIZE: u64 = 1024 * 1024 * 2;

        let content_length = base
            .body()
            .size_hint()
            .upper()
            .unwrap_or(MAX_ALLOWED_RESPONSE_SIZE + 1);

        if content_length > MAX_ALLOWED_RESPONSE_SIZE {
            let error = ErrorResponse::new()
                .message("Request body too large")
                .status(StatusCode::PAYLOAD_TOO_LARGE);

            return Err(error.into());
        }

        let bytes = to_bytes(base.body_mut()).await?;

        let headers: Headers = base
            .headers()
            .iter()
            .map(|(key, value)| {
                let key = key.to_string();
                let value = value.to_str().unwrap_or_default().to_string();

                (key, value)
            })
            .collect();

        let request = Request {
            route_parameters: self.parameters(base.uri()),
            query_parameters: Request::query_parameters_from(base.uri()),
            method: base.method().clone(),
            uri: base.uri().clone(),
            version: base.version(),
            headers,
            body: bytes.escape_ascii().to_string(),
        };

        Ok(request)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;

    use crate::http::Method;
    use crate::http::Request;
    use crate::http::Result as ResponseResult;
    use crate::http::Uri;
    use crate::routing::Routable;
    use crate::routing::Router;
    use crate::Application;
    use crate::Error;

    struct App;

    #[async_trait]
    impl Application for App {
        async fn create() -> Result<Self, Error> {
            Ok(Self)
        }
    }

    impl Routable for App {
        type Application = App;

        fn router() -> Router<Self::Application> {
            let mut router = Router::default();

            router.get("/foo/:bar", handler);

            router
        }
    }

    async fn handler(_app: Arc<App>, _request: Request) -> ResponseResult {
        unimplemented!()
    }

    #[test]
    fn it_can_parametrize_routes() {
        let router = App::router();
        let matcher = router.into_matcher().unwrap();
        let uri = Uri::from_static("/foo/asd123");

        assert!(matcher.matches(&Method::GET, &uri));

        let route = matcher.find(&Method::GET, &uri);

        assert!(route.is_some());

        let params = route.unwrap().parameters(&uri);

        assert_eq!(params.len(), 1);
        assert_eq!(params.get("bar"), Some(&"asd123".to_string()));
    }
}
