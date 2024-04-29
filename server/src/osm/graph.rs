/// Exposes DB interactions as a Graph interface
use super::{db, Graph, LocationDistance, Neighbor, NodeId, Way, WayId};
use serde::Serialize;
use geojson::ser::serialize_geometry;
use geo_types::{geometry::Geometry, Coord, Line};

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

#[derive(Serialize, Debug)]
pub struct TraversalGeom {
    #[serde(serialize_with = "serialize_geometry")]
    geometry: Geometry,
    depth: u8,
}

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

    pub fn traverse_from(
        &self,
        start: Coord,
        _depth: u8,
    ) -> Result<Vec<TraversalGeom>, anyhow::Error> {
        let neighbors = self.guess_neighbors(start)?;
        println!("traverse:: found neighbors: {neighbors:#?}");

        // continue traversing the graph for `depth` iterations

        let results = neighbors.into_iter().map(|n| {
            TraversalGeom {
                geometry: Geometry::Line(n.unwrap().geometry),
                depth: 1,
            }
        }).collect();

        Ok(results)
    }

    /// Gets the closest 2 Edges to the location provided
    ///
    /// Implementation notes:
    /// - We cannot guarantee that the first Way returned from the R*tree query will be
    /// the closest Way, because of how R*Trees work
    /// - TODO: locations on corners are edge cases
    /// - TODO: locations directly on Nodes are edge cases (or will this be accounted for by the alg's
    /// cost model?)
    /// - TODO: handle no Ways returned, empty case
    pub fn guess_neighbors(&self, start: Coord) -> Result<Vec<Option<Edge>>, anyhow::Error> {
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

        // then, use the wayId + signs of the lat_diff / lon_diff to find either:
        // - the next node on the way on the other side of the lat/lon spectrum OR
        // - the closest node that happens to be on a different Way (for starting positions on corners)
        let mut next_closest: Option<Edge> = None;
        for edge in results_iter {
            println!("{:?}", edge.distance);
            println!("\t{:?}", edge.geometry);
            println!("\tlat_diff sign: {:?}", edge.distance.lat_diff.signum());
            // this Node is not on the same Way as the `closest`
            // assuming this means the starting position is on / near an intersection
            // and we can start on either of these Ways at fairly similar cost
            if closest.way != edge.way {
                next_closest = Some(edge);
                break;
            }

            // This Node is on the same Way as the `closest`
            // but on the other side of the lat/lon spectrum
            // so we can start our alg choosing from one of these two Nodes
            if closest.distance.lat_diff.signum() != edge.distance.lat_diff.signum()
                || closest.distance.lon_diff.signum() != edge.distance.lon_diff.signum()
            {
                next_closest = Some(edge);
                break;
            }
        }

        Ok(vec![Some(closest), next_closest])
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
    /// returns a Vec of NodeId-WayId pairs, or the Node neighbor + the Way that connects them
    pub fn get_neighbors(&self, id: NodeId) -> Result<Vec<Neighbor>, anyhow::Error> {
        let mut stmt = self
            .conn
            .prepare_cached("SELECT way, n2 FROM Segments WHERE n1 = ?1")?;
        let result = stmt.query_map([id], |row| {
            Ok(Neighbor {
                way: row.get(0)?,
                node: row.get(1)?,
            })
        })?;

        Ok(result.map(|r| r.unwrap()).collect())
    }
}
