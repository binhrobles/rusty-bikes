# Architecture & Data Flow

## System Overview

```
OSM Overpass API
    ↓ (download_osm_data.sh)
out.geom.json (~50MB GeoJSON)
    ↓ (init-db + populate-db)
db.db3 (SQLite with R*Tree)
    ↓ (packaged as Lambda Layer)
AWS Lambda (Rust, arm64)
    ↓
API Gateway (CORS enabled)
    ↓
Leaflet client (GitHub Pages)
```

## API Endpoints

All endpoints go through API Gateway → single Lambda function (`lambda-handler`).

### `POST /route`
Find optimal bike route between two points.

Request:
```json
{
  "start": { "lat": 40.683, "lon": -73.961 },
  "end": { "lat": 40.690, "lon": -73.970 },
  "with_traversal": true,
  "heuristic_weight": 0.75,
  "cost_model": {
    "cycleway_coefficient": 0.3,
    "road_coefficient": 0.4,
    "salmon_coefficient": 1.3,
    "cycleway_weights": { "Track": 0.5, "Lane": 1.0, "Shared": 1.5, "No": 1.7 },
    "road_weights": { "Bike": 0.5, "Pedestrian": 1.2, "Local": 1.2, "Collector": 1.4, "Arterial": 2.0 }
  }
}
```

Response: GeoJSON FeatureCollection with route segments + traversal segments + metadata (max_depth, cost_range).

### `POST /traverse`
Explore reachable nodes from a point using A* with a depth limit.

### `GET /ping`
Health check / cold start warmer.

## Data Flow: User Click → Route Display

1. User clicks map → marker stores update (`store/marker.ts`)
2. When both markers set → `store/fetch.ts` batched task fires
3. POST to `/route` with cost model from `store/cost.ts`
4. Response GeoJSON → `store/route.ts` → `store/render.ts`
5. Render store draws polylines on Leaflet map with cost-gradient coloring

## Database Schema

SQLite with R*Tree spatial indexing for geospatial node lookup.

Tables: `node`, `way`, `waynode`, `segment`, `waylabel`, plus R*Tree virtual tables.

- **node**: id, lat, lon
- **way**: id, min_lat, max_lat, min_lon, max_lon (bounding box)
- **waynode**: way_id, node_id, pos (ordering)
- **segment**: n1, n2, way_id (adjacent node pairs within a way)
- **waylabel**: way_id, road, cycleway, salmon (ETL-derived labels)

See `services/README.md` for full ER diagram and design rationale.

## Lambda Deployment Model

- SQLite DB (~50MB) deployed as a Lambda Layer → mounted at `/opt/lib/db.db3`
- Rust binary compiled for arm64 via Cargo Lambda
- Thread-local `Graph` singleton avoids re-initialization across invocations
- Response compression: gzip, deflate, brotli (based on Accept-Encoding)
- No external network calls during routing — all data is local to the Lambda
