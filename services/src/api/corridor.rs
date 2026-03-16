use crate::graph::{Cost, TraversalSegment};
use crate::osm::NodeId;
use geo::{HaversineDistance, Point};
use serde_json::Value;
use std::collections::{HashMap, HashSet, VecDeque};
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

    // Route nodes and edges for exclusion
    let route_edges: HashSet<(NodeId, NodeId)> =
        route.iter().map(|s| (s.from.id, s.to.id)).collect();
    let route_node_ids: HashSet<NodeId> = route.iter().flat_map(|s| [s.from.id, s.to.id]).collect();

    // Pre-compute route segment midpoints and accumulated costs for local comparison
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

    // Phase 1: filter candidates by cost, proximity, not-on-route, and backward reachability
    let candidates: Vec<&'a TraversalSegment> = forward_traversal
        .iter()
        .filter(|seg| {
            if seg.from.id == START_NODE_ID || seg.to.id == END_NODE_ID {
                return false;
            }
            if route_edges.contains(&(seg.from.id, seg.to.id)) {
                return false;
            }

            // Must be reachable from finish (backward A* tree)
            if !backward_reachable.contains(&seg.to.id) {
                return false;
            }

            let mid = Point::new(
                (seg.geometry.start.x + seg.geometry.end.x) / 2.0,
                (seg.geometry.start.y + seg.geometry.end.y) / 2.0,
            );

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
        "corridor phase1: {} candidates from {} forward, {} backward reachable",
        candidates.len(),
        forward_traversal.len(),
        backward_reachable.len(),
    );

    // Phase 2: BFS connectivity — only keep candidates reachable from the route
    // through other candidates (not through the route itself).
    //
    // Without this, we get isolated floating segments that passed the cost/proximity
    // filters but don't form connected paths from the route.

    // Build forward adjacency from candidates only
    let mut forward_adj: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
    for seg in &candidates {
        forward_adj.entry(seg.from.id).or_default().push(seg.to.id);
    }

    // Forward BFS from route nodes through candidate adjacency
    let reachable_from_route = bfs(&route_node_ids, &forward_adj);

    debug!(
        "corridor phase2: {} nodes reachable from route through candidates",
        reachable_from_route.len(),
    );

    let result: Vec<_> = candidates
        .into_iter()
        .filter(|seg| reachable_from_route.contains(&seg.from.id))
        .collect();

    debug!(
        "corridor result: {} segments after connectivity filter",
        result.len()
    );

    result
}

/// BFS from a set of seed nodes through a directed adjacency list.
fn bfs(seeds: &HashSet<NodeId>, adj: &HashMap<NodeId, Vec<NodeId>>) -> HashSet<NodeId> {
    let mut visited: HashSet<NodeId> = seeds.clone();
    let mut queue: VecDeque<NodeId> = seeds.iter().copied().collect();
    while let Some(node) = queue.pop_front() {
        if let Some(neighbors) = adj.get(&node) {
            for &neighbor in neighbors {
                if visited.insert(neighbor) {
                    queue.push_back(neighbor);
                }
            }
        }
    }
    visited
}

/// Serialize corridor segments to a GeoJSON FeatureCollection Value.
pub fn serialize_corridor(segments: &[&TraversalSegment]) -> Result<Value, anyhow::Error> {
    let owned: Vec<TraversalSegment> = segments.iter().map(|s| (*s).clone()).collect();
    Ok(serde_json::from_str(
        &geojson::ser::to_feature_collection_string(&owned)?,
    )?)
}
