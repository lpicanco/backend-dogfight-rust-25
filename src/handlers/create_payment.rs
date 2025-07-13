use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use axum::response::IntoResponse;
use crate::model::{App, Payment, PAYMENT_QUEUE};

use deadpool_redis::redis::AsyncCommands;
use serde_json;

pub async fn handle(
    State(app): State<App>,
    Json(payment): Json<Payment>,
) -> Result<impl IntoResponse, StatusCode> {
    let mut conn = app.redis_pool.get().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let json = serde_json::to_string(&payment).unwrap();
    conn.lpush::<&str, String, i64>(PAYMENT_QUEUE, json).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::CREATED)
}
