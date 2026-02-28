# Build, Deploy & Data Pipeline

## Make Targets Reference

### Local Development
| Target | Description |
|--------|-------------|
| `make service-watch` | Run Rust backend locally at localhost:9000 (cargo-lambda, hot reload) |
| `make client-watch` | Run Svelte dev server (Vite) |
| `make service-test` | Run Rust tests (requires `db.db3`) |
| `make service-bench` | Run benchmarks → `services/benches/current.out` |
| `make service-flamegraph` | Generate flamegraph for profiling |

Pass `release=1` for release builds: `make service-watch release=1`

### Deployment
| Target | Description |
|--------|-------------|
| `make service-deploy` | Full deploy: sam-build + sam-deploy |
| `make sam-build` | Validate + build SAM template |
| `make sam-deploy` | Deploy to AWS (no confirmation prompt) |
| `make sam-clean` | Remove `.aws-sam/build/` |

### Data Pipeline
| Target | Description |
|--------|-------------|
| `make osm-download` | Download NYC OSM data → `out.geom.json` |
| `make db-build` | Build SQLite from GeoJSON (backs up existing db.db3) |
| `make layer-build` | Package db.db3 into Lambda layer zip |
| `make layer-upload` | Upload layer zip to `s3://rusty-bikes-osm-data/` |

## Data Pipeline Flow

```
make osm-download          # Overpass API → out.geom.json (~50MB)
    ↓
make db-build              # init-db creates schema, populate-db runs ETL
    ↓                      #   (OSM tags → Road/Cycleway/Salmon labels)
make layer-build           # Zips db.db3 into Lambda layer artifact
    ↓
make layer-upload          # Uploads to S3 bucket
    ↓
# Update template.yaml LayerName to trigger new version
make service-deploy        # Deploys Lambda + new layer reference
```

After uploading a new layer, you must update the `LayerName` in `template.yaml` to trigger a new layer version association.

## AWS SAM Configuration

Defined in `template.yaml`:
- **Runtime**: `provided.al2023` (custom runtime, Rust binary)
- **Architecture**: arm64
- **Memory**: 1024 MB
- **Timeout**: 30 seconds
- **DB path**: `/opt/lib/db.db3` (Lambda layer mount point)
- **API Gateway**: CORS enabled (all origins, POST + GET + OPTIONS)
- **S3 bucket**: `rusty-bikes-osm-data` for layer storage

SAM deploy config in `samconfig.toml`.

## GitHub Actions

Single workflow: `.github/workflows/deploy-gh-pages.yml`
- **Trigger**: Push to `main` with changes in `client/**`, or manual dispatch
- **Action**: Build Svelte client with Yarn → deploy to GitHub Pages
- **Node**: v22, uses Yarn with frozen lockfile

## Environment Requirements

| Tool | Purpose |
|------|---------|
| Rust (2021 edition) | Backend compilation |
| Cargo Lambda | Local dev + Lambda build |
| Node 22 + Yarn | Client development |
| AWS SAM CLI | Lambda deployment |
| AWS CLI | Layer upload to S3 |
| SQLite3 | Optional: inspect db.db3 directly |
