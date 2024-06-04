/// Structs and logic specific to traversing a Graph
use super::Graph;
use crate::geojson::{serialize_float_as_int, serialize_node_simple};
use crate::osm::{Distance, Neighbor, Node, NodeId, WayId};
use anyhow::anyhow;
use geo::{HaversineDistance, Line, Point};
use geojson::ser::serialize_geometry;
use serde::Serialize;
use std::collections::{HashMap, VecDeque};

pub const START_NODE_ID: NodeId = -1;
pub const END_NODE_ID: NodeId = -2;

pub type Depth = usize;
pub type Route = Vec<TraversalSegment>;
pub type Traversal = Vec<TraversalSegment>;

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct TraversalSegment {
    #[serde(serialize_with = "serialize_node_simple")]
    pub from: Node,
    #[serde(serialize_with = "serialize_node_simple")]
    pub to: Node,

    #[serde(serialize_with = "serialize_geometry")]
    pub geometry: Line,

    // segment metadata for weighing / constructing the route
    pub way: WayId,
    pub depth: Depth,

    #[serde(serialize_with = "serialize_float_as_int")]
    pub length: Distance,
    #[serde(serialize_with = "serialize_float_as_int")]
    pub distance_so_far: Distance,
    // cost
}

pub struct TraversalSegmentBuilder {
    from: Node,
    to: Node,
    way: WayId,
    length: Distance,
    geometry: Line,

    depth: Depth,
    distance_so_far: Distance,
}

impl TraversalSegmentBuilder {
    pub fn new_from_neighbor(from: &Node, to: &Neighbor) -> Self {
        Self {
            from: *from,
            to: to.node,
            way: to.way,
            length: to.distance,
            geometry: Line::new(from.geometry, to.node.geometry),

            depth: 0,
            distance_so_far: to.distance,
        }
    }

    pub fn new_from_node(from: &Node, to: &Node, way: WayId) -> Self {
        let length = from.geometry.haversine_distance(&to.geometry);
        Self {
            from: *from,
            to: *to,
            way,
            length,
            geometry: Line::new(from.geometry, to.geometry),

            depth: 0,
            distance_so_far: length,
        }
    }

    pub fn with_depth(mut self, depth: Depth) -> Self {
        self.depth = depth;
        self
    }

    /// distance_so_far = this provided distance + initialized segment length
    pub fn with_prev_distance(mut self, distance: Distance) -> Self {
        self.distance_so_far += distance;
        self
    }

    pub fn build(self) -> TraversalSegment {
        TraversalSegment {
            from: self.from,
            to: self.to,
            way: self.way,
            length: self.length,
            geometry: self.geometry,

            depth: self.depth,
            distance_so_far: self.distance_so_far,
        }
    }
}

impl TraversalSegment {
    pub fn build_to_neighbor(from: &Node, to: &Neighbor) -> TraversalSegmentBuilder {
        TraversalSegmentBuilder::new_from_neighbor(from, to)
    }

    pub fn build_to_node(from: &Node, to: &Node, way: WayId) -> TraversalSegmentBuilder {
        TraversalSegmentBuilder::new_from_node(from, to, way)
    }
}

pub struct TraversalContext {
    pub queue: VecDeque<TraversalSegment>,
    pub came_from: HashMap<NodeId, TraversalSegment>,
}

impl TraversalContext {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            came_from: HashMap::new(),
        }
    }
}

impl Default for TraversalContext {
    fn default() -> Self {
        Self::new()
    }
}

/// initializes the structures required to traverse this graph, leveraging the guess_neighbors
/// function to snap the starting Point into the graph
pub fn initialize_traversal(
    graph: &Graph,
    start: &Point,
) -> Result<TraversalContext, anyhow::Error> {
    let start_node = Node::new(START_NODE_ID, start);
    let starting_neighbors = graph.guess_neighbors(*start)?;

    let mut context = TraversalContext::new();

    for neighbor in starting_neighbors {
        let segment = TraversalSegment::build_to_neighbor(&start_node, &neighbor).build();
        context.queue.push_back(segment.clone());
        context.came_from.insert(neighbor.node.id, segment);
    }

    Ok(context)
}

/// Generates a collection of all TraversalSegments examined while routing between the start and
/// end Points. TraversalSegments will be decorated with both the depth of the traversal and
/// the cost assigned, given the designated cost model
pub fn traverse_between(
    graph: &Graph,
    context: &mut TraversalContext,
    target_neighbor_node_ids: &[NodeId],
    end_node: &Node,
) -> Result<(), anyhow::Error> {
    while let Some(current) = context.queue.pop_front() {
        if target_neighbor_node_ids.contains(&current.to.id) {
            // on exit, append the final segment to the ending node
            let segment = TraversalSegment::build_to_node(&current.to, end_node, current.way)
                .with_depth(current.depth + 1)
                .with_prev_distance(current.distance_so_far)
                .build();
            context.came_from.insert(END_NODE_ID, segment);
            return Ok(());
        }

        let adjacent_neighbors = graph.get_neighbors(current.to.id)?;

        for neighbor in adjacent_neighbors {
            context
                .came_from
                .entry(neighbor.node.id)
                .or_insert_with(|| {
                    let segment = TraversalSegment::build_to_neighbor(&current.to, &neighbor)
                        .with_depth(current.depth + 1)
                        .with_prev_distance(current.distance_so_far)
                        .build();
                    context.queue.push_back(segment.clone());
                    segment
                });
        }
    }

    Err(anyhow!("Traversal failed"))
}

/// Return a collection of TraversalSegments from traversing the Graph from the start point to
/// the depth specified
pub fn traverse_from(
    graph: &Graph,
    context: &mut TraversalContext,
    max_depth: usize,
) -> Result<(), anyhow::Error> {
    while let Some(current) = context.queue.pop_front() {
        if current.depth == max_depth {
            return Ok(());
        }

        let adjacent_neighbors = graph.get_neighbors(current.to.id)?;

        for neighbor in adjacent_neighbors {
            context
                .came_from
                .entry(neighbor.node.id)
                .or_insert_with(|| {
                    let segment = TraversalSegment::build_to_neighbor(&current.to, &neighbor)
                        .with_depth(current.depth + 1)
                        .build();
                    context.queue.push_back(segment.clone());
                    segment
                });
        }
    }

    Err(anyhow!("Traversal failed"))
}