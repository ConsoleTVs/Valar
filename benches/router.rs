use criterion::{black_box, criterion_group, criterion_main, Criterion};
use valar::http::{Method, Uri};
use valar::routing::Router;

struct Context {
    // pub name: String,
}

fn router_matcher_match(c: &mut Criterion) {
    let mut router = Router::<Context>::default();

    router.get("/", move |_| async move { unimplemented!() });

    let matcher = router.into_matcher().unwrap();
    let uri = Uri::from_static("/");
    let method = Method::GET;

    c.bench_function("router matcher match", |b| {
        b.iter(|| matcher.matches(black_box(&method), black_box(&uri)))
    });
}

criterion_group!(benches, router_matcher_match);
criterion_main!(benches);
