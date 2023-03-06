use std::collections::HashMap;
use std::future::Future;
use std::ops::Deref;
use std::sync::Arc;

use regex::Error as RegexError;
use regex::Regex;

use crate::http::Handler;
use crate::http::Method;
use crate::http::Request;
use crate::http::Response;
use crate::http::Result as HttpResult;
use crate::http::Uri;
use crate::routing::middleware::Middleware;
use crate::routing::middleware::Middlewares;
use crate::routing::Router;
use crate::Application;

/// Routes are used to match requests to handlers. They
/// store information about the path, the HTTP method and
/// the handler function.
pub struct Data<App: Application> {
    path: String,
    methods: Vec<Method>,
    handler: Handler<App>,
    parameters: HashMap<String, String>,
    middlewares: Middlewares,
}

#[derive(Default)]
pub struct Config {
    middlewares: Middlewares,
    parameters: HashMap<String, String>,
}

pub struct Group<App: Application> {
    config: Config,
    routes: Vec<Builder<App>>,
}

pub enum Builder<App: Application> {
    Data(Data<App>),
    Group(Group<App>),
}

pub struct Route<App: Application> {
    regex: Regex,
    path: String,
    method: Method,
    handler: Handler<App>,
}

impl Config {
    pub fn from_middlewares(middlewares: Middlewares) -> Self {
        Self {
            middlewares,
            parameters: Default::default(),
        }
    }
}

impl Clone for Config {
    fn clone(&self) -> Self {
        Self {
            middlewares: self.middlewares.clone(),
            parameters: self.parameters.clone(),
        }
    }
}

impl<'a> FromIterator<&'a Config> for Config {
    fn from_iter<T: IntoIterator<Item = &'a Self>>(iter: T) -> Self {
        let mut parameters = HashMap::new();
        let mut middlewares = Middlewares::new();

        for config in iter {
            parameters.extend(config.parameters.clone());
            middlewares.extend(config.middlewares.clone());
        }

        Self {
            middlewares,
            parameters,
        }
    }
}

async fn not_found_handler<App: Application>(_app: Arc<App>, request: Request) -> HttpResult {
    Response::not_found()
        .message(format!(
            "No route found for {} {}",
            request.method(),
            request.uri()
        ))
        .as_ok()
}

impl<App: Application> Builder<App> {
    pub fn fallback() -> Self {
        Builder::any(".*", not_found_handler)
    }

    pub fn group<I>(routes: I) -> Self
    where
        I: Into<Vec<Builder<App>>>,
    {
        let group = Group {
            config: Config {
                middlewares: Default::default(),
                parameters: Default::default(),
            },
            routes: routes.into(),
        };

        Self::Group(group)
    }

    /// Adds a GET route to the router.
    pub fn get<P, H, R>(path: P, handler: H) -> Self
    where
        P: Into<String>,
        R: Future<Output = HttpResult> + Send + 'static,
        H: Fn(Arc<App>, Request) -> R + Send + Sync + 'static,
    {
        let handler: Handler<App> = Arc::new(move |app, req| Box::pin(handler(app, req)));

        let data = Data {
            path: path.into(),
            methods: vec![Method::GET],
            handler,
            parameters: Default::default(),
            middlewares: Default::default(),
        };

        Self::Data(data)
    }

    /// Adds a route to the router that matches all http
    /// methods.
    pub fn any<P, H, R>(path: P, handler: H) -> Self
    where
        P: Into<String>,
        R: Future<Output = Result<Response, anyhow::Error>> + Send + 'static,
        H: Fn(Arc<App>, Request) -> R + Send + Sync + 'static,
    {
        let handler: Handler<App> = Arc::new(move |app, req| Box::pin(handler(app, req)));

        let methods = vec![
            Method::OPTIONS,
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::HEAD,
            Method::TRACE,
            Method::CONNECT,
            Method::PATCH,
        ];

        let data = Data {
            path: path.into(),
            methods,
            handler,
            parameters: Default::default(),
            middlewares: Default::default(),
        };

        Self::Data(data)
    }

    pub fn middleware<M, R>(mut self, middleware: M) -> Self
    where
        M: Middleware + Send + Sync + 'static,
    {
        let middlewares = match &mut self {
            Self::Data(data) => &mut data.middlewares,
            Self::Group(group) => &mut group.config.middlewares,
        };

        middlewares.push(Arc::new(middleware));

        self
    }

    pub fn where_parameter<N, V>(mut self, name: N, value: V) -> Self
    where
        N: Into<String>,
        V: Into<String>,
    {
        let parameters = match &mut self {
            Self::Data(data) => &mut data.parameters,
            Self::Group(group) => &mut group.config.parameters,
        };

        parameters.insert(name.into(), value.into());

        self
    }

    pub fn compile(self, previous: Config) -> Result<Vec<Route<App>>, RegexError> {
        match self {
            Builder::Data(data) => data.compile(previous),
            Builder::Group(group) => group.compile(previous),
        }
    }
}

impl<App: Application> Group<App> {
    pub fn compile(self, config: Config) -> Result<Vec<Route<App>>, RegexError> {
        let mut routes = Vec::new();

        for route in self.routes {
            let config = Config::from_iter([&config, &self.config]);
            let compiled_routes = route.compile(config)?;

            routes.extend(compiled_routes);
        }

        Ok(routes)
    }
}

impl<App: Application> Data<App> {
    /// Returns the regex string literal for the given
    /// route.
    fn to_regex_string(&self) -> String {
        let segments = self.path.trim_matches('/').split('/');

        let regex_path = segments
            .map(|segment| match segment.starts_with(':') {
                true => self
                    .parameters
                    .get(segment.trim_matches(':'))
                    .map(|segment| segment.deref())
                    .unwrap_or("[a-zA-Z0-9-_]+"),
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
    pub fn to_regex(&self) -> Result<Regex, RegexError> {
        Regex::new(&self.to_regex_string())
    }

    pub fn compile(self, config: Config) -> Result<Vec<Route<App>>, RegexError> {
        let mut routes = Vec::new();

        let regex = self.to_regex()?;

        let middlewares = Middlewares::from_iter([&config.middlewares, &self.middlewares]);
        let handler = middlewares.wrap(self.handler.clone());

        for method in self.methods {
            let route = Route {
                regex: regex.clone(),
                path: self.path.clone(),
                method,
                handler: handler.clone(),
            };

            routes.push(route);
        }

        Ok(routes)
    }
}

impl<App: Application> Route<App> {
    pub fn method(&self) -> &Method {
        &self.method
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn regex(&self) -> &Regex {
        &self.regex
    }

    pub fn handler(&self) -> &Handler<App> {
        &self.handler
    }

    /// Handles the route with the given app and request.
    pub async fn handle(&self, app: Arc<App>, request: Request) -> Response {
        match (self.handler)(app, request).await {
            Ok(response) => response,
            Err(error) => Router::<App>::error_response(error),
        }
    }

    /// Get the parameters of the route given a path.
    pub(crate) fn parameters(&self, uri: &Uri) -> HashMap<String, String> {
        self.path
            .trim_matches('/')
            .split('/')
            .zip(uri.path().trim_matches('/').split('/'))
            .filter_map(|(route_segment, path_segment)| {
                route_segment.starts_with(':').then(|| {
                    let parameter = route_segment.trim_start_matches(':').to_string();
                    let value = path_segment.to_string();

                    (parameter, value)
                })
            })
            .collect()
    }
}

// impl<'a, App: Application>
// Builder<App> {     /// Creates a new route.
//     pub fn new<P>(
//         path: P,
//         method: Method,
//         handler: Handler<App>,
//         middlewares: Middlewares<App>,
//     ) -> Self
//     where
//         P: Into<String>,
//     {
//         Self {
//             path: path.into(),
//             method,
//             handler,
//             middlewares,
//         }
//     }

//     /// Returns the path of the route.
//     pub fn path(&self) -> &str {
//         &self.path
//     }

//     /// Returns the HTTP method of the route.
//     pub fn method(&self) -> &Method {
//         &self.method
//     }

//     pub fn middleware<M>(&mut self, middleware: M)
//     where
//         M: Middleware<App>,
//     {
//         let f = Arc::new(middleware);
//         self.middlewares.push(f);
//     }

//     pub fn middleware_from_arc(
//         &mut self,
//         middleware: Arc<dyn Middleware<App> + Send + Sync
// + 'static>,     ) {
//         self.middlewares.push(middleware);
//     }

//     /// Handles the route with the given app and request.
//     pub async fn handle(&self, app: Arc<App>, mut
// request: Request) -> Response {         let wants_json =
// request.wants_json();         let (response, middlewares)
// = self.middlewares.before(app.clone(), &mut
// request).await;

//         if let Some(mut response) = response {
//             // Some middleware bailed out and
//             // returned a response. Therefore we
//             // don't need to call the handler.
//             middlewares.after(app, &mut response).await;

//             return response;
//         }

//         let mut response = match
// (self.handler)(app.clone(), request).await {
// Ok(response) => response,             Err(error) =>
// Matcher::<App>::error_response(wants_json, error),
//         };

//         middlewares.after(app, &mut response).await;

//         response
//     }

//     /// Returns the regex string literal for the given
//     /// route.
//     fn to_regex_string(&self) -> String {
//         let regex_path = self
//             .path
//             .trim_matches('/')
//             .split('/')
//             .map(|segment| match segment.starts_with(':')
// {                 true => "[a-zA-Z0-9-_]+",
//                 false => segment,
//             })
//             .collect::<Vec<_>>()
//             .join("/");

//         match regex_path.is_empty() {
//             true => "^/$".to_string(),
//             false => format!("^/{regex_path}/?$"),
//         }
//     }

//     /// Returns the regex of the route.
//     /// This generates a new regex every time it is
// called.     pub(crate) fn to_regex(&self) ->
// Result<Regex, RegexError> {         Regex::new(&self.
// to_regex_string())     }

//     /// Get the parameters of the route given a path.
//     pub(crate) fn parameters(&self, uri: &Uri) ->
// HashMap<String, String> {         self.path
//             .trim_matches('/')
//             .split('/')
//             .zip(uri.path().trim_matches('/').split('/'))
//             .filter_map(|(route_segment, path_segment)| {
//                 route_segment.starts_with(':').then(|| {
//                     let parameter =
// route_segment.trim_start_matches(':').to_string();
//                     let value = path_segment.to_string();

//                     (parameter, value)
//                 })
//             })
//             .collect()
//     }

//     /// Turns a request into a base `Request` object.
//     pub(crate) async fn into_request(&self, mut base:
// BaseRequest<Body>) -> Result<Request, Error> {         //
// TODO: Allow this to be dynamic. Current hardcoded 2MB
//         // limit.
//         const MAX_ALLOWED_RESPONSE_SIZE: u64 = 1024 *
// 1024 * 2;

//         let content_length = base
//             .body()
//             .size_hint()
//             .upper()
//             .unwrap_or(MAX_ALLOWED_RESPONSE_SIZE + 1);

//         if content_length > MAX_ALLOWED_RESPONSE_SIZE {
//             let error = ErrorResponse::new()
//                 .message("Request body too large")
//                 .status(StatusCode::PAYLOAD_TOO_LARGE);

//             return Err(error.into());
//         }

//         let bytes = to_bytes(base.body_mut()).await?;

//         let headers: Headers = base
//             .headers()
//             .iter()
//             .map(|(key, value)| {
//                 let key = key.to_string();
//                 let value =
// value.to_str().unwrap_or_default().to_string();

//                 (key, value)
//             })
//             .collect();

//         let request = Request::builder()
//
// .route_parameters(self.parameters(base.uri()))
//             .method(base.method().clone())
//             .uri(base.uri().clone())
//             .version(base.version().clone())
//             .headers(headers)
//             .body(bytes.escape_ascii().to_string())
//             .build();

//         Ok(request)
//     }
// }

// #[cfg(test)]
// mod tests {
//     use std::sync::Arc;

//     use async_trait::async_trait;

//     use crate::http::Method;
//     use crate::http::Request;
//     use crate::http::Result as ResponseResult;
//     use crate::http::Uri;
//     use crate::routing::Routable;
//     use crate::routing::Router;
//     use crate::Application;
//     use crate::Error;

//     struct App;

//     #[async_trait]
//     impl Application for App {
//         async fn create() -> Result<Self, Error> {
//             Ok(Self)
//         }
//     }

//     impl Routable for App {
//         type Application = App;

//         fn router() -> Router<Self::Application> {
//             let mut router = Router::new();

//             router.get("/foo/:bar", handler);

//             router
//         }
//     }

//     async fn handler(_app: Arc<App>, _request: Request)
// -> ResponseResult {         unimplemented!()
//     }

//     #[test]
//     fn it_can_parametrize_routes() {
//         let router = App::router();
//         let matcher = router.into_matcher().unwrap();
//         let uri = Uri::from_static("/foo/asd123");

//         assert!(matcher.matches(&Method::GET, &uri));

//         let route = matcher.find(&Method::GET, &uri);

//         assert!(route.is_some());

//         let params = route.unwrap().parameters(&uri);

//         assert_eq!(params.len(), 1);
//         assert_eq!(params.get("bar"),
// Some(&"asd123".to_string()));     }
// }
