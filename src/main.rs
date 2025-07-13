mod client;
mod handlers;
mod model;
mod workers;

use std::env;

use crate::handlers::{create_payment, get_payments_summary, reset_handler};
use crate::model::App;
use crate::workers::health_check_worker::health_check_worker;
use crate::workers::payment_worker::payment_worker;
use axum::routing::post;
use axum::{Router, routing::get};
use deadpool_redis::{Config, Runtime};
use log::info;

#[tokio::main]
async fn main() {
    if env::var_os("RUST_LOG").is_none() {
        unsafe {
            env::set_var("RUST_LOG", "info");
        }
    }
    env_logger::init();

    let port = env::var("PORT").unwrap_or("9999".to_string());
    let redis_url = env::var("REDIS_URL").unwrap_or("redis://localhost:6379".to_string());
    let cfg = Config::from_url(redis_url);
    let redis_pool = cfg.create_pool(Some(Runtime::Tokio1)).unwrap();

    let payment_endpoint =
        env::var("PAYMENT_ENDPOINT").unwrap_or("http://localhost:8001".to_string());
    let payment_fallback_endpoint =
        env::var("PAYMENT_FALLBACK_ENDPOINT").unwrap_or("http://localhost:8002".to_string());

    let app_state = App::new(redis_pool, payment_endpoint, payment_fallback_endpoint);

    let worker_app = app_state.clone();
    tokio::spawn(async move {
        health_check_worker(worker_app).await;
    });

    let payment_worker_app = app_state.clone();
    tokio::spawn(async move {
        payment_worker(payment_worker_app).await;
    });

    let app = Router::new()
        .route("/payments", post(create_payment::handle))
        .route("/payments-summary", get(get_payments_summary::handle))
        .route("/purge-payments", post(reset_handler::handle))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .unwrap();

    info!("🦀backend running at http://localhost:{}/", port);
    axum::serve(listener, app).await.unwrap();
}
