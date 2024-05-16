/// Middleware when formatting Graph structures into Geojson
use crate::osm::{traversal::TraversalSegment, Distance, NodeId, WayId};
use geo::{Coord, LineString};
use geojson::ser::serialize_geometry;
use serde::{Serialize, Serializer};

#[derive(Serialize, Clone, Debug)]
pub struct Route {
    steps: Vec<RouteStep>,
}

impl Route {
    pub fn new(segment: &TraversalSegment) -> Self {
        Self {
            steps: vec![RouteStep::new(segment)],
        }
    }

    /// extends this route with the specified TraversalSegment
    /// attempts to add to the last RouteStep (if on the same Way).
    /// otherwise, inits a new RouteStep
    pub fn extend_with(&mut self, segment: &TraversalSegment) {
        // check last RouteStep
        let len = self.steps.len();
        let last_step = self.steps.get_mut(len - 1).unwrap();

        // if still on the same way, extend the existing step
        if last_step.way == segment.way {
            last_step.extend_with(segment);
        } else {
            // otherwise, create and append a new step
            self.steps.push(RouteStep::new(segment));
        }
    }
}

impl From<&[TraversalSegment]> for Route {
    fn from(segments: &[TraversalSegment]) -> Self {
        let mut iter = segments.iter();
        let mut route = Route::new(iter.next().unwrap());

        for segment in iter {
            route.extend_with(segment);
        }

        route
    }
}

#[derive(Serialize, Clone, Debug)]
pub struct RouteStep {
    #[serde(serialize_with = "serialize_route_step")]
    geometry: Vec<Coord>,

    // route step metadata from TraversalSegment
    pub from: NodeId,
    pub to: NodeId,
    pub way: WayId,
    pub distance: Distance,
}

impl RouteStep {
    pub fn new(segment: &TraversalSegment) -> Self {
        Self {
            geometry: vec![segment.geometry.start, segment.geometry.end],

            distance: segment.distance,
            from: segment.from.id,
            to: segment.to.id,
            way: segment.way,
        }
    }

    pub fn extend_with(&mut self, segment: &TraversalSegment) {
        self.geometry.push(segment.geometry.end);
        self.distance += segment.distance;
        self.to = segment.to.id;
    }
}

/// custom serialization to first create a LineString from a Vec<Coord>
/// and then serialize into geojson
pub fn serialize_route_step<S>(geometry: &[Coord], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let line_string = LineString::new(geometry.to_vec());
    serialize_geometry(&line_string, serializer)
}

pub fn aggregate_traversal_geoms(traversal: &[TraversalSegment]) -> Result<String, anyhow::Error> {
    Ok(geojson::ser::to_feature_collection_string(traversal)?)
}

pub fn route_geom(segments: &[TraversalSegment]) -> Result<String, anyhow::Error> {
    let route: Route = segments.into();

    Ok(geojson::ser::to_feature_collection_string(&route.steps)?)
}
