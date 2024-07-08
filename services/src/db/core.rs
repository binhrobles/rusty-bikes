/// Governs interface w/ underlying SQLite db
use anyhow::anyhow;
use reqwest::blocking::Client;
use rusqlite::{Connection, Transaction};

use geo::prelude::*;
use geo::{point, Point};
use serde::Deserialize;

use super::{Element, ElevationCache, OSMMapper};
use crate::osm::{Distance, Elevation, Grade, Location, NodeId, Way};
use std::env;

#[derive(Debug, Deserialize)]
pub struct ElevationResult {
    elevation: Option<Elevation>,
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

/// Insert a OSM-parsed Way element into the DB, synchronously
pub fn insert_way_element(
    tx: &Transaction,
    client: &Client,
    elevation_cache: &mut ElevationCache,
    element: Element,
) -> anyhow::Result<()> {
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

    // get elevation data for all the nodes in this way
    // first by formatting all the latlngs into a query string
    // NOTE: with ~56k Ways, and an avg ~25ms per query,
    //       we spend ~23 mins just waiting for this service to return
    let formatted_lat_lngs: Vec<String> = node_ids
        .iter()
        .enumerate()
        .filter_map(|(pos, _)| {
            let p = get_point_for_way_node(&node_coords, pos);
            let elevation_index = get_elevation_index(p);

            // only queue up for a request if it's not present in the cache
            if !elevation_cache.contains_key(&elevation_index) {
                return Some(elevation_index);
            }

            None
        })
        .collect();

    if !formatted_lat_lngs.is_empty() {
        let elevation_query = String::from("http://localhost:5001/v1/ned10m?locations=")
            + &formatted_lat_lngs.join("|");

        // and then by executing the query against the locally running opentopo server
        let response = client
            .get(elevation_query)
            .send()?
            .json::<ElevationResponse>()?;
        // and update the cache
        for (pos, res) in response.results.iter().enumerate() {
            let elevation = res.elevation.unwrap() as Elevation;
            elevation_cache.insert(formatted_lat_lngs.get(pos).unwrap().to_owned(), elevation);
        }
    }

    let mut prev_node: Option<(NodeId, Point)> = None;

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

        // attach this and the previous node as Segments
        if let Some((prev_node_id, prev_point)) = prev_node {
            let distance = p.haversine_distance(&prev_point).ceil() as i32;
            let prev_elevation = elevation_cache
                .get(&get_elevation_index(prev_point))
                .unwrap();
            let current_elevation = elevation_cache.get(&get_elevation_index(p)).unwrap();
            let grade = get_grade(prev_elevation, current_elevation, distance);

            let segment_params = (prev_node_id, current_node_id, &way.id, distance, grade);
            segment_insert_stmt
                .execute(segment_params)
                .map_err(|e| anyhow!("Failed WayNode:\n{:#?}\n{e}", segment_params))?;

            // also insert the inverse segment, flipping the WayId sign
            // to indicate that the segment will refer to the reverse OSM direction
            let segment_params = (current_node_id, prev_node_id, -&way.id, distance, -grade);
            segment_insert_stmt
                .execute(segment_params)
                .map_err(|e| anyhow!("Failed WayNode:\n{:#?}\n{e}", segment_params))?;
        }

        prev_node = Some((*current_node_id, p));
    }

    Ok(())
}

/// returns a String index from a Point
fn get_elevation_index(p: Point) -> String {
    format!("{:.5},{:.5}", p.y(), p.x()).as_str().to_owned()
}

/// gets the grade % between two elevations
fn get_grade(from: &Elevation, to: &Elevation, distance: Distance) -> Grade {
    ((to - from) / (distance as f32) * 100.0) as Grade
}

fn get_point_for_way_node(node_coords: &[Location], pos: usize) -> Point {
    let p = point!(
        x: node_coords.get(pos).unwrap().lon,
        y: node_coords.get(pos).unwrap().lat,
    );
    p
}
