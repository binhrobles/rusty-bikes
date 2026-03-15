use geo::Point;
use rusty_router::graph::{CostModel, Graph};

/// Verify that bidirectional corridor extraction produces non-empty results
/// and that no corridor segment overlaps with the route.
#[test]
fn corridor_has_segments_and_no_route_overlap() -> Result<(), anyhow::Error> {
    let graph = Graph::new()?;

    let start = Point::new(-73.963711, 40.6955785); // Brooklyn
    let end = Point::new(-73.990386, 40.736004); // Midtown-ish

    // 1. Get route + forward traversal
    let (route, traversal, _meta) = graph.calculate_route(start, end, true, None, None)?;
    let forward_traversal = traversal.unwrap();

    // 2. Backward A* from finish→start with inverted salmon
    let mut backward_cost_model = CostModel::default();
    backward_cost_model.reverse_salmon = true;
    let (_, backward_trav, _) =
        graph.calculate_route(end, start, true, Some(backward_cost_model), None)?;
    let backward_traversal = backward_trav.unwrap();

    // 3. Extract corridor
    let optimal_cost = route
        .iter()
        .rev()
        .find(|s| s.cost > 0.0)
        .map(|s| s.cost)
        .unwrap_or(0.0);

    let corridor = rusty_router::api::corridor::extract_corridor(
        &forward_traversal,
        &backward_traversal,
        &route,
        optimal_cost,
    );

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
