use axum::{
    extract::Path,
    http::{Method, StatusCode},
    routing::{get, post},
    Router,
};
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

async fn directions_handler(
    Path((profile, result_type)): Path<(String, String)>,
) -> Result<String, StatusCode> {
    println!("request of {profile} in {result_type} received");
    if let Ok(contents) = fs::read_to_string("./static_responses/single_route.geojson").await {
        Ok(contents)
    } else {
        Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}
