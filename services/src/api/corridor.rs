/// Extracts near-optimal "corridor" segments from A* traversal data.
/// These are alternate paths that experienced cyclists might take on a grid.
///
/// Key challenge: `came_from` is a tree (one predecessor per node), so alternate
/// paths never literally rejoin route nodes via stored edges. We query the actual
/// graph DB to find which candidate nodes have edges back to route nodes.
use crate::graph::{Cost, GraphRepository, TraversalSegment};
use crate::osm::NodeId;
use geo::{HaversineDistance, Point};
use serde_json::Value;
use std::collections::{HashMap, HashSet, VecDeque};

use crate::graph::{END_NODE_ID, START_NODE_ID};

const MAX_COST_RATIO: f32 = 2.0;
const MAX_DISTANCE_METERS: f64 = 1000.0;

pub fn extract_corridor<'a>(
    traversal: &'a [TraversalSegment],
    route: &[TraversalSegment],
    optimal_cost: Cost,
    db: &dyn GraphRepository,
) -> Vec<&'a TraversalSegment> {
    let cost_threshold = optimal_cost * MAX_COST_RATIO;

    // Build set of route edges for exclusion, and route node IDs
    let route_edges: HashSet<(NodeId, NodeId)> =
        route.iter().map(|s| (s.from.id, s.to.id)).collect();
    let route_node_ids: HashSet<NodeId> = route
        .iter()
        .flat_map(|s| [s.from.id, s.to.id])
        .collect();
    let route_node_ids_vec: Vec<NodeId> = route_node_ids.iter().copied().collect();

    // Pre-compute route segment midpoints for broad proximity filter
    let route_midpoints: Vec<Point> = route
        .iter()
        .map(|s| {
            Point::new(
                (s.geometry.start.x + s.geometry.end.x) / 2.0,
                (s.geometry.start.y + s.geometry.end.y) / 2.0,
            )
        })
        .collect();

    // Phase 1: filter candidates by cost, proximity, and not-on-route
    let candidates: Vec<&'a TraversalSegment> = traversal
        .iter()
        .filter(|seg| {
            if seg.from.id == START_NODE_ID || seg.to.id == END_NODE_ID {
                return false;
            }
            if route_edges.contains(&(seg.from.id, seg.to.id)) {
                return false;
            }
            if seg.cost > cost_threshold {
                return false;
            }
            let mid = Point::new(
                (seg.geometry.start.x + seg.geometry.end.x) / 2.0,
                (seg.geometry.start.y + seg.geometry.end.y) / 2.0,
            );
            route_midpoints
                .iter()
                .any(|rp| rp.haversine_distance(&mid) < MAX_DISTANCE_METERS)
        })
        .collect();

    // Phase 2: find reconnection nodes via DB query.
    //
    // `came_from` is a tree — alternate paths that rejoin the route don't have
    // their reconnection edge stored. Query the actual graph to find which
    // candidate `to` nodes have a real edge back to a route node.
    //
    // These "reconnection nodes" become seeds for the backward BFS, alongside
    // the actual route nodes.

    // Collect all unique candidate `to` node IDs
    let candidate_to_nodes: Vec<NodeId> = candidates
        .iter()
        .map(|s| s.to.id)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    // Query DB: which candidate endpoints have a direct edge to a route node?
    let reconnection_nodes = db
        .get_nodes_with_edge_to(&candidate_to_nodes, &route_node_ids_vec)
        .unwrap_or_default();

    // Expanded seeds = route nodes + reconnection nodes
    let mut backward_seeds: HashSet<NodeId> = route_node_ids.clone();
    backward_seeds.extend(&reconnection_nodes);

    // Build directed adjacency from candidates
    let mut forward_adj: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
    let mut backward_adj: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
    for seg in &candidates {
        forward_adj.entry(seg.from.id).or_default().push(seg.to.id);
        backward_adj.entry(seg.to.id).or_default().push(seg.from.id);
    }

    // Forward BFS from route nodes: which candidate nodes are reachable?
    let reachable_from_route = bfs(&route_node_ids, &forward_adj);

    // Backward BFS from route + reconnection nodes: which candidate nodes
    // can reach the route (through the reconnection edges)?
    let can_reach_route = bfs(&backward_seeds, &backward_adj);

    candidates
        .into_iter()
        .filter(|seg| {
            reachable_from_route.contains(&seg.from.id) && can_reach_route.contains(&seg.to.id)
        })
        .collect()
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
