# Rust Backend Conventions

## Module Structure

```
services/src/
├── bin/
│   ├── lambda-handler.rs    # Lambda entry point, routes /traverse, /route, /ping
│   ├── init-db.rs           # Creates SQLite schema
│   ├── populate-db.rs       # ETL: GeoJSON → SQLite
│   └── basic-benchmarking.rs
├── graph/
│   ├── core.rs              # Graph struct, Node/Way/Segment types
│   ├── traversal.rs         # A* implementation, TraversalSegment, Traversable trait
│   ├── repository.rs        # DB query interface for neighbor lookups
│   └── cost.rs              # CostModel, weight calculations
├── db/
│   ├── core.rs              # SQLite connection, query execution
│   ├── etl.rs               # OSM → internal model transform
│   └── mapping.rs           # OSM tag → Road/Cycleway/Salmon mapping rules
├── api/
│   ├── compression.rs       # gzip/deflate/brotli response compression
│   └── geojson.rs           # Internal types → GeoJSON conversion
├── osm.rs                   # Core data types: Node, Way, Cycleway, Road, WayLabels, etc.
└── lib.rs                   # Module re-exports
```

## Key Types

| Type | Location | Purpose |
|------|----------|---------|
| `Graph` | `graph/core.rs` | Holds DB connection, provides traversal interface |
| `CostModel` | `graph/cost.rs` | Configurable weights for route cost calculation |
| `TraversalSegment` | `graph/traversal.rs` | Single edge in traversal result (from, to, way, cost, depth) |
| `Cycleway` | `osm.rs` | Enum: Track, Lane, Shared, No |
| `Road` | `osm.rs` | Enum: Bike, Pedestrian, Local, Collector, Arterial |
| `Node` | `osm.rs` | id, lat, lon |
| `Way` | `osm.rs` | id + bounding box |
| `WayLabels` | `osm.rs` | Tuple: (Cycleway, Road, bool/salmon) |

## Directional Way IDs

Each OSM Way gets two entries in the DB: one with its positive ID (OSM-normal direction) and one with the negative ID (reverse). The `Road` type is the same for both, but `Cycleway` and `Salmon` may differ based on directional bike infrastructure.

This is fundamental to how the cost function evaluates routes. See `services/README.md` for detailed examples.

## Cost Function

In `graph/cost.rs`:

```
cost_factor = (cycleway_coefficient * cycleway_weight) + (road_coefficient * road_weight)
cost_factor *= salmon_coefficient (if salmoning, else 1.0)
true_cost = cost_factor * way_length
```

Default coefficients: cycleway=0.3, road=0.4, salmon=1.3. All are client-configurable per request.

## A* Implementation

In `graph/traversal.rs`:
- Uses `BinaryHeap` (min-heap via `Reverse` or custom `Ord`) for the priority queue
- `Traversable` trait with `traverse_from` (BFS-like exploration) and `traverse_between` (A* routing)
- Heuristic: Haversine distance to target, weighted by configurable `heuristic_weight` (default 0.75)
- `START_NODE_ID = -1`, `END_NODE_ID = -2` as sentinel values

## OSM Tag Mapping

`db/mapping.rs` is the rules engine that converts OSM tags into the internal {Road, Cycleway, Salmon} model. This runs at ETL time (during `populate-db`), not at query time.

Key rules:
- `highway` tag → Road type (pedestrian, cycleway, residential, secondary, primary, etc.)
- `cycleway:left/right/both` tags → Cycleway type per direction
- `oneway` + `oneway:bicycle` + `cycleway:*:oneway` → Salmon determination
- Collector is the default Road type when no specific match

See `services/README.md` for the full tag mapping tables.

## Testing

Integration tests in `services/tests/` use real NYC street data:
- `way-labeling.rs` — Validates OSM tag → label mapping for known NYC streets
- `snapping.rs` — Tests geospatial node snapping

Run with: `make service-test` (requires `db.db3` at project root)

## Error Handling

Uses `anyhow` for error propagation. Lambda handler returns appropriate HTTP status codes with error messages in the response body.
