use criterion::{criterion_group, criterion_main, Criterion};
use geo::Point;
use rusty_router::graph::Graph;

/// Cross-city route from the performance issue:
/// Upper West Side → Flatbush, Brooklyn (~10km)
fn bench_cross_city_route(c: &mut Criterion) {
    let graph = Graph::new().unwrap();
    let start = Point::new(-73.98002, 40.7751769);
    let end = Point::new(-73.9091492, 40.6270216);

    c.bench_function("cross_city_route", |b| {
        b.iter(|| {
            graph
                .calculate_route(start, end, false, None, Some(0.75))
                .unwrap()
        })
    });
}

criterion_group!(benches, bench_cross_city_route);
criterion_main!(benches);
