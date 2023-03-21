use std::sync::Arc;

use valar::http::middleware::Logger;
use valar::routing::route::Builder as Route;
use valar::routing::router::Compiled;
use valar::routing::router::Error;
use valar::routing::Router;

use crate::http::controllers::dashboard;
use crate::App;

impl App {
    pub fn router() -> Result<Arc<Router<Self, Compiled>>, Error> {
        let api = Route::group([
            Route::get("/", dashboard::index),
            Route::get("/user/:id", dashboard::show).where_parameter("id", "[0-9]+"),
        ]);

        let router = Router::from_iter([api]).middleware(Logger);
        let router = Arc::new(router.compile()?);

        Ok(router)
    }
}
