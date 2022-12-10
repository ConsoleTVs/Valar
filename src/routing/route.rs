use crate::core::Application;
use crate::http::Handler;
use crate::http::Method;
use crate::http::Request;
use crate::http::Uri;
use http::Request as BaseRequest;
use hyper::Body;
use std::collections::HashMap;
use std::sync::Arc;

/// Routes are used to match requests to handlers.
/// They store information about the path, the HTTP method
/// and the handler function.
pub struct Route<Context: Sync + Send + 'static> {
    /// The path where the route will match on.
    pub(crate) path: String,

    /// The HTTP method that the route will match on.
    pub(crate) method: Method,

    /// The handler function that will be called when
    /// the route matches.
    pub(crate) handler: Handler<Context>,
}

impl<Context: Sync + Send + 'static> Route<Context> {
    /// Creates a new route.
    pub fn new(path: impl Into<String>, method: Method, handler: Handler<Context>) -> Self {
        Self {
            path: path.into(),
            method,
            handler,
        }
    }

    /// Returns the regex string literal for the given route.
    fn to_regex_string(&self) -> String {
        let path = self
            .path
            .trim_matches('/')
            .split('/')
            .map(|segment| match segment.starts_with(':') {
                true => "[a-zA-Z0-9-_]+",
                false => segment,
            })
            .collect::<Vec<_>>()
            .join("/");

        match path.as_str() {
            "" => "^/$".to_string(),
            _ => format!("^/{path}/?$"),
        }
    }

    /// Returns the regex of the route.
    pub(crate) fn to_regex(&self) -> Result<regex::Regex, regex::Error> {
        regex::Regex::new(&self.to_regex_string())
    }

    /// Get the parameters of the route given a path.
    pub(crate) fn parameters(&self, uri: &Uri) -> HashMap<String, String> {
        self.path
            .trim_matches('/')
            .split('/')
            .zip(uri.path().trim_matches('/').split('/'))
            .filter(|(route_segment, _)| route_segment.starts_with(':'))
            .map(|(route_segment, path_segment)| {
                (
                    route_segment.trim_start_matches(':').to_string(),
                    path_segment.to_string(),
                )
            })
            .collect::<HashMap<_, _>>()
    }

    pub(crate) fn to_request(
        &self,
        base: &BaseRequest<Body>,
        application: Arc<Application>,
        context: Arc<Context>,
    ) -> Request<Context> {
        let route_parameters = self.parameters(base.uri());

        Request {
            application,
            context,
            route_parameters,
            method: base.method().clone(),
            uri: base.uri().clone(),
            version: base.version(),
            headers: HashMap::new(), // TODO
            body: Body::from(""),    // TODO
        }
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
    fn it_can_parametrize_routes() {
        let mut router = Router::<Context>::default();

        router.get("/foo/:bar", handler);

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
