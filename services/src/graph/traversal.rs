use super::{serialize_cost_simple, Cost, CostModel, Graph, Weight};
use crate::osm::{
    serialize_node_simple, Cycleway, Distance, Neighbor, Node, NodeId, Road, WayId, WayLabels,
};
use anyhow::anyhow;
use geo::{HaversineDistance, Line, Point};
use serde::Serialize;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::f32::{MAX, MIN};

// define a thread local variable to reference during traversal ordering
thread_local! {
    static HEURISTIC_WEIGHT: RefCell<f32> = const { RefCell::new(0.75) };
}

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

    #[serde(serialize_with = "serialize_cost_simple")]
    pub cost: Cost,
    #[serde(serialize_with = "serialize_cost_simple")]
    pub cost_factor: Cost,
    #[serde(serialize_with = "serialize_cost_simple")]
    pub cost_so_far: Cost,

    pub distance_remaining: Distance,
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
        let heuristic_weight = TraversalContext::get_heuristic_weight();
        let other_total = other.cost_so_far + (other.distance_remaining as f32 * heuristic_weight);
        let self_total = self.cost_so_far + (self.distance_remaining as f32 * heuristic_weight);
        other_total.total_cmp(&self_total)
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
    cost: Cost,
    cost_factor: Cost,
    cost_so_far: Cost,
    distance_remaining: Distance,
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
            cost_factor: 0.0,
            cost_so_far: 0.0,
            distance_remaining: 0,
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
            cost_factor: 0.0,
            cost_so_far: 0.0,
            distance_remaining: 0,
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
        way_labels: &WayLabels,
        cost_so_far: Cost,
    ) -> Self {
        let cost_factor = cost_model.calculate_cost(way_labels);
        self.cost_factor = cost_factor;
        self.cost = cost_factor * self.length as f32;
        self.cost_so_far = cost_so_far + self.cost;
        self.labels = *way_labels;
        self
    }

    /// add the distance to end node heuristic
    fn with_heuristic(mut self, end_node: &Node) -> Self {
        self.distance_remaining =
            self.to.geometry.haversine_distance(&end_node.geometry) as Distance;
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
            labels: self.labels,
            cost: self.cost,
            cost_factor: self.cost_factor,
            cost_so_far: self.cost_so_far,
            distance_remaining: self.distance_remaining,
        }
    }
}

pub struct TraversalContext {
    pub queue: BinaryHeap<TraversalSegment>,
    pub came_from: HashMap<NodeId, TraversalSegment>,
    pub cost_model: CostModel,

    pub max_depth: Depth,
    pub cost_range: (Cost, Cost),
}

impl TraversalContext {
    pub fn new(cost_model: Option<CostModel>, heuristic_weight: Option<f32>) -> Self {
        if let Some(weight) = heuristic_weight {
            TraversalContext::set_heuristic_weight(weight);
        }

        Self {
            queue: BinaryHeap::new(),
            came_from: HashMap::new(),
            cost_model: cost_model.unwrap_or_default(),
            max_depth: 0,
            cost_range: (MAX, MIN),
        }
    }

    fn set_heuristic_weight(weight: f32) {
        HEURISTIC_WEIGHT.with(|w| {
            *w.borrow_mut() = weight;
        });
    }

    #[inline]
    fn get_heuristic_weight() -> f32 {
        HEURISTIC_WEIGHT.with(|w| *w.borrow())
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
                .calculate_cost(&context.cost_model, &way_labels, current.cost_so_far)
                .with_heuristic(end_node)
                .build();
            context.cost_range.0 = context.cost_range.0.min(segment.cost_factor);
            context.cost_range.1 = context.cost_range.1.max(segment.cost_factor);
            context.max_depth = context.max_depth.max(segment.depth);

            if let Some(existing_segment) = context.came_from.get(&neighbor.node.id) {
                // if we already have a path to this neighbor, compare costs, take the cheaper
                // also queue up this neighbor, so a possibly better route can be identified
                if segment.cost_so_far < existing_segment.cost_so_far {
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
                        .calculate_cost(&context.cost_model, &way_labels, current.cost_so_far)
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
