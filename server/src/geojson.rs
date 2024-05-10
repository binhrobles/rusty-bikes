// Collection of functions for formatting Graph structures into Geojson
use crate::osm::graph::Route;

// reduces traversal geom lines to their minimal form
// TODO: either implement this, or do reductions at traversal-time
pub fn aggregate_traversal_geoms(traversal: &[Route]) -> Result<String, anyhow::Error> {
    Ok(geojson::ser::to_feature_collection_string(traversal)?)
}

pub fn route_geom(route: &Route) -> Result<String, anyhow::Error> {
    Ok(geojson::ser::to_feature_string(route)?)
}
