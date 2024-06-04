/// Exposes DB interactions as a Graph interface
use super::{db, Graph, Neighbor, Node, NodeId};
use crate::osm::traversal::{
    self, Traversable, Traversal, TraversalMap, TraversalRoute, TraversalSegment, END_NODE_ID,
    START_NODE_ID,
};
use anyhow::anyhow;
use geo::prelude::*;
use geo::{HaversineBearing, Point};
use std::collections::VecDeque;

impl Graph {
    pub fn new() -> Result<Self, anyhow::Error> {
        Ok(Self {
            conn: db::get_conn()?,
        })
    }

    pub fn route_between(
        &self,
        start: Point,
        end: Point,
        with_traversal: bool,
    ) -> Result<(TraversalRoute, Option<Traversal>), anyhow::Error> {
        let traversal_map = self.traverse_between(start, end)?;

        // construct route from traversal information
        let mut current_segment = traversal_map.get(&END_NODE_ID).unwrap();
        let mut result: VecDeque<TraversalSegment> = VecDeque::from([current_segment.clone()]);

        loop {
            if current_segment.from.id == START_NODE_ID {
                break;
            }
            current_segment = traversal_map.get(&current_segment.from.id).unwrap();
            result.push_front(current_segment.clone());
        }

        // include the traversal if requested
        let traversal = if with_traversal {
            Some(traversal::get_traversal(&traversal_map))
        } else {
            None
        };

        Ok((result.make_contiguous().to_vec(), traversal))
    }

    /// Return a collection of all TraversalSegments examined while routing between the start and
    /// end Points. TraversalSegments will be decorated with both the depth of the traversal and
    /// the cost assigned, given the designated cost model
    fn traverse_between(&self, start: Point, end: Point) -> Result<TraversalMap, anyhow::Error> {
        let end_node = Node::new(END_NODE_ID, &end);
        let target_neighbors = self.guess_neighbors(end)?;
        let target_neighbor_node_ids: Vec<NodeId> =
            target_neighbors.iter().map(|n| n.node.id).collect();

        let context = self.initialize_traversal(&start)?;

        self.traverse(
            &context,
            |current| target_neighbor_node_ids.contains(&current.to.id),
            |current| {
                // on exit, append the final segment to the ending node
                let segment = TraversalSegment::build_to_node(&current.to, &end_node, current.way)
                    .with_depth(current.depth + 1)
                    .with_prev_distance(current.distance_so_far)
                    .build();
                context.came_from.borrow_mut().insert(END_NODE_ID, segment);
            },
        )?;

        let traversal = context.came_from.borrow().clone();

        Ok(traversal)
    }

    /// Return a collection of TraversalSegments from traversing the Graph from the start point to
    /// the depth specified
    pub fn traverse_from(
        &self,
        start: Point,
        max_depth: usize,
    ) -> Result<Vec<TraversalSegment>, anyhow::Error> {
        let context = self.initialize_traversal(&start)?;

        self.traverse(
            &context,
            |current| current.depth == max_depth,
            |_current| {},
        )?;

        let traversal = traversal::get_traversal(&context.came_from.borrow());

        Ok(traversal)
    }

    /// Returns Edges to the closest Node(s) to the location provided
    ///
    /// Implementation notes:
    /// - We cannot guarantee that the first Way returned from the R*tree query will be
    /// the closest Way, because of how R*Trees work
    /// - TODO: locations on corners are edge cases
    /// - TODO: locations directly on Nodes are edge cases (or will this be accounted for by the alg's
    /// cost model?)
    /// - TODO: handle no Ways returned, empty case
    /// - TODO: more than 2 neighbors?
    pub fn guess_neighbors(&self, start: Point) -> Result<Vec<Neighbor>, anyhow::Error> {
        type Bearing = f64;

        let mut stmt = self.conn.prepare_cached(
            "
            SELECT WayNodes.node, lon, lat, WayNodes.way
            FROM Ways
            JOIN WayNodes ON WayNodes.way=Ways.id
            JOIN Nodes ON WayNodes.node=Nodes.id
            WHERE minLat <= ?2
              AND maxLat >= ?2
              AND minLon <= ?1
              AND maxLon >= ?1
        ",
        )?;
        let results = stmt.query_map([start.x(), start.y()], |row| {
            let lon = row.get(1)?;
            let lat = row.get(2)?;

            let loc = Point::new(lon, lat);

            // for each returned Node, calculate the distance from the start point
            Ok((
                Neighbor {
                    node: Node::new(row.get(0)?, &loc),
                    way: row.get(3)?,
                    distance: start.haversine_distance(&loc),
                },
                start.haversine_bearing(loc),
            ))
        })?;

        let mut results: Vec<(Neighbor, Bearing)> = results.map(|r| r.unwrap()).collect();

        // at this point we have a sorted list of nodes by distance
        // the first, closest node is clearly the best candidate
        if results.is_empty() {
            // TODO: do something better, with a wider search radius?
            return Err(anyhow!("Could not snap coords to graph nodes"));
        }

        // sort these results by the total distance from the start point
        results.sort_by(|(n1, _), (n2, _)| n1.distance.partial_cmp(&n2.distance).unwrap());

        let mut results_iter = results.into_iter();
        let (closest_neighbor, closest_bearing) = results_iter.next().unwrap();

        // then, use the wayId + the bearing relationship to find
        // the next node on the way on the other side of the start point
        let mut next_closest: Option<Neighbor> = None;
        for (neighbor, bearing) in results_iter {
            // This Node is on the same Way as the `closest`
            // so find the next segment that had a fairly different bearing than the closest
            // TODO: better alg here than just more than "normal" to closest bearing
            if closest_neighbor.way == neighbor.way && ((closest_bearing - bearing).abs() >= 90.) {
                next_closest = Some(neighbor);
                break;
            }
        }

        let mut results = vec![closest_neighbor];

        if let Some(n) = next_closest {
            results.push(n)
        }

        Ok(results)
    }

    /// given a NodeId, gets the neighbors from the Segments table
    /// returns a Vec of Edges to the neighbors
    pub fn get_neighbors(&self, id: NodeId) -> Result<Vec<Neighbor>, anyhow::Error> {
        let mut stmt = self.conn.prepare_cached(
            "
            SELECT way, n2, N2.lon, N2.lat, distance
            FROM Segments
            JOIN Nodes as N2 ON n2=N2.id
            WHERE n1 = ?1
        ",
        )?;
        let result = stmt.query_map([id], |row| {
            Ok(Neighbor {
                way: row.get(0)?,
                node: Node::new(row.get(1)?, &Point::new(row.get(2)?, row.get(3)?)),
                distance: row.get(4)?,
            })
        })?;

        Ok(result.map(|r| r.unwrap()).collect())
    }
}
