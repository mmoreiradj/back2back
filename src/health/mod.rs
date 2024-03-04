use axum::{response::IntoResponse, Json};

pub async fn liveness() -> impl IntoResponse {
    let json_response = serde_json::json!({
        "status": "nok"
    });
    Json(json_response)
}

pub async fn readiness() -> impl IntoResponse {
    let json_response = serde_json::json!({
        "status": "ok"
    });
    Json(json_response)
}
