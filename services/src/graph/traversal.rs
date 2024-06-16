use super::{serialize_float_rounded, serialize_as_int, Cost, CostModel, Graph, Weight};
use crate::osm::{
    serialize_node_simple, Cycleway, Distance, Neighbor, Node, NodeId, Road, WayId, WayLabels,
};
use anyhow::anyhow;
use geo::{HaversineDistance, Line, Point};
use serde::Serialize;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::f32::{MAX, MIN};

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
    pub labels: WayLabels,

    #[serde(serialize_with = "serialize_float_rounded")]
    pub cost: Cost,
    #[serde(serialize_with = "serialize_float_rounded")]
    pub cost_factor: Cost,
    #[serde(serialize_with = "serialize_as_int")]
    pub cost_so_far: Cost,

    #[serde(serialize_with = "serialize_as_int")]
    pub heuristic: Weight,
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

/// TraversalSegment comparisons make use of cost_so_far from traversal start and a factor of the distance
/// remaining to the end node
/// TODO: do less in the cmp function and load more into the Segments themselves? this seems like a
///       lot of work to do inside a cmp which will be called (possibly recursively?) on each
///       PriorityQueue insert
impl Ord for TraversalSegment {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        (other.cost + other.heuristic).total_cmp(&(self.cost + self.heuristic))
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

pub struct TraversalSegmentBuilder {
    from: Node,
    to: Node,
    way: WayId,

    geometry: Line,

    depth: Depth,
    length: Distance,
    distance_so_far: Distance,
    labels: WayLabels,
    cost_factor: Cost,
    cost_so_far: Cost,
    heuristic: Weight,
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
            cost_factor: 0.0,
            cost_so_far: 0.0,
            heuristic: 0.0,
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
            cost_factor: 0.0,
            cost_so_far: 0.0,
            heuristic: 0.0,
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

    pub fn with_cost(
        mut self,
        cost_model: &CostModel,
        way_labels: &WayLabels,
        cost_so_far: Cost,
    ) -> Self {
        self.cost_factor = cost_model.calculate_cost(way_labels);
        self.cost_so_far = cost_so_far;
        self.labels = *way_labels;
        self
    }

    /// add the distance to end node heuristic
    fn with_heuristic(mut self, end_node: &Node, heuristic_weight: &Weight) -> Self {
        self.heuristic = heuristic_weight * self.to.geometry.haversine_distance(&end_node.geometry) as f32;
        self
    }

    pub fn build(self) -> TraversalSegment {
        // generates "true segment cost" at build time, incorporating the cost factor, length of
        // the segment, the accumulated cost to get here
        // these should 0 out if any of these haven't been built into the segment
        let cost = self.cost_factor * self.length as f32 + self.cost_so_far;

        TraversalSegment {
            from: self.from,
            to: self.to,
            way: self.way,

            geometry: self.geometry,

            length: self.length,
            depth: self.depth,
            distance_so_far: self.distance_so_far,
            labels: self.labels,
            cost_factor: self.cost_factor,
            cost_so_far: self.cost_so_far,
            heuristic: self.heuristic,
            cost,
        }
    }
}

pub struct TraversalContext {
    pub queue: BinaryHeap<TraversalSegment>,
    pub came_from: HashMap<NodeId, TraversalSegment>,
    pub cost_model: CostModel,
    pub heuristic_weight: Weight,

    pub max_depth: Depth,
    pub cost_range: (Cost, Cost),
}

impl TraversalContext {
    pub fn new(cost_model: Option<CostModel>, heuristic_weight: Option<Weight>) -> Self {
        Self {
            queue: BinaryHeap::new(),
            came_from: HashMap::new(),
            cost_model: cost_model.unwrap_or_default(),
            heuristic_weight: heuristic_weight.unwrap_or(0.75),

            max_depth: 0,
            cost_range: (MAX, MIN),
        }
    }
}

impl Default for TraversalContext {
    fn default() -> Self {
        Self::new(None, None)
    }
}

/// initializes the structures required to traverse this graph, leveraging the snap_to_graph
/// function to snap the starting Point into the graph
/// TODO: be able to create a "virtual" node location _midway_ along a Way, rather than starting
///       from the clicked `start location
pub fn initialize_traversal(
    graph: &Graph,
    start: &Point,
    cost_model: Option<CostModel>,
    heuristic_weight: Option<Weight>,
) -> Result<TraversalContext, anyhow::Error> {
    let start_node = Node::new(START_NODE_ID, start);
    let starting_neighbors = graph.snap_to_graph(*start, None)?;

    let mut context = TraversalContext::new(cost_model, heuristic_weight);

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
                .build();
            context.came_from.insert(END_NODE_ID, segment);
            return Ok(());
        }

        let edges = graph.get_neighbors_with_labels(current.to.id)?;

        for (neighbor, way_labels) in edges {
            let segment = TraversalSegment::build_to_neighbor(&current.to, &neighbor)
                .with_depth(current.depth + 1)
                .with_prev_distance(current.distance_so_far)
                .with_cost(&context.cost_model, &way_labels, current.cost)
                .with_heuristic(end_node, &context.heuristic_weight)
                .build();
            context.cost_range.0 = context.cost_range.0.min(segment.cost_factor);
            context.cost_range.1 = context.cost_range.1.max(segment.cost_factor);
            context.max_depth = context.max_depth.max(segment.depth);

            if let Some(existing_segment) = context.came_from.get(&neighbor.node.id) {
                // if we already have a path to this neighbor, compare costs, take the cheaper
                // also queue up this neighbor, so a possibly better route can be identified
                if segment.cost < existing_segment.cost {
                    context.queue.push(segment.clone());
                    context.came_from.insert(neighbor.node.id, segment);
                }
            } else {
                context.queue.push(segment.clone());
                context.came_from.insert(neighbor.node.id, segment);
            }
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
            context.max_depth = max_depth;
            return Ok(());
        }

        let edges = graph.get_neighbors_with_labels(current.to.id)?;

        for (neighbor, way_labels) in edges {
            context
                .came_from
                .entry(neighbor.node.id)
                .or_insert_with(|| {
                    let segment = TraversalSegment::build_to_neighbor(&current.to, &neighbor)
                        .with_depth(current.depth + 1)
                        .with_prev_distance(current.distance_so_far)
                        .with_cost(&context.cost_model, &way_labels, current.cost)
                        .build();
                    context.cost_range.0 = context.cost_range.0.min(segment.cost_factor);
                    context.cost_range.1 = context.cost_range.1.max(segment.cost_factor);
                    context.queue.push(segment.clone());
                    segment
                });
        }
    }

    Err(anyhow!("Traversal failed"))
}
