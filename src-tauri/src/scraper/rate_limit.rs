use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, OwnedSemaphorePermit, Semaphore};

pub struct ProviderRateLimiter {
    next_request: Mutex<Instant>,
    interval: Duration,
    concurrency: Arc<Semaphore>,
}

impl ProviderRateLimiter {
    pub fn per_second(requests: u32, threads: u32) -> Self {
        Self {
            next_request: Mutex::new(Instant::now()),
            interval: Duration::from_secs_f64(1.0 / requests.max(1) as f64),
            concurrency: Arc::new(Semaphore::new(threads.clamp(1, 32) as usize)),
        }
    }

    pub async fn acquire(&self) -> OwnedSemaphorePermit {
        let permit = self
            .concurrency
            .clone()
            .acquire_owned()
            .await
            .expect("rate limiter semaphore closed");
        let mut next = self.next_request.lock().await;
        let now = Instant::now();
        if *next > now {
            tokio::time::sleep(*next - now).await;
        }
        *next = Instant::now() + self.interval;
        permit
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
    format_response_error(provider, status.as_u16(), retry_after.as_deref(), &body)
}

fn format_response_error(
    provider: &str,
    status: u16,
    retry_after: Option<&str>,
    body: &str,
) -> String {
    match status {
        401 | 403 => format!("{provider} 鉴权失败，请检查账号或 API 凭据"),
        426 if provider == "ScreenScraper" => {
            "ScreenScraper 已拒绝当前客户端版本，请升级应用后重试".to_string()
        }
        429 => format!(
            "{provider} 已限流{}",
            retry_after.map_or(String::new(), |value| format!("，{} 秒后重试", value))
        ),
        430 if provider == "ScreenScraper" => "ScreenScraper 当日抓取额度已用尽".to_string(),
        431 if provider == "ScreenScraper" => {
            "ScreenScraper 当日未识别 ROM 查询额度已用尽".to_string()
        }
        _ => {
            let normalized = body.split_whitespace().collect::<Vec<_>>().join(" ");
            let sensitive = [
                "password",
                "devpassword",
                "sspassword",
                "devid=",
                "ssid=",
                "http://",
                "https://",
            ]
            .iter()
            .any(|marker| normalized.to_ascii_lowercase().contains(marker));
            let detail = if normalized.is_empty() || sensitive {
                "服务端拒绝了请求".to_string()
            } else {
                normalized.chars().take(200).collect::<String>()
            };
            format!("{provider} HTTP {status}: {detail}")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::format_response_error;

    #[test]
    fn screenscraper_status_codes_are_actionable() {
        assert!(format_response_error("ScreenScraper", 401, None, "").contains("鉴权失败"));
        assert!(format_response_error("ScreenScraper", 426, None, "").contains("升级应用"));
        assert!(format_response_error("ScreenScraper", 429, Some("12"), "").contains("12 秒后重试"));
        assert!(format_response_error("ScreenScraper", 430, None, "").contains("抓取额度"));
        assert!(format_response_error("ScreenScraper", 431, None, "").contains("未识别 ROM"));
    }

    #[test]
    fn error_body_never_echoes_credential_urls() {
        let error = format_response_error(
            "ScreenScraper",
            500,
            None,
            "failed https://api.example/?devid=secret&sspassword=secret",
        );
        assert!(!error.contains("secret"));
        assert!(!error.contains("https://"));
    }
}
