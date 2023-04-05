pub mod app;
pub mod http;
pub mod routes;

use std::sync::Arc;

use valar::http::Server;

use crate::app::App;

#[tokio::main]
async fn main() {
    let app = Arc::new(App::create().await);
    let router = App::router().unwrap();

    Server::builder()
        .address(([127, 0, 0, 1], 8080))
        .build()
        .start(app, router)
        .await;
}
