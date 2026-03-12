/// Gmail OAuth2 authentication using saved token.json.
/// First-run: prints URL for user to open; subsequent runs reuse saved token.
use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use common::config;

#[allow(dead_code)]
const SCOPES: &[&str] = &[
    "https://www.googleapis.com/auth/gmail.readonly",
    "https://www.googleapis.com/auth/gmail.send",
    "https://www.googleapis.com/auth/gmail.compose",
];

fn token_path() -> PathBuf {
    config::get().credentials_dir.join("gmail_token.json")
}

fn creds_path() -> PathBuf {
    config::get().credentials_dir.join("gmail_credentials.json")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    pub access_token: String,
    #[serde(default)]
    pub refresh_token: String,
    #[serde(default)]
    pub expires_in: u64,
}

/// Load the saved access token from disk.
/// Does NOT refresh automatically — call refresh_token() if needed.
pub async fn load_token() -> Result<Token> {
    let path = token_path();
    if !path.exists() {
        bail!(
            "Gmail token not found at {}. Run `cargo run --bin gmail-auth` to authenticate.",
            path.display()
        );
    }
    let json = std::fs::read_to_string(&path)?;
    Ok(serde_json::from_str(&json)?)
}

/// Save a token to disk.
pub fn save_token(token: &Token) -> Result<()> {
    let path = token_path();
    std::fs::create_dir_all(path.parent().unwrap())?;
    std::fs::write(&path, serde_json::to_string_pretty(token)?)?;
    Ok(())
}

/// Refresh an expired access token using the stored refresh_token.
pub async fn refresh_access_token(refresh_token: &str) -> Result<Token> {
    #[derive(Deserialize)]
    struct OAuthCreds {
        installed: OAuthInstalled,
    }
    #[derive(Deserialize)]
    struct OAuthInstalled {
        client_id: String,
        client_secret: String,
        token_uri: String,
    }
    let creds_json = std::fs::read_to_string(creds_path())?;
    let creds: OAuthCreds = serde_json::from_str(&creds_json)?;
    let c = &creds.installed;

    let params = [
        ("client_id", c.client_id.as_str()),
        ("client_secret", c.client_secret.as_str()),
        ("refresh_token", refresh_token),
        ("grant_type", "refresh_token"),
    ];
    let resp: serde_json::Value = reqwest::Client::new()
        .post(&c.token_uri)
        .form(&params)
        .send()
        .await?
        .json()
        .await?;

    Ok(Token {
        access_token: resp["access_token"].as_str().unwrap_or("").to_string(),
        refresh_token: refresh_token.to_string(),
        expires_in: resp["expires_in"].as_u64().unwrap_or(3600),
    })
}

/// Get a valid access token, refreshing if the saved one is expired/missing.
pub async fn get_access_token() -> Result<String> {
    let token = load_token().await?;
    // Simple check: try to refresh if refresh_token is present.
    // For full expiry tracking we'd store expires_at, but this is sufficient.
    if !token.refresh_token.is_empty() {
        match refresh_access_token(&token.refresh_token).await {
            Ok(new_token) => {
                let _ = save_token(&new_token);
                return Ok(new_token.access_token);
            }
            Err(_) => {} // fall through to saved access_token
        }
    }
    Ok(token.access_token)
}
