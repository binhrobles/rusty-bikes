/// Structs and logic specific to traversing a Graph
use super::Graph;
use crate::osm::{Distance, Neighbor, Node, NodeId, WayId};
use anyhow::anyhow;
use geo::{HaversineDistance, Line, Point};
use serde::Serialize;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

pub const START_NODE_ID: NodeId = -1;
pub const END_NODE_ID: NodeId = -2;

pub type Depth = usize;
pub type Route = Vec<TraversalSegment>;
pub type Traversal = Vec<TraversalSegment>;

#[derive(Clone, Debug, Serialize)]
pub struct TraversalSegment {
    pub from: NodeId,
    pub to: NodeId,
    pub way: WayId,

    #[serde(serialize_with = "geojson::ser::serialize_geometry")]
    pub geometry: Line,

    // segment metadata for weighing / constructing the route
    pub depth: Depth,
    pub length: Distance,
    pub distance_so_far: Distance,
    // cost
}

/// TraversalSegments are roughly equivalent when they connect the same points along the same way
impl PartialEq for TraversalSegment {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.from == other.from && self.to == other.to && self.way == other.way
    }
}

// manually implementing Eq so that the `geometry` field isn't implicitly added to the derived
// implementation
impl Eq for TraversalSegment {}

/// TraversalSegment comparisons make use of distance_so_far from traversal start
/// TODO: eventually cost
impl PartialOrd for TraversalSegment {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.distance_so_far.partial_cmp(&other.distance_so_far)
    }
}

impl Ord for TraversalSegment {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        // Notice that the we flip the ordering on costs.
        // In case of a tie we compare positions - this step is necessary
        // to make implementations of `PartialEq` and `Ord` consistent.
        other.distance_so_far.cmp(&self.distance_so_far)
    }
}

pub struct TraversalSegmentBuilder {
    from: NodeId,
    to: NodeId,
    way: WayId,

    geometry: Line,

    depth: Depth,
    length: Distance,
    distance_so_far: Distance,
}

impl TraversalSegmentBuilder {
    pub fn new_from_neighbor(from: &Node, to: &Neighbor) -> Self {
        Self {
            from: from.id,
            to: to.node.id,
            way: to.way,

            geometry: Line::new(from.geometry, to.node.geometry),

            depth: 0,
            length: to.distance,
            distance_so_far: to.distance,
        }
    }

    pub fn new_from_node(from: &Node, to: &Node, way: WayId) -> Self {
        let length = from.geometry.haversine_distance(&to.geometry) as Distance;
        Self {
            from: from.id,
            to: to.id,
            way,

            geometry: Line::new(from.geometry, to.geometry),

            depth: 0,
            length,
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

            geometry: self.geometry,

            length: self.length,
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
    pub queue: BinaryHeap<TraversalSegment>,
    pub came_from: HashMap<NodeId, TraversalSegment>,
}

impl TraversalContext {
    pub fn new() -> Self {
        Self {
            queue: BinaryHeap::new(),
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
                        .with_prev_distance(current.distance_so_far)
                        .build();
                    context.queue.push_back(segment.clone());
                    segment
                });
        }
    }

    Err(anyhow!("Traversal failed"))
}
