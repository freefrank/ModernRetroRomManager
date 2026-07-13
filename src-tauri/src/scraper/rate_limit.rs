use std::time::{Duration, Instant};
use tokio::sync::Mutex;

pub struct ProviderRateLimiter {
    next_request: Mutex<Instant>,
    interval: Duration,
}

impl ProviderRateLimiter {
    pub fn per_second(requests: u32) -> Self {
        Self {
            next_request: Mutex::new(Instant::now()),
            interval: Duration::from_secs_f64(1.0 / requests.max(1) as f64),
        }
    }

    pub async fn acquire(&self) {
        let mut next = self.next_request.lock().await;
        let now = Instant::now();
        if *next > now {
            tokio::time::sleep(*next - now).await;
        }
        *next = Instant::now() + self.interval;
    }
}

pub async fn response_error(provider: &str, response: reqwest::Response) -> String {
    let status = response.status();
    let retry_after = response
        .headers()
        .get(reqwest::header::RETRY_AFTER)
        .and_then(|value| value.to_str().ok())
        .map(str::to_string);
    let body = response.text().await.unwrap_or_default();
    if status.as_u16() == 429 {
        format!(
            "{provider} 已限流{}",
            retry_after.map_or(String::new(), |value| format!("，{} 秒后重试", value))
        )
    } else {
        format!(
            "{provider} HTTP {status}: {}",
            body.chars().take(200).collect::<String>()
        )
    }
}
