use rusqlite::Connection;
use serde::Deserialize;

pub mod db;
pub mod etl;
pub mod graph;

pub struct Graph {
    conn: Connection,
}

pub type NodeId = i64;
pub type WayId = i64;

#[derive(Debug)]
pub struct Neighbor {
    pub way: WayId,
    pub node: NodeId,
    // TODO: distance
}

#[derive(Debug, Deserialize)]
pub struct Node {
    pub id: NodeId,
    pub lat: f32,
    pub lon: f32,
}

#[derive(Debug, Deserialize)]
pub struct Way {
    pub id: WayId,
    pub min_lat: f32,
    pub max_lat: f32,
    pub min_lon: f32,
    pub max_lon: f32,
}

pub type WayNodePosition = usize;

#[derive(Debug, Deserialize)]
pub struct Location {
    pub lat: f64,
    pub lon: f64,
}

#[derive(Debug)]
pub struct LocationDistance {
    pub lat_diff: f64,
    pub lon_diff: f64,
    pub total: f64,
}

impl Location {
    /// get the distance b/w two locations, in cartesian units
    fn distance(&self, loc: &Location) -> LocationDistance {
        let lat_diff = loc.lat - self.lat;
        let lon_diff = loc.lon - self.lon;
        LocationDistance {
            lat_diff,
            lon_diff,
            total: (lat_diff.powi(2) + lon_diff.powi(2)).sqrt(),
        }
    }
}
