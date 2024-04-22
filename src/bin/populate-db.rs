use std::env;
use std::fs::File;
use std::io::BufReader;
use std::process;

use rusty_router::osm;

// ../osm-data/nyc_bk_highways_no_footways.geom.json
// ./src/bin/sample-osm.json
fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("must supply a path to an OSM JSON file");
        process::exit(1);
    }

    // Open the file in read-only mode with buffer.
    let filename = &args[1];
    let file = File::open(filename).unwrap();
    let reader = BufReader::new(file);

    let out: osm::Output = serde_json::from_reader(reader).unwrap();
    println!("out: {:?}", out);
}