use std::{collections::HashMap, process::Output};

use rusty_router::osm;
use dotenvy::dotenv;

#[tokio::main]
async fn main() {
    dotenv().expect(".env file not found");

    let j = r#"{
        "version": 0.6,
        "generator": "Overpass API 0.7.62.1 084b4234",
        "osm3s": {
            "timestamp_osm_base": "2024-04-12T01:39:29Z",
            "copyright": "The data included in this document is from www.openstreetmap.org. The data is made available under ODbL."
        },
        "elements": [
            {
                "type": "node",
                "id": 42421877,
                "lat": 40.7328569,
                "lon": -73.9959291,
                "tags": {
                    "highway": "traffic_signals"
                }
            },
            {
                "type": "way",
                "id": 1271806812,
                "bounds": {
                    "minlat": 40.7522174,
                    "minlon": -73.9510943,
                    "maxlat": 40.7524399,
                    "maxlon": -73.9509023
                },
                "nodes": [
                    42826843,
                    5034184382,
                    11524814888,
                    11524814890
                ],
                "geometry": [
                    { "lat": 40.7524399, "lon": -73.9509023 },
                    { "lat": 40.7523957, "lon": -73.9509417 },
                    { "lat": 40.7523349, "lon": -73.9509937 },
                    { "lat": 40.7522174, "lon": -73.9510943 }
                ],
                "tags": {
                    "bicycle": "yes",
                    "cycleway:left": "shared_lane",
                    "cycleway:left:lane": "pictogram",
                    "cycleway:right": "lane",
                    "cycleway:right:oneway": "no",
                    "hgv": "local",
                    "highway": "secondary",
                    "lanes": "2",
                    "maxspeed": "25 mph",
                    "name": "Vernon Boulevard",
                    "surface": "asphalt",
                    "tiger:cfcc": "A41",
                    "tiger:county": "Queens, NY",
                    "tiger:name_base": "Vernon",
                    "tiger:name_type": "Blvd",
                    "tiger:reviewed": "no"
                }
            }
        ]
    }"#;

    let out: osm::Output = serde_json::from_str(j).unwrap();
    println!("out: {:?}", out);

    // let out = osm::Element { 
    //     id: 0,
    //     r#type: "node".to_owned(),
    //     tags: HashMap::new(),
    //     lat: Some(0.0),
    //     lon: Some(0.0),
    //     bounds: None,
    //     geometry: None,
    //     nodes: None,
    // };

    // println!("out: {}", serde_json::to_string(&out).unwrap());
}
