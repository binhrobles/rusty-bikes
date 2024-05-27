use anyhow::anyhow;
use axum::{extract, http::StatusCode as AxumStatus};
use dotenvy::dotenv;
use geo::Point;
use lambda_http::{
    http::{Response as LambdaResponse, StatusCode},
    run, service_fn, Error as LambdaError, IntoResponse, Request as LambdaRequest, RequestExt,
};
use query_map::QueryMap;
use serde::Deserialize;
use serde_json::{json, Value};
use tracing::error;

use rusty_router::geojson;
use rusty_router::osm::Graph;

#[tokio::main]
async fn main() {
    dotenv().expect(".env file not found");

    tracing_subscriber::fmt()
        .compact()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    run(service_fn(traverse_handler)).await.unwrap();
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

async fn traverse_handler(event: LambdaRequest) -> Result<impl IntoResponse, LambdaError> {
    let params = TraversalParams::try_from(&event.query_string_parameters()).map_err(|e| {
        error!("Parsing Error: {:?}", e);
        e
    })?;

    let starting_coord = Point::new(params.lon, params.lat);

    let graph = Graph::new().unwrap();
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
    start: String,
    end: String,
    with_traversal: Option<bool>,
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

async fn route_handler(query: extract::Query<RouteParams>) -> Result<String, AxumStatus> {
    let start = parse_point(&query.start).map_err(|_| AxumStatus::BAD_REQUEST)?;
    let end = parse_point(&query.end).map_err(|_| AxumStatus::BAD_REQUEST)?;
    let with_traversal = query.with_traversal.unwrap_or_default();

    let graph = Graph::new().unwrap();

    let (route, traversal) = graph
        .route_between(start, end, with_traversal)
        .map_err(|e| {
            error!("Routing Error: {e}");
            AxumStatus::INTERNAL_SERVER_ERROR
        })?;

    let route =
        geojson::serialize_route_geom(&route).map_err(|_| AxumStatus::INTERNAL_SERVER_ERROR)?;
    let traversal = match traversal {
        Some(t) => {
            geojson::serialize_traversal_geoms(&t).map_err(|_| AxumStatus::INTERNAL_SERVER_ERROR)?
        }
        None => "null".to_string(),
    };

    // TODO: oh god how do we use struct serialization to help us here
    //       had issues trying to leverage the geojson serialization helpers in a custom
    //       Serializer impl for a RouteResponse struct
    Ok(format!(
        "{{ \"route\": {route}, \"traversal\": {traversal} }}"
    ))
}
