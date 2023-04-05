use std::sync::Arc;

use valar::http::middleware::Logger;
use valar::http::middleware::Session;
use valar::routing::route::Builder as Route;
use valar::routing::router::Compiled;
use valar::routing::router::Error;
use valar::routing::Router;

use crate::app::App;
// use crate::http::controllers::dashboard;
use crate::http::controllers::counter;

impl App {
    pub fn router() -> Result<Arc<Router<Self, Compiled>>, Error> {
        let web = Route::group([
            Route::get("/", counter::show),
            Route::post("/", counter::increment),
        ]);

        let router = Router::from_iter([web.middleware(Session)]).middleware(Logger);
        let router = Arc::new(router.compile()?);

        Ok(router)
    }
}
