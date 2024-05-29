use anyhow::anyhow;
use geo::Point;
use lambda_http::{
    run, service_fn, Error as LambdaError, IntoResponse, Request as LambdaRequest, RequestExt,
};
use query_map::QueryMap;
use serde::Deserialize;
use tracing::error;

use rusty_router::geojson;
use rusty_router::osm::Graph;

// create a singleton of the Graph struct on lambda boot
thread_local! {
    static GRAPH: Graph = Graph::new().unwrap();
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .compact()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    run(service_fn(handler)).await.unwrap();
}

async fn handler(event: LambdaRequest) -> Result<impl IntoResponse, LambdaError> {
    GRAPH.with(|graph| match event.raw_http_path() {
        "/traverse" => traverse_handler(graph, event),
        "/route" => route_handler(graph, event),
        _ => Err(anyhow!("invalid path").into()),
    })
}

#[derive(Debug, Deserialize)]
struct TraversalParams {
    lat: f64,
    lon: f64,
    depth: usize,
}

impl TryFrom<&QueryMap> for TraversalParams {
    type Error = anyhow::Error;

    // TODO: DRY or a lib?
    fn try_from(query_map: &QueryMap) -> Result<Self, Self::Error> {
        let lon = query_map
            .first("lon")
            .ok_or_else(|| anyhow!("missing lon"))?
            .parse::<f64>()
            .map_err(|_| anyhow!("invalid lon"))?;
        let lat = query_map
            .first("lat")
            .ok_or_else(|| anyhow!("missing lat"))?
            .parse::<f64>()
            .map_err(|_| anyhow!("invalid lat"))?;
        let depth = query_map
            .first("depth")
            .ok_or_else(|| anyhow!("missing depth"))?
            .parse::<usize>()
            .map_err(|_| anyhow!("invalid depth"))?;

        Ok(Self { lon, lat, depth })
    }
}

fn traverse_handler(graph: &Graph, event: LambdaRequest) -> Result<String, LambdaError> {
    let params = TraversalParams::try_from(&event.query_string_parameters()).map_err(|e| {
        error!("Parsing Error: {:?}", e);
        e
    })?;

    let starting_coord = Point::new(params.lon, params.lat);

    let traversal = graph
        .traverse_from(starting_coord, params.depth)
        .map_err(|e| {
            error!("Routing Error: {e}");
            e
        })?;

    let traversal = geojson::serialize_traversal_geoms(&traversal).map_err(|e| {
        error!("Serialization Error: {e}");
        e
    })?;

    Ok(format!("{{ \"traversal\": {traversal} }}"))
}

#[derive(Debug, Deserialize)]
struct RouteParams {
    start: Point,
    end: Point,
    with_traversal: bool,
}

fn parse_point(param: &str) -> Result<Point, anyhow::Error> {
    if let Some((lon, lat)) = param.split_once(',') {
        let lon: f64 = lon.parse()?;
        let lat: f64 = lat.parse()?;
        Ok(Point::new(lon, lat))
    } else {
        Err(anyhow!("Couldn't parse Point"))
    }
}

impl TryFrom<&QueryMap> for RouteParams {
    type Error = anyhow::Error;

    // TODO: DRY or a lib?
    fn try_from(query_map: &QueryMap) -> Result<Self, Self::Error> {
        let start = query_map
            .first("start")
            .ok_or_else(|| anyhow!("missing start"))?;
        let start = parse_point(start).map_err(|_| anyhow!("invalid start"))?;

        let end = query_map
            .first("end")
            .ok_or_else(|| anyhow!("missing end"))?;
        let end = parse_point(end).map_err(|_| anyhow!("invalid end"))?;

        let with_traversal: bool = query_map
            .first("with_traversal")
            .unwrap_or("false")
            .parse()
            .map_err(|_| anyhow!("invalid with_traversal"))?;

        Ok(Self {
            start,
            end,
            with_traversal,
        })
    }
}

fn route_handler(graph: &Graph, event: LambdaRequest) -> Result<String, LambdaError> {
    let params = RouteParams::try_from(&event.query_string_parameters()).map_err(|e| {
        error!("Parsing Error: {:?}", e);
        e
    })?;

    let (route, traversal) = graph
        .route_between(params.start, params.end, params.with_traversal)
        .map_err(|e| {
            error!("Routing Error: {e}");
            e
        })?;

    let route = geojson::serialize_route_geom(&route).map_err(|e| {
        error!("Serialization Error: {e}");
        e
    })?;
    let traversal = match traversal {
        Some(t) => geojson::serialize_traversal_geoms(&t).map_err(|e| {
            error!("Serialization Error: {e}");
            e
        })?,
        None => "null".to_string(),
    };

    // TODO: oh god how do we use struct serialization to help us here
    //       had issues trying to leverage the geojson serialization helpers in a custom
    //       Serializer impl for a RouteResponse struct
    Ok(format!(
        "{{ \"route\": {route}, \"traversal\": {traversal} }}"
    ))
}
