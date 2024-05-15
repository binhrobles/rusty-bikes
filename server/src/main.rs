use anyhow::anyhow;
use axum::{
    extract,
    http::{Method, StatusCode},
    routing::get,
    Router,
};
use dotenvy::dotenv;
use geo::Point;
use serde::Deserialize;
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing::{info, error};

use rusty_router::geojson;
use rusty_router::osm::Graph;

#[tokio::main]
async fn main() {
    dotenv().expect(".env file not found");

    tracing_subscriber::fmt()
        .compact()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let trace = TraceLayer::new_for_http();

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_headers(Any)
        .allow_origin(Any);

    let app = Router::new()
        .route("/heartbeat", get(|| async { "OK" }))
        // .route("/traverse", get(traverse_handler))
        .route("/route", get(route_handler))
        // applies a collection of Tower Layers to all of this Router's routes
        .layer(ServiceBuilder::new().layer(trace).layer(cors));

    // run app w/ hyper, bind to 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    info!("listening on 3000!");
    axum::serve(listener, app).await.unwrap();
}

#[derive(Debug, Deserialize)]
struct TraversalParams {
    lat: f64,
    lon: f64,
    depth: usize,
}

// TODO: figure out how to translate these traversals to geojson nicely!
// async fn traverse_handler(query: extract::Query<TraversalParams>) -> Result<String, StatusCode> {
//     info!(
//         "traverse:: traversing from (lat, lon): ({}, {}) to depth {}",
//         query.lat, query.lon, query.depth
//     );
//     let starting_coord = Point::new(query.lon, query.lat);

//     let graph = Graph::new().unwrap();
//     let traversal = graph
//         .traverse_from(starting_coord, query.depth)
//         .map_err(|e| {
//             error!("Routing Error: {e}");
//             StatusCode::INTERNAL_SERVER_ERROR
//         })?;

//     geojson::aggregate_traversal_geoms(&traversal).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
// }

#[derive(Debug, Deserialize)]
struct RouteParams {
    start: String,
    end: String,
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

async fn route_handler(query: extract::Query<RouteParams>) -> Result<String, StatusCode> {
    let start = parse_point(&query.start).map_err(|_| StatusCode::BAD_REQUEST)?;
    let end = parse_point(&query.end).map_err(|_| StatusCode::BAD_REQUEST)?;

    info!("{:?} to {:?}", start, end);

    let graph = Graph::new().unwrap();

    let route = graph
        .route_between(start, end)
        .map_err(|e| {
            error!("Routing Error: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    geojson::route_geom(&route).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}
