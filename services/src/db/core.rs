/// Governs interface w/ underlying SQLite db
use rusqlite::{Connection, Transaction};

use geo::prelude::*;
use geo::{point, Point};

use super::Element;
use crate::osm::{Cycleway, Road, Salmoning, Way, WayId};
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
#[derive(Debug)]
struct OSMMapper {
    way: WayId,
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
impl From<&Element> for OSMMapper {
    fn from(element: &Element) -> Self {
        let tags = &element.tags;
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
            .get("oneway:bicycle")
            .cloned()
            .unwrap_or("none".to_owned());

        OSMMapper {
            way: element.id,
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

impl OSMMapper {
    /// Given these OSM tags, calculate road label
    fn get_road_label(&self) -> Road {
        match self.highway.as_str() {
            "pedestrian" | "crossing" | "corridor" | "footway" | "path" => Road::Pedestrian,
            "cycleway" => Road::Bike,
            "residential" | "living_street" | "unclassified" | "service" | "track" => Road::Local,
            "secondary" | "secondary_link" | "tertiary" | "tertiary_link" | "none" => {
                Road::Collector
            }
            "primary" | "primary_link" => Road::Arterial,
            _ => {
                eprintln!("{}: Unexpected highway value: {}", self.way, self.highway);
                Road::Collector
            }
        }
    }

    /// Given these OSM tags, get the forward and reverse cycleways and directionality
    // opted to make this a mega function, since the logic for
    // determining these 3 was always coupled
    fn get_cycleways_and_directionality(&self) -> (Cycleway, Cycleway, Salmoning) {
        // easiest solution, leverage cycleway both if specified
        if let Some(cycleway) = self.get_cycleway_if_specified(&self.cycleway_both) {
            return (cycleway, cycleway, false);
        }

        // Leverage indicators for designated bike paths
        if self.highway == "cycleway" || self.bicycle == "designated" {
            return self.handle_designated_paths();
        }

        // Now handle oneway roads
        if self.oneway == "yes" {
            return self.handle_oneway_roads();
        }

        // finally, handle bidirectional roads
        self.handle_bidirectional_roads()
    }

    fn get_cycleway_from_tag(&self, val: &str) -> Cycleway {
        match val {
            "track" | "separate" => Cycleway::Track,
            "lane" => Cycleway::Lane,
            "shared_lane" | "share_busway" | "no" | "none" => Cycleway::Shared,
            _ => {
                eprintln!("{}: Unexpected cycleway value: {val}", self.way);
                Cycleway::Shared
            }
        }
    }

    fn get_cycleway_if_specified(&self, tag: &str) -> Option<Cycleway> {
        if tag != "none" {
            Some(self.get_cycleway_from_tag(tag))
        } else {
            None
        }
    }

    fn handle_designated_paths(&self) -> (Cycleway, Cycleway, Salmoning) {
        // Just need to check if this designated path is a oneway
        // A lack of these tags is an implicit _oneway=yes_
        if self.oneway == "no" || self.oneway_bicycle == "no" {
            (Cycleway::Track, Cycleway::Track, false)
        } else {
            (Cycleway::Track, Cycleway::Track, true)
        }
    }

    fn handle_oneway_roads(&self) -> (Cycleway, Cycleway, Salmoning) {
        // if right side is specified, use that
        if let Some(labels) = self.check_cycleway_side(
            &self.cycleway_right,
            &self.cycleway_right_oneway,
            &self.cycleway_left,
            &self.cycleway_left_oneway,
        ) {
            return labels;
        }

        // if left side is specified, use that
        if let Some(labels) = self.check_cycleway_side(
            &self.cycleway_left,
            &self.cycleway_left_oneway,
            &self.cycleway_right,
            &self.cycleway_right_oneway,
        ) {
            return labels;
        }

        // if we are this far down, there are no forward direction lanes
        // so begin checking for contraflow lanes
        if let Some(reverse_cycleway) =
            self.get_cycleway_if_contraflow(&self.cycleway_right_oneway, &self.cycleway_right)
        {
            return (Cycleway::Shared, reverse_cycleway, false);
        }
        if let Some(reverse_cycleway) =
            self.get_cycleway_if_contraflow(&self.cycleway_left_oneway, &self.cycleway_left)
        {
            return (Cycleway::Shared, reverse_cycleway, false);
        }

        // if there are no forward or backward bike lanes on this oneway road, default!
        (Cycleway::Shared, Cycleway::Shared, true)
    }

    /// For use with oneway roads: if there is non-contraflow bike infra on the specified side, use it
    /// Also check the opposite side for an explicit, different reverse lane
    fn check_cycleway_side(
        &self,
        cycleway_side: &str,
        cycleway_side_oneway: &str,
        opposite_side: &str,
        opposite_side_oneway: &str,
    ) -> Option<(Cycleway, Cycleway, Salmoning)> {
        if cycleway_side != "none" && cycleway_side != "no" && cycleway_side_oneway != "-1" {
            let cycleway = self.get_cycleway_from_tag(cycleway_side);

            // is this a bidirectional cycleway?
            let mut salmon = true;
            if cycleway_side_oneway == "no" || self.oneway_bicycle == "no" {
                salmon = false;
            }

            // does the opposite side have an explicit reverse lane?
            let mut reverse_cycleway = cycleway;
            if opposite_side_oneway == "-1" {
                reverse_cycleway = self.get_cycleway_from_tag(opposite_side);
                salmon = false;
            }

            return Some((cycleway, reverse_cycleway, salmon));
        }
        None
    }

    fn get_cycleway_if_contraflow(&self, oneway_tag: &str, cycleway_tag: &str) -> Option<Cycleway> {
        if oneway_tag == "-1" {
            Some(self.get_cycleway_from_tag(cycleway_tag))
        } else {
            None
        }
    }

    fn handle_bidirectional_roads(&self) -> (Cycleway, Cycleway, Salmoning) {
        // first, check if either side uses bidirectional bike infra
        if let Some(cycleway) =
            self.get_cycleway_if_bidirectional(&self.cycleway_left, &self.cycleway_left_oneway)
        {
            return (cycleway, cycleway, false);
        }
        if let Some(cycleway) =
            self.get_cycleway_if_bidirectional(&self.cycleway_right, &self.cycleway_right_oneway)
        {
            return (cycleway, cycleway, false);
        }

        // otherwise, right side is the forward cycleway, left side is reverse cycleway
        let forward_cycleway = self.get_cycleway_from_tag(&self.cycleway_right);
        let reverse_cycleway = self.get_cycleway_from_tag(&self.cycleway_left);

        (forward_cycleway, reverse_cycleway, false)
    }

    fn get_cycleway_if_bidirectional(
        &self,
        cycleway_tag: &str,
        oneway_tag: &str,
    ) -> Option<Cycleway> {
        if oneway_tag == "no" {
            Some(self.get_cycleway_from_tag(cycleway_tag))
        } else {
            None
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
    let osm_mapper: OSMMapper = (&element).into();

    let road = osm_mapper.get_road_label();
    let (forward_cycleway, reverse_cycleway, salmon) =
        osm_mapper.get_cycleways_and_directionality();

    let params = (&way.id, forward_cycleway as isize, road as isize, false);
    stmt.execute(params).unwrap_or_else(|e| {
        eprintln!("Failed WayLabel:\n{:#?}", params);
        panic!("{e}");
    });

    let params = (-&way.id, reverse_cycleway as isize, road as isize, salmon);
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
