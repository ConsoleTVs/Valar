pub mod app;
pub mod http;
pub mod routes;

pub use app::App;
use valar::http::Server;

#[tokio::main]
async fn main() {
    let app = App::create().await;
    let router = App::router().unwrap();

    Server::builder()
        .address(([127, 0, 0, 1], 8080))
        .build()
        .start(app, router)
        .await;
}
