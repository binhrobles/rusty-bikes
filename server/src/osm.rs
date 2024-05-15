use geo::Point;
use rusqlite::Connection;
use serde::Deserialize;

pub mod db;
pub mod etl;
pub mod graph;

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
}

#[derive(Debug, Copy, Clone, PartialEq, Deserialize)]
pub struct Node {
    pub id: NodeId,
    pub lon: f64,
    pub lat: f64,

    #[serde(serialize_with = "serialize_geometry")]
    pub geometry: Point,
}

impl Node {
    pub fn new(id: NodeId, lon: f64, lat: f64) -> Node {
        Node {
            id,
            lon,
            lat,
            geometry: Point::new(lon, lat),
        }
    }

    pub fn from_point(id: NodeId, point: &Point) -> Node {
        Node {
            id,
            lon: point.x(),
            lat: point.y(),
            geometry: *point,
        }
    }
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
