use actix_web::{get, HttpResponse, Responder};

#[get("/health/live")]
pub async fn liveness() -> impl Responder {
    let json_response = serde_json::json!({
        "status": "ok"
    });
    HttpResponse::Ok().json(json_response)
}

#[get("/health/ready")]
pub async fn readiness() -> impl Responder {
    let json_response = serde_json::json!({
        "status": "ok"
    });
    HttpResponse::Ok().json(json_response)
}
