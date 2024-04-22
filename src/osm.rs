use serde::de::{SeqAccess, Visitor};
use serde::{Deserializer, Deserialize};
use std::collections::HashMap;

use std::fmt;
use std::marker::PhantomData;

use crate::db;

#[derive(std::fmt::Debug, Deserialize)]
pub struct Bounds {
    pub minlat: f32,
    pub minlon: f32,
    pub maxlat: f32,
    pub maxlon: f32,
}

#[derive(std::fmt::Debug, Deserialize)]
pub struct Geometry {
    pub lat: f32,
    pub lon: f32,
}

#[derive(std::fmt::Debug, Deserialize)]
pub struct Element {
    #[serde(deserialize_with = "deserialize_string_from_int")]
    pub id: String,
    pub r#type: String,
    pub tags: HashMap<String, String>,

    // Node
    pub lat: Option<f32>,
    pub lon: Option<f32>,

    // Way
    pub bounds: Option<Bounds>,
    #[serde(default, deserialize_with = "deserialize_strings_from_int_array")]
    pub nodes: Option<Vec<String>>,
    pub geometry: Option<Vec<Geometry>>,
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

/// Casts an int to a String during deserialization
fn deserialize_string_from_int<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(u128::deserialize(deserializer)?.to_string())
}

/// Casts an array of ints to an array of Strings during deserialization
fn deserialize_strings_from_int_array<'de, D>(deserializer: D) -> Result<Option<Vec<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    let t: Option<Vec<u128>> = Option::deserialize(deserializer)?;
    if let Some(t) = t {
        let strings: Vec<String> = t.iter().map(|u| u.to_string()).collect();
        return Ok(Some(strings));
    }

    Ok(None)
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
            while let Some(el) = seq.next_element::<Element>()? {
                count += 1;

                match el.r#type.as_str() {
                    "node" => {
                        // insert to Node table
                        db::insert_node(el).unwrap();

                        // can we assume all Nodes will appear before Ways?
                    }
                    "way" => {
                        // insert to Way table
                        db::insert_way(el).unwrap();

                        // also walk Nodes and update their adjacency matrices
                        // insert if Node is not present?
                    }
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