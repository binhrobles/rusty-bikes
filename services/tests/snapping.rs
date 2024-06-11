use rusty_router::graph::Graph;
use geo::Point;

#[test]
fn get_corner_coords_in_lot() -> Result<(), anyhow::Error> {
    let graph = Graph::new()?;
    graph.guess_neighbors(Point::new(-73.9791875, 40.690155), None)?;
    Ok(())
}

#[test]
fn get_corner_coords_on_road() -> Result<(), anyhow::Error> {
    let graph = Graph::new()?;
    let neighbors = graph.guess_neighbors(Point::new(-73.9790797, 40.6898084), None)?;
    println!("{:#?}", neighbors);
    Ok(())
}
