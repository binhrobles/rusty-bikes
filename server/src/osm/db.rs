/// Governs interface w/ underlying SQLite db
use rusqlite::{Connection, Transaction};

use crate::osm::{etl::Element, Way};

const DB_PATH: &str = "./db.db3";

/// get a SQLite Connection for queries and stuff
pub fn get_conn() -> anyhow::Result<Connection> {
    let conn = Connection::open(DB_PATH)?;
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
        DROP TABLE IF EXISTS WayTags;
        DROP TABLE IF EXISTS Nodes;
        DROP TABLE IF EXISTS Ways;

        CREATE TABLE Nodes (
            id INTEGER PRIMARY KEY,
            lat REAL NOT NULL,
            lon REAL NOT NULL
        );

        CREATE VIRTUAL TABLE Ways USING rtree(
            id,
            minLat,
            maxLat,
            minLon,
            maxLon
        );

        CREATE TABLE WayNodes (
            way   integer NOT NULL,
            node  integer NOT NULL,
            pos   integer NOT NULL,
            PRIMARY KEY (way, pos),
            FOREIGN KEY (node) REFERENCES Nodes(id)
        );
        CREATE INDEX way_index ON WayNodes(way);

        CREATE TABLE Segments (
            n1  integer NOT NULL,
            n2  integer NOT NULL,
            way integer NOT NULL,
            PRIMARY KEY (n1, n2, way),
            FOREIGN KEY (n1) REFERENCES Nodes(id),
            FOREIGN KEY (n2) REFERENCES Nodes(id)
        );
        CREATE INDEX n1_index ON Segments(n1);

        CREATE TABLE WayTags (
            id  integer NOT NULL,
            key TEXT NOT NULL,
            value TEXT NOT NULL,
            PRIMARY KEY (id, key)
        );
        CREATE INDEX way_tag_index ON WayTags(id);
    ",
    )?;
    println!("Tables created");

    Ok(())
}

/// Insert a OSM-parsed Node element into the DB, synchronously
pub fn insert_node_element(tx: &Transaction, element: Element) -> anyhow::Result<()> {
    let mut stmt = tx.prepare_cached("INSERT INTO Nodes (id, lat, lon) VALUES (?1, ?2, ?3)")?;
    stmt.execute((&element.id, &element.lat, &element.lon))
        .unwrap_or_else(|e| {
            eprintln!("Failed Node:\n{:#?}", element);
            panic!("{e}");
        });

    Ok(())
}

/// Insert a OSM-parsed Way element into the DB, synchronously
pub fn insert_way_element(tx: &Transaction, element: Element) -> anyhow::Result<()> {
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
        .unwrap_or_else(|e| {
            eprintln!("Failed Way: {:#?}", way);
            panic!("{e}");
        });

    let mut stmt = tx.prepare_cached("INSERT INTO WayTags (id, key, value) VALUES (?1, ?2, ?3)")?;
    for (key, value) in &element.tags {
        let params = (&way.id, &key, &value);
        stmt.execute(params).unwrap_or_else(|e| {
            eprintln!("Failed WayTag:\n{:#?}", params);
            panic!("{e}");
        });
    }

    let mut node_insert_stmt =
        tx.prepare_cached("INSERT OR IGNORE INTO Nodes (id, lat, lon) VALUES (?1, ?2, ?3)")?;
    let mut wn_insert_stmt =
        tx.prepare_cached("INSERT INTO WayNodes (way, node, pos) VALUES (?1, ?2, ?3)")?;
    let mut segment_insert_stmt =
        tx.prepare_cached("INSERT INTO Segments (n1, n2, way) VALUES (?1, ?2, ?3)")?;

    let node_ids = element.nodes.unwrap_or_default();
    let node_coords = element.geometry.unwrap_or_default();
    assert!(
        node_ids.len() == node_coords.len(),
        "Ways should always have nodes[] and geometry[] of equal length"
    );

    let mut prev_n_id: Option<i64> = None;

    // walk the Way's Nodes
    for (pos, n_id) in node_ids.iter().enumerate() {
        // ensure each Node exists in Nodes
        let node_params = (
            n_id,
            node_coords.get(pos).unwrap().lat,
            node_coords.get(pos).unwrap().lon,
        );
        node_insert_stmt.execute(node_params).unwrap_or_else(|e| {
            eprintln!("Failed implied Node: {:#?}", node_params);
            panic!("{e}");
        });

        // insert each Node at position in WayNodes
        let wn_params = (&way.id, n_id, pos);
        wn_insert_stmt.execute(wn_params).unwrap_or_else(|e| {
            eprintln!("Failed WayNode: {:#?}", wn_params);
            panic!("{e}");
        });

        // attach this and the previous node as Segments
        // TODO: supplement this with a distance calculation?
        //       can we just do bird's eye here?
        //       curves seem to be heavily node-d:
        //       https://www.openstreetmap.org/way/495991868
        if let Some(prev_n_id) = prev_n_id {
            let segment_params = (prev_n_id, n_id, &way.id);
            segment_insert_stmt
                .execute(segment_params)
                .unwrap_or_else(|e| {
                    eprintln!("Failed WayNode: {:#?}", segment_params);
                    panic!("{e}");
                });

            // also insert the inverse segment
            // so that we only have to query on n1
            // also because n1/n2 have no signifance wrt
            // cardinal directions or anything
            let segment_params = (n_id, prev_n_id, &way.id);
            segment_insert_stmt
                .execute(segment_params)
                .unwrap_or_else(|e| {
                    eprintln!("Failed WayNode: {:#?}", segment_params);
                    panic!("{e}");
                });
        }

        prev_n_id = Some(*n_id);
    }

    Ok(())
}
