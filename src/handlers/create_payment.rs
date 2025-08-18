use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use axum::response::IntoResponse;
use crate::model::{App, Payment};

pub async fn handle(
    State(app): State<App>,
    Json(payment): Json<Payment>,
) -> Result<impl IntoResponse, StatusCode> {
    app.channel_tx.send(payment).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(StatusCode::CREATED)
}
