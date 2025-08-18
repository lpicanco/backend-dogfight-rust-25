use axum::{extract::{Query, State}, http::StatusCode, response::IntoResponse};
use crate::model::{App, Payment, REDIS_KEY_PAYMENT_DEFAULT, REDIS_KEY_PAYMENT_FALLBACK};
use chrono::{DateTime, Utc};
use serde_json;
use std::collections::HashMap;
use deadpool_redis::redis::AsyncCommands;

pub async fn handle(
    State(app): State<App>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<impl IntoResponse, StatusCode> {
    let from = parse_date(params.get("from"), "2025-01-01T00:00:00Z")?;
    let to = parse_date(params.get("to"), "2025-12-01T00:00:00Z")?;

    let mut conn = app
        .redis_pool
        .get()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let payments_default: HashMap<String, String> = conn
        .hgetall(REDIS_KEY_PAYMENT_DEFAULT)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let payments_fallback: HashMap<String, String> = conn
        .hgetall(REDIS_KEY_PAYMENT_FALLBACK)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // TODO: refactor this code.
    let summarize = |map: HashMap<String, String>| -> (usize, f64) {
        map.values()
            .filter_map(|json| serde_json::from_str::<Payment>(json).ok())
            .filter(|p| {
                DateTime::parse_from_rfc3339(&p.requested_at)
                    .map(|dt| {
                        let dt = dt.with_timezone(&Utc);
                        dt >= from && dt <= to
                    })
                    .unwrap_or(false)
            })
            .fold((0, 0.0), |mut acc, p| {
                acc.0 += 1; // total_requests
                acc.1 += p.amount; // total_amount
                acc
            })
    };

    let (def_requests, def_amount) = summarize(payments_default);
    let (fbk_requests, fbk_amount) = summarize(payments_fallback);

    let response_body = format!(
        r#"{{"default":{{"totalRequests":{},"totalAmount":{:.2}}},"fallback":{{"totalRequests":{},"totalAmount":{:.2}}}}}"#,
        def_requests, def_amount, fbk_requests, fbk_amount
    );

    Ok((
        StatusCode::OK,
        [("content-type", "application/json")],
        response_body
    ))
}

fn parse_date(param: Option<&String>, default: &str) -> Result<DateTime<Utc>, StatusCode> {
    let param_value = param.map(|s| s.as_str()).unwrap_or(default);
    let date = DateTime::parse_from_rfc3339(param_value)
        .map_err(|_| StatusCode::BAD_REQUEST)?
        .with_timezone(&Utc);
    Ok(date)
}