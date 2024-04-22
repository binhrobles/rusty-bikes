# Rusty Bikes

A project for learning Rust.

Implementing [OpenRouteService's Direction API](https://giscience.github.io/openrouteservice/api-reference/endpoints/directions/) to get a feel for serving an API, threads, pathfinding algs, and OSM data, in Rust :crab:

Used w/ my [ORS map client fork](https://github.com/binhrobles/ors-map-client-rusty-fork)

### Usage

```bash
# host it at localhost:3000
cargo run --bin rusty_router

# enable request/response tracing
RUST_LOG=tower_http=trace cargo run --bin rusty_router

# init the db
cargo run --bin init_db
```

## How it's happening

### OSM Data

The underlying data is coming from [OSM's Overpass API](https://wiki.openstreetmap.org/wiki/Overpass_API). The initial data dump is the result of the OverpassQL query:

```
data=[out:json][timeout:90];
nw(40.647941,-74.028837,40.755695,-73.907988)
  ["highway"]
  [!"footway"]
  ["highway"!="footway"]
  ["highway"!="steps"]
  ["highway"!="street_lamp"]
  ["highway"!="elevator"]
  ["highway"!="bus_stop"]
  ;
out geom;
```

which gives us all relevant [Way](https://wiki.openstreetmap.org/wiki/Way)'s tagged with the [key "highway"](https://wiki.openstreetmap.org/wiki/Key:highway) in roughly Lower Manhattan + over the bridge BK, along with their geometry data (lat-longs + referenced Node lat-longs).

### Primary Alg Considerations

To support an efficient A\* implementation:

- Looking up Node neighbors must be as fast as possible
  - add adjacency matrix to Node table
- Costs must be calculated quickly
  - Way tags should be quickly available (_future: bring up to top level as columns?_)
    - ie: highway, bicycle, oneway, height, cycleway?, ...
  - _future: add Edges as another table, to make distance calculations O(1) lookups?_
- We must be able to locate the Way that is closest to our start / end points
  - Enable R\*Tree support on Ways, easily done due to their min/max coords
  - Given a way and a coordinate, where along the Way is this coordinate?

This results in a SQLite schema of:

```
Node
---
id
lat
long
neighbors: {
	nodeId: wayId, (FK Way ID)
	...
}

Way
---
id
minLat
minLong
maxLat
maxLong
nodes: [
	nodeId, (FK Node ID)
    ...
] 
tags: {
	{key}: value
}
```

### Future things to consider
- run sql operations on a [dedicated thread](https://ryhl.io/blog/async-what-is-blocking/)? does that gain anything for me?
