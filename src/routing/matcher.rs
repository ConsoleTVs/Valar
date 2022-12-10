use crate::http::Method;
use crate::http::Uri;
use crate::routing::Route;
use crate::routing::Router;
use regex::Error as RegexError;
use regex::Regex;

pub struct Matcher<Context: Sync + Send + 'static>(Vec<(Regex, Route<Context>)>);

impl<Context: Sync + Send + 'static> Matcher<Context> {
    /// Creates a new route matcher.
    pub fn new(routes: impl IntoIterator<Item = Route<Context>>) -> Result<Self, regex::Error> {
        let result = routes
            .into_iter()
            .map(|route| Ok((route.to_regex()?, route)))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self(result))
    }

    /// Returns the route that matches the given method and URL path.
    pub fn find(&self, method: &Method, path: &Uri) -> Option<&Route<Context>> {
        self.0
            .iter()
            .find(|(regex, route)| regex.is_match(path.path()) && route.method == *method)
            .map(|(_, route)| route)
    }

    /// Returns true if the given method and URI matches a route.
    pub fn matches(&self, method: &Method, path: &Uri) -> bool {
        self.find(method, path).is_some()
    }
}

impl<Context: Sync + Send + 'static> TryFrom<Router<Context>> for Matcher<Context> {
    type Error = RegexError;

    fn try_from(router: Router<Context>) -> Result<Matcher<Context>, Self::Error> {
        Self::new(router.routes)
    }
}

#[cfg(test)]
mod tests {
    use crate::http::Method;
    use crate::http::Request;
    use crate::http::Result as ResponseResult;
    use crate::http::Uri;
    use crate::routing::Router;

    struct Context;

    async fn handler(_request: Request<Context>) -> ResponseResult {
        unimplemented!()
    }

    #[test]
    fn it_can_match_router_routes() {
        let mut router = Router::<Context>::default();

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
