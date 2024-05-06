/// Exposes DB interactions as a Graph interface
use super::{db, Graph, LocationDistance, Neighbor, Node, NodeId, Way, WayId};
use geo_types::{Coord, LineString};
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
    distance: LocationDistance,

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
        let start = Coord {
            x: from.lon,
            y: from.lat,
        };

        let end = Coord {
            x: to.lon,
            y: to.lat,
        };
        Route {
            geometry: vec![start, end],
            from: from.id,
            to: to.id,
            way,
            distance: calculate_distance(&start, &end),
            depth,
        }
    }

    /// extends this route with the specified node
    pub fn extend_with(&mut self, node: &mut Node) {
        let coord = Coord {x: node.lon, y: node.lat};
        self.depth += 1;
        self.to = node.id;
        self.distance = self.distance + calculate_distance(self.geometry.last().unwrap(), &coord);
        self.geometry.push(coord);
    }
}

// TODO: where should this actually live? at ETL time
/// get the distance b/w two coordinates, in cartesian units
fn calculate_distance(from: &Coord, to: &Coord) -> LocationDistance {
    let lat_diff = to.y - from.y;
    let lon_diff = to.x - from.x;
    LocationDistance {
        lat_diff,
        lon_diff,
        total: (lat_diff.powi(2) + lon_diff.powi(2)).sqrt(),
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
        start: Coord,
        max_depth: u8,
    ) -> Result<VecDeque<Route>, anyhow::Error> {
        let mut queue: VecDeque<Route> = self.guess_neighbors(start)?.into();

        let mut visited: HashSet<NodeId> = HashSet::new();

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
    pub fn guess_neighbors(&self, start: Coord) -> Result<Vec<Route>, anyhow::Error> {
        let mut stmt = self.conn.prepare_cached(
            "
            SELECT WayNodes.way, WayNodes.node, lon, lat
            FROM Ways
            JOIN WayNodes ON WayNodes.way=Ways.id
            JOIN Nodes ON WayNodes.node=Nodes.id
            WHERE minLat <= ?1
              AND maxLat >= ?1
              AND minLon <= ?2
              AND maxLon >= ?2
        ",
        )?;
        let results = stmt.query_map([start.y, start.x], |row| {
            let loc = Coord {
                x: row.get(2)?,
                y: row.get(3)?,
            };

            // for each returned Node, calculate the distance from the start point
            Ok(Route {
                way: row.get(0)?,
                from: 0, // TODO: other representation for a virtual node?
                to: row.get(1)?,
                geometry: vec![start, loc],
                distance: calculate_distance(&start, &loc),
                depth: 0,
            })
        })?;

        let mut results: Vec<Route> = results.map(|r| r.unwrap()).collect();

        // sort these results by the total distance from the start point
        results.sort_by(|a, b| a.distance.total.partial_cmp(&b.distance.total).unwrap());

        // at this point we have a sorted list of nodes by distance
        // the first, closest node is clearly the best candidate
        let mut results_iter = results.into_iter();
        let closest = results_iter.next().unwrap();

        // then, use the wayId + signs of the lat_diff / lon_diff to find
        // the next node on the way on the other side of the lat/lon spectrum
        let mut next_closest: Option<Route> = None;
        for edge in results_iter {
            // This Node is on the same Way as the `closest`
            // but on the other side of the lat/lon spectrum
            // so we can start our alg choosing from one of these two Nodes
            if closest.way == edge.way
                && (closest.distance.lat_diff.signum() != edge.distance.lat_diff.signum()
                    || closest.distance.lon_diff.signum() != edge.distance.lon_diff.signum())
            {
                next_closest = Some(edge);
                break;
            }
        }

        let mut results = vec![closest];

        if let Some(n) = next_closest {
            results.push(n)
        }

        Ok(results)
    }

    pub fn get_bounding_ways(&self, location: Coord) -> Result<Vec<Way>, anyhow::Error> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT id, minLat, maxLat, minLon, maxLon FROM Ways
            WHERE minLat <= ?1
            AND maxLat >= ?1
            AND minLon <= ?2
            AND maxLon >= ?2",
        )?;
        let result = stmt.query_map([location.y, location.x], |row| {
            Ok(Way {
                id: row.get(0)?,
                min_lat: row.get(1)?,
                max_lat: row.get(2)?,
                min_lon: row.get(3)?,
                max_lon: row.get(4)?,
            })
        })?;

        Ok(result.map(|r| r.unwrap()).collect())
    }

    /// given a NodeId, gets the neighbors from the Segments table
    /// returns a Vec of Edges to the neighbors
    pub fn get_neighbors(
        &self,
        id: NodeId,
    ) -> Result<Vec<Neighbor>, anyhow::Error> {
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
