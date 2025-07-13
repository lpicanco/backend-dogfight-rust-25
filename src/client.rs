use crate::model::{App, HealthCheck, HealthCheckResult, Payment};
use log::debug;

pub async fn health_check(app: &App) -> Result<HealthCheckResult, reqwest::Error> {
    let default_health_check = health_check_endpoint(app, &app.payment_endpoint).await?;
    let fallback_health_check = health_check_endpoint(app, &app.payment_fallback_endpoint).await?;

    Ok(HealthCheckResult {
        default_health_check,
        fallback_health_check,
    })
}

async fn health_check_endpoint(app: &App, endpoint: &str) -> Result<HealthCheck, reqwest::Error> {
    app.http_client
        .get(format!("{}/payments/service-health", endpoint))
        .send()
        .await?
        .json::<HealthCheck>()
        .await
}

pub async fn create_payment(
    app: &App,
    endpoint: &str,
    payment: Payment,
) -> Result<(), reqwest::Error> {
    debug!(
        "correlationId: {} and endpoint {}",
        payment.correlation_id, endpoint
    );
    app.http_client
        .post(format!("{}/payments", endpoint))
        .json(&payment)
        .send()
        .await?
        .error_for_status()?;
    Ok(())
}

pub async fn purge(app: &App) -> Result<(), reqwest::Error> {
    purge_endpoint(app, &app.payment_endpoint).await?;
    purge_endpoint(app, &app.payment_fallback_endpoint).await?;

    Ok(())
}

async fn purge_endpoint(app: &App, endpoint: &str) -> Result<(), reqwest::Error> {
    app.http_client
        .post(format!("{}/admin/purge-payments", endpoint))
        .header("X-Rinha-Token", "123")
        .send()
        .await?
        .error_for_status()?;
    Ok(())
}
