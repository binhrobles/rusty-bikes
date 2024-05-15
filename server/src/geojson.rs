/// Middleware when formatting Graph structures into Geojson
use crate::osm::graph::{Route, TraversalSegment};

pub fn aggregate_traversal_geoms(traversal: &[TraversalSegment]) -> Result<String, anyhow::Error> {
    Ok(geojson::ser::to_feature_collection_string(traversal)?)
}

pub fn route_geom(route: &Route) -> Result<String, anyhow::Error> {
    Ok(geojson::ser::to_feature_string(route)?)
}
