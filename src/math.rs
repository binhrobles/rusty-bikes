use crate::db::Location;

/// get the distance b/w two locations, in cartesian units
pub fn distance(p1: Location, p2: Location) -> f32 {
    ((p2.lat - p1.lat).powi(2) + (p2.lon - p1.lon).powi(2)).sqrt()
}
