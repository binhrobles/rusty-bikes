# Bidirectional Corridor Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the reconnection-BFS corridor extraction with a bidirectional intersection approach — run a second forward exploration from the finish point and include nodes explored from both directions with competitive combined cost.

**Architecture:** Two forward explorations (existing A* from start, new Dijkstra from finish) are intersected by node ID. Corridor candidates must appear in both, be near the route, and have combined cost within a ratio of optimal. Salmon flags are inverted for the backward exploration to correct directional cost asymmetry.

**Tech Stack:** Rust 2021, SQLite (in-memory), A* / Dijkstra traversal

**Spec:** `docs/superpowers/specs/2026-03-15-bidirectional-corridor-design.md`

---

## Task 1: Add `reverse_salmon` to CostModel + Clone derive

**Files:**
- Modify: `services/src/graph/cost.rs:56-70` (CostModel struct)
- Modify: `services/src/graph/cost.rs:129-141` (calculate_cost method)

- [ ] **Step 1: Add `Clone` derive and `reverse_salmon` field to CostModel**

```rust
// cost.rs:56 — add Clone to derive
#[derive(Debug, Clone)]
pub struct CostModel {
    cycleway_coefficient: Cost,
    road_coefficient: Cost,
    salmon_coefficient: Cost,
    distance_coefficient: Cost,
    elevation_coefficient: Cost,
    cycleway_weights: [Cost; 4],
    road_weights: [Cost; 5],
    /// When true, invert the salmon flag during cost calculation.
    /// Used for backward exploration where the traversal direction is
    /// opposite to the cyclist's actual direction of travel.
    pub reverse_salmon: bool,
}
```

- [ ] **Step 2: Add `reverse_salmon: false` to all construction sites**

In `Default::default()` (line 117-126):
```rust
Self {
    cycleway_coefficient: 0.3,
    road_coefficient: 0.4,
    salmon_coefficient: 1.3,
    distance_coefficient: 0.0,
    elevation_coefficient: 0.0,
    cycleway_weights,
    road_weights,
    reverse_salmon: false,
}
```

In `From<CostModelInput>` (line 84-93):
```rust
Self {
    cycleway_coefficient: input.cycleway_coefficient,
    road_coefficient: input.road_coefficient,
    salmon_coefficient: input.salmon_coefficient,
    distance_coefficient: input.distance_coefficient,
    elevation_coefficient: input.elevation_coefficient,
    cycleway_weights,
    road_weights,
    reverse_salmon: false,
}
```

In `MobileCostModel::resolve()` (line 289-297):
```rust
CostModel {
    cycleway_coefficient,
    road_coefficient,
    salmon_coefficient,
    distance_coefficient,
    elevation_coefficient,
    cycleway_weights,
    road_weights,
    reverse_salmon: false,
}
```

- [ ] **Step 3: Invert salmon flag in calculate_cost when reverse_salmon is true**

```rust
// cost.rs:131-141
#[inline]
pub fn calculate_cost(&self, way_labels: &WayLabels) -> Cost {
    let (cycleway, road, salmon) = way_labels;
    let salmon_flag = if self.reverse_salmon { !*salmon } else { *salmon };
    let cycleway_cost = self.cycleway_coefficient * self.cycleway_weights[*cycleway as usize];
    let road_cost = self.road_coefficient * self.road_weights[*road as usize];
    let salmon_cost = if salmon_flag {
        self.salmon_coefficient
    } else {
        1.0
    };
    (cycleway_cost + road_cost + self.distance_coefficient) * salmon_cost
}
```

- [ ] **Step 4: Verify it compiles**

Run: `cd services && cargo check`
Expected: compiles with no errors

- [ ] **Step 5: Commit**

```bash
git add services/src/graph/cost.rs
git commit -m "feat(cost): add reverse_salmon flag and Clone to CostModel

Used by backward corridor exploration to invert salmon penalties,
correcting directional cost asymmetry on one-way streets."
```

---

## Task 2: Fix `traverse_from` to return Ok on queue exhaustion

**Files:**
- Modify: `services/src/graph/traversal.rs:389-435` (traverse_from method)

- [ ] **Step 1: Change traverse_from to return Ok(()) when queue empties**

Replace the final `Err(anyhow!("Traversal failed"))` (line 434) with `Ok(())`:

```rust
// traversal.rs:389-435 — the traverse_from method
// Change the final line from:
//     Err(anyhow!("Traversal failed"))
// To:
        Ok(())
    }
```

The while loop naturally exits when the queue is empty. Currently this returns Err, discarding all explored nodes. Exhausting the search space is valid termination — the explored nodes in `came_from` are the result.

- [ ] **Step 2: Verify it compiles**

Run: `cd services && cargo check`
Expected: compiles. The `anyhow` import may become unused — check and clean up if needed.

- [ ] **Step 3: Run existing tests**

Run: `cd services && cargo test`
Expected: all existing tests pass (this change only affects behavior when queue empties before max_depth, which is the backward exploration case)

- [ ] **Step 4: Commit**

```bash
git add services/src/graph/traversal.rs
git commit -m "fix(traversal): return Ok on queue exhaustion in traverse_from

Queue exhaustion means all reachable nodes were explored — this is
valid termination, not failure. Previously returned Err which caused
calculate_traversal to discard all partial results."
```

---

## Task 3: Rewrite corridor extraction

**Files:**
- Modify: `services/src/api/corridor.rs` (complete rewrite of extract_corridor, delete bfs)

- [ ] **Step 1: Write the new extract_corridor function**

Replace the entire `extract_corridor` function and delete the `bfs` helper. Keep `serialize_corridor` unchanged.

```rust
use crate::graph::{Cost, TraversalSegment};
use crate::osm::NodeId;
use geo::{HaversineDistance, Point};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use tracing::debug;

use crate::graph::{END_NODE_ID, START_NODE_ID};

/// Combined cost of a detour through a corridor node can be at most this ratio
/// above the optimal route cost. 1.5 = 50% more expensive than optimal.
const CORRIDOR_COST_RATIO: f32 = 1.5;
const MAX_DISTANCE_METERS: f64 = 1000.0;

pub fn extract_corridor<'a>(
    forward_traversal: &'a [TraversalSegment],
    backward_traversal: &[TraversalSegment],
    route: &[TraversalSegment],
    optimal_cost: Cost,
) -> Vec<&'a TraversalSegment> {
    if optimal_cost <= 0.0 {
        return vec![];
    }

    // Build backward cost lookup: node_id -> cost from finish exploration
    let backward_costs: HashMap<NodeId, Cost> = backward_traversal
        .iter()
        .map(|s| (s.to.id, s.cost))
        .collect();

    // Build set of route edges for exclusion
    let route_edges: HashSet<(NodeId, NodeId)> =
        route.iter().map(|s| (s.from.id, s.to.id)).collect();

    // Pre-compute route segment midpoints for proximity filtering
    let route_refs: Vec<(Point, Cost)> = route
        .iter()
        .map(|s| {
            (
                Point::new(
                    (s.geometry.start.x + s.geometry.end.x) / 2.0,
                    (s.geometry.start.y + s.geometry.end.y) / 2.0,
                ),
                s.cost,
            )
        })
        .collect();

    let result: Vec<&'a TraversalSegment> = forward_traversal
        .iter()
        .filter(|seg| {
            // Exclude virtual nodes and route edges
            if seg.from.id == START_NODE_ID || seg.to.id == END_NODE_ID {
                return false;
            }
            if route_edges.contains(&(seg.from.id, seg.to.id)) {
                return false;
            }

            // Must exist in backward exploration (reachable from finish)
            let backward_cost = match backward_costs.get(&seg.to.id) {
                Some(c) => *c,
                None => return false,
            };

            // Proximity filter
            let mid = Point::new(
                (seg.geometry.start.x + seg.geometry.end.x) / 2.0,
                (seg.geometry.start.y + seg.geometry.end.y) / 2.0,
            );
            let nearest_dist = route_refs
                .iter()
                .map(|(rp, _)| rp.haversine_distance(&mid))
                .min_by(|a, b| a.total_cmp(b))
                .unwrap_or(f64::MAX);

            if nearest_dist >= MAX_DISTANCE_METERS {
                return false;
            }

            // Combined cost ratio: (cost from start to node + cost from finish to node) / optimal
            let combined_cost = seg.cost + backward_cost;
            combined_cost / optimal_cost <= CORRIDOR_COST_RATIO
        })
        .collect();

    debug!(
        "corridor: {} segments from {} forward x {} backward, optimal_cost={:.1}",
        result.len(),
        forward_traversal.len(),
        backward_traversal.len(),
        optimal_cost,
    );

    result
}

/// Serialize corridor segments to a GeoJSON FeatureCollection Value.
pub fn serialize_corridor(segments: &[&TraversalSegment]) -> Result<Value, anyhow::Error> {
    let owned: Vec<TraversalSegment> = segments.iter().map(|s| (*s).clone()).collect();
    Ok(serde_json::from_str(
        &geojson::ser::to_feature_collection_string(&owned)?,
    )?)
}
```

- [ ] **Step 2: Verify it compiles**

Run: `cd services && cargo check`
Expected: compiles. May get warnings about removed imports (`VecDeque`, `GraphRepository`) — that's expected since we removed the BFS logic.

- [ ] **Step 3: Commit**

```bash
git add services/src/api/corridor.rs
git commit -m "refactor(corridor): replace reconnection BFS with bidirectional intersection

extract_corridor now takes forward and backward traversals and includes
segments that appear in both with competitive combined cost. Removes
the 3-hop reconnection BFS, bfs() helper, and DB dependency."
```

---

## Task 4: Update navigate handler to run backward exploration

**Files:**
- Modify: `services/src/bin/lambda-handler.rs:258-307` (corridor block in navigate_handler)

- [ ] **Step 1: Clone cost_model before it's consumed by calculate_route**

The `cost_model` variable (line 227-230) is `Option<CostModel>` and gets moved into `calculate_route` (line 237). We need it again for the backward exploration. Add a clone before the route calculation:

```rust
    // Clone cost_model before calculate_route consumes it
    let cost_model_for_corridor = cost_model.clone();
```

Insert this line just before the `let (route_segments, traversal, _) = graph.calculate_route(...)` call (before line 232).

- [ ] **Step 2: Replace the corridor extraction block**

Replace lines 258-307 (the `let corridor_value = if with_corridor { ... }` block) with:

```rust
    // Extract corridor from traversal if requested
    let corridor_value = if with_corridor {
        let forward_traversal = traversal.unwrap_or_default();

        // Backward exploration from finish with inverted salmon
        let mut backward_cost_model = cost_model_for_corridor.unwrap_or_default();
        backward_cost_model.reverse_salmon = true;
        let backward_traversal = graph
            .calculate_traversal(end_point, 40, Some(backward_cost_model), params.heuristic_weight)
            .unwrap_or_default();

        // Second-to-last segment has the real accumulated cost;
        // the last segment is the virtual END_NODE with cost=0
        let optimal_cost = route_segments
            .iter()
            .rev()
            .find(|s| s.cost > 0.0)
            .map(|s| s.cost)
            .unwrap_or(0.0);

        let corridor_segments = corridor::extract_corridor(
            &forward_traversal,
            &backward_traversal,
            &route_segments,
            optimal_cost,
        );
        Some(
            corridor::serialize_corridor(&corridor_segments).map_err(|e| {
                error!("Corridor serialization error: {e}");
                e
            })?,
        )
    } else {
        None
    };
```

- [ ] **Step 3: Remove unused imports if any**

Check if `HashMap` import (line 17) is still needed — it was used by the merged_traversal logic. The `TraversalSegment` import is still needed. Remove `HashMap` from the import if unused.

- [ ] **Step 4: Verify it compiles**

Run: `cd services && cargo check`
Expected: compiles with no errors

- [ ] **Step 5: Commit**

```bash
git add services/src/bin/lambda-handler.rs
git commit -m "feat(navigate): use bidirectional corridor extraction

Replace deep-exploration-from-start + reconnection BFS with a second
forward exploration from the finish point with inverted salmon. Corridor
candidates are the intersection of both explorations filtered by
combined cost ratio and proximity."
```

---

## Task 5: Remove `get_nodes_with_edge_to` from GraphRepository

**Files:**
- Modify: `services/src/graph/repository.rs:28-32` (trait method), `services/src/graph/repository.rs:246-280` (SqliteGraphRepository impl)
- Modify: `services/src/graph/in_memory_repository.rs:178+` (InMemoryGraphRepository impl)

- [ ] **Step 1: Remove from the trait definition**

In `repository.rs`, remove the `get_nodes_with_edge_to` method from the `GraphRepository` trait (around line 28-32).

- [ ] **Step 2: Remove from SqliteGraphRepository impl**

In `repository.rs`, remove the `get_nodes_with_edge_to` implementation (around lines 246-280).

- [ ] **Step 3: Remove from InMemoryGraphRepository impl**

In `in_memory_repository.rs`, remove the `get_nodes_with_edge_to` implementation (around line 178+).

- [ ] **Step 4: Verify it compiles**

Run: `cd services && cargo check`
Expected: compiles with no errors or warnings about dead code

- [ ] **Step 5: Run all tests**

Run: `cd services && cargo test`
Expected: all tests pass

- [ ] **Step 6: Commit**

```bash
git add services/src/graph/repository.rs services/src/graph/in_memory_repository.rs
git commit -m "chore: remove get_nodes_with_edge_to from GraphRepository

No longer needed after corridor extraction switched to bidirectional
intersection approach."
```

---

## Task 6: Integration test — corridor with backward exploration

**Files:**
- Create: `services/tests/corridor.rs`

- [ ] **Step 1: Write integration test**

```rust
use geo::Point;
use rusty_router::graph::{CostModel, Graph, Traversable};

/// Verify that bidirectional corridor extraction produces non-empty results
/// and that no corridor segment overlaps with the route.
#[test]
fn corridor_has_segments_and_no_route_overlap() -> Result<(), anyhow::Error> {
    let graph = Graph::new()?;

    let start = Point::new(-73.963711, 40.6955785); // Brooklyn
    let end = Point::new(-73.990386, 40.736004); // Midtown-ish

    // 1. Get route + forward traversal
    let (route, traversal, _meta) =
        graph.calculate_route(start, end, true, None, None)?;
    let forward_traversal = traversal.unwrap();

    // 2. Backward exploration from finish with inverted salmon
    let mut backward_cost_model = CostModel::default();
    backward_cost_model.reverse_salmon = true;
    let backward_traversal = graph.calculate_traversal(end, 40, Some(backward_cost_model), None)?;

    // 3. Extract corridor
    let optimal_cost = route
        .iter()
        .rev()
        .find(|s| s.cost > 0.0)
        .map(|s| s.cost)
        .unwrap_or(0.0);

    let corridor =
        rusty_router::api::corridor::extract_corridor(&forward_traversal, &backward_traversal, &route, optimal_cost);

    // Corridor should have segments for a Brooklyn→Midtown route
    assert!(
        !corridor.is_empty(),
        "Expected non-empty corridor for Brooklyn→Midtown route"
    );

    // No corridor segment should be on the route
    let route_edges: std::collections::HashSet<(i64, i64)> =
        route.iter().map(|s| (s.from.id, s.to.id)).collect();
    for seg in &corridor {
        assert!(
            !route_edges.contains(&(seg.from.id, seg.to.id)),
            "Corridor segment ({} -> {}) overlaps with route",
            seg.from.id,
            seg.to.id,
        );
    }

    Ok(())
}
```

- [ ] **Step 2: Run the test**

Run: `cd services && cargo test --test corridor`
Expected: PASS

- [ ] **Step 3: Commit**

```bash
git add services/tests/corridor.rs
git commit -m "test: add integration test for bidirectional corridor extraction

Verifies corridor is non-empty for Brooklyn→Midtown and has no
overlap with the route."
```

---

## Task 7: Manual validation + tune CORRIDOR_COST_RATIO

- [ ] **Step 1: Start the backend**

Run: `make service-watch`

- [ ] **Step 2: Test corridor output**

```bash
curl -s -X POST http://localhost:9000/lambda-url/lambda-handler/navigate \
    -H "Content-Type: application/json" \
    -d '{"start":{"lat":40.6955785,"lon":-73.963711},"end":{"lat":40.736004,"lon":-73.990386},"with_corridor":true}' \
    -o /tmp/corridor-response.json

jq '.corridor.features | length' /tmp/corridor-response.json
jq '.route.features | length' /tmp/corridor-response.json
```

Expected: corridor has meaningful number of features (10-100+), route has its usual steps.

- [ ] **Step 3: Evaluate corridor density and tune ratio**

If corridor is too sparse, increase `CORRIDOR_COST_RATIO` in `corridor.rs` (try 1.8, 2.0).
If corridor is too dense, decrease (try 1.3, 1.2).

Adjust and re-test until the corridor provides useful parallel street coverage without overwhelming the map.

- [ ] **Step 4: Commit any ratio changes**

```bash
git add services/src/api/corridor.rs
git commit -m "tune: adjust CORRIDOR_COST_RATIO based on manual testing"
```
