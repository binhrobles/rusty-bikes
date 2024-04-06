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
