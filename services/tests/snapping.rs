use geo::Point;
use rusty_router::graph::{GraphRepository, SqliteGraphRepository};

#[test]
fn get_corner_coords_in_lot() -> Result<(), anyhow::Error> {
    let db = SqliteGraphRepository::new()?;
    db.get_snapped_neighbors(Point::new(-73.9791875, 40.690155), None)?;
    Ok(())
}

#[test]
fn get_corner_coords_on_road() -> Result<(), anyhow::Error> {
    let db = SqliteGraphRepository::new()?;
    db.get_snapped_neighbors(Point::new(-73.9790797, 40.6898084), None)?;
    Ok(())
}
