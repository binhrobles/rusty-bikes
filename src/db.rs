use rusqlite::{Connection, Result};
use serde::{Deserialize, Serialize};

type NodeId = u64;
type WayId = u64;

// Note: future hashmap? does that matter?
type Neighbors = Vec<(NodeId, WayId)>;

#[derive(Debug, Serialize, Deserialize)]
struct Node {
    lat: f32,
    long: f32,
    id: NodeId,
    neighbors: Neighbors,
}

pub fn init() -> Result<(), rusqlite::Error> {
    let path = "./db.db3";
    let conn = Connection::open(path)?;

    conn.execute(
        "CREATE TABLE Node (
        id INTEGER PRIMARY KEY,
        lat REAL NOT NULL,
        long REAL NOT NULL,
        neighbors BLOB
    )",
        (), // empty list of parameters.
    )?;
    let seed = Node {
        id: 0,
        lat: 40.5,
        long: 70.5,
        neighbors: vec![(1, 1), (2, 2)],
    };
    conn.execute(
        "INSERT INTO Node (id, lat, long, neighbors) VALUES (?1, ?2, ?3, ?4)",
        (
            &seed.id,
            &seed.lat,
            &seed.long,
            serde_json::to_string(&seed.neighbors).unwrap(),
        ),
    )?;

    Ok(())
}

// fn get_neighbors(node: NodeId) -> Result<Vec<(NodeId, WayId)>> {
pub fn get_neighbors(id: NodeId) -> Result<()> {
    let path = "./db.db3";
    let conn = Connection::open(path)?;

    let row: String = conn.query_row("SELECT neighbors FROM Node WHERE id = ?1", [id], |row| {
        row.get(0)
    })?;

    dbg!(row);

    Ok(())
}
