use boilerplate::App;
use valar::Application;

#[tokio::test]
async fn sample() {
    let app = App::fake().await.unwrap();
    let response = app.get("/").send().await;

    response.assert_ok().assert_is_json();
}
