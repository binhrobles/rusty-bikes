use rusty_router::{graph::Graph};
use geo::Point;

#[test]
fn get_liu_corner_coords_in_lot() -> Result<(), anyhow::Error> {
    let graph = Graph::new()?;
    graph.guess_neighbors(Point::new(-73.979187, 40.69015))?;
    Ok(())
}

#[test]
fn get_liu_corner_coords_on_road() -> Result<(), anyhow::Error> {
    let graph = Graph::new()?;
    graph.guess_neighbors(Point::new(-73.9790797, 40.6898084))?;
    Ok(())
}
