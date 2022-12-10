use crate::http::Method;
use crate::http::Request;
use crate::http::Response;
use crate::routing::Matcher;
use crate::routing::Route;
use std::future::Future;

/// A router is used to store routes and match them
/// against requests.
pub struct Router<Context: Sync + Send + 'static> {
    /// Stores the routes that the router will use to
    /// match requests.
    pub(crate) routes: Vec<Route<Context>>,
}

impl<Context: Sync + Send + 'static> Router<Context> {
    /// Creates a new router.
    pub fn new(routes: Vec<Route<Context>>) -> Self {
        Self { routes }
    }

    /// Returns the routes of the router.
    pub fn routes(&self) -> &[Route<Context>] {
        &self.routes
    }

    /// Adds a new route to the router.
    pub fn add_route(&mut self, route: Route<Context>) {
        self.routes.push(route);
    }

    /// Determines if the router has a route that matches the given criteria.
    pub fn has_route(&self, path: &str, method: &Method) -> bool {
        self.routes
            .iter()
            .any(|route| route.method == *method && route.path == path)
    }

    /// Adds a GET route to the router.
    pub fn get<P, H, R>(&mut self, path: P, handler: H)
    where
        P: Into<String>,
        R: Future<Output = Result<Response, anyhow::Error>> + Send + 'static,
        H: Fn(Request<Context>) -> R + Send + Sync + 'static,
    {
        self.add_route(Route::new(
            path,
            Method::GET,
            Box::new(move |req| Box::pin(handler(req))),
        ));
    }

    /// Adds a POST route to the router.
    /// This performs an allocation on the path.
    /// If you don't want to allocate, use `add_route`.
    // pub fn post(&mut self, path: impl Into<String>, handler: Handler<Context>) {
    //     self.add_route(Route::post(path, handler));
    // }

    /// Creates a route matcher from the current router.
    pub fn into_matcher(self) -> Result<Matcher<Context>, regex::Error> {
        self.try_into()
    }
}

impl<Context: Sync + Send + 'static> Default for Router<Context> {
    /// Creates a new router with an empty list of routes.
    fn default() -> Self {
        Self::new(Vec::new())
    }
}
