use rusqlite::Connection;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use crate::osm;

type NodeId = String;
type WayId = String;

// TODO: enum?
type TagKey = String;
type TagValue = String;

// just using serde_json to/from_str before/after sqlite calls
// which is essentially what's happening in the JSON From/ToSql implementations
// here: https://docs.rs/rusqlite/0.31.0/src/rusqlite/types/serde_json.rs.html#17-29
type Neighbors = HashMap<NodeId, WayId>;
type Tags = HashMap<TagKey, TagValue>;

#[derive(Debug, Serialize, Deserialize)]
struct Node {
    id: NodeId,
    lat: f32,
    lon: f32,
    neighbors: Neighbors,
    tags: Tags,
}

#[derive(Debug, Serialize, Deserialize)]
struct Way {
    id: WayId,
    min_lat: f32,
    min_lon: f32,
    max_lat: f32,
    max_lon: f32,
    nodes: Vec<NodeId>,
    tags: Tags,
}

const DB_PATH: &str = "./db.db3";

/// initializes a sqlite database at DATABASE_URL with the requisite tables
pub fn create_tables() -> Result<(), anyhow::Error> {
    let conn = Connection::open(DB_PATH)?;

    conn.execute_batch("
        DROP TABLE IF EXISTS Node;
        CREATE TABLE Node (
            id TEXT PRIMARY KEY,
            lat REAL NOT NULL,
            lon REAL NOT NULL,
            neighbors TEXT,
            tags TEXT
        );

        DROP TABLE IF EXISTS Way;
        CREATE TABLE Way (
            id TEXT PRIMARY KEY,
            minLat REAL NOT NULL,
            minLon REAL NOT NULL,
            maxLat REAL NOT NULL,
            maxLon REAL NOT NULL,
            nodes TEXT,
            tags TEXT
        );
    ")?;
    println!("Tables created");

    let seed = Node {
        id: "0".to_owned(),
        lat: 40.5,
        lon: 70.5,
        neighbors: HashMap::from([
                    ("1".to_owned(), "1".to_owned()),
                    ("2".to_owned(), "2".to_owned()),
                ]),
        tags: HashMap::from([
                    ("highway".to_owned(), "traffic_signals".to_owned()),
                ]),
    };

    conn.execute(
        "INSERT INTO Node (id, lat, lon, neighbors, tags) VALUES (?1, ?2, ?3, ?4, ?5)",
        (
            &seed.id,
            &seed.lat,
            &seed.lon,
            serde_json::to_string(&seed.neighbors).unwrap(),
            serde_json::to_string(&seed.tags).unwrap(),
        ),
    )?;

    let seed = Way {
        id: "0".to_owned(),
        min_lat: 40.5,
        min_lon: 70.5,
        max_lat: 48.5,
        max_lon: 78.5,
        nodes: vec!["0".to_owned()],
        tags: HashMap::from([
                    ("highway".to_owned(), "traffic_signals".to_owned()),
                ]),
    };

    conn.execute(
        "INSERT INTO Way (id, minLat, minLon, maxLat, maxLon, nodes, tags) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        (
            &seed.id,
            &seed.min_lat,
            &seed.min_lon,
            &seed.max_lat,
            &seed.max_lon,
            serde_json::to_string(&seed.nodes).unwrap(),
            serde_json::to_string(&seed.tags).unwrap(),
        ),
    )?;

    Ok(())
}

// TODO: batch insert queries...if they're useful
/// Insert a Node to the DB, synchronously
pub fn insert_node(node: osm::Element) -> anyhow::Result<()> {
    let conn = Connection::open(DB_PATH)?;

    conn.execute(
        "INSERT INTO Node (id, lat, lon, neighbors, tags) VALUES (?1, ?2, ?3, ?4, ?5)",
        (
            &node.id,
            &node.lat,
            &node.lon,
            "{}", // inits w/ empty `neighbors` adjacency matrix
            serde_json::to_string(&node.tags).unwrap(),
        ),
    ).unwrap_or_else(|e| {
        eprintln!("Failed Node:\n{:#?}", node); 
        panic!("{e}");
    });

    Ok(())
}

/// Insert a Way to the DB, synchronously
pub fn insert_way(way: osm::Element) -> anyhow::Result<()> {
    let conn = Connection::open(DB_PATH)?;

    let Some(bounds) = way.bounds else {
        eprintln!("No bounds present on Way {}", way.id);
        return Ok(());
    };

    let nodes = way.nodes.unwrap_or_default();

    conn.execute(
        "INSERT INTO Way (id, minLat, minLon, maxLat, maxLon, nodes, tags) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        (
            &way.id,
            bounds.minlat,
            bounds.minlon,
            bounds.maxlat,
            bounds.maxlon,
            serde_json::to_string(&nodes).unwrap(),
            serde_json::to_string(&way.tags).unwrap(),
        ),
    ).unwrap_or_else(|e| {
        eprintln!("Failed Way: {}", way.id); 
        panic!("{e}");
    });

    Ok(())
}

pub fn get_neighbors(id: NodeId) -> Result<Neighbors, anyhow::Error> {
    let conn = Connection::open(DB_PATH)?;

    let row: String = conn.query_row("SELECT neighbors FROM Node WHERE id = ?1", [id], |row| {
        row.get(0)
    })?;

    Ok(serde_json::from_str(&row)?)
}
