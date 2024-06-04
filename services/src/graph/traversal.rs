/// Structs and logic specific to traversing a Graph
use super::{Cost, CostModel, Graph};
use crate::osm::{serialize_node_simple, Distance, Neighbor, Node, NodeId, WayId};
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
    #[serde(serialize_with = "serialize_node_simple")]
    pub from: Node,
    #[serde(serialize_with = "serialize_node_simple")]
    pub to: Node,
    pub way: WayId,

    #[serde(serialize_with = "geojson::ser::serialize_geometry")]
    pub geometry: Line,

    // segment metadata for weighing / constructing the route
    pub depth: Depth,
    pub length: Distance,
    pub distance_so_far: Distance,

    pub cost: Cost,
}

/// TraversalSegments are equivalent when they connect the same points along the same way
impl PartialEq for TraversalSegment {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.from == other.from && self.to == other.to && self.way == other.way
    }
}

// manually implementing Eq so that the `geometry`, `from`, and `to` fields aren't
// implicitly added to the derived implementation
impl Eq for TraversalSegment {}

impl PartialOrd for TraversalSegment {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// TraversalSegment comparisons make use of distance_so_far from traversal start
/// TODO: eventually use cost for sorting
impl Ord for TraversalSegment {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        other.distance_so_far.total_cmp(&self.distance_so_far)
    }
}

pub struct TraversalSegmentBuilder {
    from: Node,
    to: Node,
    way: WayId,

    geometry: Line,

    depth: Depth,
    length: Distance,
    distance_so_far: Distance,
    cost: Cost,
}

impl TraversalSegmentBuilder {
    pub fn new_from_neighbor(from: &Node, to: &Neighbor) -> Self {
        Self {
            from: *from,
            to: to.node,
            way: to.way,

            geometry: Line::new(from.geometry, to.node.geometry),

            depth: 0,
            length: to.distance,
            distance_so_far: to.distance,
            cost: 0.0,
        }
    }

    pub fn new_from_node(from: &Node, to: &Node, way: WayId) -> Self {
        let length = from.geometry.haversine_distance(&to.geometry) as Distance;
        Self {
            from: *from,
            to: *to,
            way,

            geometry: Line::new(from.geometry, to.geometry),

            depth: 0,
            length,
            distance_so_far: length,
            cost: 0.0,
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

    pub fn calculate_cost(
        mut self,
        cost_model: &CostModel,
        graph: &Graph,
    ) -> Result<Self, anyhow::Error> {
        self.cost = cost_model.calculate_cost(graph, self.way)?;
        Ok(self)
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
            cost: self.cost,
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
    pub cost_model: CostModel,
}

impl TraversalContext {
    pub fn new() -> Self {
        Self {
            queue: BinaryHeap::new(),
            came_from: HashMap::new(),
            cost_model: CostModel::default(),
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
        context.queue.push(segment.clone());
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
    while let Some(current) = context.queue.pop() {
        if target_neighbor_node_ids.contains(&current.to.id) {
            // on exit, append the final segment to the ending node
            let segment = TraversalSegment::build_to_node(&current.to, end_node, current.way)
                .with_depth(current.depth + 1)
                .with_prev_distance(current.distance_so_far)
                .calculate_cost(&context.cost_model, graph)?
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
                        .calculate_cost(&context.cost_model, graph)
                        .unwrap() // TODO: not this
                        .build();
                    context.queue.push(segment.clone());
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
    while let Some(current) = context.queue.pop() {
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
                        .calculate_cost(&context.cost_model, graph)
                        .unwrap() // TODO: not this
                        .build();
                    context.queue.push(segment.clone());
                    segment
                });
        }
    }

    Err(anyhow!("Traversal failed"))
}
