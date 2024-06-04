/// Middleware for formatting Graph structures into Geojson
use crate::graph::{Depth, TraversalSegment};
use crate::osm::{Distance, Node, NodeId, WayId};
use geo::{Coord, LineString};
use geojson::ser::serialize_geometry;
use serde::{Serialize, Serializer};

/// simple serialization of a Node to just its ID
pub fn serialize_node_simple<S>(node: &Node, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_i64(node.id)
}

/// serialization of a float to an int
pub fn serialize_float_as_int<S>(float: &f64, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_i64(*float as i64)
}

#[derive(Serialize, Clone, Debug)]
pub struct Route {
    steps: Vec<RouteStep>,

    len: usize,
    last_step_way: WayId,
}

impl Route {
    pub fn new(segment: &TraversalSegment) -> Self {
        let init_step = RouteStep::new(segment, 0);
        Self {
            steps: vec![init_step],
            len: 1,
            last_step_way: segment.way,
        }
    }

    /// extends this route with the specified TraversalSegment
    /// attempts to add to the last RouteStep (if on the same Way).
    /// otherwise, inits a new RouteStep
    pub fn extend_with(&mut self, segment: &TraversalSegment) {
        // if still on the same way, extend the existing step
        if self.last_step_way == segment.way {
            let last_step = self.steps.get_mut(self.len - 1).unwrap();
            last_step.extend_with(segment);
        } else {
            // otherwise, create and append a new step
            self.len += 1;
            self.last_step_way = segment.way;
            self.steps.push(RouteStep::new(segment, self.len));
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
    pub depth: Depth,
    pub idx: usize,
}

impl RouteStep {
    pub fn new(segment: &TraversalSegment, idx: usize) -> Self {
        Self {
            geometry: vec![segment.geometry.start, segment.geometry.end],

            distance: segment.length,
            from: segment.from.id,
            to: segment.to.id,
            way: segment.way,
            depth: segment.depth,
            idx,
        }
    }

    pub fn extend_with(&mut self, segment: &TraversalSegment) {
        self.geometry.push(segment.geometry.end);
        self.distance += segment.length;
        self.to = segment.to.id;
        self.depth = segment.depth; // takes the depth of the last segment appended
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

pub fn serialize_traversal_geoms(traversal: &[TraversalSegment]) -> Result<String, anyhow::Error> {
    Ok(geojson::ser::to_feature_collection_string(traversal)?)
}

pub fn serialize_route_geom(segments: &[TraversalSegment]) -> Result<String, anyhow::Error> {
    let route: Route = segments.into();
    Ok(geojson::ser::to_feature_collection_string(&route.steps)?)
}
