use super::traversal::{
    Route, Traversable, Traversal, TraversalSegment, END_NODE_ID, START_NODE_ID,
};
use super::SqliteGraphRepository;
use super::{repository::GraphRepository, Cost, CostModel, Depth, Weight};
use crate::osm::{Node, NodeId};
use geo::Point;
use serde::Serialize;
use std::collections::VecDeque;

/// The Graph "service object", through which routing interfaces are exposed
pub struct Graph {
    pub db: Box<dyn GraphRepository>,
}

#[derive(Debug, Serialize)]
pub struct RouteMetadata {
    max_depth: Depth,
    cost_range: (Cost, Cost),
}

impl Graph {
    pub fn new() -> Result<Self, anyhow::Error> {
        Ok(Self {
            db: Box::new(SqliteGraphRepository::new()?),
        })
    }

    /// Calculates a Route between the start and end points, optionally attaching the raw underlying traversal
    pub fn calculate_route(
        &self,
        start: Point,
        end: Point,
        with_traversal: bool,
        cost_model: Option<CostModel>,
        heuristic_weight: Option<Weight>,
    ) -> Result<(Route, Option<Traversal>, RouteMetadata), anyhow::Error> {
        let end_node = Node::new(END_NODE_ID, &end);
        let target_neighbors = self.db.get_snapped_neighbors(end, None)?;
        let target_neighbor_node_ids: Vec<NodeId> =
            target_neighbors.iter().map(|n| n.node.id).collect();

        let mut context = self.initialize_traversal(&start, cost_model, heuristic_weight)?;

        self.traverse_between(&mut context, &target_neighbor_node_ids, &end_node)?;

        // construct route from traversal information, tracing backwards from the end node
        let mut current_segment = context.came_from.get(&END_NODE_ID).unwrap();
        let mut result: VecDeque<TraversalSegment> = VecDeque::from([current_segment.clone()]);

        loop {
            if current_segment.from.id == START_NODE_ID {
                break;
            }
            current_segment = context.came_from.get(&current_segment.from.id).unwrap();
            result.push_front(current_segment.clone());
        }

        // include the traversal if requested
        let traversal = if with_traversal {
            Some(context.came_from.values().cloned().collect())
        } else {
            None
        };

        let meta = RouteMetadata {
            max_depth: context.max_depth,
            cost_range: context.cost_range,
        };
        Ok((result.make_contiguous().to_vec(), traversal, meta))
    }

    /// Generates a breadth-first traversal from the start point to the depth specified
    pub fn calculate_traversal(
        &self,
        start: Point,
        max_depth: usize,
        cost_model: Option<CostModel>,
        heuristic_weight: Option<Weight>,
    ) -> Result<Traversal, anyhow::Error> {
        let mut context = self.initialize_traversal(&start, cost_model, heuristic_weight)?;

        self.traverse_from(&mut context, max_depth)?;

        Ok(context.came_from.values().cloned().collect())
    }
}
