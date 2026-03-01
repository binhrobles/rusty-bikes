use super::{serialize_as_int, serialize_float_rounded, Cost, CostModel, Graph, Weight};
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

pub trait Traversable {
    fn initialize_traversal(
        &self,
        start: &Point,
        cost_model: Option<CostModel>,
        heuristic_weight: Option<Weight>,
    ) -> Result<TraversalContext, anyhow::Error>;
    fn traverse_from(
        &self,
        context: &mut TraversalContext,
        max_depth: usize,
    ) -> Result<(), anyhow::Error>;
    fn traverse_between(
        &self,
        context: &mut TraversalContext,
        target_neighbor_node_ids: &[NodeId],
        end_node: &Node,
    ) -> Result<(), anyhow::Error>;
}

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
    fn eq(&self, other: &Self) -> bool {
        self.from == other.from && self.to == other.to && self.way == other.way
    }
}

// manually implementing Eq so that the `geometry`, `from`, and `to` fields aren't
// implicitly added to the derived implementation
impl Eq for TraversalSegment {}

impl PartialOrd for TraversalSegment {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// TraversalSegment comparisons make use of cost_so_far from traversal start and a factor of the distance
/// remaining to the end node
impl Ord for TraversalSegment {
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

    /// generates and attaches the "true" cost factor and accumulated cost
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
        self.heuristic =
            heuristic_weight * self.to.geometry.haversine_distance(&end_node.geometry) as f32;
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

/// Lightweight heap entry: replaces storing full TraversalSegments in the priority queue.
/// The full segment lives only in `came_from`; the heap just tracks ordering + stale detection.
#[derive(PartialEq)]
pub struct HeapEntry {
    /// cost + heuristic (A* f-value) used for ordering
    pub priority: Cost,
    /// which node this entry routes to
    pub to_node_id: NodeId,
    /// g-value: total cost to reach to_node_id (used for lazy-deletion stale check)
    pub cost_at_node: Cost,
}

impl Eq for HeapEntry {}

impl PartialOrd for HeapEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Min-heap on priority (lower cost = higher priority)
impl Ord for HeapEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.priority.total_cmp(&self.priority)
    }
}

/// Context object representing the state of a single routing or traversal operation
pub struct TraversalContext {
    pub queue: BinaryHeap<HeapEntry>,
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
            came_from: HashMap::with_capacity(4096),
            cost_model: cost_model.unwrap_or_default(),
            heuristic_weight: heuristic_weight.unwrap_or(0.75),

            max_depth: 0,
            cost_range: (f32::MAX, f32::MIN),
        }
    }
}

impl Traversable for Graph {
    /// initializes the context and structures required to perform a traversal
    /// TODO: be able to create a "virtual" node location _midway_ along a Way, rather than starting
    ///       from the clicked `start location
    fn initialize_traversal(
        &self,
        start: &Point,
        cost_model: Option<CostModel>,
        heuristic_weight: Option<Weight>,
    ) -> Result<TraversalContext, anyhow::Error> {
        let start_node = Node::new(START_NODE_ID, start);
        let starting_neighbors = self.db.get_snapped_neighbors(*start, None)?;

        let mut context = TraversalContext::new(cost_model, heuristic_weight);

        for neighbor in starting_neighbors {
            let segment = TraversalSegment::build_to_neighbor(&start_node, &neighbor).build();
            context.queue.push(HeapEntry {
                priority: segment.cost + segment.heuristic,
                to_node_id: neighbor.node.id,
                cost_at_node: segment.cost,
            });
            context.came_from.insert(neighbor.node.id, segment);
        }

        Ok(context)
    }

    /// Generates a collection of all TraversalSegments examined while routing between the start and
    /// end Points. TraversalSegments will be decorated with both the depth of the traversal and
    /// the cost assigned, given the designated cost model
    fn traverse_between(
        &self,
        context: &mut TraversalContext,
        target_neighbor_node_ids: &[NodeId],
        end_node: &Node,
    ) -> Result<(), anyhow::Error> {
        while let Some(entry) = context.queue.pop() {
            // Lazy deletion: skip stale heap entries where we already found a cheaper path
            let current_cost = context
                .came_from
                .get(&entry.to_node_id)
                .map(|s| s.cost)
                .unwrap_or(f32::MAX);
            if current_cost < entry.cost_at_node {
                continue;
            }

            if target_neighbor_node_ids.contains(&entry.to_node_id) {
                // Reached the target — reconstruct the final segment to the virtual end node
                let current = context.came_from.get(&entry.to_node_id).unwrap();
                let segment = TraversalSegment::build_to_node(&current.to, end_node, current.way)
                    .with_depth(current.depth + 1)
                    .with_prev_distance(current.distance_so_far)
                    .build();
                context.came_from.insert(END_NODE_ID, segment);
                return Ok(());
            }

            // Extract what we need before the mutable came_from borrows below
            let (current_to, current_cost, current_depth, current_distance) = {
                let seg = context.came_from.get(&entry.to_node_id).unwrap();
                (seg.to, seg.cost, seg.depth, seg.distance_so_far)
            };

            let edges = self.db.get_neighbors_with_labels(entry.to_node_id)?;

            for (neighbor, way_labels) in edges {
                let segment = TraversalSegment::build_to_neighbor(&current_to, &neighbor)
                    .with_depth(current_depth + 1)
                    .with_prev_distance(current_distance)
                    .with_cost(&context.cost_model, &way_labels, current_cost)
                    .with_heuristic(end_node, &context.heuristic_weight)
                    .build();
                context.cost_range.0 = context.cost_range.0.min(segment.cost_factor);
                context.cost_range.1 = context.cost_range.1.max(segment.cost_factor);
                context.max_depth = context.max_depth.max(segment.depth);

                let should_push = context
                    .came_from
                    .get(&neighbor.node.id)
                    .map_or(true, |existing| segment.cost < existing.cost);

                if should_push {
                    context.queue.push(HeapEntry {
                        priority: segment.cost + segment.heuristic,
                        to_node_id: neighbor.node.id,
                        cost_at_node: segment.cost,
                    });
                    context.came_from.insert(neighbor.node.id, segment);
                }
            }
        }

        Err(anyhow!("Traversal failed"))
    }

    /// Return a collection of TraversalSegments from traversing the Graph from the start point to
    /// the depth specified
    fn traverse_from(
        &self,
        context: &mut TraversalContext,
        max_depth: usize,
    ) -> Result<(), anyhow::Error> {
        while let Some(entry) = context.queue.pop() {
            // Extract what we need before any mutable borrows of came_from
            let (current_depth, current_to, current_cost, current_distance) = {
                let seg = context.came_from.get(&entry.to_node_id).unwrap();
                (seg.depth, seg.to, seg.cost, seg.distance_so_far)
            };

            if current_depth == max_depth {
                context.max_depth = max_depth;
                return Ok(());
            }

            let edges = self.db.get_neighbors_with_labels(entry.to_node_id)?;

            for (neighbor, way_labels) in edges {
                if context.came_from.contains_key(&neighbor.node.id) {
                    continue;
                }
                let segment = TraversalSegment::build_to_neighbor(&current_to, &neighbor)
                    .with_depth(current_depth + 1)
                    .with_prev_distance(current_distance)
                    .with_cost(&context.cost_model, &way_labels, current_cost)
                    .build();
                context.cost_range.0 = context.cost_range.0.min(segment.cost_factor);
                context.cost_range.1 = context.cost_range.1.max(segment.cost_factor);
                context.queue.push(HeapEntry {
                    priority: segment.cost,
                    to_node_id: neighbor.node.id,
                    cost_at_node: segment.cost,
                });
                context.came_from.insert(neighbor.node.id, segment);
            }
        }

        Err(anyhow!("Traversal failed"))
    }
}
