use std::ops::Add;

use rusqlite::Connection;
use serde::{Deserialize, Serialize};

pub mod db;
pub mod etl;
pub mod graph;

pub struct Graph {
    conn: Connection,
}

pub type NodeId = i64;
pub type WayId = i64;

// TODO: this needs context about who it's a neighbor TO!
// at which point...is this just an Edge / Segment?
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Neighbor {
    pub way: WayId,
    pub node: Node,
}

#[derive(Debug, Copy, Clone, PartialEq, Deserialize)]
pub struct Node {
    pub id: NodeId,
    pub lat: f64,
    pub lon: f64,
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

#[derive(Serialize, Copy, Clone, Debug)]
pub struct LocationDistance {
    pub lat_diff: f64,
    pub lon_diff: f64,
    pub total: f64,
}

impl Add for LocationDistance {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            lat_diff: self.lat_diff + other.lat_diff,
            lon_diff: self.lon_diff + other.lon_diff,
            total: self.total + other.total,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Location {
    pub lat: f64,
    pub lon: f64,
}
