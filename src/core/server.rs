use crate::http::ErrorResponse;
use crate::http::Response;
use crate::http::StatusCode;
use crate::routing::Router;
use crate::Application;
use http::Result as HttpResult;
use hyper::service::make_service_fn;
use hyper::service::service_fn;
use hyper::Body;
use hyper::Response as BaseResponse;
use hyper::Server as BaseServer;
use log::debug;
use log::info;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;

pub struct Server<Context: Sync + Send + 'static> {
    address: SocketAddr,
    router: Router<Context>,
}

impl<Context: Sync + Send + 'static> Server<Context> {
    pub fn builder() -> ServerBuilder<Context> {
        ServerBuilder::new()
    }

    fn error_response(wants_json: bool, error: anyhow::Error) -> HttpResult<BaseResponse<Body>> {
        if let Some(error) = error.downcast_ref::<ErrorResponse>() {
            if wants_json {
                return error.to_json_response().into_base_response();
            }

            return error.to_response().into_base_response();
        }

        let message = error.to_string();

        Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .html(format!("<h1>INTERNAL SERVER ERROR</h1><h2>{message}</h2>"))
            .build()
            .into_base_response()
    }

    pub async fn serve(self, context: Context) {
        println!("=============================");
        println!("| V A L A R                 |");
        println!("| DX-Oriented Web Framework |");
        println!("=============================");
        println!();

        let application = Arc::new(Application {});
        let context = Arc::new(context);
        let matcher = Arc::new(self.router.into_matcher().unwrap());

        let service = make_service_fn(move |conn| {
            debug!("Incoming connection: {:?}", conn);
            let application = application.clone();
            let context = context.clone();
            let matcher = matcher.clone();

            async move {
                Ok::<_, Infallible>(service_fn(move |req| {
                    info!("{} {}", req.method(), req.uri());
                    let application = application.clone();
                    let context = context.clone();
                    let matcher = matcher.clone();

                    async move {
                        if let Some(route) = matcher.find(req.method(), req.uri()) {
                            let request = route.to_request(&req, application, context);
                            let wants_json = request.wants_json();
                            let response = (route.handler)(request).await;

                            return match response {
                                Ok(response) => response.into_base_response(),
                                Err(error) => Self::error_response(wants_json, error),
                            };
                        }

                        Response::builder()
                            .status(StatusCode::NOT_FOUND)
                            .html("<h1>NOT FOUND</h1><h2>We were unable to find this page</h2>")
                            .build()
                            .into_base_response()
                    }
                }))
            }
        });

        let server = BaseServer::bind(&self.address).serve(service);

        println!("Server running at: {}", self.address);
        println!();

        if let Err(e) = server.await {
            eprintln!("server error: {}", e);
        }
    }
}

pub struct ServerBuilder<Context: Sync + Send + 'static> {
    address: Option<SocketAddr>,
    router: Option<Router<Context>>,
}

impl<Context: Sync + Send + 'static> ServerBuilder<Context> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn address(mut self, address: impl Into<SocketAddr>) -> Self {
        self.address = Some(address.into());

        self
    }

    pub fn router(mut self, router: Router<Context>) -> Self {
        self.router = Some(router);

        self
    }

    pub fn build(self) -> Server<Context> {
        Server {
            address: self
                .address
                .unwrap_or_else(|| SocketAddr::from(([127, 0, 0, 1], 3000))),
            router: self.router.unwrap_or_default(),
        }
    }
}

impl<Context: Sync + Send + 'static> Default for ServerBuilder<Context> {
    fn default() -> Self {
        Self {
            address: None,
            router: None,
        }
    }
}
