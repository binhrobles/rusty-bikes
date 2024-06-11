use anyhow::anyhow;
use geo::Point;
use lambda_http::{
    run, service_fn, Body, Error as LambdaError, Request, RequestExt, RequestPayloadExt, Response,
};
use rusty_router::osm::Location;
use serde::Deserialize;
use tracing::{error, info};

use rusty_router::geojson;
use rusty_router::graph::{CostModel, Graph};

// create a singleton of the Graph struct on lambda boot
thread_local! {
    static GRAPH: Graph = Graph::new().unwrap();
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .compact()
        .with_max_level(tracing::Level::DEBUG)
        .with_target(false)
        .without_time()
        .init();

    run(service_fn(handler)).await.unwrap();
}

async fn handler(event: Request) -> Result<Response<Body>, LambdaError> {
    let body = GRAPH.with(|graph| match event.raw_http_path() {
        "/traverse" => traverse_handler(graph, event),
        "/route" => route_handler(graph, event),
        _ => Err(anyhow!("invalid path")),
    })?;

    let response = Response::builder()
        .status(200)
        .header("content-type", "application/json")
        .header("Access-Control-Allow-Headers", "Content-Type")
        .header("Access-Control-Allow-Origin", "*")
        .header("Access-Control-Allow-Methods", "GET")
        .body(body.into())?;

    Ok(response)
}

#[derive(Debug, Deserialize)]
struct TraversalParams {
    lat: f64,
    lon: f64,
    depth: usize,
    cost_model: Option<CostModel>,
}

fn traverse_handler(graph: &Graph, event: Request) -> Result<String, anyhow::Error> {
    let params = event
        .payload::<TraversalParams>()?
        .ok_or_else(|| anyhow!("Missing traversal params"))?;

    let starting_coord = Point::new(params.lon, params.lat);
    if let Some(cost_model) = params.cost_model.as_ref() {
        info!("custom cost model: {:#?}", cost_model);
    }

    let traversal = graph
        .traverse_from(starting_coord, params.depth, params.cost_model)
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
    start: Location,
    end: Location,
    with_traversal: Option<bool>,
    cost_model: Option<CostModel>,
}

fn route_handler(graph: &Graph, event: Request) -> Result<String, anyhow::Error> {
    let params = event
        .payload::<RouteParams>()?
        .ok_or_else(|| anyhow!("Missing route params"))?;

    let with_traversal = params.with_traversal.unwrap_or(false);
    if let Some(cost_model) = params.cost_model.as_ref() {
        info!("custom cost model: {:#?}", cost_model);
    }

    let (route, traversal) = graph
        .route_between(
            params.start.into(),
            params.end.into(),
            with_traversal,
            params.cost_model,
        )
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
