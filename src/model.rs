use serde::{Deserialize, Serialize};

pub const PAYMENT_QUEUE: &str = "payment_queue";
pub const REDIS_KEY_PAYMENT_DEFAULT: &str = "payments_default";
pub const REDIS_KEY_PAYMENT_FALLBACK: &str = "payments_fallback";
fn default_requested_at() -> String {
    chrono::Utc::now().to_rfc3339()
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Payment {
    #[serde(rename = "correlationId")]
    pub correlation_id: String,
    pub amount: f64,
    #[serde(default = "default_requested_at", rename = "requestedAt")]
    pub requested_at: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct HealthCheck {
    pub failing: bool,
    #[serde(rename = "minResponseTime")]
    pub min_response_time: u32,
    #[serde(default)]
    pub failure_start_time: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct HealthCheckResult {
    pub default_health_check: HealthCheck,
    pub fallback_health_check: HealthCheck,
}

#[derive(Serialize)]
pub struct Summary {
    #[serde(rename = "totalRequests")]
    pub total_requests: usize,
    #[serde(rename = "totalAmount")]
    pub total_amount: f64,
}

#[derive(Serialize)]
pub struct PaymentsSummary {
    pub default: Summary,
    pub fallback: Summary,
}

#[derive(Clone)]
pub struct App {
    pub http_client: reqwest::Client,
    pub redis_pool: deadpool_redis::Pool,
    pub payment_endpoint: String,
    pub payment_fallback_endpoint: String,
}

impl App {
    pub fn new(
        redis_pool: deadpool_redis::Pool,
        payment_endpoint: String,
        payment_fallback_endpoint: String,
    ) -> Self {
        App {
            http_client: reqwest::Client::new(),
            redis_pool,
            payment_endpoint,
            payment_fallback_endpoint,
        }
    }
}
