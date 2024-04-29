use axum::{
    extract,
    http::{Method, StatusCode},
    routing::{get, post},
    Router,
};
use dotenvy::dotenv;
use serde::Deserialize;
use tokio::fs;
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

use rusty_router::osm::{Graph, Location};

#[tokio::main]
async fn main() {
    dotenv().expect(".env file not found");

    tracing_subscriber::fmt::init();
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_headers(Any)
        .allow_origin(Any);

    let trace = TraceLayer::new_for_http();
    let app = Router::new()
        .route("/heartbeat", get(|| async { "OK" }))
        .route(
            "/graph",
            get(traverse_handler),
        )
        // applies a collection of Tower Layers to all of this Router's routes
        .layer(ServiceBuilder::new().layer(trace).layer(cors));

    // run app w/ hyper, bind to 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!("listening on 3000...");
    axum::serve(listener, app).await.unwrap();
}

#[derive(Debug, Deserialize)]
struct TraversalParams {
    lat: f64,
    lon: f64,
    depth: u8,
}

async fn traverse_handler(query: extract::Query<TraversalParams>) -> Result<String, StatusCode> {
    println!("traverse:: traversing from (lat, lon): ({}, {}) to depth {}", query.lat, query.lon, query.depth);
    let starting_location = Location {
        lat: query.lat,
        lon: query.lon,
    };

    let graph = Graph::new().unwrap();
    let neighbors = graph
        .guess_neighbors(starting_location)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    println!("traverse:: found neighbors: {neighbors:#?}");

    // TODO: recurse until depth is 0

    // Ok(format!("{:#?}", neighbors))

    fs::read_to_string("./static_responses/multi_bushwick_greenpoint.geojson")
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}
