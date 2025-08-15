mod client;
mod handlers;
mod model;
mod workers;

use std::env;
use std::os::unix::fs::PermissionsExt;

use crate::handlers::{create_payment, get_payments_summary, reset_handler};
use crate::model::App;
use crate::workers::health_check_worker::health_check_worker;
use crate::workers::payment_worker::payment_worker;
use axum::routing::post;
use axum::{Router, routing::get};
use deadpool_redis::{Config, Runtime};
use log::info;
use tokio::net::UnixListener;
use tokio::signal;

#[tokio::main]
async fn main() {
    if env::var_os("RUST_LOG").is_none() {
        unsafe {
            env::set_var("RUST_LOG", "info");
        }
    }
    env_logger::init();

    let uds_path = env::var("UDS_PATH").unwrap_or("/tmp/uds42".to_string());
    let redis_url = env::var("REDIS_URL").unwrap_or("redis://dev-server:6379".to_string());
    let redis_pool = Config::from_url(redis_url)
        .builder().unwrap()
        .max_size(20)
        .runtime(Runtime::Tokio1)
        .build().unwrap();

    let payment_endpoint =
        env::var("PAYMENT_ENDPOINT").unwrap_or("http://dev-server:8001".to_string());
    let payment_fallback_endpoint =
        env::var("PAYMENT_FALLBACK_ENDPOINT").unwrap_or("http://dev-server:8002".to_string());

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

    std::fs::remove_file(uds_path.clone()).ok();
    let listener = UnixListener::bind(uds_path.clone()).unwrap();

    let mut perms = std::fs::metadata(&uds_path).unwrap().permissions();
    perms.set_mode(0o666);
    std::fs::set_permissions(&uds_path, perms).unwrap();

    info!("🦀backend running at http://localhost:{}/", uds_path);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
