/// Exposes DB interactions as a Graph interface
use super::{db, serialize_node_simple, Graph, Neighbor, Node, NodeId, WayId};
use anyhow::anyhow;
use geo::prelude::*;
use geo::{Coord, HaversineBearing, Line, LineString, Point};
use geojson::ser::serialize_geometry;
use serde::{Serialize, Serializer};
use std::collections::{HashMap, VecDeque};

const START_NODE_ID: NodeId = 0;
const END_NODE_ID: NodeId = -1;

type Depth = usize;

#[derive(Serialize, Clone, Debug)]
pub struct Route {
    #[serde(serialize_with = "serialize_route")]
    geometry: Vec<Coord>,
}

impl Route {
    pub fn new(node: &Node) -> Route {
        Route {
            geometry: vec![node.geometry.into()],
        }
    }

    /// extends this route with the specified node
    pub fn extend_with(&mut self, node: &Node) {
        // TODO: push into LineString or MultiLineString instead of Vec<Coord>
        // so we don't have to do custom serialization
        self.geometry.push(node.geometry.into());
    }
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

#[derive(Clone, Debug, Serialize)]
pub struct TraversalSegment {
    #[serde(serialize_with = "serialize_node_simple")]
    from: Node,
    #[serde(serialize_with = "serialize_node_simple")]
    to: Node,

    #[serde(serialize_with = "serialize_geometry")]
    geometry: Line,

    // segment metadata for weighing / constructing the route
    depth: Depth,
    distance: f64,
    way: WayId,
    // cost
}

impl TraversalSegment {
    pub fn new_to_neighbor(from: &Node, to: &Neighbor, depth: usize) -> TraversalSegment {
        TraversalSegment {
            from: *from,
            to: to.node,
            geometry: Line::new(from.geometry, to.node.geometry),
            distance: to.distance,
            depth,
            way: to.way,
        }
    }

    pub fn new_to_node(from: &Node, to: &Node, way: WayId, depth: usize) -> TraversalSegment {
        TraversalSegment {
            from: *from,
            to: *to,
            geometry: Line::new(from.geometry, to.geometry),
            distance: from.geometry.haversine_distance(&to.geometry),
            depth,
            way,
        }
    }
}

impl Graph {
    pub fn new() -> Result<Self, anyhow::Error> {
        Ok(Self {
            conn: db::get_conn()?,
        })
    }

    /// This boring function returns the route bw the points provided, as a single, pretty Route
    /// struct with no fun decorations or anything
    /// Uses the most ADVANCED algorithm currently known to {me}
    pub fn route_between(&self, start: Point, end: Point) -> Result<Route, anyhow::Error> {
        let traversal = self.traverse_between(start, end)?;

        let end_node = Node::new(END_NODE_ID, &end);

        println!("constructing route from {:#?}", traversal);

        // construct route from traversal information
        let mut route = Route::new(&end_node);
        let mut current_segment = traversal.get(&END_NODE_ID).unwrap();

        loop {
            println!("extending with {}", current_segment.from.id);
            route.extend_with(&current_segment.from);

            if current_segment.from.id == START_NODE_ID {
                break;
            }

            current_segment = traversal.get(&current_segment.from.id).unwrap();
        }

        // TODO: reverse?
        // TODO: retain route metadata during Route extensions? condense per Way?

        Ok(route)
    }

    /// Return a collection of all TraversalSegments examined while routing between the start and
    /// end Points. TraversalSegments will be decorated with both the depth of the traversal and
    /// the cost assigned, given the designated cost model
    pub fn traverse_between(
        &self,
        start: Point,
        end: Point,
    ) -> Result<HashMap<NodeId, TraversalSegment>, anyhow::Error> {
        let start_node = Node::new(START_NODE_ID, &start);
        let end_node = Node::new(END_NODE_ID, &end);

        let starting_neighbors = self.guess_neighbors(start)?;

        // get all the possible Nodes we can connect to as "destination" targets
        let target_neighbors = self.guess_neighbors(end)?;
        let target_neighbor_node_ids: Vec<NodeId> =
            target_neighbors.iter().map(|n| n.node.id).collect();

        // init the traversal queue and visited set w/ the starting neighbors
        let mut queue: VecDeque<TraversalSegment> = VecDeque::new();
        let mut came_from: HashMap<NodeId, TraversalSegment> = HashMap::new();

        for neighbor in starting_neighbors {
            let segment = TraversalSegment::new_to_neighbor(&start_node, &neighbor, 0);
            came_from.insert(neighbor.node.id, segment.clone());
            queue.push_back(segment);
        }

        while !queue.is_empty() {
            let current = queue.pop_front().unwrap();

            // exit condition -- entrypoint to DRY-able?
            if target_neighbor_node_ids.contains(&current.to.id) {
                let segment = TraversalSegment::new_to_node(
                    &current.to,
                    &end_node,
                    current.way,
                    current.depth + 1,
                );
                came_from.insert(END_NODE_ID, segment);
                return Ok(came_from);
            }

            // find neighbors at the end of this segment
            let adjacent_neighbors = self.get_neighbors(current.to.id)?;

            for neighbor in adjacent_neighbors {
                // only act for neighbors that haven't been visited already
                came_from.entry(neighbor.node.id).or_insert({
                    let segment = TraversalSegment::new_to_neighbor(
                        &current.to,
                        &neighbor,
                        current.depth + 1,
                    );
                    queue.push_back(segment.clone());

                    segment
                });
            }
        }

        Err(anyhow!("No route found!"))
    }

    /// Return a collection of TraversalSegments from traversing the Graph from the start point to
    /// the depth specified
    pub fn traverse_from(
        &self,
        start: Point,
        max_depth: usize,
    ) -> Result<Vec<TraversalSegment>, anyhow::Error> {
        let start_node = Node::new(START_NODE_ID, &start);

        let starting_neighbors = self.guess_neighbors(start)?;

        // init the traversal queue and visited set w/ the starting neighbors
        let mut queue: VecDeque<TraversalSegment> = VecDeque::new();
        let mut came_from: HashMap<NodeId, TraversalSegment> = HashMap::new();
        for neighbor in starting_neighbors {
            let segment = TraversalSegment::new_to_neighbor(&start_node, &neighbor, 0);
            came_from.insert(neighbor.node.id, segment.clone());
            queue.push_back(segment);
        }

        while !queue.is_empty() {
            // println!(
            //     "queue: {:#?}",
            //     queue
            //         .clone()
            //         .iter()
            //         .map(|s| s.to.id)
            //         .collect::<Vec<NodeId>>()
            // );
            // println!(
            //     "came_from: {:#?}",
            //     came_from
            //         .clone()
            //         .iter()
            //         .map(|(k, v)| format!("{}: {}->{}", k, v.from.id, v.to.id))
            //         .collect::<Vec<String>>()
            // );
            // println!("============");

            let current = queue.pop_front().unwrap();

            // exit condition
            if current.depth == max_depth {
                break;
            }

            // find outbound segments for this node
            let adjacent_neighbors = self.get_neighbors(current.to.id)?;

            for neighbor in adjacent_neighbors {
                came_from.entry(neighbor.node.id).or_insert_with(|| {
                    let segment = TraversalSegment::new_to_neighbor(
                        &current.to,
                        &neighbor,
                        current.depth + 1,
                    );
                    queue.push_back(segment.clone());

                    segment
                });
            }
        }

        Ok(came_from.values().cloned().collect())
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
