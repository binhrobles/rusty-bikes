/// Governs interface w/ underlying SQLite db
use rusqlite::{Connection, Transaction};

use geo::prelude::*;
use geo::{point, Point};

use super::{Element, Tags};
use crate::osm::{Cycleway, Road, Salmoning, Way};
use std::env;

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
            distance REAL NOT NULL,
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
        .unwrap_or_else(|e| {
            eprintln!("Failed Node:\n{:#?}", element);
            panic!("{e}");
        });

    Ok(())
}

/// Provides helper methods for interpreting and labeling the relevant OSM tags for bike routing
struct BikeTags {
    highway: String,
    bicycle: String,
    oneway: String,
    cycleway_right: String,
    cycleway_left: String,
    cycleway_both: String,
    cycleway_right_oneway: String,
    cycleway_left_oneway: String,
    oneway_bicycle: String,
}

/// Parses out the tags we want from this JSON map
impl From<Tags> for BikeTags {
    fn from(tags: Tags) -> Self {
        let highway = tags.get("highway").cloned().unwrap_or("none".to_owned());
        let bicycle = tags.get("bicycle").cloned().unwrap_or("none".to_owned());
        let oneway = tags.get("oneway").cloned().unwrap_or("none".to_owned());
        let cycleway_right = tags
            .get("cycleway:right")
            .cloned()
            .unwrap_or("none".to_owned());
        let cycleway_left = tags
            .get("cycleway:left")
            .cloned()
            .unwrap_or("none".to_owned());
        let cycleway_both = tags
            .get("cycleway:both")
            .cloned()
            .unwrap_or("none".to_owned());
        let cycleway_right_oneway = tags
            .get("cycleway:right:oneway")
            .cloned()
            .unwrap_or("none".to_owned());
        let cycleway_left_oneway = tags
            .get("cycleway:left:oneway")
            .cloned()
            .unwrap_or("none".to_owned());
        let oneway_bicycle = tags
            .get("oneway_bicycle")
            .cloned()
            .unwrap_or("none".to_owned());

        BikeTags {
            highway,
            bicycle,
            oneway,
            cycleway_right,
            cycleway_left,
            cycleway_both,
            cycleway_right_oneway,
            cycleway_left_oneway,
            oneway_bicycle,
        }
    }
}

impl BikeTags {
    /// Given these OSM tags, calculate road label
    fn get_road_label(&self) -> Road {
        match self.highway.as_str() {
            "pedestrian" | "crossing" | "corridor" | "footway" | "path" => Road::Pedestrian,
            "cycleway" => Road::Bike,
            "residential" | "living_street" | "unclassified" | "track" => Road::Local,
            "secondary" | "secondary_link" | "tertiary" | "tertiary_link" => Road::Collector,
            "primary" | "primary_link" => Road::Arterial,
            _ => Road::Collector,
        }
    }

    #[inline]
    /// helper method for determining if the tag indicates a Bike Track
    fn indicates_track(&self, val: &str) -> bool {
        val == "track" || val == "separate"
    }

    #[inline]
    /// helper method for determining if the tags show we can use the left side cycleway
    fn can_use_left_side(&self) -> bool {
        // if road is a oneway, or the left side is a bidirectional bike lane
        self.oneway == "yes" || self.oneway_bicycle == "yes" || self.cycleway_left_oneway == "no"
    }

    /// calculate cycleway label for "normal" direction
    fn get_normal_cycleway(&self) -> Cycleway {
        // first, just leverage big general indicators
        if self.bicycle == "designated" || self.highway == "cycleway"
            || self.indicates_track(&self.cycleway_both)
            // then we can use the cycleway on the left side
            || self.can_use_left_side() && self.indicates_track(&self.cycleway_left)
            // otherwise, just use the right side cycleway
            || self.indicates_track(&self.cycleway_right)
        {
            Cycleway::Track
        } else if self.cycleway_both == "lane"
            || (self.can_use_left_side() && self.cycleway_left == "lane")
            || self.cycleway_right == "lane"
        {
            Cycleway::Lane
        } else {
            Cycleway::Shared
        }
    }

    /// Calculates the cycleway label for the reverse direction
    fn get_reverse_cycleway(&self) -> Cycleway {
        // TODO!
        Cycleway::Shared
    }

    /// determines if we're salmoning when going the reverse direction
    fn get_reverse_salmon(&self) -> Salmoning {
        // if it's a one way street
        if self.oneway == "yes" {
            // then we are salmoning, and we need an explicit indicator that bikes are _not_ one way
            !(self.cycleway_left_oneway == "no" || self.cycleway_right_oneway == "no" || self.oneway_bicycle == "no")
        } else {
            // otherwise, it's a two way road, and we are probably not salmoning
            false
        }
    }
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

    let mut stmt = tx.prepare_cached(
        "INSERT INTO WayLabels (id, cycleway, road, salmon) VALUES (?1, ?2, ?3, ?4)",
    )?;

    // OSM tags -> internal labeling
    let tags: BikeTags = element.tags.into();

    let road = tags.get_road_label() as isize;
    let cycleway = tags.get_normal_cycleway() as isize;

    let params = (&way.id, cycleway, road, false);
    stmt.execute(params).unwrap_or_else(|e| {
        eprintln!("Failed WayLabel:\n{:#?}", params);
        panic!("{e}");
    });

    // calculate cycleway and salmon label for reverse direction
    let cycleway = tags.get_reverse_cycleway() as isize;
    let salmon = tags.get_reverse_salmon() as isize;

    let params = (-&way.id, cycleway, road, salmon);
    stmt.execute(params).unwrap_or_else(|e| {
        eprintln!("Failed WayLabel:\n{:#?}", params);
        panic!("{e}");
    });

    let mut node_insert_stmt =
        tx.prepare_cached("INSERT OR IGNORE INTO Nodes (id, lon, lat) VALUES (?1, ?2, ?3)")?;
    let mut wn_insert_stmt =
        tx.prepare_cached("INSERT INTO WayNodes (way, node, pos) VALUES (?1, ?2, ?3)")?;
    let mut segment_insert_stmt =
        tx.prepare_cached("INSERT INTO Segments (n1, n2, way, distance) VALUES (?1, ?2, ?3, ?4)")?;

    let node_ids = element.nodes.unwrap_or_default();
    let node_coords = element.geometry.unwrap_or_default();
    assert!(
        node_ids.len() == node_coords.len(),
        "Ways should always have nodes[] and geometry[] of equal length"
    );

    let mut prev_node: Option<(i64, Point)> = None;

    // walk the Way's Nodes
    for (pos, n_id) in node_ids.iter().enumerate() {
        let p = point!(
            x: node_coords.get(pos).unwrap().lon,
            y: node_coords.get(pos).unwrap().lat,
        );

        // ensure each Node exists in Nodes
        let node_params = (n_id, p.x(), p.y());
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
        if let Some(prev_node) = prev_node {
            let distance = p.haversine_distance(&prev_node.1);
            // TODO: also pre-calculate `default` cost for this segment

            let segment_params = (prev_node.0, n_id, &way.id, distance);
            segment_insert_stmt
                .execute(segment_params)
                .unwrap_or_else(|e| {
                    eprintln!("Failed WayNode: {:#?}", segment_params);
                    panic!("{e}");
                });

            // also insert the inverse segment, flipping the WayId sign
            // to indicate that the segment will refer to the reverse OSM direction
            let segment_params = (n_id, prev_node.0, -&way.id, distance);
            segment_insert_stmt
                .execute(segment_params)
                .unwrap_or_else(|e| {
                    eprintln!("Failed WayNode: {:#?}", segment_params);
                    panic!("{e}");
                });
        }

        prev_node = Some((*n_id, p));
    }

    Ok(())
}
