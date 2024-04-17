use axum::{
    extract,
    http::{Method, StatusCode},
    routing::{get, post},
    Router,
};
use serde::Deserialize;
use tokio::fs;
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use dotenvy::dotenv;
use rusty_router::db;

#[tokio::main]
async fn main() {
    dotenv().expect(".env file not found");

    match db::get_neighbors("0".into()).await {
        Ok(n) => {
            println!("neighbors: {:?}", n)
        }
        Err(e) => {
            println!("get neighbors {:?}", e);
        }
    }

    tracing_subscriber::fmt::init();
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_headers(Any)
        .allow_origin(Any);

    let trace = TraceLayer::new_for_http();
    // TODO: support single GET /v2/directions/:profile spec
    let app = Router::new()
        .route("/heartbeat", get(|| async { "OK" }))
        .route(
            "/v2/directions/:profile/*result_type",
            post(directions_handler),
        )
        // applies a collection of Tower Layers to all of this Router's routes
        .layer(ServiceBuilder::new().layer(trace).layer(cors));

    // run app w/ hyper, bind to 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("listening on 3000...");
    axum::serve(listener, app).await.unwrap();
}

#[derive(Debug, Deserialize)]
struct AlternativeRoutes {
    target_count: u8,
    share_factor: f32,
    weight_factor: f32,
}

#[derive(Debug, Deserialize)]
struct ORSDirectionsRequestBody {
    coordinates: Vec<Vec<f64>>,
    elevation: bool,
    instructions_format: String,
    extra_info: Vec<String>,
    language: String,
    units: String,
    preference: String,
    alternative_routes: Option<AlternativeRoutes>,
}

async fn directions_handler(
    extract::Path((profile, result_type)): extract::Path<(String, String)>,
    extract::Json(payload): extract::Json<ORSDirectionsRequestBody>,
) -> Result<String, StatusCode> {
    println!("request of {profile} in {result_type} received");
    println!("{payload:?}");

    let response_file: &str = match payload.alternative_routes {
        Some(_) => "./static_responses/multi_bushwick_greenpoint.geojson",
        None => "./static_responses/single_bushwick_greenpoint.geojson",
    };

    fs::read_to_string(response_file)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}
