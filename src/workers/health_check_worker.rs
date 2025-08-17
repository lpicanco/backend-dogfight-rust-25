use crate::client;
use crate::model::HealthCheckResult;
use crate::App;
use deadpool_redis::redis;
use deadpool_redis::redis::AsyncCommands;
use log::{debug, error, info};
use serde_json;
use std::time::Duration;
use tokio::time::sleep;

pub async fn health_check_worker(app: App) {
    let lock_key = "health_check_lock";
    let result_key = "health_check_result";
    const LOCK_TIMEOUT: u64 = 5;

    loop {
        let Ok(mut conn) = app.redis_pool.get().await else {
            error!("Failed to get Redis connection. Retrying...");
            sleep(Duration::from_millis(50)).await;
            continue;
        };

        let acquired: bool = redis::cmd("SET")
            .arg(lock_key)
            .arg("locked")
            .arg("NX")
            .arg("EX")
            .arg(LOCK_TIMEOUT)
            .query_async(&mut conn)
            .await
            .map(|res: Option<String>| res.is_some())
            .unwrap_or_else(|e| {
                info!("Lock not acquired, retrying...: {}", e);
                false
            });

        if !acquired {
            sleep(Duration::from_secs(1)).await;
            continue;
        }

        debug!("Lock acquired, performing health check...");
        let health = match client::health_check(&app).await {
            Ok(health) => health,
            Err(e) => {
                error!("Health check failed: {}", e);
                sleep(Duration::from_secs(1)).await;
                continue;
            }
        };

        let previous_health = get_previous_health_check(&mut conn, result_key).await;
        let updated_health = update_health_check_with_failure_tracking(health, previous_health);

        let health_json = serde_json::to_string(&updated_health).unwrap();
        if let Err(e) = conn.set::<_, _, ()>(result_key, health_json).await {
            error!("Failed to set result: {}", e);
        }
        sleep(Duration::from_secs(5)).await;
    }
}

async fn get_previous_health_check(
    conn: &mut deadpool_redis::Connection,
    result_key: &str,
) -> Option<HealthCheckResult> {
    if let Ok(result) = conn.get::<_, String>(result_key).await {
        serde_json::from_str(&result).ok()
    } else {
        None
    }
}

fn update_health_check_with_failure_tracking(
    mut current: HealthCheckResult,
    previous: Option<HealthCheckResult>,
) -> HealthCheckResult {
    let now = chrono::Utc::now();

    current.default_health_check.failure_start_time = if current.default_health_check.failing {
        match previous {
            Some(ref prev) if prev.default_health_check.failing => {
                prev.default_health_check.failure_start_time
            }
            _ => Some(now),
        }
    } else {
        None
    };

    current.fallback_health_check.failure_start_time = if current.fallback_health_check.failing {
        match previous {
            Some(ref prev) if prev.fallback_health_check.failing => {
                prev.fallback_health_check.failure_start_time
            }
            _ => Some(now),
        }
    } else {
        None
    };
    current
}
