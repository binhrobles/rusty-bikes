use axum::{
    extract,
    http::{Method, StatusCode},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use tokio::fs;
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

#[tokio::main]
async fn main() {
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
struct DirectionsRequestBody {
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
    extract::Json(payload): extract::Json<DirectionsRequestBody>,
) -> Result<String, StatusCode> {
    println!("request of {profile} in {result_type} received");
    println!("{payload:?}");

    let response_file: &str = match payload.alternative_routes {
        Some(_) => "./static_responses/two_routes.geojson",
        None => "./static_responses/single_route.geojson",
    };

    if let Ok(contents) = fs::read_to_string(response_file).await {
        Ok(contents)
    } else {
        Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}
