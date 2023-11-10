use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;

use colored::Colorize;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use log::debug;
use log::info;
use tokio::net::TcpListener;

use crate::routing::router::Compiled;
use crate::routing::Router;

pub struct Server {
    address: SocketAddr,
}

impl Server {
    pub fn builder() -> ServerBuilder {
        ServerBuilder::new()
    }

    async fn handler(
        _: Request<hyper::body::Incoming>,
    ) -> Result<Response<Full<Bytes>>, Infallible> {
        Ok(BaseResponse::new(Full::new(Bytes::from("Hello, World!"))))
    }

    pub async fn start<App: Send + Sync + 'static>(
        &self,
        app: Arc<App>,
        router: Arc<Router<App, Compiled>>,
    ) {
        println!("{} • Supercharged Async Web Framework", "Valar".bold());
        println!("{}", "Lambda Studio • https://λ.studio".italic().dimmed());
        println!();

        let Ok(listener) = TcpListener::bind(&self.address).await else {
            eprintln!("Failed to bind to address: {}", self.address);
            return;
        };

        tokio::task::spawn(async move {
            loop {
                let Ok((stream, _)) = listener.accept().await else {
                    eprintln!("Failed to accept connection");
                    continue;
                };

                let io = TokioIo::new(stream);

                if let Err(err) = http1::Builder::new()
                    // `service_fn` converts our function in a `Service`
                    .serve_connection(io, service_fn(handler))
                    .await
                {
                    println!("Error serving connection: {:?}", err);
                }
            }
        });

        // let service = make_service_fn(move |conn| {
        //     debug!("Incoming connection: {:?}", conn);
        //     let app = app.clone();
        //     let router = router.clone();

        //     async move {
        //         Ok::<_, Infallible>(service_fn(move |request| {
        //             info!("{} {}", request.method(),
        // request.uri());             let app =
        // app.clone();             let router =
        // router.clone();

        //             async move { router.handle_base(app,
        // request).await.into_base_response() }         }))
        //     }
        // });

        // let server =
        // BaseServer::bind(&self.address).serve(service);

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

        // if let Err(e) = server.await {
        //     eprintln!("server error: {}", e);
        // }
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
