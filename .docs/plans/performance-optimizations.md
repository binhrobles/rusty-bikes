# Routing Engine Performance Optimizations

## Problem

Cross-city route (~40.77 → ~40.63 lat) takes 3.5s locally and 5s on Lambda. Goal: 4-10x improvement.

**Root cause** (`services/src/graph/repository.rs:166`):
> "flamegraphs show we spend 95%+ of our time in this query"

`get_neighbors_with_labels()` — a 2-JOIN SQL query — is called once **per A\* expansion** (thousands of calls per route against an on-disk SQLite database).

## Optimization Iterations

### Iteration 1: In-Memory HashMap Adjacency List

**Approach:** Load the entire routing graph (Nodes + Segments + WayLabels) into a Rust `HashMap<NodeId, Vec<InMemoryEdge>>` at startup via one bulk SQL query. Hot path goes from a per-expansion SQL query with 2 JOINs → O(1) HashMap lookup.

**Files changed:**
- `services/Cargo.toml` — added `criterion` dev-dep + `[[bench]]` section
- `services/benches/routing.rs` — Criterion benchmark (cross-city route from issue)
- `services/src/graph/in_memory_repository.rs` — `InMemoryGraphRepository` impl
- `services/src/graph/mod.rs` — module registration
- `services/src/graph/core.rs` — `Graph::new()` uses `InMemoryGraphRepository`

**Bulk load query:**
```sql
SELECT S.n1, S.way, S.n2, N2.lon, N2.lat, S.distance, WL.cycleway, WL.road, WL.salmon
FROM Segments S
JOIN Nodes N2 ON S.n2 = N2.id
JOIN WayLabels WL ON S.way = WL.id
```

**R\*Tree snapping** (called only 2× per route) retained on the original `SqliteGraphRepository`.

**Expected improvement:** 10-100x on `get_neighbors_with_labels` call.

**Measured improvement:** TBD (run `cargo bench --bench routing`)

---

## Running Benchmarks

```bash
# Criterion benchmark (statistical, HTML report)
cd services && DB_PATH=../db.db3 cargo bench --bench routing

# Flamegraph (find next hotspot after primary optimization)
make service-flamegraph

# Sanity check
make service-test
```

## Known Future Optimizations (post-flamegraph)

- `target_neighbor_node_ids.contains()` at `traversal.rs:270` — Vec O(n) → `HashSet<NodeId>`
- `BinaryHeap<TraversalSegment>` stores/clones large structs — use `(Cost, NodeId)` + lazy deletion
- `HashMap::with_capacity(est)` to reduce rehashing during load
