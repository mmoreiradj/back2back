use tokio::net::TcpListener;

use axum::{routing::get, Router};
mod health;

#[tokio::main]
async fn main() {
    let router = Router::new()
        .route("/health/live", get(health::liveness))
        .route("/health/ready", get(health::readiness));

    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, router).await.unwrap();
}
