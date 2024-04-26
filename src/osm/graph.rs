/// Exposes DB interactions as a Graph interface
use super::{db, Graph, Location, LocationDistance, Neighbor, NodeId, Way, WayNodePosition};

impl Graph {
    pub fn new() -> Result<Self, anyhow::Error> {
        Ok(Self {
            conn: db::get_conn()?,
        })
    }

    /// Gets the closest 2 Neighbors to the location provided
    ///
    /// Implementation notes:
    /// - We cannot guarantee that the first Way returned from the R*tree query will be
    /// the closest Way, because of how R*Trees work
    /// - TODO: locations on corners are edge cases
    /// - TODO: locations directly on Nodes are edge cases (or will this be accounted for by the alg's
    /// cost model?)
    /// - TODO: handle no Ways returned, empty case
    pub fn guess_neighbors(
        &self,
        location: Location,
    ) -> Result<Vec<Option<Neighbor>>, anyhow::Error> {
        let mut stmt = self.conn.prepare_cached(
            "
            SELECT WayNodes.way, WayNodes.node, lat, lon, WayNodes.pos
            FROM Ways
            JOIN WayNodes ON WayNodes.way=Ways.id
            JOIN Nodes ON WayNodes.node=Nodes.id
            WHERE minLat <= ?1
              AND maxLat >= ?1
              AND minLon <= ?2
              AND maxLon >= ?2
        ",
        )?;
        let results = stmt.query_map([location.lat, location.lon], |row| {
            let nbr: Neighbor = Neighbor {
                way: row.get(0)?,
                node: row.get(1)?,
            };
            let loc: Location = Location {
                lat: row.get(2)?,
                lon: row.get(3)?,
            };

            // for each returned Node, calculate the distance from the start point
            let dist = location.distance(&loc);
            Ok((nbr, loc, dist, row.get(4)?))
        })?;

        let mut results: Vec<(Neighbor, Location, LocationDistance, WayNodePosition)> =
            results.map(|r| r.unwrap()).collect();

        // sort these results by the total distance from the start point
        results.sort_by(|a, b| a.2.total.partial_cmp(&b.2.total).unwrap());

        // at this point we have a sorted list of nodes by distance
        // the first, closest node is clearly the best candidate
        let mut results_iter = results.into_iter();
        let closest = results_iter.next().unwrap();

        // then, use the wayId + signs of the lat_diff / lon_diff to find either:
        // - the next node on the way on the other side of the lat/lon spectrum OR
        // - the closest node that happens to be on a different Way (for starting positions on corners)
        let mut next_closest: Option<Neighbor> = None;
        for (neighbor, _, distance, pos) in results_iter {
            println!("{:?} {:?}", distance, pos);
            println!("\t{:?}", neighbor);
            println!("\tlat_diff sign: {:?}", distance.lat_diff.signum());
            // this Node is not on the same Way as the `closest`
            // assuming this means the starting position is on / near an intersection
            // and we can start on either of these Ways at fairly similar cost
            if closest.0.way != neighbor.way {
                next_closest = Some(neighbor);
                break;
            }

            // This Node is on the same Way as the `closest`
            // but on the other side of the lat/lon spectrum
            // so we can start our alg choosing from one of these two Nodes
            if closest.2.lat_diff.signum() != distance.lat_diff.signum()
                || closest.2.lon_diff.signum() != distance.lon_diff.signum()
            {
                next_closest = Some(neighbor);
                break;
            }
        }

        Ok(vec![Some(closest.0), next_closest])
    }

    pub fn ind_bounding_ways(&self, location: Location) -> Result<Vec<Way>, anyhow::Error> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT id, minLat, maxLat, minLon, maxLon FROM Ways
            WHERE minLat <= ?1
            AND maxLat >= ?1
            AND minLon <= ?2
            AND maxLon >= ?2",
        )?;
        let result = stmt.query_map([location.lat, location.lon], |row| {
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
