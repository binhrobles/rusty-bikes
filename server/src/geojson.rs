// Collection of functions for formatting Graph structures into Geojson

use crate::osm::graph::TraversalGeom;

// reduces traversal geom lines to their minimal form
// TODO: either implement this, or do reductions at traversal-time
pub fn aggregate_traversal_geoms(traversal: &mut [TraversalGeom]) -> Result<String, anyhow::Error> {
    Ok(geojson::ser::to_feature_collection_string(traversal)?)
}
