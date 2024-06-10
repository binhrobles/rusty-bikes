/// Structs and logic specific to traversing a Graph
use super::{Cost, CostModel, CostModelConfiguration, Graph};
use crate::osm::{
    serialize_node_simple, Cycleway, Distance, Neighbor, Node, NodeId, Road, WayId, WayLabels,
};
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
    #[serde(serialize_with = "serialize_node_simple", rename(serialize = "f"))]
    pub from: Node,
    #[serde(serialize_with = "serialize_node_simple", rename(serialize = "t"))]
    pub to: Node,
    #[serde(rename(serialize = "w"))]
    pub way: WayId,

    #[serde(serialize_with = "geojson::ser::serialize_geometry")]
    pub geometry: Line,

    // segment metadata for weighing / constructing the route
    #[serde(rename(serialize = "d"))]
    pub depth: Depth,
    #[serde(rename(serialize = "l"))]
    pub length: Distance,
    #[serde(rename(serialize = "da"))]
    pub distance_so_far: Distance,
    #[serde(rename(serialize = "wl"))]
    labels: WayLabels,

    #[serde(rename(serialize = "c"))]
    pub cost: Cost,
    #[serde(rename(serialize = "ca"))]
    pub cost_so_far: Cost,
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

/// TraversalSegment comparisons make use of cost_so_far from traversal start
impl Ord for TraversalSegment {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        other.cost_so_far.total_cmp(&self.cost_so_far)
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
    labels: WayLabels,
    cost: Cost,
    cost_so_far: Cost,
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
            labels: (Cycleway::Shared, Road::Collector, false),
            cost: 0.0,
            cost_so_far: 0.0,
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
            labels: (Cycleway::Shared, Road::Collector, false),
            cost: 0.0,
            cost_so_far: 0.0,
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
        cost_so_far: Cost,
    ) -> Result<Self, anyhow::Error> {
        let (cost, labels) = cost_model.calculate_cost(graph, self.way, self.length)?;
        self.cost = cost;
        self.cost_so_far = cost_so_far + self.cost;
        self.labels = labels;
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
            labels: self.labels,
            cost: self.cost,
            cost_so_far: self.cost_so_far,
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
    pub fn new(cost_params: Option<CostModelConfiguration>) -> Self {
        Self {
            queue: BinaryHeap::new(),
            came_from: HashMap::new(),
            cost_model: CostModel::default(),
        }
    }
}

impl Default for TraversalContext {
    fn default() -> Self {
        Self::new(None)
    }
}

/// initializes the structures required to traverse this graph, leveraging the guess_neighbors
/// function to snap the starting Point into the graph
pub fn initialize_traversal(
    graph: &Graph,
    start: &Point,
    cost_model_configuration: Option<CostModelConfiguration>,
) -> Result<TraversalContext, anyhow::Error> {
    let start_node = Node::new(START_NODE_ID, start);
    let starting_neighbors = graph.guess_neighbors(*start)?;

    let mut context = TraversalContext::new(cost_model_configuration);

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
                .calculate_cost(&context.cost_model, graph, current.cost_so_far)?
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
                        .calculate_cost(&context.cost_model, graph, current.cost_so_far)
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
                        .calculate_cost(&context.cost_model, graph, current.cost_so_far)
                        .unwrap() // TODO: not this
                        .build();
                    context.queue.push(segment.clone());
                    segment
                });
        }
    }

    Err(anyhow!("Traversal failed"))
}
