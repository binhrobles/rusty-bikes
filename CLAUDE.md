# Rusty Bikes

NYC bike routing app: OSM data → Rust A* pathfinding → Leaflet map UI. Users click two points and get a bike-optimized route with configurable cost preferences.

## Issue Tracking

This project uses **bd (beads)** for issue tracking.
Run `bd prime` for workflow context, hooks are also installed for auto-injection.

**Quick reference:**
- `bd ready` - Find unblocked work
- `bd create "Title" --type task --priority 2` - Create issue
- `bd close <id>` - Complete work
- `bd sync` - Sync with git (run at session end)

For full workflow details: `bd prime`

## Tech Stack

| Layer | Tech |
|-------|------|
| Backend | Rust 2021, AWS Lambda (arm64), SQLite + R*Tree |
| Frontend | Svelte 4, TypeScript, Vite, Leaflet, Nanostores |
| Infra | AWS SAM, API Gateway, Lambda Layers, GitHub Pages |
| Data | OpenStreetMap Overpass API |

## Quick Start

```bash
make service-watch     # Rust backend at localhost:9000 (hot reload via cargo-lambda)
make client-watch      # Svelte client (Vite dev server)
make service-test      # Run Rust tests (needs db.db3)
make service-deploy    # Build + deploy Lambda to AWS
make osm-download      # Fetch OSM data → out.geom.json
make db-build          # Build SQLite DB from GeoJSON
make layer-build       # Package DB as Lambda layer zip
make layer-upload      # Upload layer to S3
```

## Project Layout

```
rusty-bikes/
├── services/                # Rust backend
│   ├── src/
│   │   ├── bin/             # lambda-handler, init-db, populate-db, basic-benchmarking
│   │   ├── graph/           # core.rs, traversal.rs, repository.rs, cost.rs
│   │   ├── db/              # core.rs, etl.rs, mapping.rs
│   │   ├── api/             # compression.rs, geojson.rs
│   │   ├── osm.rs           # OSM data types (Node, Way, Cycleway, Road, etc.)
│   │   └── lib.rs
│   ├── tests/               # Integration tests (way-labeling.rs, snapping.rs)
│   └── scripts/             # download_osm_data.sh
├── client/                  # Svelte frontend
│   └── src/
│       ├── components/      # App, Control, Icon, LoadingIndicator, DebugPopup
│       ├── store/           # Nanostores: map, route, cost, fetch, render, marker
│       ├── modules/         # map.mts, control.mts (Leaflet integration)
│       ├── config.ts        # API URL (prod vs local)
│       └── consts.ts        # Enums, defaults
├── template.yaml            # SAM CloudFormation template
├── Makefile                 # All build/deploy targets
└── db.db3                   # SQLite database (gitignored, ~144MB)
```

## Key Conventions

- **Directional Way IDs**: positive = OSM-normal direction, negative = reverse. See `.docs/services.md`
- **Cost function**: `(road_coeff * road_weight + cycleway_coeff * cycleway_weight) * salmon_multiplier * way_length`. All weights are client-configurable. See `services/src/graph/cost.rs`
- **State management**: Nanostores atoms → computed stores → batched tasks. See `.docs/client.md`
- **OSM tag mapping**: Lossy ETL from OSM tags → {Road, Cycleway, Salmon} labels at DB build time, not runtime. See `services/src/db/mapping.rs`
- **Testing**: Integration tests use real NYC street data for edge cases. See `services/tests/`
- **No frontend tests**: Client uses ESLint + Prettier only

## Testing Endpoints Locally

Start the backend with `make service-watch`, then use curl to test. Save response to a file and parse with `jq`:

```bash
# /navigate — mobile-optimized lean response
curl -s -X POST http://localhost:9000/lambda-url/lambda-handler/navigate \
    -H "Content-Type: application/json" \
    -d @services/tests/navigate-request.json \
    -o /tmp/navigate-response.json

# /route — desktop response with optional traversal
curl -s -X POST http://localhost:9000/lambda-url/lambda-handler/route \
    -H "Content-Type: application/json" \
    -d @services/tests/route-request.json \
    -o /tmp/route-response.json

# Inline body for shorter routes (useful for quick tests)
curl -s -X POST http://localhost:9000/lambda-url/lambda-handler/navigate \
    -H "Content-Type: application/json" \
    -d '{"start":{"lat":40.6955785,"lon":-73.963711},"end":{"lat":40.736004,"lon":-73.990386}}'
```

Useful jq queries for `/navigate` responses:
```bash
jq '.meta' /tmp/navigate-response.json                                    # total_distance, total_time_estimate
jq '.route.features[:3][] | {way_name: .properties.way_name, distance: .properties.distance}' /tmp/navigate-response.json  # first 3 steps
jq '[.route.features[].properties.way_name | select(. == "")] | length' /tmp/navigate-response.json  # count unnamed steps
```

## Detailed References

- [Architecture & data flow](.docs/architecture.md)
- [Rust backend conventions](.docs/services.md)
- [Frontend conventions](.docs/client.md)
- [Build, deploy & data pipeline](.docs/deployment.md)
- [Detailed backend docs](services/README.md) — OSM tags, cost function design, DB schema
