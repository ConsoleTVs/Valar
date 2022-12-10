#[cfg(test)]
mod tests {
    use valar::http::Method;
    use valar::routing::Router;

    struct Context;

    #[test]
    fn it_can_create_routers() {
        let router = Router::<Context>::default();

        assert_eq!(router.routes().len(), 0);
    }

    #[test]
    fn it_can_create_routers_with_routes() {
        let mut router = Router::<Context>::default();

        router.get("/", move |_| async move { unimplemented!() });
        // router.post("/", |_| unimplemented!());

        assert_eq!(router.routes().len(), 1);
        assert!(router.has_route("/", &Method::GET));
        // assert!(router.has_route("/", &Method::POST));
    }
}
