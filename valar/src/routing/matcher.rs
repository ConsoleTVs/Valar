use std::sync::Arc;

use http::Request as BaseRequest;
use hyper::Body;
use regex::Error as RegexError;
use regex::Regex;

use crate::http::error::ErrorResponse;
use crate::http::Method;
use crate::http::Response;
use crate::http::StatusCode;
use crate::http::Uri;
use crate::routing::Route;
use crate::routing::Router;
use crate::Application;
use crate::Error;

pub struct Matcher<App: Application>(Vec<(Regex, Route<App>)>);

impl<App: Application> Matcher<App> {
    /// Creates a new route matcher.
    pub fn new<R>(routes: R) -> Result<Self, regex::Error>
    where
        R: IntoIterator<Item = Route<App>>,
    {
        let result: Result<Vec<(Regex, Route<App>)>, RegexError> = routes
            .into_iter()
            .map(|route| Ok((route.to_regex()?, route)))
            .collect();

        Ok(Self(result?))
    }

    /// Returns the route that matches the given method and
    /// URL path.
    pub fn find(&self, method: &Method, uri: &Uri) -> Option<&Route<App>> {
        self.0
            .iter()
            .find(|(regex, route)| {
                regex.is_match(uri.path()) && route.method() == method
            })
            .map(|(_, route)| route)
    }

    /// Returns true if the given method and URI matches a
    /// route.
    pub fn matches(&self, method: &Method, uri: &Uri) -> bool {
        self.find(method, uri).is_some()
    }

    pub(crate) fn not_found(&self, wants_json: bool) -> Response {
        let response = ErrorResponse::new().status(StatusCode::NOT_FOUND);

        match wants_json {
            true => response.into_json_response(),
            false => response.into_response(),
        }
    }

    pub(crate) fn error_response(wants_json: bool, error: Error) -> Response {
        let error = error.downcast::<ErrorResponse>().unwrap_or_else(|error| {
            ErrorResponse::new()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .message(error.to_string())
        });

        match wants_json {
            true => error.into_json_response(),
            false => error.into_response(),
        }
    }

    pub(crate) fn wants_json(request: &BaseRequest<Body>) -> bool {
        match request.headers().get("Accept") {
            Some(accept) => accept
                .to_str()
                .map(|accept| accept.contains("application/json"))
                .unwrap_or(false),
            None => false,
        }
    }

    pub(crate) async fn handle(
        &self,
        application: Arc<App>,
        request: BaseRequest<Body>,
    ) -> Response {
        let wants_json = Self::wants_json(&request);

        let route = match self.find(request.method(), request.uri()) {
            Some(route) => route,
            None => return self.not_found(wants_json),
        };

        let request = match route.into_request(request).await {
            Ok(request) => request,
            Err(error) => return Self::error_response(wants_json, error),
        };

        let response = route.handle(application, request).await;

        match response {
            Ok(response) => response,
            Err(error) => Self::error_response(wants_json, error),
        }
    }
}

impl<App: Application> TryFrom<Router<App>> for Matcher<App> {
    type Error = RegexError;

    fn try_from(router: Router<App>) -> Result<Matcher<App>, Self::Error> {
        Self::new(router)
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
    fn it_can_match_router_routes() {
        let mut router = Router::default();

        router.get("/", handler);
        router.get("/foo", handler);
        router.get("/foo/:bar", handler);
        router.get("/foo/bar/", handler);

        let matcher = router.into_matcher().unwrap();

        assert!(matcher.matches(&Method::GET, &Uri::from_static("/")));
        assert!(matcher.matches(&Method::GET, &Uri::from_static("/foo")));
        assert!(matcher.matches(&Method::GET, &Uri::from_static("/foo/bar")));
        assert!(matcher.matches(&Method::GET, &Uri::from_static("/foo/bar/")));
        assert!(matcher.matches(&Method::GET, &Uri::from_static("/foo/asd123")));
        assert!(!matcher.matches(&Method::GET, &Uri::from_static("/bar")));
        assert!(!matcher.matches(&Method::GET, &Uri::from_static("/bar/")));
        assert!(!matcher.matches(&Method::GET, &Uri::from_static("/bar/baz")));
        assert!(!matcher.matches(&Method::GET, &Uri::from_static("/bar/baz/")));
    }
}
