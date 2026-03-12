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
    let resp = reqwest::get(format!("{CDP_HTTP}/json/new?{url}")).await?;
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

    /// Send a CDP command and wait for its response.
    pub async fn send(&mut self, method: &str, params: Value) -> Result<Value> {
        let id = MSG_ID.fetch_add(1, Ordering::Relaxed);
        let msg = json!({"id": id, "method": method, "params": params});
        self.ws.send(Message::Text(msg.to_string().into())).await?;

        // Read until we get a message with matching id
        loop {
            match self.ws.next().await {
                Some(Ok(Message::Text(text))) => {
                    let v: Value = serde_json::from_str(&text).unwrap_or_default();
                    if v["id"].as_u64() == Some(id) {
                        return Ok(v["result"].clone());
                    }
                }
                Some(Err(e)) => bail!("CDP WS error: {e}"),
                None => bail!("CDP WS closed"),
                _ => {}
            }
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
