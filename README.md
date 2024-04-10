# Rusty Bikes
A project for learning Rust.

Implementing [OpenRouteService's Direction API](https://giscience.github.io/openrouteservice/api-reference/endpoints/directions/) to get a feel for serving an API, threads, pathfinding algs, and OSM data, in Rust :crab:

Used w/ my [ORS map client fork](https://github.com/binhrobles/ors-map-client-rusty-fork)

### Usage
```bash
# host it at localhost:3000
cargo run

# enable request/response tracing
RUST_LOG=tower_http=trace cargo run
```

## How it's happening
### Data Structures
The underlying data is coming from [OSM's Overpass API](https://wiki.openstreetmap.org/wiki/Overpass_API). The initial data dump is the result of the OverpassQL query:
```
data=[out:json]
[timeout:90]
;

(
    way(40.647941,-74.028837,40.755695,-73.907988)["highway"];
);

out geom;
```
which gives us all [Way](https://wiki.openstreetmap.org/wiki/Way)'s tagged with the [key "highway"](https://wiki.openstreetmap.org/wiki/Key:highway) in roughly Lower Manhattan + over the bridge BK, along with their geometry data (lat-longs + referenced Node lat-longs).

This gets loaded into SQLite under the schema:
```mermaid
```
JSONB id maps from Nodes to Ways? or Ways to Nodes? both?
or...just save it as a graph? Nodes are linked to the Nodes that are accessible to it via the Ways data
