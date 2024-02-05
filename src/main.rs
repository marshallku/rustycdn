use axum::{
    body::Body,
    extract::{Path, Request},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Router,
};
use std::env;
use tower_http::trace::{self, TraceLayer};
use tracing::{info, Level, Span};

async fn handle_files_request(Path(path): Path<String>) -> impl IntoResponse {
    (StatusCode::OK, format!("Hello {}!", path))
}

async fn handle_image_request(Path(path): Path<String>) -> impl IntoResponse {
    (StatusCode::OK, format!("Hello {}!", path))
}

fn trace_layer_on_request(request: &Request<Body>, _span: &Span) {
    let user_agent = request
        .headers()
        .get("user-agent")
        .map_or("<no user-agent>", |h| {
            h.to_str().unwrap_or("<invalid utf8>")
        });

    let referer = request
        .headers()
        .get("referer")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("<no referer>");

    let ip_address = request
        .headers()
        .get("x-forwarded-for")
        .or_else(|| request.headers().get("x-real-ip"))
        .and_then(|value| value.to_str().ok())
        .unwrap_or("<no ip>");

    tracing::info!(
        "User-Agent: {:?} Referrer: {:?} IP: {:?}",
        user_agent,
        referer,
        ip_address
    )
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    let app = Router::new()
        .route("/files/*path", get(handle_files_request))
        .route("/images/*path", get(handle_image_request))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO))
                .on_request(trace_layer_on_request),
        );

    let bind_address = env::var("BIND_ADDRESS").unwrap_or_else(|_| String::from("127.0.0.1"));
    let addr = format!("{}:41890", bind_address);
    let listener = tokio::net::TcpListener::bind(addr.as_str()).await.unwrap();
    info!("Server running at http://{}", addr);
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
