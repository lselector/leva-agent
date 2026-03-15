/// CDP browser client — connects to Chrome via WebSocket on port 9222.
use anyhow::{bail, Result};
use serde_json::{json, Value};
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};
use futures::{SinkExt, StreamExt};

const CDP_HTTP: &str = "http://127.0.0.1:9222";
static MSG_ID: AtomicU64 = AtomicU64::new(1);

// ---------------------------------------------------------------
// Tab info returned by /json endpoint

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TabInfo {
    pub id: String,
    pub url: String,
    pub title: String,
    #[serde(rename = "webSocketDebuggerUrl", default)]
    pub ws_url: String,
}

/// List open Chrome tabs via the HTTP /json API.
pub async fn list_tabs() -> Result<Vec<TabInfo>> {
    let resp = reqwest::get(format!("{CDP_HTTP}/json")).await?;
    let tabs: Vec<TabInfo> = resp.json().await?;
    Ok(tabs)
}

/// Open a new Chrome tab and return its TabInfo.
pub async fn new_tab(url: &str) -> Result<TabInfo> {
    let resp = reqwest::Client::new()
        .put(format!("{CDP_HTTP}/json/new?{url}"))
        .send()
        .await?;
    let tab: TabInfo = resp.json().await?;
    Ok(tab)
}

/// Close a Chrome tab by ID.
pub async fn close_tab(tab_id: &str) -> Result<()> {
    reqwest::get(format!("{CDP_HTTP}/json/close/{tab_id}")).await?;
    Ok(())
}

// ---------------------------------------------------------------
// WebSocket CDP session

pub struct CdpSession {
    ws: WebSocketStream<MaybeTlsStream<TcpStream>>,
}

impl CdpSession {
    pub async fn connect(ws_url: &str) -> Result<Self> {
        let (ws, _) = connect_async(ws_url).await?;
        Ok(Self { ws })
    }

    /// Send a CDP command and wait for its response (10 s timeout).
    pub async fn send(&mut self, method: &str, params: Value) -> Result<Value> {
        let id = MSG_ID.fetch_add(1, Ordering::Relaxed);
        let msg = json!({"id": id, "method": method, "params": params});
        self.ws.send(Message::Text(msg.to_string().into())).await?;

        // Read until we get a message with matching id, with a hard timeout.
        let result = tokio::time::timeout(std::time::Duration::from_secs(10), async {
            loop {
                match self.ws.next().await {
                    Some(Ok(Message::Text(text))) => {
                        let v: Value = serde_json::from_str(&text).unwrap_or_default();
                        if v["id"].as_u64() == Some(id) {
                            return Ok::<Value, anyhow::Error>(v["result"].clone());
                        }
                    }
                    Some(Err(e)) => bail!("CDP WS error: {e}"),
                    None => bail!("CDP WS closed"),
                    _ => {}
                }
            }
        }).await;
        match result {
            Ok(inner) => inner,
            Err(_) => bail!("CDP command '{method}' timed out after 10s"),
        }
    }

    pub async fn navigate(&mut self, url: &str) -> Result<()> {
        self.send("Page.navigate", json!({"url": url})).await?;
        // Wait for load
        self.wait_load().await
    }

    pub async fn wait_load(&mut self) -> Result<()> {
        for _ in 0..30 {
            tokio::time::sleep(std::time::Duration::from_millis(300)).await;
            let result = self.evaluate("document.readyState").await?;
            if result.as_str() == Some("complete") {
                return Ok(());
            }
        }
        Ok(()) // timeout — continue anyway
    }

    /// Poll until `document.querySelector(selector)` returns non-null, or `max_ms` elapses.
    pub async fn wait_for_selector(&mut self, selector: &str, max_ms: u64) -> Result<()> {
        let script = format!(
            "document.querySelector({:?}) !== null",
            selector
        );
        let steps = (max_ms / 100).max(1);
        for _ in 0..steps {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            let result = self.evaluate(&script).await?;
            if result.as_bool() == Some(true) {
                return Ok(());
            }
        }
        Ok(()) // timeout — continue anyway
    }

    pub async fn evaluate(&mut self, script: &str) -> Result<Value> {
        let result = self.send("Runtime.evaluate", json!({
            "expression": script,
            "returnByValue": true,
        })).await?;
        Ok(result["result"]["value"].clone())
    }

    pub async fn screenshot(&mut self) -> Result<Vec<u8>> {
        let result = self.send("Page.captureScreenshot", json!({"format": "png"})).await?;
        let data = result["data"].as_str().unwrap_or("");
        Ok(base64::Engine::decode(&base64::engine::general_purpose::STANDARD, data)?)
    }
}

/// Open a new tab, return a connected CdpSession.
pub async fn open_session(url: &str) -> Result<(String, CdpSession)> {
    let tab = new_tab(url).await?;
    let ws_url = if tab.ws_url.is_empty() {
        format!("ws://127.0.0.1:9222/devtools/page/{}", tab.id)
    } else {
        tab.ws_url.clone()
    };
    let mut session = CdpSession::connect(&ws_url).await?;
    session.send("Page.enable", json!({})).await?;
    session.navigate(url).await?;
    Ok((tab.id, session))
}

/// Attach to an existing responsive tab whose URL starts with `url_prefix`.
/// Probes each candidate with a quick evaluate to skip zombie tabs.
/// Returns `None` if no responsive tab is found.
pub async fn attach_session(url_prefix: &str) -> Result<Option<(String, CdpSession)>> {
    let tabs = list_tabs().await?;
    let mut candidates: Vec<_> = tabs.into_iter()
        .filter(|t| t.url.starts_with(url_prefix) && !t.ws_url.is_empty())
        .collect();
    // Prefer fully-loaded tabs first.
    candidates.sort_by_key(|t| if t.title.contains("Feed") { 0usize } else { 1 });

    for t in candidates {
        if let Ok(mut session) = CdpSession::connect(&t.ws_url).await {
            // Quick probe: if evaluate responds within 3s, this tab is healthy.
            let probe = tokio::time::timeout(
                std::time::Duration::from_secs(3),
                session.evaluate("1")
            ).await;
            if probe.is_ok() {
                return Ok(Some((t.id, session)));
            }
            // Unresponsive zombie tab — close it to avoid accumulation.
            let _ = close_tab(&t.id).await;
        }
    }
    Ok(None)
}
