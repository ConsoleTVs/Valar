use crate::http::controllers::dashboard;
use crate::App;
use valar::routing::Router;

pub fn api(router: &mut Router<App>) {
    router.get("/", dashboard::index);
    router.get("/:id", dashboard::show);
}
