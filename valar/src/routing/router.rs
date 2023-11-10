use std::marker::PhantomData;
use std::sync::Arc;

use http_body_util::{combinators::BoxBody, BodyExt, Empty, Full};
use hyper::body::Body;
use hyper::body::Buf;
use hyper::body::Bytes;
use hyper::body::Frame;
use hyper::body::Incoming;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::Request as BaseRequest;
use hyper::Uri;
use hyper::{Method, Response as BaseResponse, StatusCode};
use regex::Error as RegexError;
use thiserror::Error as ThisError;
use tokio::net::TcpListener;

use crate::http::Headers;
use crate::http::Method;
use crate::http::Request;
use crate::http::Response;
use crate::routing::middleware::Middleware;
use crate::routing::middleware::Middlewares;
use crate::routing::route::Builder;
use crate::routing::route::Config;
use crate::routing::route::Route;
use crate::utils::TruncatableToFit;

#[derive(Debug, ThisError)]
#[error(transparent)]
pub struct Error(#[from] RegexError);

pub enum Pending {}

pub enum Compiled {}

enum Routes<App: Send + Sync + 'static> {
    Pending(Vec<Builder<App>>),
    Compiled(Vec<Route<App>>),
}

/// A router is used to store routes and match them
/// against requests.
pub struct Router<App: Send + Sync + 'static, State = Pending> {
    /// Stores the current router configuration.
    middlewares: Middlewares<App>,

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

impl<App: Send + Sync + 'static> Router<App, Pending> {
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

    pub fn compile(self) -> Result<Router<App, Compiled>, Error> {
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

impl<App: Send + Sync + 'static> Router<App, Compiled> {
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

    pub fn summary(&self) -> Vec<String> {
        let summary: Vec<String> = self
            .routes()
            .iter()
            .rev()
            .map(|route| {
                format!(
                    "{:.<7} ⟶ {:.<33} ⟶ {:.<34}",
                    route.method().to_string().truncate_to_fit(7),
                    route.path().truncate_to_fit(34),
                    route.regex().to_string().truncate_to_fit(34)
                )
            })
            .collect();

        summary
    }

    pub(crate) async fn handle_base(
        &self,
        app: Arc<App>,
        request: BaseRequest<Incoming>,
    ) -> Response {
        let request = match Self::build_request(request, app.clone()).await {
            Ok(request) => request,
            Err(response) => return response,
        };

        self.handle(request).await
    }

    pub async fn handle(&self, request: Request<App>) -> Response {
        let route = self.find(request.method(), request.uri());
        let request = request.parematrized(route);

        route.handle(request).await
    }

    /// Turns a request into a base `Request` object.
    pub(crate) async fn build_request(
        mut base: BaseRequest<Incoming>,
        app: Arc<App>,
    ) -> Result<Request<App>, Response> {
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

        let bytes = base.body_mut();

        let headers: Headers<Request<App>> = base
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
            .build(app);

        Ok(builder)
    }
}

impl<App: Send + Sync + 'static> FromIterator<Builder<App>> for Router<App> {
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

// pub trait Routable: Sized + Application + Send + Sync +
// 'static {     fn router() -> Router<Self>;
//     fn compiled_router() -> Result<Arc<Router<Self,
// Compiled>>, Error> {         let router =
// Self::router().compile()?;         let router =
// Arc::new(router);

//         Ok(router)
//     }
// }

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use tokio::join;

    use crate::http::Request;
    use crate::http::Response;
    use crate::http::Result as ResponseResult;
    use crate::http::Uri;
    use crate::routing::route::Builder as Route;
    use crate::routing::Router;

    struct App;

    async fn handler(_request: Request<App>) -> ResponseResult {
        Response::ok().into_ok()
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

        let r1 = router.handle(Request::get(Uri::from_static("/")).build(app.clone()));
        let r2 = router.handle(Request::get(Uri::from_static("/foo")).build(app.clone()));
        let r3 = router.handle(Request::get(Uri::from_static("/foo/bar")).build(app.clone()));
        let r4 = router.handle(Request::get(Uri::from_static("/foo/bar/")).build(app.clone()));
        let r5 = router.handle(Request::get(Uri::from_static("/foo/asd123")).build(app.clone()));

        let r6 = router.handle(Request::get(Uri::from_static("/bar")).build(app.clone()));
        let r7 = router.handle(Request::get(Uri::from_static("/bar/")).build(app.clone()));
        let r8 = router.handle(Request::get(Uri::from_static("/bar/baz")).build(app.clone()));
        let r9 = router.handle(Request::get(Uri::from_static("/bar/baz/")).build(app.clone()));

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
