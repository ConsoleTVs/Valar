use crate::Application;
use hyper::service::make_service_fn;
use hyper::service::service_fn;
use hyper::Server as BaseServer;
use log::debug;
use log::info;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;

pub struct Server {
    address: SocketAddr,
}

impl Server {
    pub fn builder() -> ServerBuilder {
        ServerBuilder::new()
    }

    pub async fn start<App: Application + Send + Sync + 'static>(&self, app: App) {
        println!("VALAR");
        println!("Developer Centric Web Framework");
        println!();

        let app = Arc::new(app);
        let router = App::router();
        let matcher = Arc::new(router.into_matcher().unwrap());

        let service = make_service_fn(move |conn| {
            debug!("Incoming connection: {:?}", conn);
            let app = app.clone();
            let matcher = matcher.clone();

            async move {
                Ok::<_, Infallible>(service_fn(move |request| {
                    info!("{} {}", request.method(), request.uri());
                    let app = app.clone();
                    let matcher = matcher.clone();

                    async move { matcher.handle(app, request).await.into_base_response() }
                }))
            }
        });

        let server = BaseServer::bind(&self.address).serve(service);

        println!("Server running at: http://{}", self.address);
        println!();

        if let Err(e) = server.await {
            eprintln!("server error: {}", e);
        }
    }
}

#[derive(Default)]
pub struct ServerBuilder {
    address: Option<SocketAddr>,
}

impl ServerBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn address<A>(mut self, address: A) -> Self
    where
        A: Into<SocketAddr>,
    {
        self.address = Some(address.into());

        self
    }

    pub fn build(self) -> Server {
        Server {
            address: self
                .address
                .unwrap_or_else(|| SocketAddr::from(([127, 0, 0, 1], 3000))),
        }
    }
}
