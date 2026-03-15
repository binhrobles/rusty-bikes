use crate::graph::{Cost, TraversalSegment};
use crate::osm::NodeId;
use geo::{HaversineDistance, Point};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use tracing::debug;

use crate::graph::{END_NODE_ID, START_NODE_ID};

/// Combined cost of a detour through a corridor node can be at most this ratio
/// above the optimal route cost. 1.5 = 50% more expensive than optimal.
const CORRIDOR_COST_RATIO: f32 = 1.5;
const MAX_DISTANCE_METERS: f64 = 1000.0;

pub fn extract_corridor<'a>(
    forward_traversal: &'a [TraversalSegment],
    backward_traversal: &[TraversalSegment],
    route: &[TraversalSegment],
    optimal_cost: Cost,
) -> Vec<&'a TraversalSegment> {
    if optimal_cost <= 0.0 {
        return vec![];
    }

    // Build backward cost lookup: node_id -> cost from finish exploration
    let backward_costs: HashMap<NodeId, Cost> = backward_traversal
        .iter()
        .map(|s| (s.to.id, s.cost))
        .collect();

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

            // Must exist in backward exploration (reachable from finish)
            let backward_cost = match backward_costs.get(&seg.to.id) {
                Some(c) => *c,
                None => return false,
            };

            // Proximity filter
            let mid = Point::new(
                (seg.geometry.start.x + seg.geometry.end.x) / 2.0,
                (seg.geometry.start.y + seg.geometry.end.y) / 2.0,
            );
            let nearest_dist = route_refs
                .iter()
                .map(|(rp, _)| rp.haversine_distance(&mid))
                .min_by(|a, b| a.total_cmp(b))
                .unwrap_or(f64::MAX);

            if nearest_dist >= MAX_DISTANCE_METERS {
                return false;
            }

            // Combined cost ratio: (cost from start to node + cost from finish to node) / optimal
            let combined_cost = seg.cost + backward_cost;
            combined_cost / optimal_cost <= CORRIDOR_COST_RATIO
        })
        .collect();

    debug!(
        "corridor: {} segments from {} forward x {} backward, optimal_cost={:.1}",
        result.len(),
        forward_traversal.len(),
        backward_traversal.len(),
        optimal_cost,
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
