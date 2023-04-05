use std::sync::Arc;

use boilerplate::App;
use valar::http::Request;
use valar::http::Uri;

#[tokio::test]
async fn it_has_a_homepage() {
    let app = App {
        ..App::fake().await
    };
    let app = Arc::new(app);
    let router = App::router().unwrap();
    let request = Request::get(Uri::from_static("/"));
    let response = router.handle(app, request).await;

    response.assert_ok();
}
