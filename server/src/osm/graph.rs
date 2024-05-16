/// Exposes DB interactions as a Graph interface
use super::{db, Graph, Neighbor, Node, NodeId};
use crate::osm::traversal::{Traversal, TraversalSegment, END_NODE_ID, START_NODE_ID};
use anyhow::anyhow;
use geo::prelude::*;
use geo::{HaversineBearing, Point};
use std::collections::{HashMap, VecDeque};

impl Graph {
    pub fn new() -> Result<Self, anyhow::Error> {
        Ok(Self {
            conn: db::get_conn()?,
        })
    }

    /// the route bw the points provided, as a vector
    /// pretty Route struct with no fun decorations or anything
    pub fn route_between(&self, start: Point, end: Point) -> Result<Vec<TraversalSegment>, anyhow::Error> {
        let traversal = self.traverse_between(start, end)?;
        println!("got traversal of len: {}", traversal.clone().keys().len());

        // construct route from traversal information
        let mut current_segment = traversal.get(&END_NODE_ID).unwrap();
        let mut result: VecDeque<TraversalSegment> = VecDeque::from([current_segment.clone()]);

        loop {
            if current_segment.from.id == START_NODE_ID {
                break;
            }
            current_segment = traversal.get(&current_segment.from.id).unwrap();
            result.push_front(current_segment.clone());
        }

        Ok(result.make_contiguous().to_vec())
    }

    /// Return a collection of all TraversalSegments examined while routing between the start and
    /// end Points. TraversalSegments will be decorated with both the depth of the traversal and
    /// the cost assigned, given the designated cost model
    fn traverse_between(
        &self,
        start: Point,
        end: Point,
    ) -> Result<HashMap<NodeId, TraversalSegment>, anyhow::Error> {
        let end_node = Node::new(END_NODE_ID, &end);
        let target_neighbors = self.guess_neighbors(end)?;
        let target_neighbor_node_ids: Vec<NodeId> =
            target_neighbors.iter().map(|n| n.node.id).collect();

        let mut context = self.initialize_traversal(&start)?;

        self.traverse(
            &mut context,
            |current| target_neighbor_node_ids.contains(&current.to.id),
            |current, came_from| {
                let segment = TraversalSegment::new_to_node(
                    &current.to,
                    &end_node,
                    current.way,
                    current.depth + 1,
                );
                came_from.insert(END_NODE_ID, segment);
            },
        )?;

        Ok(context.came_from)
    }

    /// Return a collection of TraversalSegments from traversing the Graph from the start point to
    /// the depth specified
    pub fn traverse_from(
        &self,
        start: Point,
        max_depth: usize,
    ) -> Result<Vec<TraversalSegment>, anyhow::Error> {
        let mut context = self.initialize_traversal(&start)?;

        self.traverse(
            &mut context,
            |current| current.depth == max_depth,
            |_current, _came_from| {},
        )?;

        Ok(context.came_from.values().cloned().collect())
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
