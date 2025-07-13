use crate::model::HealthCheckResult;
use crate::App;
use deadpool_redis::redis::AsyncCommands;
use serde_json;

pub async fn select_endpoint(app: &App) -> Result<String, String> {
    let Ok(mut conn) = app.redis_pool.get().await else {
        return Err("Redis connection error".to_string());
    };

    let Ok(result) = conn.get::<_, String>("health_check_result").await else {
        return Err("Failed to retrieve health check".to_string());
    };
    let health: HealthCheckResult = serde_json::from_str(&result).unwrap();

    if !health.default_health_check.failing {
        Ok(app.payment_endpoint.clone())
    } else if !health.fallback_health_check.failing {
        Ok(app.payment_fallback_endpoint.clone())
    } else {
        Err("Both endpoints are failing".to_string())
    }
}
