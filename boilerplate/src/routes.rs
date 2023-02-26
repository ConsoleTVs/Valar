use valar::http::middleware::Logger;
use valar::routing::route::Builder as Route;
use valar::routing::Routable;
use valar::routing::Router;

use crate::http::controllers::dashboard;
use crate::App;

impl Routable for App {
    type Application = App;

    fn router() -> Router<Self::Application> {
        let api = Route::group([
            Route::get("/", dashboard::index),
            Route::get("/user/:id", dashboard::show).where_parameter("id", "[0-9]+"),
        ]);

        // let web = Route::group([
        //     Route::get("/", dashboard::index),
        //     Route::get("/other", dashboard::index),
        //     Route::get("/another", dashboard::index),
        // ]);

        Router::from_iter([api]).middleware(Logger)
    }
}
