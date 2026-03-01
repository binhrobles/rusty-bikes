use crate::db::{self, DBConnection};
use crate::osm::{Distance, Neighbor, Node, NodeId, WayId, WayLabels};
use anyhow::anyhow;
use geo::prelude::*;
use geo::Point;
use std::collections::HashMap;
use tracing::debug;

const MAX_SNAP_RADIUS: f64 = 0.001;
const SNAP_INCREMENT: f64 = 0.0002;

/// SQLite abstraction for Graph operations
pub trait GraphRepository {
    fn get_snapped_neighbors(
        &self,
        center: Point,
        snap_radius: Option<f64>,
    ) -> Result<Vec<Neighbor>, anyhow::Error>;
    fn get_neighbors(&self, id: NodeId) -> Result<Vec<Neighbor>, anyhow::Error>;
    fn get_neighbors_with_labels(
        &self,
        id: NodeId,
    ) -> Result<Vec<(Neighbor, WayLabels)>, anyhow::Error>;
    fn get_way_labels(&self, way: WayId) -> Result<WayLabels, anyhow::Error>;
    fn get_way_names(&self, way_ids: &[WayId]) -> Result<HashMap<WayId, String>, anyhow::Error>;
}

pub struct SqliteGraphRepository {
    conn: DBConnection,
}

impl SqliteGraphRepository {
    pub fn new() -> Result<Self, anyhow::Error> {
        Ok(Self {
            conn: db::get_conn()?,
        })
    }
}

impl GraphRepository for SqliteGraphRepository {
    /// Returns the closest Node(s) to the location provided
    /// Creates a buffer around the location and searches a small square area
    ///
    /// Implementation notes:
    /// - We cannot guarantee that the first Way returned from the R*tree query will be
    /// the closest Way, because of how R*Trees work
    /// - TODO: this sometimes returns the wrong street (ie: the next one over)
    /// existing Node
    /// - TODO: handle no Ways returned, empty case
    fn get_snapped_neighbors(
        &self,
        center: Point,
        snap_radius: Option<f64>,
    ) -> Result<Vec<Neighbor>, anyhow::Error> {
        type Bearing = f64;
        let snap_radius = snap_radius.unwrap_or(SNAP_INCREMENT);

        let mut stmt = self.conn.prepare_cached(
            "
            SELECT WayNodes.node, lon, lat, WayNodes.way
            FROM Ways
            JOIN WayNodes ON WayNodes.way=Ways.id
            JOIN Nodes ON WayNodes.node=Nodes.id
            WHERE minLon <= ?1
              AND maxLon >= ?2
              AND minLat <= ?3
              AND maxLat >= ?4
        ",
        )?;

        let center_lon = center.x();
        let center_lat = center.y();
        let results = stmt.query_map(
            [
                center_lon + snap_radius,
                center_lon - snap_radius,
                center_lat + snap_radius,
                center_lat - snap_radius,
            ],
            |row| {
                let lon = row.get(1)?;
                let lat = row.get(2)?;

                let loc = Point::new(lon, lat);

                // for each returned Node, calculate the distance from the center point
                Ok((
                    Neighbor {
                        node: Node::new(row.get(0)?, &loc),
                        way: row.get(3)?,
                        distance: center.haversine_distance(&loc) as Distance,
                    },
                    center.haversine_bearing(loc),
                ))
            },
        )?;

        let mut results: Vec<(Neighbor, Bearing)> = results.map(|r| r.unwrap()).collect();

        // if no results returned w/ this snap radius, expand it by a bit
        if results.is_empty() {
            if snap_radius >= MAX_SNAP_RADIUS {
                return Err(anyhow!("Could not snap coords to graph"));
            }

            debug!("Could not snap coords to graph, expanding");
            return self.get_snapped_neighbors(center, Some(snap_radius + SNAP_INCREMENT));
        }

        // sort these results by the total distance from the center point
        results.sort_by(|(n1, _), (n2, _)| n1.distance.partial_cmp(&n2.distance).unwrap());

        // at this point we have a sorted list of nodes by distance
        // the first, closest node is clearly the best candidate
        let mut results_iter = results.into_iter();
        let (closest_neighbor, closest_bearing) = results_iter.next().unwrap();

        // then, use the wayId + the bearing relationship to find
        // the next node on the way on the other side of the center point
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
    fn get_neighbors(&self, id: NodeId) -> Result<Vec<Neighbor>, anyhow::Error> {
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

    /// given a NodeId, gets the neighbors from the Segments table and labels from WayLabels table
    /// returns a Vec of (Neighbor, WayLabels) representing each "edge" out of this Node
    fn get_neighbors_with_labels(
        &self,
        id: NodeId,
    ) -> Result<Vec<(Neighbor, WayLabels)>, anyhow::Error> {
        // flamegraphs show we spend 95%+ of our time in this query
        let mut stmt = self.conn.prepare_cached(
            "
            SELECT way, n2, N2.lon, N2.lat, distance, WL.cycleway, WL.road, WL.salmon
            FROM Segments
            JOIN Nodes as N2 ON n2=N2.id
            JOIN WayLabels as WL ON way=WL.id
            WHERE n1 = ?1
        ",
        )?;
        let result = stmt.query_map([id], |row| {
            Ok((
                Neighbor {
                    way: row.get(0)?,
                    node: Node::new(row.get(1)?, &Point::new(row.get(2)?, row.get(3)?)),
                    distance: row.get(4)?,
                },
                (row.get(5)?, row.get(6)?, row.get(7)?),
            ))
        })?;

        Ok(result.map(|r| r.unwrap()).collect())
    }

    fn get_way_labels(&self, way: WayId) -> Result<WayLabels, anyhow::Error> {
        let mut stmt = self.conn.prepare_cached(
            "
            SELECT cycleway, road, salmon
            FROM WayLabels
            WHERE id = ?1
        ",
        )?;

        let results = stmt.query_row([way], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?;

        Ok(results)
    }

    fn get_way_names(&self, way_ids: &[WayId]) -> Result<HashMap<WayId, String>, anyhow::Error> {
        if way_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let placeholders: Vec<String> = way_ids.iter().map(|_| "?".to_string()).collect();
        let sql = format!(
            "SELECT id, name FROM WayLabels WHERE id IN ({})",
            placeholders.join(",")
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let params: Vec<&dyn rusqlite::types::ToSql> = way_ids
            .iter()
            .map(|id| id as &dyn rusqlite::types::ToSql)
            .collect();
        let rows = stmt.query_map(params.as_slice(), |row| {
            Ok((row.get::<_, WayId>(0)?, row.get::<_, String>(1)?))
        })?;

        let mut result = HashMap::new();
        for row in rows {
            let (id, name) = row?;
            result.insert(id, name);
        }
        Ok(result)
    }
}
