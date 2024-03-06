use tokio::net::TcpListener;

use axum::{routing::get, Router};
use tracing::{error, info};
mod health;

#[derive(Debug)]
enum ServerError {
    #[allow(dead_code)]
    InvalidBindAddress(std::io::Error),
    #[allow(dead_code)]
    ServerStartFailed(std::io::Error),
    #[allow(dead_code)]
    ConfigurationError(std::env::VarError),
}

#[tokio::main]
async fn main() -> Result<(), ServerError> {
    tracing_subscriber::fmt::init();

    // display the name and version of the application
    info!(
        "Starting {} version {}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );

    let router = Router::new()
        .route("/health/live", get(health::liveness))
        .route("/health/ready", get(health::readiness));

    let host = std::env::var("SERVER_HOST").map_err(|e| {
        error!("Failed to read SERVER_HOST: {}", e);
        ServerError::ConfigurationError(e)
    })?;
    let port = std::env::var("SERVER_PORT").map_err(|e| {
        error!("Failed to read SERVER_PORT: {}", e);
        ServerError::ConfigurationError(e)
    })?;
    let addr = format!("{}:{}", host, port);

    let listener = TcpListener::bind(addr.clone()).await.map_err(|e| {
        error!("Failed to bind to address {}: {}", addr, e);
        ServerError::InvalidBindAddress(e)
    })?;

    info!("Server listening on {}", addr);

    if let Err(error) = axum::serve(listener, router).await {
        error!("Server failed to start: {}", error);
        Err(ServerError::ServerStartFailed(error))?
    }

    Ok(())
}
