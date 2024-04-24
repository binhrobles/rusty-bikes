use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::osm;

type NodeId = i64;
type WayId = i64;
type Neighbor = (NodeId, WayId);
// TODO: enum?
type TagKey = String;
type TagValue = String;

// just using serde_json to/from_str before/after sqlite calls
// which is essentially what's happening in the JSON From/ToSql implementations
// here: https://docs.rs/rusqlite/0.31.0/src/rusqlite/types/serde_json.rs.html#17-29
type Tags = HashMap<TagKey, TagValue>;

#[derive(Debug, Serialize, Deserialize)]
struct Node {
    id: NodeId,
    lat: f32,
    lon: f32,
    tags: Tags,
}

#[derive(Debug, Serialize, Deserialize)]
struct Way {
    id: WayId,
    min_lat: f32,
    min_lon: f32,
    max_lat: f32,
    max_lon: f32,
    tags: Tags,
}

const DB_PATH: &str = "./db.db3";

/// initializes a sqlite database at DATABASE_URL with the requisite tables
pub fn create_tables() -> Result<(), anyhow::Error> {
    let conn = Connection::open(DB_PATH)?;

    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.execute_batch(
        "
        DROP TABLE IF EXISTS Nodes;
        CREATE TABLE Nodes (
            id INTEGER PRIMARY KEY,
            lat REAL NOT NULL,
            lon REAL NOT NULL,
            tags TEXT
        );

        DROP TABLE IF EXISTS Ways;
        CREATE VIRTUAL TABLE Ways USING rtree(
            id,
            minLat,
            maxLat,
            minLon,
            maxLon,
            +tags TEXT
        );

        DROP TABLE IF EXISTS WayNodes;
        CREATE TABLE WayNodes (
            way   integer NOT NULL,
            node  integer NOT NULL,
            pos   integer NOT NULL,
            PRIMARY KEY (way, pos),
            FOREIGN KEY (way) REFERENCES Ways(id)
            FOREIGN KEY (node) REFERENCES Nodes(id)
        );
        CREATE INDEX way_index ON WayNodes(way);

        DROP TABLE IF EXISTS Segments;
        CREATE TABLE Segments (
            n1  integer NOT NULL,
            n2  integer NOT NULL,
            way integer NOT NULL,
            PRIMARY KEY (n1, n2, way)
            FOREIGN KEY (way) REFERENCES Ways(id)
            FOREIGN KEY (n1) REFERENCES Nodes(id)
            FOREIGN KEY (n2) REFERENCES Nodes(id)
        );

        CREATE INDEX n1_index ON Segments(n1);
        CREATE INDEX n2_index ON Segments(n2);
    ",
    )?;
    println!("Tables created");

    let seed = Node {
        id: 0,
        lat: 40.5,
        lon: 70.5,
        tags: HashMap::from([("highway".to_owned(), "traffic_signals".to_owned())]),
    };

    conn.execute(
        "INSERT INTO Nodes (id, lat, lon, tags) VALUES (?1, ?2, ?3, ?4)",
        (
            &seed.id,
            &seed.lat,
            &seed.lon,
            serde_json::to_string(&seed.tags).unwrap(),
        ),
    )?;

    let seed = Way {
        id: 0,
        min_lat: 40.5,
        min_lon: 70.5,
        max_lat: 48.5,
        max_lon: 78.5,
        tags: HashMap::from([("highway".to_owned(), "traffic_signals".to_owned())]),
    };

    conn.execute(
        "INSERT INTO Ways (id, minLat, maxLat, minLon, maxLon, tags) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        (
            &seed.id,
            &seed.min_lat,
            &seed.max_lat,
            &seed.min_lon,
            &seed.max_lon,
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
        "INSERT INTO Nodes (id, lat, lon, tags) VALUES (?1, ?2, ?3, ?4)",
        (
            &node.id,
            &node.lat,
            &node.lon,
            serde_json::to_string(&node.tags).unwrap(),
        ),
    )
    .unwrap_or_else(|e| {
        eprintln!("Failed Node:\n{:#?}", node);
        panic!("{e}");
    });

    Ok(())
}

/// Insert a Way to the DB, synchronously
pub fn insert_way(way: osm::Element) -> anyhow::Result<()> {
    let conn = Connection::open(DB_PATH)?;

    conn.pragma_update(None, "foreign_keys", "ON")?;

    let Some(bounds) = &way.bounds else {
        eprintln!("No bounds present on Way {}", way.id);
        return Ok(());
    };

    conn.execute(
        "INSERT INTO Ways (id, minLat, maxLat, minLon, maxLon, tags) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        (
            &way.id,
            bounds.minlat,
            bounds.maxlat,
            bounds.minlon,
            bounds.maxlon,
            serde_json::to_string(&way.tags).unwrap(),
        ),
    ).unwrap_or_else(|e| {
        eprintln!("Failed Way: {:#?}", way); 
        panic!("{e}");
    });

    // TODO: ensure each Node exists in the Node table

    let mut stmt = conn.prepare("INSERT INTO WayNodes (way, node, pos) VALUES (?1, ?2, ?3)")?;
    let nodes = way.nodes.unwrap_or_default();

    // walk the Way and insert each Node at position into the WayNodes table
    for (pos, n) in nodes.iter().enumerate() {
        let params = (&way.id, n, pos);
        stmt.execute(params).unwrap_or_else(|e| {
            eprintln!("Failed Segment: {:#?}", params);
            panic!("{e}");
        });
    }

    Ok(())
}

/// given a NodeId, gets the neighbors from the Segments table
/// returns a Vec of NodeId-WayId pairs, or the Node neighbor + the Way that connects them
pub fn get_neighbors(id: NodeId) -> Result<Vec<Neighbor>, anyhow::Error> {
    let conn = Connection::open(DB_PATH)?;

    let mut stmt = conn.prepare("SELECT n1, n2, way FROM Segments WHERE n1 = ?1 OR n2 = ?1")?;
    let result = stmt.query_map([id], |row| {
        // since we only store each relation once, we need to query both n1 and n2
        // here, return the "other" nodeId
        // TODO: what's the storage<>speed tradeoff for duplicating every Segment?
        let n1: NodeId = row.get(0)?;
        let node = if n1 != id { n1 } else { row.get(1)? };
        let way: WayId = row.get(2)?;

        Ok((node, way))
    })?;

    Ok(result.map(|r| r.unwrap()).collect())
}
