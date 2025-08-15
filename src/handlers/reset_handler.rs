use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use deadpool_redis::redis::AsyncCommands;
use crate::model::{App, REDIS_KEY_PAYMENT_DEFAULT, REDIS_KEY_PAYMENT_FALLBACK};
use crate::client;

pub async fn handle(
    State(app): State<App>,
) -> Result<impl IntoResponse, StatusCode> {

    let mut conn = app.redis_pool.get().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let _: () = conn.del(REDIS_KEY_PAYMENT_DEFAULT).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let _: () = conn.del(REDIS_KEY_PAYMENT_FALLBACK).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    client::purge(&app).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(())
}
