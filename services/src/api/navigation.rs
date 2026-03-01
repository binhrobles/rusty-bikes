/// Lean response serialization for the /navigate endpoint (mobile-optimized).
/// Drops from/to/way IDs, adds way_name (empty until rb-1.6 adds DB support).
use crate::graph::TraversalSegment;
use crate::osm::{Distance, WayLabels};
use geo::{Coord, LineString};
use geojson::ser::serialize_geometry;
use serde::{Serialize, Serializer};
use serde_json::Value;

/// Estimated average cycling speed in m/s (~15 km/h)
const AVG_CYCLING_SPEED_MPS: f64 = 4.2;

#[derive(Serialize, Clone, Debug)]
pub struct NavigationStep {
    #[serde(serialize_with = "serialize_nav_step_geom")]
    geometry: Vec<Coord>,

    pub distance: Distance,
    pub way_name: String,
    pub labels: WayLabels,
}

impl NavigationStep {
    pub fn new(segment: &TraversalSegment) -> Self {
        Self {
            geometry: vec![segment.geometry.start, segment.geometry.end],
            distance: segment.length,
            way_name: String::new(), // TODO rb-1.6: populate from DB
            labels: segment.labels,
        }
    }

    pub fn extend_with(&mut self, segment: &TraversalSegment) {
        self.geometry.push(segment.geometry.end);
        self.distance += segment.length;
    }
}

fn serialize_nav_step_geom<S>(geometry: &[Coord], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let line_string = LineString::new(geometry.to_vec());
    serialize_geometry(&line_string, serializer)
}

#[derive(Serialize, Debug)]
pub struct NavigationMeta {
    pub total_distance: Distance,
    pub total_time_estimate: u32, // seconds
}

#[derive(Serialize, Debug)]
pub struct NavigationResponse {
    pub route: Value,
    pub meta: NavigationMeta,
}

/// Build lean navigation steps from route segments, merging consecutive segments
/// on the same way into a single step (same logic as Route in geojson.rs).
pub fn build_navigation_steps(segments: &[TraversalSegment]) -> Vec<NavigationStep> {
    let mut iter = segments.iter();
    let first = iter.next().unwrap();
    let mut steps = vec![NavigationStep::new(first)];
    let mut last_way = first.way;

    for segment in iter {
        if segment.way == last_way {
            steps.last_mut().unwrap().extend_with(segment);
        } else {
            last_way = segment.way;
            steps.push(NavigationStep::new(segment));
        }
    }

    steps
}

/// Serialize navigation steps into a GeoJSON FeatureCollection and compute meta.
pub fn serialize_navigation(segments: &[TraversalSegment]) -> Result<NavigationResponse, anyhow::Error> {
    let steps = build_navigation_steps(segments);

    let total_distance: Distance = steps.iter().map(|s| s.distance).sum();
    let total_time_estimate = (total_distance as f64 / AVG_CYCLING_SPEED_MPS).round() as u32;

    let route: Value = serde_json::from_str(
        &geojson::ser::to_feature_collection_string(&steps)?,
    )?;

    Ok(NavigationResponse {
        route,
        meta: NavigationMeta {
            total_distance,
            total_time_estimate,
        },
    })
}
