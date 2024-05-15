/// Governs the initial ETL process from the OSM export JSON to SQLite
use serde::de::{SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;

use std::fmt;
use std::marker::PhantomData;

use crate::osm::{db, Location, Node, Way};

#[derive(std::fmt::Debug, Deserialize)]
pub struct Bounds {
    pub minlat: f64,
    pub minlon: f64,
    pub maxlat: f64,
    pub maxlon: f64,
}

#[derive(std::fmt::Debug, Deserialize)]
pub struct Element {
    pub id: i64,
    pub r#type: String,
    pub tags: HashMap<String, String>,

    // Node
    pub lat: Option<f64>,
    pub lon: Option<f64>,

    // Way
    pub bounds: Option<Bounds>,
    pub nodes: Option<Vec<i64>>,
    pub geometry: Option<Vec<Location>>,
}

impl From<&Element> for Node {
    fn from(value: &Element) -> Self {
        Node::new(value.id, value.lon.unwrap(), value.lat.unwrap())
    }
}

impl From<&Element> for Way {
    fn from(value: &Element) -> Self {
        let Some(bounds) = &value.bounds else {
            panic!("No bounds present on Way {}", value.id);
        };

        Self {
            id: value.id,
            min_lat: bounds.minlat,
            max_lat: bounds.maxlat,
            min_lon: bounds.minlon,
            max_lon: bounds.maxlon,
        }
    }
}

#[derive(std::fmt::Debug, Deserialize)]
pub struct Output {
    // Deserialize this field by adding the element to SQLite
    #[serde(deserialize_with = "deserialize_into_sqlite")]
    // Despite the struct field being named `num_rows`, we are parsing
    // from a JSON field called `elements`.
    #[serde(rename(deserialize = "elements"))]
    pub num_rows: u128,
}

/// Deserialize the OSM JSON elements into SQLite. The entire OSM file
/// is not buffered into memory as it would be if we deserialize to Vec<T>
/// and then format / insert into SQLite later.
fn deserialize_into_sqlite<'de, D>(deserializer: D) -> Result<u128, D::Error>
where
    D: Deserializer<'de>,
{
    // TODO: what is PhantomData ?
    struct ElementsVisitor<T>(PhantomData<fn() -> T>);

    impl<'de, T> Visitor<'de> for ElementsVisitor<T>
    where
        T: Deserialize<'de> + std::fmt::Debug,
    {
        /// Return type of this visitor. This visitor computes the max of a
        /// sequence of values of type T, so the type of the maximum is T.
        type Value = u128;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a nonempty sequence of Node / Way objects")
        }

        fn visit_seq<S>(self, mut seq: S) -> Result<u128, S::Error>
        where
            S: SeqAccess<'de>,
        {
            let mut count = 0;
            let mut conn = db::get_conn().unwrap();
            let tx = conn.transaction().unwrap();

            while let Some(el) = seq.next_element::<Element>()? {
                count += 1;

                match el.r#type.as_str() {
                    "node" => {
                        // insert to Node table
                        // can we assume all Nodes will appear before Ways?
                        db::insert_node_element(&tx, el).unwrap();
                    }
                    "way" => {
                        // insert to Way table
                        db::insert_way_element(&tx, el).unwrap();
                    }
                    other => panic!("unsupported type {}\nelement: {:?}", other, el),
                }
            }

            tx.commit().unwrap();

            Ok(count)
        }
    }

    // Create the visitor and ask the deserializer to drive it. The
    // deserializer will call visitor.visit_seq() if a seq is present in
    // the input data.
    let visitor = ElementsVisitor::<Element>(PhantomData);
    deserializer.deserialize_seq(visitor)
}
