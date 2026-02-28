use super::repository::GraphRepository;
use crate::db::{self, DBConnection};
use crate::osm::{Distance, Neighbor, Node, NodeId, WayId, WayLabels};
use geo::Point;
use std::collections::HashMap;
use tracing::info;

/// A compact edge in the adjacency list
#[derive(Clone)]
struct InMemoryEdge {
    way: WayId,
    node: Node,
    distance: Distance,
    labels: WayLabels,
}

/// In-memory graph repository: loads Segments + Nodes + WayLabels into a HashMap at startup,
/// eliminating per-expansion SQL queries from the A* hot loop.
///
/// R*Tree snapping queries (called only 2× per route) are delegated to a retained SQLite
/// connection so spatial indexing is preserved.
pub struct InMemoryGraphRepository {
    /// Retained SQLite connection for R*Tree-backed snapping queries only
    snap_db: DBConnection,
    /// Adjacency list: NodeId → outgoing edges (with labels pre-joined)
    adjacency: HashMap<NodeId, Vec<InMemoryEdge>>,
}

impl InMemoryGraphRepository {
    pub fn new() -> Result<Self, anyhow::Error> {
        let snap_db = db::get_conn()?;
        let load_conn = db::get_conn()?;

        info!("Loading graph into memory...");
        let adjacency = Self::load_adjacency(&load_conn)?;
        info!("Graph loaded: {} nodes in adjacency list", adjacency.len());

        Ok(Self { snap_db, adjacency })
    }

    /// Bulk-load the full adjacency list from SQLite in a single query.
    /// Each row becomes one InMemoryEdge stored under its source node (n1).
    fn load_adjacency(conn: &DBConnection) -> Result<HashMap<NodeId, Vec<InMemoryEdge>>, anyhow::Error> {
        let mut stmt = conn.prepare(
            "
            SELECT S.n1, S.way, S.n2, N2.lon, N2.lat, S.distance, WL.cycleway, WL.road, WL.salmon
            FROM Segments S
            JOIN Nodes N2 ON S.n2 = N2.id
            JOIN WayLabels WL ON S.way = WL.id
            ",
        )?;

        let mut adjacency: HashMap<NodeId, Vec<InMemoryEdge>> = HashMap::new();

        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            let n1: NodeId = row.get(0)?;
            let edge = InMemoryEdge {
                way: row.get(1)?,
                node: Node::new(row.get(2)?, &Point::new(row.get(3)?, row.get(4)?)),
                distance: row.get(5)?,
                labels: (row.get(6)?, row.get(7)?, row.get(8)?),
            };
            adjacency.entry(n1).or_default().push(edge);
        }

        Ok(adjacency)
    }
}

impl GraphRepository for InMemoryGraphRepository {
    /// Delegate to SQLite — uses R*Tree spatial index, only called 2× per route
    fn get_snapped_neighbors(
        &self,
        center: Point,
        snap_radius: Option<f64>,
    ) -> Result<Vec<Neighbor>, anyhow::Error> {
        const DEFAULT_SNAP_RADIUS: f64 = 0.0002;
        snap_snapped_neighbors_from_conn(&self.snap_db, center, snap_radius.unwrap_or(DEFAULT_SNAP_RADIUS))
    }

    fn get_neighbors(&self, id: NodeId) -> Result<Vec<Neighbor>, anyhow::Error> {
        Ok(self
            .adjacency
            .get(&id)
            .map(|edges| {
                edges
                    .iter()
                    .map(|e| Neighbor {
                        way: e.way,
                        node: e.node,
                        distance: e.distance,
                    })
                    .collect()
            })
            .unwrap_or_default())
    }

    fn get_neighbors_with_labels(
        &self,
        id: NodeId,
    ) -> Result<Vec<(Neighbor, WayLabels)>, anyhow::Error> {
        Ok(self
            .adjacency
            .get(&id)
            .map(|edges| {
                edges
                    .iter()
                    .map(|e| {
                        (
                            Neighbor {
                                way: e.way,
                                node: e.node,
                                distance: e.distance,
                            },
                            e.labels,
                        )
                    })
                    .collect()
            })
            .unwrap_or_default())
    }

    fn get_way_labels(&self, way: WayId) -> Result<WayLabels, anyhow::Error> {
        // Find any edge with this way_id and return its labels.
        // Called rarely (only for snapped starting edges), so linear scan is acceptable.
        for edges in self.adjacency.values() {
            if let Some(e) = edges.iter().find(|e| e.way == way) {
                return Ok(e.labels);
            }
        }
        anyhow::bail!("Way {} not found in adjacency list", way)
    }
}

/// Snapping logic extracted from SqliteGraphRepository so we can reuse it
/// against the snap_db connection without constructing a full SqliteGraphRepository.
fn snap_snapped_neighbors_from_conn(
    conn: &DBConnection,
    center: Point,
    snap_radius: f64,
) -> Result<Vec<Neighbor>, anyhow::Error> {
    use crate::osm::Distance;
    use anyhow::anyhow;
    use geo::prelude::*;

    const MAX_SNAP_RADIUS: f64 = 0.001;
    const SNAP_INCREMENT: f64 = 0.0002;

    type Bearing = f64;

    let mut stmt = conn.prepare_cached(
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

    if results.is_empty() {
        if snap_radius >= MAX_SNAP_RADIUS {
            return Err(anyhow!("Could not snap coords to graph"));
        }
        return snap_snapped_neighbors_from_conn(conn, center, snap_radius + SNAP_INCREMENT);
    }

    results.sort_by(|(n1, _), (n2, _)| n1.distance.partial_cmp(&n2.distance).unwrap());

    let mut results_iter = results.into_iter();
    let (closest_neighbor, closest_bearing) = results_iter.next().unwrap();

    let mut next_closest: Option<Neighbor> = None;
    for (neighbor, bearing) in results_iter {
        if closest_neighbor.way == neighbor.way && ((closest_bearing - bearing).abs() >= 90.) {
            next_closest = Some(neighbor);
            break;
        }
    }

    let mut out = vec![closest_neighbor];
    if let Some(n) = next_closest {
        out.push(n);
    }
    Ok(out)
}
