use std::future::Future;
use std::sync::Arc;

use crate::http::Method;
use crate::http::Request;
use crate::http::Response;
use crate::routing::Matcher;
use crate::routing::Route;
use crate::Application;

/// A router is used to store routes and match them
/// against requests.
pub struct Router<App: Application> {
    /// Stores the routes that the router will use to
    /// match requests.
    routes: Vec<Route<App>>,
}

impl<App: Application> Router<App> {
    /// Creates a new router.
    pub fn new<R>(routes: R) -> Self
    where
        R: Into<Vec<Route<App>>>,
    {
        Self {
            routes: routes.into(),
        }
    }

    /// Returns the routes of the router.
    pub fn routes(&self) -> &[Route<App>] {
        &self.routes
    }

    /// Adds a new route to the router.
    pub fn add_route(&mut self, route: Route<App>) {
        self.routes.push(route);
    }

    /// Determines if the router has a route that matches
    /// the given criteria.
    pub fn has_route(&self, path: &str, method: &Method) -> bool {
        self.routes
            .iter()
            .any(|route| route.method() == method && route.path() == path)
    }

    /// Adds a GET route to the router.
    pub fn get<P, H, R>(&mut self, path: P, handler: H)
    where
        P: Into<String>,
        R: Future<Output = Result<Response, anyhow::Error>> + Send + 'static,
        H: Fn(Arc<App>, Request) -> R + Send + Sync + 'static,
    {
        let route = Route::new(
            path,
            Method::GET,
            Box::new(move |app, req| Box::pin(handler(app, req))),
        );

        self.add_route(route);
    }

    pub fn through<F>(mut self, handler: F) -> Self
    where
        F: FnOnce(&mut Self),
    {
        handler(&mut self);

        self
    }

    /// Creates a route matcher from the current router.
    pub fn into_matcher(self) -> Result<Matcher<App>, regex::Error> {
        self.try_into()
    }
}

impl<App: Application> IntoIterator for Router<App> {
    type IntoIter = std::vec::IntoIter<Self::Item>;
    type Item = Route<App>;

    fn into_iter(self) -> Self::IntoIter {
        self.routes.into_iter()
    }
}

impl<App: Application> Default for Router<App> {
    /// Creates a new router with an empty list of routes.
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

pub trait Routable {
    type Application: Application;

    fn router() -> Router<Self::Application>;
}
