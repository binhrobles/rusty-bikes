/// Structs and logic specific to traversing a Graph
use super::{
    serialize_float_as_int, serialize_node_simple, Distance, Graph, Neighbor, Node, NodeId, WayId,
};
use anyhow::anyhow;
use geo::{HaversineDistance, Line, Point};
use geojson::ser::serialize_geometry;
use serde::Serialize;
use std::collections::{HashMap, VecDeque};

pub const START_NODE_ID: NodeId = -1;
pub const END_NODE_ID: NodeId = -2;

pub type Depth = usize;

#[derive(Clone, Debug, Serialize)]
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

pub type TraversalQueue = VecDeque<TraversalSegment>;
pub type TraversalMap = HashMap<NodeId, TraversalSegment>;
pub type TraversalRoute = Vec<TraversalSegment>;
pub type Traversal = Vec<TraversalSegment>;

pub fn get_traversal(map: &TraversalMap) -> Traversal {
    map.values().cloned().collect()
}

pub struct TraversalContext {
    pub queue: TraversalQueue,
    pub came_from: TraversalMap,
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

pub trait Traversable {
    fn initialize_traversal(&self, start: &Point) -> Result<TraversalContext, anyhow::Error>;
    fn traverse<F, G>(
        &self,
        context: &mut TraversalContext,
        exit_condition: F,
        exit_action: G,
    ) -> Result<(), anyhow::Error>
    where
        F: Fn(&TraversalSegment) -> bool,
        G: Fn(&TraversalSegment, &mut TraversalMap);
}

impl Traversable for Graph {
    /// initializes the structures required to traverse this graph, leveraging the guess_neighbors
    /// function to snap the starting Point into the graph
    fn initialize_traversal(&self, start: &Point) -> Result<TraversalContext, anyhow::Error> {
        let start_node = Node::new(START_NODE_ID, start);
        let starting_neighbors = self.guess_neighbors(*start)?;

        let mut context = TraversalContext::new();

        for neighbor in starting_neighbors {
            let segment = TraversalSegment::build_to_neighbor(&start_node, &neighbor).build();
            context.came_from.insert(neighbor.node.id, segment.clone());
            context.queue.push_back(segment);
        }

        Ok(context)
    }

    /// performs the traversal on the graph, given a starting context
    /// will perform the specified action and exit when the specified condition is met
    fn traverse<F, G>(
        &self,
        context: &mut TraversalContext,
        exit_condition: F,
        exit_action: G,
    ) -> Result<(), anyhow::Error>
    where
        F: Fn(&TraversalSegment) -> bool,
        G: Fn(&TraversalSegment, &mut TraversalMap),
    {
        while let Some(current) = context.queue.pop_front() {
            if exit_condition(&current) {
                exit_action(&current, &mut context.came_from);
                return Ok(());
            }

            let adjacent_neighbors = self.get_neighbors(current.to.id)?;

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
}
