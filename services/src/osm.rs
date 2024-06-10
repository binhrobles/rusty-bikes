/// holds types and structs related to Open Street Map data
use geo::Point;
use geojson::ser::serialize_geometry;
use rusqlite::types::FromSql;
use serde::{Deserialize, Serialize, Serializer};

pub type NodeId = i64;
pub type WayId = i64;
pub type Distance = f32;

pub type WayLabels = (Cycleway, Road, Salmoning);

/// Whether we are swimming against the stream (of traffic)
pub type Salmoning = bool;

/// Designations for varying levels of bike lanes
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize)]
#[repr(u8)]
pub enum Cycleway {
    /// There is no bike-specific infra
    Shared = 0,
    /// The bike lane is designated, but on the road itself
    Lane,
    /// The bike lane is separate from the road / protected
    Track,
}

/// Designations for varying levels of roads
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize)]
#[repr(u8)]
pub enum Road {
    /// Walking path, generally for pedestrians
    Pedestrian = 0,
    /// A dedicated bike path ❤️
    Bike,
    /// A low-speed street, residential or otherwise
    Local,
    /// A medium-speed road connecting arterial roads to local roads
    Collector,
    /// A high-speed, high traffic road
    Arterial,
}

// Allow interpretation of SQLite stored ints as enums again
impl FromSql for Cycleway {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        Ok(unsafe { ::std::mem::transmute(value.as_i64()? as u8) })
    }
}

// Allow interpretation of SQLite stored ints as enums again
impl FromSql for Road {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        Ok(unsafe { ::std::mem::transmute(value.as_i64()? as u8) })
    }
}

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

impl From<Location> for Point {
    fn from(l: Location) -> Point {
        Point::new(l.lon, l.lat)
    }
}
