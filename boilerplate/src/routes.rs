mod api;
mod web;

use crate::App;
use api::api;
use valar::routing::Routable;
use valar::routing::Router;
use web::web;

impl Routable for App {
    type Application = App;

    fn router() -> Router<Self::Application> {
        Router::default().through(web).through(api)
    }
}
