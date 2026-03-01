use anyhow::anyhow;
use geo::Point;
use lambda_http::{
    run, service_fn, Body, Error as LambdaError, Request, RequestExt, RequestPayloadExt, Response,
};
use rusty_router::api::compression::Encoding;
use rusty_router::osm::Location;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::error;

use rusty_router::api::{compression, geojson, navigation};
use rusty_router::graph::{CostModel, Graph, RouteMetadata, Weight};

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
    let origin = event
        .headers()
        .get("origin")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_owned();

    let is_prod = std::env::var("STAGE").unwrap_or_default() == "Prod";
    let allowed = origin == "https://binhrobles.com"
        || (!is_prod && origin.starts_with("http://localhost"));
    if !allowed {
        return Ok(Response::builder().status(403).body(Body::Empty)?);
    }

    let body = GRAPH.with(|graph| match event.raw_http_path() {
        "/traverse" => traverse_handler(graph, &event),
        "/route" => route_handler(graph, &event),
        "/navigate" => navigate_handler(graph, &event),
        "/ping" => ping_handler(graph),
        _ => Err(anyhow!("invalid path")),
    })?;

    let mut response = Response::builder()
        .status(200)
        .header("content-type", "application/json")
        .header("Access-Control-Allow-Headers", "Content-Type")
        .header("Access-Control-Allow-Origin", &origin)
        .header("Access-Control-Allow-Methods", "GET,POST");

    // perform compression, if specified
    if let Some(accept_encoding) = event.headers().get("Accept-Encoding") {
        let accept_encoding = accept_encoding
            .to_str()
            .map_err(|_| anyhow!("could not parse Accept-Encoding header"))?;

        let (compression_output, encoding) =
            compression::compress_with_encoding(&body, accept_encoding).map_err(|e| {
                error!("Compression Error: {e}");
                anyhow!(e)
            })?;

        if encoding != Encoding::No {
            response = response.header("content-encoding", encoding.to_string());
            return Ok(response.body(compression_output.unwrap().into())?);
        }
    }

    // otherwise, return raw response body
    Ok(response.body(body.into())?)
}

/// handler for waking up the lambda
/// ensures that the Graph singleton is instantiated and traversable
fn ping_handler(graph: &Graph) -> Result<String, anyhow::Error> {
    graph
        .calculate_traversal(
            Point::new(-73.961677, 40.683762),
            10,
            None,
            None,
        )
        .map_err(|e| {
            error!("Routing Error: {e}");
            e
        })?;

    Ok("ok!".to_owned())
}

#[derive(Debug, Deserialize)]
struct TraversalParams {
    lat: f64,
    lon: f64,
    depth: usize,
    cost_model: Option<CostModel>,
    heuristic_weight: Option<Weight>,
}

#[derive(Serialize)]
struct TraversalResponse {
    traversal: Value,
}

fn traverse_handler(graph: &Graph, event: &Request) -> Result<String, anyhow::Error> {
    let params = event
        .payload::<TraversalParams>()?
        .ok_or_else(|| anyhow!("Missing traversal params"))?;

    let starting_coord = Point::new(params.lon, params.lat);

    let traversal = graph
        .calculate_traversal(
            starting_coord,
            params.depth,
            params.cost_model,
            params.heuristic_weight,
        )
        .map_err(|e| {
            error!("Routing Error: {e}");
            e
        })?;
    let traversal = geojson::serialize_traversal_geoms(&traversal).map_err(|e| {
        error!("Serialization Error: {e}");
        e
    })?;

    let response = TraversalResponse { traversal };

    // TODO: vec -> string -> json::Value -> string ?
    Ok(serde_json::to_string(&response)?)
}

#[derive(Debug, Deserialize)]
struct RouteParams {
    start: Location,
    end: Location,
    with_traversal: Option<bool>,
    cost_model: Option<CostModel>,
    heuristic_weight: Option<Weight>,
}

#[derive(Serialize)]
struct RouteResponse {
    route: Value,
    traversal: Option<Value>,
    meta: RouteMetadata,
}

fn route_handler(graph: &Graph, event: &Request) -> Result<String, anyhow::Error> {
    let params = event
        .payload::<RouteParams>()?
        .ok_or_else(|| anyhow!("Missing route params"))?;

    let with_traversal = params.with_traversal.unwrap_or(false);

    let (route, traversal, meta) = graph
        .calculate_route(
            params.start.into(),
            params.end.into(),
            with_traversal,
            params.cost_model,
            params.heuristic_weight,
        )
        .map_err(|e| {
            error!("Routing Error: {e}");
            e
        })?;

    let route = geojson::serialize_route_geom(&route).map_err(|e| {
        error!("Serialization Error: {e}");
        e
    })?;
    let traversal = traversal
        .map(|t| {
            geojson::serialize_traversal_geoms(&t).map_err(|e| {
                error!("Serialization Error: {e}");
                e
            })
        })
        .transpose()?;

    let response = RouteResponse {
        route,
        traversal,
        meta,
    };

    // TODO: vec -> string -> json::Value -> string ?
    Ok(serde_json::to_string(&response)?)
}

/// Mobile-optimized /navigate endpoint: lean response (no from/to/way IDs),
/// merged steps per way, total_distance + total_time_estimate in meta.
#[derive(Debug, Deserialize)]
struct NavigateParams {
    start: Location,
    end: Location,
    cost_model: Option<CostModel>,
    heuristic_weight: Option<Weight>,
}

fn navigate_handler(graph: &Graph, event: &Request) -> Result<String, anyhow::Error> {
    let params = event
        .payload::<NavigateParams>()?
        .ok_or_else(|| anyhow!("Missing navigate params"))?;

    let (route_segments, _, _) = graph
        .calculate_route(
            params.start.into(),
            params.end.into(),
            false, // never include traversal for mobile
            params.cost_model,
            params.heuristic_weight,
        )
        .map_err(|e| {
            error!("Routing Error: {e}");
            e
        })?;

    let response = navigation::serialize_navigation(&route_segments).map_err(|e| {
        error!("Serialization Error: {e}");
        e
    })?;

    Ok(serde_json::to_string(&response)?)
}
