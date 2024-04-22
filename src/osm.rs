use std::collections::HashMap;
use serde::{Serialize, Deserialize, Deserializer};
use serde::de::{Visitor, SeqAccess};

use std::fmt;
use std::marker::PhantomData;

use crate::db;

#[derive(std::fmt::Debug, Serialize, Deserialize)]
pub struct Element {
    pub r#type: String,
    pub id: u128,
    pub tags: HashMap<String, String>,

    // Node
    pub lat: Option<f32>,
    pub lon: Option<f32>,

    // Way
    pub bounds: Option<HashMap<String, f32>>,
    pub nodes: Option<Vec<u128>>,
    pub geometry: Option<Vec<HashMap<String, f32>>>,
}

#[derive(std::fmt::Debug, Serialize, Deserialize)]
pub struct Node {
    pub r#type: String,
    pub id: u128,
    pub tags: HashMap<String, String>,
}

#[derive(std::fmt::Debug, Serialize, Deserialize)]
pub struct Way {
    pub r#type: String,
    pub id: u128,
    pub tags: HashMap<String, String>,
    pub bounds: HashMap<String, f32>,
    pub nodes: Vec<u128>,
    pub geometry: Vec<HashMap<String, f32>>,
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
    println!("entering de fn");
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
            while let Some(el) = seq.next_element::<Element>()? {
                count += 1;

                match el.r#type.as_str() {
                    "node" => {
                        // insert to Node table
                        // can we assume all Nodes will appear before Ways?
                        db::insert_node(el).unwrap();
                    },
                    "way" => {
                        // insert to Way table
                        // also walk Nodes and update their adjacency matrices
                        // warn if Node is not present?
                    },
                    other => panic!("unsupported type {}\nelement: {:?}", other, el),
                }
            }

            Ok(count)
        }
    }

    // Create the visitor and ask the deserializer to drive it. The
    // deserializer will call visitor.visit_seq() if a seq is present in
    // the input data.
    // TODO: research this syntax (something to do w/ not inferring comparison operators)
    let visitor = ElementsVisitor::<Element>(PhantomData);
    deserializer.deserialize_seq(visitor)
}
