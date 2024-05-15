use super::{serialize_node_simple, Graph, Neighbor, Node, NodeId, WayId};
use anyhow::anyhow;
use geo::{HaversineDistance, Line, Point};
use geojson::ser::serialize_geometry;
use serde::Serialize;
use std::collections::{HashMap, VecDeque};

pub const START_NODE_ID: NodeId = -1;
pub const END_NODE_ID: NodeId = -2;

type Depth = usize;

#[derive(Clone, Debug, Serialize)]
pub struct TraversalSegment {
    #[serde(serialize_with = "serialize_node_simple")]
    pub from: Node,
    #[serde(serialize_with = "serialize_node_simple")]
    pub to: Node,

    #[serde(serialize_with = "serialize_geometry")]
    pub geometry: Line,

    // segment metadata for weighing / constructing the route
    pub depth: Depth,
    pub distance: f64,
    pub way: WayId,
    // cost
}

impl TraversalSegment {
    pub fn new_to_neighbor(from: &Node, to: &Neighbor, depth: usize) -> TraversalSegment {
        TraversalSegment {
            from: *from,
            to: to.node,
            geometry: Line::new(from.geometry, to.node.geometry),
            distance: to.distance,
            depth,
            way: to.way,
        }
    }

    pub fn new_to_node(from: &Node, to: &Node, way: WayId, depth: usize) -> TraversalSegment {
        TraversalSegment {
            from: *from,
            to: *to,
            geometry: Line::new(from.geometry, to.geometry),
            distance: from.geometry.haversine_distance(&to.geometry),
            depth,
            way,
        }
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

pub trait Traversal {
    fn initialize_traversal(&self, start: &Point) -> Result<TraversalContext, anyhow::Error>;
    fn traverse<F, G>(
        &self,
        context: &mut TraversalContext,
        exit_condition: F,
        exit_action: G,
    ) -> Result<(), anyhow::Error>
    where
        F: Fn(&TraversalSegment) -> bool,
        G: Fn(&TraversalSegment, &mut HashMap<NodeId, TraversalSegment>);
}

impl Traversal for Graph {
    /// initializes the structures required to traverse this graph, leveraging the guess_neighbors
    /// function to snap the starting Point into the graph
    fn initialize_traversal(&self, start: &Point) -> Result<TraversalContext, anyhow::Error> {
        let start_node = Node::new(START_NODE_ID, start);
        let starting_neighbors = self.guess_neighbors(*start)?;

        let mut context = TraversalContext::new();

        for neighbor in starting_neighbors {
            let segment = TraversalSegment::new_to_neighbor(&start_node, &neighbor, 0);
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
        G: Fn(&TraversalSegment, &mut HashMap<NodeId, TraversalSegment>),
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
                        let segment = TraversalSegment::new_to_neighbor(
                            &current.to,
                            &neighbor,
                            current.depth + 1,
                        );
                        context.queue.push_back(segment.clone());
                        segment
                    });
            }
        }

        Err(anyhow!("Traversal failed"))
    }
}
