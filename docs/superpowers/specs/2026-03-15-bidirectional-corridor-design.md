# Bidirectional Corridor Extraction

## Problem

The current corridor shows alternate paths rooted at the **start** of the route. A* commits to the optimal path early, so parallel streets that become relevant mid-route are never explored. A cyclist at node N along the route wants to know "what detours from HERE still get me to the finish?" — but the current tree only answers that question from the start's perspective.

## Solution: Forward-from-Finish Exploration

Run two forward explorations and intersect them:

1. **Forward A\* (existing)**: S → F. Produces `cost_from_start[N]` for every explored node.
2. **Forward exploration from F (new)**: F → outward via `calculate_traversal`. Produces `cost_from_finish[N]` as an approximate `cost_to_finish[N]`.
3. **Corridor inclusion**: node C is a candidate if it appears in **both** explorations and:
   ```
   (cost_from_start[C] + cost_from_finish[C]) / optimal_route_cost ≤ CORRIDOR_COST_RATIO
   ```

### Why approximate is acceptable

`cost_from_finish[C]` ≠ `cost_to_finish[C]` due to elevation asymmetry, salmon (wrong-way) penalties, and one-way streets. For NYC's flat, mostly-bidirectional grid this is a good approximation. The corridor is a visual hint, not a navigation instruction — precision isn't critical.

**Known bias**: salmon penalties make `cost_from_finish[C]` **underestimate** `cost_to_finish[C]` for nodes that require traveling against one-way streets to reach the finish. A one-way street explored "the right way" from finish has salmon=1.0, but the cyclist going toward finish would pay the salmon penalty. This means the corridor filter is slightly too permissive near one-way clusters — acceptable for a visual hint.

## Architecture

### Data flow

```
navigate_handler (with_corridor=true)
  │
  ├─ calculate_route(start → end)           // existing A* forward traversal
  │   └─ returns: route_segments, forward_traversal, meta
  │
  ├─ calculate_traversal(end, depth=40)     // NEW: forward exploration from finish
  │   └─ returns: backward_traversal
  │
  └─ extract_corridor(forward_traversal, backward_traversal, route, optimal_cost)
      └─ returns: corridor segments (intersection filtered by cost ratio + proximity)
```

### Changes by file

**`services/src/bin/lambda-handler.rs` — navigate_handler**

Replace the current corridor block (lines 258-307) with:
- Always request traversal from `calculate_route` when `with_corridor` is true (already done)
- Run a second `calculate_traversal` from `end_point` with `depth=40` and the **same resolved cost model** used for the route (not default — mixing cost spaces would make the ratio filter meaningless)
- Pass both traversals to the new `extract_corridor` signature

Before (current):
```rust
// Deep exploration from start, merge with route traversal, reconnection BFS
let mut merged_traversal = traversal.clone()...;
if let Ok(deep_traversal) = graph.calculate_traversal(start_point, 40, None, ...) {
    // merge...
}
let corridor_segments = corridor::extract_corridor(&merged_vec, &route_segments, optimal_cost, &*graph.db);
```

After:
```rust
// Forward traversal from route A* + backward exploration from finish
let forward_traversal = traversal.clone().unwrap_or_default();
let backward_traversal = graph.calculate_traversal(
    end_point,
    40,
    cost_model,   // same cost model as the route — keeps cost spaces comparable
    params.heuristic_weight,
).unwrap_or_default();

let corridor_segments = corridor::extract_corridor(
    &forward_traversal,
    &backward_traversal,
    &route_segments,
    optimal_cost,
);
```

**`services/src/api/corridor.rs` — extract_corridor**

Complete rewrite. The new algorithm:

1. **Build lookup maps**: `forward_costs: HashMap<NodeId, Cost>` from forward traversal, `backward_costs: HashMap<NodeId, Cost>` from backward traversal.
2. **Identify route edges** for exclusion (same as today).
3. **Pre-compute route midpoints** for proximity filtering (same as today).
4. **Filter forward traversal segments** where:
   - Not on the route
   - Not START_NODE / END_NODE virtual nodes
   - Within `MAX_DISTANCE_METERS` (1000m) of the route
   - The segment's `to` node exists in the backward traversal (reachable from finish)
   - `(cost_from_start[to] + cost_from_finish[to]) / optimal_cost ≤ CORRIDOR_COST_RATIO`

New signature:
```rust
pub fn extract_corridor<'a>(
    forward_traversal: &'a [TraversalSegment],
    backward_traversal: &[TraversalSegment],
    route: &[TraversalSegment],
    optimal_cost: Cost,
) -> Vec<&'a TraversalSegment>
```

**What gets deleted:**
- The `db: &dyn GraphRepository` parameter — no more DB queries during corridor extraction
- All of Phase 2: reconnection BFS (3-hop expansion, `get_nodes_with_edge_to` calls)
- The `bfs()` helper function
- The forward/backward adjacency maps and reachability filtering
- The `get_nodes_with_edge_to` method on `GraphRepository` (if unused elsewhere)

**What stays:**
- `serialize_corridor()` — unchanged
- `MAX_DISTANCE_METERS` constant — still used for proximity filtering
- Route edge exclusion logic

**`services/src/graph/traversal.rs` — traverse_from**

Fix: `traverse_from` currently returns `Err("Traversal failed")` when the priority queue empties before reaching `max_depth`. This discards all explored nodes in `came_from`. Exhausting the search space is a valid termination — change to return `Ok(())` when the queue empties. This makes `calculate_traversal` return partial results instead of an error.

**`services/src/graph/repository.rs` — GraphRepository trait**

Remove `get_nodes_with_edge_to` from the trait and implementation (confirmed only used by corridor extraction).

### Constants

| Constant | Current | Proposed | Rationale |
|----------|---------|----------|-----------|
| `MAX_LOCAL_COST_RATIO` | 2.0 (local comparison) | Rename to `CORRIDOR_COST_RATIO`, start at 1.5 | Semantics changed: was local (segment vs nearest route segment), now global (total detour cost vs optimal route cost). 2.0 globally is very generous — 1.5 means "detour costs at most 50% more than optimal." Tune empirically. |
| `MAX_DISTANCE_METERS` | 1000.0 | Keep 1000.0 | Proximity filter unchanged |
| Backward exploration depth | N/A | 40 (same as current deep exploration) | Match existing exploration budget |

## Testing

### Unit test: corridor extraction

Test `extract_corridor` with synthetic traversal data:
- Forward traversal with nodes A, B, C (costs known)
- Backward traversal with nodes B, C, D (costs known)
- Route through A → D
- Assert: B and C included (in both), D excluded (not in forward), A excluded (on route)

### Integration test: navigate with corridor

Use existing test infrastructure (`services/tests/navigate-request.json` or similar):
- Request with `with_corridor: true`
- Assert corridor is non-empty
- Assert no corridor segments overlap with route segments
- Compare corridor coverage qualitatively against the old approach

### Manual validation

Use `make service-watch` + curl to visually compare corridor output for known routes (e.g., Brooklyn → Midtown) before and after. The corridor should show more parallel street options mid-route.

## Risks and mitigations

| Risk | Likelihood | Mitigation |
|------|-----------|------------|
| Backward exploration from finish explores irrelevant areas (away from route) | Medium | Proximity filter (1000m) already handles this; depth=40 limits exploration budget |
| Cost asymmetry causes poor corridor quality near one-way clusters | Low | Acceptable per design; one-way streets are rare on NYC's cycling grid |
| Two traversals slower than one + reconnection BFS | Low | DB is in-memory; traversal is fast. Reconnection BFS did multiple DB queries anyway. Net performance likely similar or better. |
| Forward A* traversal is narrower than current merged approach | Medium | The current code runs a separate depth=40 exploration from start and merges it. The new approach drops that, using only the A* `came_from` tree for the forward side. Start-side corridor coverage may be thinner. If this is a problem, we can add back a merged start exploration later — but try the simpler approach first. |
