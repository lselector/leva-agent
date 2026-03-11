/// Perplexity API fallback for web research (no browser required).
use anyhow::{bail, Result};
use common::config;

const PPLX_URL: &str = "https://api.perplexity.ai/chat/completions";
const PPLX_MODEL: &str = "sonar";

pub async fn web_research(query: &str) -> Result<String> {
    let api_key = &config::get().perplexity_api_key;
    if api_key.is_empty() {
        bail!("PERPLEXITY_API_KEY not set in .env");
    }
    let body = serde_json::json!({
        "model": PPLX_MODEL,
        "messages": [{"role": "user", "content": query}],
    });
    let resp = reqwest::Client::new()
        .post(PPLX_URL)
        .bearer_auth(api_key)
        .json(&body)
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .await?;
    let data: serde_json::Value = resp.json().await?;
    Ok(data["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("")
        .to_string())
}
