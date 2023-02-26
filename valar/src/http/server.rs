use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;

use colored::Colorize;
use hyper::service::make_service_fn;
use hyper::service::service_fn;
use hyper::Server as BaseServer;
use log::debug;
use log::info;

use crate::Application;

pub struct Server {
    address: SocketAddr,
}

impl Server {
    pub fn builder() -> ServerBuilder {
        ServerBuilder::new()
    }

    pub async fn start<App: Application + Send + Sync + 'static>(&self, app: App) {
        println!("{} • Supercharged Async Web Framework", "Valar".bold());
        println!("{}", "Lambda Studio • https://λ.studio".italic().dimmed());
        println!();

        let app = Arc::new(app);
        let router = Arc::new(App::router().compile().unwrap());

        let service = make_service_fn(move |conn| {
            debug!("Incoming connection: {:?}", conn);
            let app = app.clone();
            let router = router.clone();

            async move {
                Ok::<_, Infallible>(service_fn(move |request| {
                    info!("{} {}", request.method(), request.uri());
                    let app = app.clone();
                    let router = router.clone();

                    async move { router.handle(app, request).await.into_base_response() }
                }))
            }
        });

        let server = BaseServer::bind(&self.address).serve(service);

        println!(
            "Server running at: {}{}",
            "http://".bold(),
            self.address.to_string().bold()
        );
        println!();

        println!(
            "{}",
            "Valar is still under development. Use at your own risk."
                .yellow()
                .italic()
        );
        println!(
            "{}",
            "Please report any bugs or feature requests at:"
                .yellow()
                .italic()
        );
        println!(
            "{}",
            "https://github.com/ConsoleTVs/Valar/issues."
                .yellow()
                .italic()
        );
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
