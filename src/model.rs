use serde::{Deserialize, Serialize};

pub const REDIS_KEY_PAYMENT_DEFAULT: &str = "payments_default";
pub const REDIS_KEY_PAYMENT_FALLBACK: &str = "payments_fallback";

#[derive(Deserialize, Serialize)]
pub struct Payment {
    #[serde(rename = "correlationId")]
    pub correlation_id: String,
    pub amount: f64,
    #[serde(default, rename = "requestedAt")]
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
    pub channel_tx: async_channel::Sender<Payment>,
    pub channel_rx: async_channel::Receiver<Payment>,
}

impl App {
    pub fn new(
        redis_pool: deadpool_redis::Pool,
        payment_endpoint: String,
        payment_fallback_endpoint: String,
    ) -> Self {
        let (tx,rx) = async_channel::unbounded();
        App {
            http_client: reqwest::Client::new(),
            redis_pool,
            payment_endpoint,
            payment_fallback_endpoint,
            channel_tx: tx,
            channel_rx: rx,
        }
    }
}
