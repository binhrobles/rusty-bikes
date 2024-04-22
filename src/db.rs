use rusqlite::Connection;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use crate::osm;

type NodeId = String;
type WayId = String;

// just using serde_json to/from_str before/after sqlite calls
// which is essentially what's happening in the JSON From/ToSql implementations
// here: https://docs.rs/rusqlite/0.31.0/src/rusqlite/types/serde_json.rs.html#17-29
type Neighbors = HashMap<NodeId, WayId>;

#[derive(Debug, Serialize, Deserialize)]
struct Node {
    id: NodeId,
    lat: f32,
    lon: f32,
    neighbors: Neighbors,
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
            neighbors TEXT
        );"
    )?;
    println!("Table created");

    let seed = Node {
        id: "0".to_owned(),
        lat: 40.5,
        lon: 70.5,
        neighbors: HashMap::from([
                        ("1".to_owned(), "1".to_owned()),
                        ("2".to_owned(), "2".to_owned()),
                    ]),
    };

    conn.execute(
        "INSERT INTO Node (id, lat, lon, neighbors) VALUES (?1, ?2, ?3, ?4)",
        (
            &seed.id,
            &seed.lat,
            &seed.lon,
            serde_json::to_string(&seed.neighbors).unwrap(),
        ),
    )?;

    Ok(())
}

/// Insert a Node to the DB, synchronously
// TODO: create a batch insert query...if it's useful
pub fn insert_node(node: osm::Element) -> anyhow::Result<()> {
    let conn = Connection::open(DB_PATH)?;
    conn.execute(
        "INSERT INTO Node (id, lat, lon, neighbors) VALUES (?1, ?2, ?3, ?4)",
        (
            &node.id,
            &node.lat,
            &node.lon,
            "{}", // inits w/ empty `neighbors` adjacency matrix
        ),
    )?;
    Ok(())
}

pub fn get_neighbors(id: NodeId) -> Result<Neighbors, anyhow::Error> {
    let conn = Connection::open(DB_PATH)?;

    let row: String = conn.query_row("SELECT neighbors FROM Node WHERE id = ?1", [id], |row| {
        row.get(0)
    })?;

    Ok(serde_json::from_str(&row)?)
}
