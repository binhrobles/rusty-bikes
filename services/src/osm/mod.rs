use geo::Point;
use geojson::ser::serialize_geometry;
use rusqlite::Connection;
use serde::{Deserialize, Serialize, Serializer};

pub mod db;
pub mod etl;
pub mod graph;
pub mod traversal;

#[derive(Debug)]
pub struct Graph {
    conn: Connection,
}

pub type NodeId = i64;
pub type WayId = i64;
pub type Distance = f64;

// TODO: this needs context about who it's a neighbor TO!
// at which point...is this just an Edge / Segment?
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Neighbor {
    pub way: WayId,
    pub node: Node,
    pub distance: Distance,
    // TODO: bearing?
}

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct Node {
    pub id: NodeId,

    #[serde(serialize_with = "serialize_geometry")]
    pub geometry: Point,
}

impl Node {
    pub fn new(id: NodeId, point: &Point) -> Self {
        Self {
            id,
            geometry: *point,
        }
    }
}

/// simple serialization of a Node to just its ID
pub fn serialize_node_simple<S>(node: &Node, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_i64(node.id)
}

/// serialization of a float to an int
pub fn serialize_float_as_int<S>(float: &f64, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_i64(*float as i64)
}

#[derive(Debug, Deserialize)]
pub struct Way {
    pub id: WayId,
    pub min_lat: f64,
    pub max_lat: f64,
    pub min_lon: f64,
    pub max_lon: f64,
}

pub type WayNodePosition = usize;

#[derive(Debug, Deserialize)]
pub struct Location {
    pub lat: f64,
    pub lon: f64,
}
