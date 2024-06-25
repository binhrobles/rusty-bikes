/// Governs interface w/ underlying SQLite db
use anyhow::anyhow;
use rayon::prelude::*;
use reqwest::blocking::Client;
use rusqlite::{Connection, Transaction};

use geo::prelude::*;
use geo::{point, Point};
use serde::Deserialize;

use super::{Element, OSMMapper};
use crate::osm::{Distance, Elevation, Location, NodeId, Way};
use std::env;

#[derive(Debug, Deserialize)]
pub struct ElevationResult {
    elevation: Option<f32>,
}
#[derive(Debug, Deserialize)]
pub struct ElevationResponse {
    results: Vec<ElevationResult>,
}

pub type DBConnection = Connection;

/// get a SQLite Connection for queries and stuff
pub fn get_conn() -> anyhow::Result<DBConnection> {
    let db_path = env::var("DB_PATH")?;
    let conn = Connection::open(db_path)?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    Ok(conn)
}

/// initializes a sqlite database at DATABASE_URL with the requisite tables
pub fn init_tables(conn: &Connection) -> Result<(), anyhow::Error> {
    // note that no foreign key relationships are allowed to virtual tables
    conn.execute_batch(
        "
        DROP TABLE IF EXISTS Segments;
        DROP TABLE IF EXISTS WayNodes;
        DROP TABLE IF EXISTS WayLabels;
        DROP TABLE IF EXISTS Nodes;
        DROP TABLE IF EXISTS Ways;

        CREATE TABLE Nodes (
            id INTEGER PRIMARY KEY,
            lon REAL NOT NULL,
            lat REAL NOT NULL
        );

        CREATE VIRTUAL TABLE Ways USING rtree(
            id,
            minLat,
            maxLat,
            minLon,
            maxLon
        );

        CREATE TABLE WayNodes (
            way   INTEGER NOT NULL,
            node  INTEGER NOT NULL,
            pos   INTEGER NOT NULL,
            PRIMARY KEY (way, pos),
            FOREIGN KEY (node) REFERENCES Nodes(id)
        );
        CREATE INDEX way_index ON WayNodes(way);

        CREATE TABLE Segments (
            n1  INTEGER NOT NULL,
            n2  INTEGER NOT NULL,
            way INTEGER NOT NULL,
            distance INTEGER NOT NULL,
            grade INTEGER NOT NULL,
            PRIMARY KEY (n1, n2, way),
            FOREIGN KEY (n1) REFERENCES Nodes(id),
            FOREIGN KEY (n2) REFERENCES Nodes(id)
        );
        CREATE INDEX n1_index ON Segments(n1);

        CREATE TABLE WayLabels (
            id       INTEGER PRIMARY KEY,
            cycleway INTEGER NOT NULL,
            road     INTEGER NOT NULL,
            salmon   INTEGER NOT NULL
        );
    ",
    )?;
    println!("Tables created");

    Ok(())
}

/// Insert a OSM-parsed Node element into the DB, synchronously
pub fn insert_node_element(tx: &Transaction, element: Element) -> anyhow::Result<()> {
    let mut stmt = tx.prepare_cached("INSERT INTO Nodes (id, lon, lat) VALUES (?1, ?2, ?3)")?;
    stmt.execute((&element.id, &element.lon, &element.lat))
        .map_err(|e| anyhow!("Failed Node:\n{:#?}\n{e}", element))?;

    Ok(())
}

type SegmentMetadata = (
    NodeId,
    NodeId,
    Distance,
    Option<Elevation>,
    Option<Elevation>,
);

/// Insert a OSM-parsed Way element into the DB, synchronously
pub fn insert_way_element(tx: &Transaction, client: &Client, element: Element) -> anyhow::Result<()> {
    let way = Way::from(&element);

    let mut way_insert_stmt = tx.prepare_cached(
        "INSERT INTO Ways (id, minLat, maxLat, minLon, maxLon) VALUES (?1, ?2, ?3, ?4, ?5)",
    )?;
    way_insert_stmt
        .execute((
            &way.id,
            &way.min_lat,
            &way.max_lat,
            &way.min_lon,
            &way.max_lon,
        ))
        .map_err(|e| anyhow!("Failed Way:\n{:#?}\n{e}", way))?;

    let mut stmt = tx.prepare_cached(
        "INSERT INTO WayLabels (id, cycleway, road, salmon) VALUES (?1, ?2, ?3, ?4)",
    )?;

    // OSM tags -> internal labeling
    let osm_mapper: OSMMapper = (&element).into();

    let road = osm_mapper.get_road_label();
    let (forward_cycleway, reverse_cycleway, salmon) =
        osm_mapper.get_cycleways_and_directionality();

    let params = (&way.id, forward_cycleway as isize, road as isize, false);
    stmt.execute(params)
        .map_err(|e| anyhow!("Failed WayLabel:\n{:#?}\n{e}", params))?;

    let params = (-&way.id, reverse_cycleway as isize, road as isize, salmon);
    stmt.execute(params)
        .map_err(|e| anyhow!("Failed WayLabel:\n{:#?}\n{e}", params))?;

    let mut node_insert_stmt =
        tx.prepare_cached("INSERT OR IGNORE INTO Nodes (id, lon, lat) VALUES (?1, ?2, ?3)")?;
    let mut wn_insert_stmt =
        tx.prepare_cached("INSERT INTO WayNodes (way, node, pos) VALUES (?1, ?2, ?3)")?;
    let mut segment_insert_stmt = tx.prepare_cached(
        "INSERT INTO Segments (n1, n2, way, distance, grade) VALUES (?1, ?2, ?3, ?4, ?5)",
    )?;

    let node_ids = element.nodes.unwrap_or_default();
    let node_coords = element.geometry.unwrap_or_default();
    assert!(
        node_ids.len() == node_coords.len(),
        "Ways should always have nodes[] and geometry[] of equal length"
    );

    // walk the Way's Nodes
    for (pos, current_node_id) in node_ids.iter().enumerate() {
        let p = get_point_for_way_node(&node_coords, pos);

        // ensure each Node exists in Nodes
        let node_params = (current_node_id, p.x(), p.y());
        node_insert_stmt
            .execute(node_params)
            .map_err(|e| anyhow!("Failed implied Node:\n{:#?}\n{e}", node_params))?;

        // insert each Node at position in WayNodes
        let wn_params = (&way.id, current_node_id, pos);
        wn_insert_stmt
            .execute(wn_params)
            .map_err(|e| anyhow!("Failed WayNode:\n{:#?}\n{e}", wn_params))?;
    }

    // gather segment metadata, leveraging parallel requests to the elevation HTTP service
    let segments: Vec<Option<SegmentMetadata>> = node_ids
        .par_iter()
        .enumerate()
        .map(|(pos, n_id)| -> Option<SegmentMetadata> {
            if pos == 0 {
                return None;
            }

            // attach this and the previous node as Segments
            let p = get_point_for_way_node(&node_coords, pos);

            let prev_node_pos = get_point_for_way_node(&node_coords, pos - 1);
            let prev_node_id = node_ids.get(pos - 1).unwrap();

            let distance = p.haversine_distance(&prev_node_pos).ceil() as Distance;
            let elevation_response: Option<ElevationResponse> = match client
                .get(format!(
                    "http://localhost:8080/api/v1/lookup?locations={},{}|{},{}",
                    p.y(),
                    p.x(),
                    prev_node_pos.y(),
                    prev_node_pos.x()
                ))
                .send()
            {
                Ok(res) => match res.json() {
                    Ok(res) => Some(res),
                    Err(e) => {
                        eprintln!("Elevation JSON Error: {e}");
                        None
                    }
                },
                Err(e) => {
                    eprintln!("Elevation Request Error: {e}");
                    None
                }
            };

            let mut current_elevation: Option<Elevation> = None;
            let mut prev_elevation: Option<Elevation> = None;

            if let Some(elevation_response) = elevation_response {
                current_elevation = Some(
                    elevation_response
                        .results
                        .first()
                        .unwrap()
                        .elevation
                        .unwrap() as Elevation,
                );
                prev_elevation = Some(
                    elevation_response
                        .results
                        .get(1)
                        .unwrap()
                        .elevation
                        .unwrap() as Elevation,
                );
            }

            Some((
                *prev_node_id,
                *n_id,
                distance,
                prev_elevation,
                current_elevation,
            ))
        })
        .collect();

    // now go through the valid segment metadata and add the segments to the Segments table
    for (prev_node_id, current_node_id, distance, prev_elevation, current_elevation) in
        segments.iter().flatten()
    {
        let segment_params = (
            prev_node_id,
            current_node_id,
            &way.id,
            distance,
            get_grade(current_elevation, prev_elevation),
        );
        segment_insert_stmt
            .execute(segment_params)
            .map_err(|e| anyhow!("Failed WayNode:\n{:#?}\n{e}", segment_params))?;

        // also insert the inverse segment, flipping the WayId sign
        // to indicate that the segment will refer to the reverse OSM direction
        let segment_params = (
            current_node_id,
            prev_node_id,
            -&way.id,
            distance,
            get_grade(prev_elevation, current_elevation),
        );
        segment_insert_stmt
            .execute(segment_params)
            .map_err(|e| anyhow!("Failed WayNode:\n{:#?}\n{e}", segment_params))?;
    }

    Ok(())
}

/// gets the grade between two elevations, defaulting to 0 in the case of any Nones
fn get_grade(current_elevation: &Option<i32>, prev_elevation: &Option<i32>) -> i32 {
    if current_elevation.is_some() && prev_elevation.is_some() {
        current_elevation.unwrap() - prev_elevation.unwrap()
    } else {
        0
    }
}

fn get_point_for_way_node(node_coords: &[Location], pos: usize) -> Point {
    let p = point!(
        x: node_coords.get(pos).unwrap().lon,
        y: node_coords.get(pos).unwrap().lat,
    );
    p
}
