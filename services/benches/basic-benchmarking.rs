use geo::Point;
use rusty_router::graph::Graph;
use std::time::Instant;

#[test]
fn run_a_long_route_lots_of_times() -> Result<(), anyhow::Error> {
    let graph = Graph::new().unwrap();
    let now = Instant::now();

    for _i in 0..20 {
        graph.route_between(
            Point::new(-73.98949, 40.75376),
            Point::new(-73.92632, 40.64338),
            true,
            None,
            None,
        )?;
    }
    println!("Routing took: {}", now.elapsed().as_millis());
    println!("avg: {}", now.elapsed().as_millis() / 100);

    Ok(())
}
