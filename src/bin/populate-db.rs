use std::fs::File;
use std::io::BufReader;

use rusty_router::osm;
use dotenvy::dotenv;

#[tokio::main]
async fn main() {
    dotenv().expect(".env file not found");

    // Open the file in read-only mode with buffer.
    // let file = File::open("../osm-data/nyc_bk_highways_no_footways.geom.json").unwrap();
    let file = File::open("./src/bin/sample-osm.json").unwrap();
    let reader = BufReader::new(file);

    let out: osm::Output = serde_json::from_reader(reader).unwrap();
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
