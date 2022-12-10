use valar::http::Response;
use valar::routing::Router;
use valar::Server;

struct Context {}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    env_logger::init();
    let context = Context {};
    let mut router = Router::<Context>::default();

    router.get("/", move |_req| async move {
        Response::ok().body("Hello, world!").produce()
    });

    router.get("/profile/:id/create", move |_req| async move {
        Response::ok().html("<h1>User created!</h1>").produce()
    });

    router.get("/profile/:id", move |req| async move {
        let id = req.parameter("ids")?.parse::<i32>()?;

        Response::ok()
            .html(format!("<h1>User ID: {id}</h1>"))
            .produce()
    });

    let app = Server::builder()
        .address(([127, 0, 0, 1], 3000))
        .router(router)
        .build();

    app.serve(context).await;

    Ok(())
}
