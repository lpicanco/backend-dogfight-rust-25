use crate::client;
use crate::model::{Payment, REDIS_KEY_PAYMENT_DEFAULT, REDIS_KEY_PAYMENT_FALLBACK};
use crate::workers::endpoint_selector::select_endpoint;
use crate::App;
use deadpool_redis::{redis::AsyncCommands, Connection};
use log::{debug, error, warn};
use serde_json;
use std::time::Duration;
use async_channel::Receiver;
use chrono::SubsecRound;
use tokio::time::sleep;

const WORKER_COUNT: usize = 4;

pub async fn payment_worker(app: App) {    
    for i in 0..WORKER_COUNT {
        let worker_app = app.clone();
        let worker_rx = app.channel_rx.clone();
        tokio::spawn(async move {
            payment_processor_worker(worker_app, worker_rx, i).await;
        });
    }

    ()
}

async fn payment_processor_worker(app: App, rx: Receiver<Payment>, worker_id: usize) {
    debug!("Payment processor worker {} started", worker_id);

    let mut conn = app.redis_pool.get().await.unwrap();
    loop {
        let Ok(mut payment) = rx.recv().await else {
            error!("Payment processor worker {} shutting down", worker_id);
            break;
        };
        payment.requested_at = chrono::Utc::now().round_subsecs(0).to_rfc3339();

        loop {
            match process_payment(&mut conn, &app, &payment).await {
                Ok(_) => {
                    break;
                }
                Err(e) => {
                    sleep(Duration::from_millis(200)).await;
                    continue;
                }
            }
        }
    }
}

async fn process_payment(conn: &mut Connection, app: &App, payment: &Payment) -> Result<(), String> {
    let endpoint = select_endpoint(&app).await?;
    let result = client::create_payment(&app, &endpoint, payment).await;
    if let Err(e) = result {
        if e.status() == Some(reqwest::StatusCode::UNPROCESSABLE_ENTITY) {
            warn!("Payment already exists: {}", e);
            return Ok(());
        }

        if e.status() != Some(reqwest::StatusCode::INTERNAL_SERVER_ERROR) {
            error!("Failed to create payment: {}", e);
        }

        return Err(e.to_string());
    }
    set_payment_in_redis(conn, &payment, &endpoint, &app).await?;

    Ok(())
}

async fn set_payment_in_redis(
    conn: &mut Connection,
    payment: &Payment,
    endpoint: &str,
    app: &App,
) -> Result<(), String> {
    let key = if endpoint == app.payment_endpoint {
        REDIS_KEY_PAYMENT_DEFAULT
    } else {
        REDIS_KEY_PAYMENT_FALLBACK
    };
    let serialized = serde_json::to_string(&payment).unwrap();
    conn.hset::<&str, &str, String, ()>(key, &payment.correlation_id, serialized)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}
