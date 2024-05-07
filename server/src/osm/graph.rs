use super::{db, Graph, Neighbor, Node, NodeId, WayId};
/// Exposes DB interactions as a Graph interface
use geo::prelude::*;
use geo::{Coord, HaversineBearing, LineString, Point};
use geojson::ser::serialize_geometry;
use serde::{Serialize, Serializer};
use std::collections::{HashSet, VecDeque};

/// A Route is a Graph abstraction built from a DB Segment and its Node / Way data
#[derive(Serialize, Clone, Debug)]
pub struct Route {
    #[serde(serialize_with = "serialize_route")]
    geometry: Vec<Coord>,
    from: NodeId, // a value of `0` represents the starter virtual node
    to: NodeId,
    way: WayId,
    distance: f64,

    /// graph traversal depth from start point
    depth: u8,
    // TODO: cost_so_far
}

/// custom serialization to first create a LineString from a Vec<Coord>
/// and then serialize into geojson
pub fn serialize_route<S>(geometry: &[Coord], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let line_string = LineString::new(geometry.to_vec());
    serialize_geometry(&line_string, serializer)
}

impl Route {
    pub fn new(from: &Node, to: &Node, way: WayId, depth: u8) -> Route {
        let start = Point::new(from.lon, from.lat);
        let end = Point::new(to.lon, to.lat);

        Route {
            geometry: vec![start.into(), end.into()],
            from: from.id,
            to: to.id,
            way,
            distance: start.haversine_distance(&end),
            depth,
        }
    }

    /// extends this route with the specified node
    // TODO: accept distance from Segments and add to Route's distance
    pub fn extend_with(&mut self, node: &mut Node) {
        let point = Point::new(node.lon, node.lat);
        self.depth += 1;
        self.to = node.id;
        self.geometry.push(point.into());
    }
}

impl Graph {
    pub fn new() -> Result<Self, anyhow::Error> {
        Ok(Self {
            conn: db::get_conn()?,
        })
    }

    /// Return a collection of Points and Lines from traversing the Graph from the start point to
    /// the depth specified
    pub fn traverse_from(
        &self,
        start: Point,
        max_depth: u8,
    ) -> Result<VecDeque<Route>, anyhow::Error> {
        let starting_neighbors = self.guess_neighbors(start)?;

        // init the traversal queue and visited set w/ those neighbors
        let mut visited: HashSet<NodeId> = starting_neighbors.iter().map(|r| r.to).collect();
        let mut queue: VecDeque<Route> = starting_neighbors.clone().into();

        while !queue.is_empty() {
            let current = queue.pop_front().unwrap();

            if current.depth == max_depth {
                break;
            }

            // find outbound segments for this node
            let adjacent_neighbors = self.get_neighbors(current.to)?;

            for mut n in adjacent_neighbors {
                // only act for neighbors that haven't been visited already
                if !visited.contains(&n.node.id) {
                    visited.insert(n.node.id);
                    let mut new_route = current.clone();
                    new_route.extend_with(&mut n.node);
                    queue.push_back(new_route);
                }
            }
        }

        Ok(queue)
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
    pub fn guess_neighbors(&self, start: Point) -> Result<Vec<Route>, anyhow::Error> {
        type Bearing = f64;
        let mut stmt = self.conn.prepare_cached(
            "
            SELECT WayNodes.way, WayNodes.node, lon, lat
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
            let loc = Point::new(row.get(2)?, row.get(3)?);

            // for each returned Node, calculate the distance from the start point
            Ok((
                Route {
                    way: row.get(0)?,
                    from: 0,
                    to: row.get(1)?,
                    geometry: vec![start.into(), loc.into()],
                    distance: start.haversine_distance(&loc),
                    depth: 0,
                },
                start.haversine_bearing(loc),
            ))
        })?;

        let mut results: Vec<(Route, Bearing)> = results.map(|r| r.unwrap()).collect();

        // sort these results by the total distance from the start point
        results.sort_by(|(a, _), (b, _)| a.distance.partial_cmp(&b.distance).unwrap());

        // at this point we have a sorted list of nodes by distance
        // the first, closest node is clearly the best candidate
        let mut results_iter = results.into_iter();
        let (closest_edge, closest_bearing) = results_iter.next().unwrap();

        // then, use the wayId + the bearing relationship to find
        // the next node on the way on the other side of the start point
        let mut next_closest: Option<Route> = None;
        for (edge, bearing) in results_iter {
            // This Node is on the same Way as the `closest`
            // so find the next edge that had a fairly different bearing than the closest
            // TODO: better alg here than just more than "normal" to closest bearing
            if closest_edge.way == edge.way && ((closest_bearing - bearing).abs() >= 90.) {
                next_closest = Some(edge);
                break;
            }
        }

        let mut results = vec![closest_edge];

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
            SELECT way, n2, N2.lon, N2.lat
            FROM Segments
            JOIN Nodes as N2 ON n2=N2.id
            WHERE n1 = ?1
        ",
        )?;
        let result = stmt.query_map([id], |row| {
            Ok(Neighbor {
                way: row.get(0)?,
                node: Node {
                    id: row.get(1)?,
                    lon: row.get(2)?,
                    lat: row.get(3)?,
                },
            })
        })?;

        Ok(result.map(|r| r.unwrap()).collect())
    }
}
