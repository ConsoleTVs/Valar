use std::marker::PhantomData;
use std::sync::Arc;

use http::Request as BaseRequest;
use http::Uri;
use hyper::body::to_bytes;
use hyper::body::HttpBody;
use hyper::Body;
use regex::Error as RegexError;

use crate::http::Headers;
use crate::http::Method;
use crate::http::Request;
use crate::http::Response;
use crate::routing::middleware::Middleware;
use crate::routing::middleware::Middlewares;
use crate::routing::route::Builder;
use crate::routing::route::Config;
use crate::routing::route::Route;
use crate::Application;

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
    middlewares: Middlewares,

    /// Stores the routes that the router will use to
    /// match requests.
    routes: Routes<App>,

    state: PhantomData<State>,
}

// impl<App: Application, State> Router<App, State> {
//     pub(crate) fn error_response(error: anyhow::Error) ->
// Response {         error.downcast::<Response>().
// unwrap_or_else(|error| {
// Response::internal_server_error()
// .message(error.to_string())                 .build()
//         })
//     }
// }

impl<App: Application + Send + Sync + 'static> Router<App, Pending> {
    /// Returns the routes of the router.
    pub fn routes(&self) -> &[Builder<App>] {
        match &self.routes {
            Routes::Pending(routes) => routes,
            _ => unreachable!(),
        }
    }

    pub fn middleware<M>(mut self, middleware: M) -> Self
    where
        M: Middleware + Send + Sync + 'static,
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

impl<App: Application + Send + Sync + 'static> Router<App, Compiled> {
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
    /// URL path. A fallback route must always be present,
    /// othersiwe it will panic. The default router builders
    /// already provide such a fallack route.
    pub fn find(&self, method: &Method, uri: &Uri) -> &Route<App> {
        self.routes()
            .iter()
            .rev()
            .find(|route| route.regex().is_match(uri.path()) && route.method() == method)
            .expect("There should always be a fallback route in a router.")
    }

    pub(crate) async fn handle_base(&self, app: Arc<App>, request: BaseRequest<Body>) -> Response {
        let request = match Self::build_request(request).await {
            Ok(request) => request,
            Err(response) => return response,
        };

        self.handle(app, request).await
    }

    pub async fn handle<R>(&self, app: Arc<App>, request: R) -> Response
    where
        R: Into<Request>,
    {
        let request = request.into();
        let route = self.find(request.method(), request.uri());
        let request = request.parematrized(route);

        route.handle(app.clone(), request).await
    }

    /// Turns a request into a base `Request` object.
    pub(crate) async fn build_request(mut base: BaseRequest<Body>) -> Result<Request, Response> {
        // TODO: Allow this to be dynamic. Current hardcoded 2MB.
        const MAX_ALLOWED_RESPONSE_SIZE: u64 = 1024 * 1024 * 2;

        let content_length = base
            .body()
            .size_hint()
            .upper()
            .unwrap_or(MAX_ALLOWED_RESPONSE_SIZE + 1);

        if content_length > MAX_ALLOWED_RESPONSE_SIZE {
            let error = Response::payload_too_large()
                .message("Request body too large")
                .build();

            return Err(error);
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
            .version(base.version())
            .headers(headers)
            .body(bytes.escape_ascii().to_string())
            .build();

        Ok(builder)
    }
}

impl<App: Application + Send + Sync + 'static> FromIterator<Builder<App>> for Router<App> {
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

pub trait Routable: Sized + Application + Send + Sync + 'static {
    fn router() -> Router<Self>;
    fn compiled_router() -> Result<Arc<Router<Self, Compiled>>, Error> {
        let router = Self::router().compile()?;
        let router = Arc::new(router);

        Ok(router)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;
    use tokio::join;

    use crate::http::Request;
    use crate::http::Response;
    use crate::http::Result as ResponseResult;
    use crate::http::Uri;
    use crate::routing::route::Builder as Route;
    use crate::routing::Router;
    use crate::Application;

    struct App;

    #[async_trait]
    impl Application for App {}

    async fn handler(_app: Arc<App>, _request: Request) -> ResponseResult {
        Response::ok().as_ok()
    }

    #[tokio::test]
    async fn it_can_match_router_routes() {
        let app = Arc::new(App);

        let router = Router::from_iter([
            Route::get("/", handler),
            Route::get("/foo", handler),
            Route::get("/foo/:bar", handler),
            Route::get("/foo/bar", handler),
        ]);

        let router = router.compile().unwrap();

        let r1 = router.handle(app.clone(), Request::get(Uri::from_static("/")));
        let r2 = router.handle(app.clone(), Request::get(Uri::from_static("/foo")));
        let r3 = router.handle(app.clone(), Request::get(Uri::from_static("/foo/bar")));
        let r4 = router.handle(app.clone(), Request::get(Uri::from_static("/foo/bar/")));
        let r5 = router.handle(app.clone(), Request::get(Uri::from_static("/foo/asd123")));

        let r6 = router.handle(app.clone(), Request::get(Uri::from_static("/bar")));
        let r7 = router.handle(app.clone(), Request::get(Uri::from_static("/bar/")));
        let r8 = router.handle(app.clone(), Request::get(Uri::from_static("/bar/baz")));
        let r9 = router.handle(app.clone(), Request::get(Uri::from_static("/bar/baz/")));

        let (r1, r2, r3, r4, r5, r6, r7, r8, r9) = join!(r1, r2, r3, r4, r5, r6, r7, r8, r9);

        r1.assert_ok();
        r2.assert_ok();
        r3.assert_ok();
        r4.assert_ok();
        r5.assert_ok();

        r6.assert_not_found();
        r7.assert_not_found();
        r8.assert_not_found();
        r9.assert_not_found();
    }
}
