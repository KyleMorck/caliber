use reqwest::Client;
use std::sync::LazyLock;
use std::time::Duration;

static HTTP_CLIENT: LazyLock<Option<Client>> = LazyLock::new(|| {
    Client::builder()
        .timeout(Duration::from_secs(30))
        .user_agent("Caliber/1.0")
        .build()
        .ok()
});

pub async fn fetch_calendar(url: &str) -> Result<String, String> {
    let client = HTTP_CLIENT
        .as_ref()
        .ok_or_else(|| "HTTP client unavailable (TLS initialization failed)".to_string())?;

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch calendar: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Calendar fetch failed with status: {}",
            response.status()
        ));
    }

    response
        .text()
        .await
        .map_err(|e| format!("Failed to read calendar response: {}", e))
}
