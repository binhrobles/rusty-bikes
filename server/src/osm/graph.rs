/// Exposes DB interactions as a Graph interface
use super::{db, Graph, LocationDistance, Neighbor, Node, NodeId, Way, WayId};
use geo_types::{geometry::Geometry, Coord, Line};
use geojson::ser::serialize_geometry;
use serde::Serialize;
use std::collections::HashSet;

/// An Edge is a Graph abstraction built from a DB Segment and its Node / Way data
#[derive(Debug)]
pub struct Edge {
    /// contains two Coords, corresponding to the `from` and `to` Nodes
    geometry: Line,
    from: NodeId, // a value of `0` represents the starter virtual node
    to: NodeId,
    way: WayId,
    distance: LocationDistance,
}

impl Edge {
    pub fn new(from: &Node, to: &Node, way: &WayId) -> Edge {
        let start = Coord {
            x: from.lon,
            y: from.lat,
        };

        let end = Coord {
            x: to.lon,
            y: to.lat,
        };
        Edge {
            geometry: Line { start, end },
            from: from.id,
            to: to.id,
            way: *way,
            distance: calculate_distance(&start, &end),
        }
    }
}

/// struct for returning a geometry from a graph traversal
/// created for demo purposes
#[derive(Serialize, Debug)]
pub struct TraversalGeom {
    #[serde(serialize_with = "serialize_geometry")]
    geometry: Geometry,
    from: NodeId,
    to: NodeId,
    depth: u8,
}

impl TraversalGeom {
    pub fn new(from: &Node, to: &Node, depth: u8) -> TraversalGeom {
        let start = Coord {
            x: from.lon,
            y: from.lat,
        };

        let end = Coord {
            x: to.lon,
            y: to.lat,
        };
        TraversalGeom {
            geometry: Geometry::Line(Line { start, end }),
            from: from.id,
            to: to.id,
            depth,
        }
    }
}

// TODO: where should this actually live?
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
    ///
    /// A function for demo purposes. Questionable use for routing.
    pub fn traverse_from(
        &self,
        start: Coord,
        max_depth: u8,
    ) -> Result<Vec<TraversalGeom>, anyhow::Error> {
        let neighbors = self.guess_neighbors(start)?;

        let mut queue: Vec<Neighbor> = neighbors.iter().map(|e| e.unwrap()).collect();

        let mut visited_nodes_set: HashSet<NodeId> = HashSet::new();
        let mut results: Vec<TraversalGeom> = Vec::new();

        // continue traversing the graph for `depth` iterations
        let mut depth = 1;
        while depth <= max_depth {
            let mut next_level_queue: Vec<Neighbor> = Vec::new();
            // for each of the last round of results
            for neighbor in queue.iter() {
                // only act for neighbors that haven't been visited already
                // avoids cycling -- but how does it play w/ A* priority queue?
                if !visited_nodes_set.contains(&neighbor.node.id) {
                    visited_nodes_set.insert(neighbor.node.id);

                    // find outbound segments this node
                    let adjacent_neighbors = self.get_neighbors(neighbor.node.id)?;

                    adjacent_neighbors.iter().for_each(|n| {
                        if !visited_nodes_set.contains(&n.node.id) {
                            // push neighbor into the queue for the next depth
                            next_level_queue.push(*n);

                            // format and push neighbors into results collection
                            results.push(TraversalGeom::new(&neighbor.node, &n.node, depth));
                        }
                    });
                }
            }

            queue = next_level_queue;

            depth += 1;
        }

        Ok(results)
    }

    /// Gets the closest 2 Nodes to the location provided
    ///
    /// Implementation notes:
    /// - We cannot guarantee that the first Way returned from the R*tree query will be
    /// the closest Way, because of how R*Trees work
    /// - TODO: locations on corners are edge cases
    /// - TODO: locations directly on Nodes are edge cases (or will this be accounted for by the alg's
    /// cost model?)
    /// - TODO: handle no Ways returned, empty case
    pub fn guess_neighbors(&self, start: Coord) -> Result<Vec<Option<Neighbor>>, anyhow::Error> {
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
            Ok(Edge {
                way: row.get(0)?,
                from: 0, // TODO: other representation for a virtual node?
                to: row.get(1)?,
                geometry: Line { start, end: loc },
                distance: calculate_distance(&start, &loc),
            })
        })?;

        let mut results: Vec<Edge> = results.map(|r| r.unwrap()).collect();

        // sort these results by the total distance from the start point
        results.sort_by(|a, b| a.distance.total.partial_cmp(&b.distance.total).unwrap());

        // at this point we have a sorted list of nodes by distance
        // the first, closest node is clearly the best candidate
        let mut results_iter = results.into_iter();
        let closest = results_iter.next().unwrap();

        // then, use the wayId + signs of the lat_diff / lon_diff to find
        // the next node on the way on the other side of the lat/lon spectrum
        let mut next_closest: Option<Edge> = None;
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

        let closest = Neighbor {
            way: closest.way,
            node: Node {
                id: closest.to,
                lon: closest.geometry.end.x,
                lat: closest.geometry.end.y,
            },
        };

        let next_closest_node: Option<Neighbor> = match next_closest {
            Some(next_closest) => Some(Neighbor {
                way: next_closest.way,
                node: Node {
                    id: next_closest.to,
                    lon: next_closest.geometry.end.x,
                    lat: next_closest.geometry.end.y,
                },
            }),
            None => None,
        };

        Ok(vec![Some(closest), next_closest_node])
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

    // TODO: tuple!
    /// given a NodeId, gets the neighbors from the Segments table
    /// returns a Vec of NodeId-WayId pairs, or the Node neighbor + the Way that connects them
    pub fn get_neighbors(&self, id: NodeId) -> Result<Vec<Neighbor>, anyhow::Error> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT way, n2, lon, lat FROM Segments JOIN Nodes ON n2=Nodes.id WHERE n1 = ?1",
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_neighbors_returns_array_of_neighbors() {
        let graph = Graph::new().unwrap();
        let neighbors = graph.get_neighbors(278630910).unwrap();

        assert_eq!(
            neighbors,
            vec![
                Neighbor {
                    node: Node {
                        id: 42496432,
                        lon: -73.9894027709961,
                        lat: 40.6910400390625,
                    },
                    way: 221605481
                },
                Neighbor {
                    node: Node {
                        id: 6224367557,
                        lon: -73.9891128540039,
                        lat: 40.6909599304199,
                    },
                    way: 221605486
                },
                Neighbor {
                    node: Node {
                        id: 10001064063,
                        lon: -73.9892196655273,
                        lat: 40.6910285949707,
                    },
                    way: 5029221
                },
                Neighbor {
                    node: Node {
                        id: 10001064066,
                        lon: -73.9892654418945,
                        lat: 40.6909446716309,
                    },
                    way: 421121604
                },
            ]
        );
    }

    // TODO: tests for querying Way R tree (ensuring determinism?)
    // TODO: tests for starting coords off a Way (ensure no None response?)
}
