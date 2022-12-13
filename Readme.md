# Valar

This is an experimental, developer-centric async Rust web framework. This is a Proof Of Concept project not meant to be used in any real applications.

```rs
use valar::http::Request;
use valar::http::Response;
use valar::http::Result;
use valar::routing::Router;
use valar::Server;

struct Context {
    version: f32,
}

#[derive(serde::Serialize)]
struct Profile {
    id: i32,
}

async fn homepage(request: Request<Context>) -> Result {
    let version = request.context().version;

    Response::ok()
        .html(format!("<h1>Hello, world</h1><h2>Version: {version}</h2>"))
        .produce()
}

async fn profile(request: Request<Context>) -> Result {
    let id = request.parameter("id")?.parse::<i32>()?;

    Response::ok()
        .html(format!("<h1>User ID: {id}</h1>"))
        .produce()
}

async fn json_profile(request: Request<Context>) -> Result {
    let id = request.parameter("id")?.parse::<i32>()?;

    Response::ok().json(&Profile { id })?.produce()
}

#[tokio::main]
async fn main() -> std::result::Result<(), anyhow::Error> {
    env_logger::init();

    let context = Context { version: 1. };
    let mut router = Router::<Context>::default();

    router.get("/", homepage);
    router.get("/profile/:id", profile);
    router.get("/profile/:id/json", json_profile);

    let app = Server::builder()
        .address(([127, 0, 0, 1], 3000))
        .router(router)
        .build();

    app.serve(context).await;

    Ok(())
}

```
