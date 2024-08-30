use geo::Point;
use rusty_router::graph::Graph;

fn main() -> Result<(), anyhow::Error> {
    let graph = Graph::new().unwrap();

    for _i in 0..20 {
        graph.calculate_route(
            Point::new(-73.98949, 40.75376),
            Point::new(-73.92632, 40.64338),
            true,
            None,
            None,
        )?;
    }

    Ok(())
}
