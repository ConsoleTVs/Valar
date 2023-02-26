use std::marker::PhantomData;
use std::sync::Arc;

use http::Request as BaseRequest;
use http::StatusCode;
use http::Uri;
use hyper::body::to_bytes;
use hyper::body::HttpBody;
use hyper::Body;
use regex::Error as RegexError;

use crate::http::ErrorResponse;
use crate::http::Headers;
use crate::http::Method;
use crate::http::Request;
use crate::http::Response;
use crate::routing::middleware::Middlewares;
use crate::routing::route::Builder;
use crate::routing::route::Config;
use crate::routing::route::Route;
use crate::routing::Middleware;
use crate::Application;
use crate::Error as GeneralError;

pub enum Pending {}

pub enum Compiled {}

enum Routes<App: Application> {
    Pending(Vec<Builder<App>>),
    Compiled(Vec<Route<App>>),
}

/// A router is used to store routes and match them
/// against requests.
pub struct Router<App: Application + Send + Sync + 'static, State = Pending> {
    /// Stores the current router configuration.
    middlewares: Middlewares<App>,

    /// Stores the routes that the router will use to
    /// match requests.
    routes: Routes<App>,

    state: PhantomData<State>,
}

impl<App: Application, State> Router<App, State> {
    pub(crate) fn error_response(wants_json: bool, error: GeneralError) -> Response {
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
}

impl<App: Application> Router<App, Pending> {
    /// Returns the routes of the router.
    pub fn routes(&self) -> &[Builder<App>] {
        match &self.routes {
            Routes::Pending(routes) => routes,
            _ => unreachable!(),
        }
    }

    pub fn middleware<M>(mut self, middleware: M) -> Self
    where
        M: Middleware<App> + Send + Sync + 'static,
    {
        self.middlewares.push(Arc::new(middleware));

        self
    }

    pub fn compile(self) -> Result<Router<App, Compiled>, RegexError> {
        let mut compiled_routes = Vec::new();

        let routes = match self.routes {
            Routes::Pending(routes) => routes,
            _ => unreachable!(),
        };

        for route in routes {
            let config = Config::from_middlewares(self.middlewares.clone());
            compiled_routes.extend(route.compile(config)?);
        }

        let router = Router {
            state: PhantomData::<Compiled>,
            middlewares: self.middlewares,
            routes: Routes::Compiled(compiled_routes),
        };

        Ok(router)
    }
}

impl<App: Application> Router<App, Compiled> {
    /// Returns the routes of the router.
    pub fn routes(&self) -> &[Route<App>] {
        match &self.routes {
            Routes::Compiled(routes) => routes,
            _ => unreachable!(),
        }
    }

    /// Determines if the router has a route that matches
    /// the given criteria.
    pub fn has_route(&self, path: &str, method: &Method) -> bool {
        self.routes()
            .iter()
            .rev()
            .any(|route| route.method() == method && route.path() == path)
    }

    /// Returns the route that matches the given method and
    /// URL path.
    pub fn find(&self, method: &Method, uri: &Uri) -> Option<&Route<App>> {
        self.routes()
            .iter()
            .rev()
            .find(|route| route.regex().is_match(uri.path()) && route.method() == method)
    }

    /// Returns true if the given method and URI matches a
    /// route.
    pub fn matches(&self, method: &Method, uri: &Uri) -> bool {
        self.find(method, uri).is_some()
    }

    pub(crate) async fn handle(&self, app: Arc<App>, request: BaseRequest<Body>) -> Response {
        let wants_json = Self::wants_json(&request);

        let request = match Self::build_request(request).await {
            Ok(request) => request,
            Err(error) => return Self::error_response(wants_json, error),
        };

        let route = match self.find(request.method(), request.uri()) {
            Some(route) => route,
            None => unreachable!("There should always be a fallback route."),
        };

        let request = request.parematrized(&route);

        route.handle(app.clone(), request).await
    }

    /// Turns a request into a base `Request` object.
    pub(crate) async fn build_request(
        mut base: BaseRequest<Body>,
    ) -> Result<Request, GeneralError> {
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

        let builder = Request::builder()
            .method(base.method().clone())
            .uri(base.uri().clone())
            .version(base.version().clone())
            .headers(headers)
            .body(bytes.escape_ascii().to_string())
            .build();

        Ok(builder)
    }
}

impl<App: Application> FromIterator<Builder<App>> for Router<App> {
    fn from_iter<I: IntoIterator<Item = Builder<App>>>(routes: I) -> Self {
        let mut routes_with_fallbacks = vec![Builder::fallback()];

        routes_with_fallbacks.extend(routes);

        Self {
            state: PhantomData::<Pending>,
            middlewares: Middlewares::new(),
            routes: Routes::Pending(routes_with_fallbacks),
        }
    }
}

pub type Error = RegexError;

pub trait Routable {
    type Application: Application + Send + Sync + 'static;

    fn router() -> Router<Self::Application>;
}
