use boilerplate::App;
use valar::http::Request;
use valar::http::Uri;

#[tokio::test]
async fn it_has_a_homepage() {
    let app = App::fake().await;
    let router = App::router().unwrap();

    let request = Request::get(Uri::from_static("/"));
    let response = router.handle(app.clone(), request).await;

    response.assert_ok().assert_is_json();
}
