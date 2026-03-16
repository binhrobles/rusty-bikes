use crate::graph::{Cost, TraversalSegment};
use crate::osm::NodeId;
use geo::{HaversineDistance, Point};
use serde_json::Value;
use std::collections::HashSet;
use tracing::debug;

use crate::graph::{END_NODE_ID, START_NODE_ID};

/// A corridor segment's accumulated cost can be at most this ratio above
/// the route's cost at the nearest point. E.g. 2.0 = up to 2x as expensive locally.
const MAX_LOCAL_COST_RATIO: f32 = 2.0;
const MAX_DISTANCE_METERS: f64 = 1000.0;

pub fn extract_corridor<'a>(
    forward_traversal: &'a [TraversalSegment],
    backward_traversal: &[TraversalSegment],
    route: &[TraversalSegment],
) -> Vec<&'a TraversalSegment> {
    // Backward A* reachability: nodes the backward tree explored can plausibly reach the finish
    let backward_reachable: HashSet<NodeId> = backward_traversal.iter().map(|s| s.to.id).collect();

    // Build set of route edges for exclusion
    let route_edges: HashSet<(NodeId, NodeId)> =
        route.iter().map(|s| (s.from.id, s.to.id)).collect();

    // Pre-compute route segment midpoints for proximity filtering
    let route_refs: Vec<(Point, Cost)> = route
        .iter()
        .map(|s| {
            (
                Point::new(
                    (s.geometry.start.x + s.geometry.end.x) / 2.0,
                    (s.geometry.start.y + s.geometry.end.y) / 2.0,
                ),
                s.cost,
            )
        })
        .collect();

    let result: Vec<&'a TraversalSegment> = forward_traversal
        .iter()
        .filter(|seg| {
            // Exclude virtual nodes and route edges
            if seg.from.id == START_NODE_ID || seg.to.id == END_NODE_ID {
                return false;
            }
            if route_edges.contains(&(seg.from.id, seg.to.id)) {
                return false;
            }

            // Must exist in backward A* tree (can plausibly reach finish)
            if !backward_reachable.contains(&seg.to.id) {
                return false;
            }

            // Proximity + local cost filter
            let mid = Point::new(
                (seg.geometry.start.x + seg.geometry.end.x) / 2.0,
                (seg.geometry.start.y + seg.geometry.end.y) / 2.0,
            );

            // Find nearest route segment: check distance and compare costs locally
            let nearest = route_refs
                .iter()
                .map(|(rp, rc)| (rp.haversine_distance(&mid), *rc))
                .min_by(|a, b| a.0.total_cmp(&b.0));

            match nearest {
                Some((dist, local_route_cost)) => {
                    dist < MAX_DISTANCE_METERS
                        && seg.cost <= local_route_cost * MAX_LOCAL_COST_RATIO
                }
                None => false,
            }
        })
        .collect();

    debug!(
        "corridor: {} segments from {} forward x {} backward",
        result.len(),
        forward_traversal.len(),
        backward_traversal.len(),
    );

    result
}

/// Serialize corridor segments to a GeoJSON FeatureCollection Value.
pub fn serialize_corridor(segments: &[&TraversalSegment]) -> Result<Value, anyhow::Error> {
    let owned: Vec<TraversalSegment> = segments.iter().map(|s| (*s).clone()).collect();
    Ok(serde_json::from_str(
        &geojson::ser::to_feature_collection_string(&owned)?,
    )?)
}
