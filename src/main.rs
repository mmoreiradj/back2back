use actix_web::{App, HttpServer};
use tracing::{error, info};
use tracing_actix_web::TracingLogger;
mod health;

#[derive(Debug)]
enum ServerError {
    #[allow(dead_code)]
    InvalidBindAddress(std::io::Error),
    #[allow(dead_code)]
    ServerStartFailed(std::io::Error),
    #[allow(dead_code)]
    ConfigurationError(std::env::VarError),
    #[allow(dead_code)]
    UnexpectedError,
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

    let host = std::env::var("SERVER_HOST").map_err(|e| {
        error!("Failed to read SERVER_HOST: {}", e);
        ServerError::ConfigurationError(e)
    })?;
    let port = std::env::var("SERVER_PORT").map_err(|e| {
        error!("Failed to read SERVER_PORT: {}", e);
        ServerError::ConfigurationError(e)
    })?;
    let addr = format!("{}:{}", host, port);

    let server = HttpServer::new(|| {
        App::new()
            .wrap(TracingLogger::default())
            .service(health::liveness)
            .service(health::readiness)
    })
    .bind(addr.clone())
    .map_err(|e| {
        error!("Failed to bind to address: {}", e);
        ServerError::InvalidBindAddress(e)
    })?
    .shutdown_timeout(5);

    info!("Server listening on {}", addr);

    server.run().await.map_err(|e| {
        error!("Server failed to start: {}", e);
        ServerError::ServerStartFailed(e)
    })?;

    Ok(())
}
