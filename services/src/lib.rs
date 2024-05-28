pub mod osm;
pub mod geojson;

use crate::osm::Graph;
use query_map::QueryMap;
use geo::Point;
use osm::traversal::TraversalSegment;
use serde::Deserialize;
use tracing::error;
use anyhow::anyhow;

#[derive(Debug, Deserialize)]
pub struct TraversalParams {
    lat: f64,
    lon: f64,
    depth: usize,
}

/// A helper for parsing TraversalParams from QueryStrings
impl TryFrom<&QueryMap> for TraversalParams {
    type Error = anyhow::Error;

    // TODO: DRY or a lib?
    fn try_from(query_map: &QueryMap) -> Result<Self, Self::Error> {
        let lon = query_map
            .first("lon")
            .ok_or_else(|| anyhow!("missing lon"))?
            .parse::<f64>()
            .map_err(|_| anyhow!("invalid lon"))?;
        let lat = query_map
            .first("lat")
            .ok_or_else(|| anyhow!("missing lat"))?
            .parse::<f64>()
            .map_err(|_| anyhow!("invalid lat"))?;
        let depth = query_map
            .first("depth")
            .ok_or_else(|| anyhow!("missing depth"))?
            .parse::<usize>()
            .map_err(|_| anyhow!("invalid depth"))?;

        Ok(Self { lon, lat, depth })
    }
}

/// performs a
pub async fn traverse(graph: &Graph, params: TraversalParams) -> Result<Vec<TraversalSegment>, anyhow::Error> {
    let starting_coord = Point::new(params.lon, params.lat);

    graph
        .traverse_from(starting_coord, params.depth)
        .map_err(|e| {
            error!("Routing Error: {e}");
            e
        })
}
