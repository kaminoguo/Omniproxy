use std::sync::Arc;

use axum::{
    extract::Query,
    response::Html,
    routing::get,
    Router,
};
use chrono::{Duration, Utc};
use serde::Deserialize;
use tokio::sync::oneshot;

use crate::accounts::Credentials;
use super::pkce::{generate_pkce, generate_state};

const AUTH_URL: &str = "https://claude.ai/oauth/authorize";
const TOKEN_URL: &str = "https://claude.ai/api/auth/oauth_token";
const CLIENT_ID: &str = "9d1c250a-e61b-44d5-b14b-f5d8b0c5fdce";
const REDIRECT_PORT: u16 = 8485;
const SCOPES: &[&str] = &["user:inference", "user:profile"];

pub struct ClaudeAuth;

impl ClaudeAuth {
    pub async fn login() -> anyhow::Result<Credentials> {
        let (code_verifier, code_challenge) = generate_pkce();
        let state = generate_state();
        let redirect_uri = format!("http://127.0.0.1:{}/auth/callback", REDIRECT_PORT);

        // Build authorization URL
        let auth_url = format!(
            "{}?response_type=code&client_id={}&redirect_uri={}&scope={}&state={}&code_challenge={}&code_challenge_method=S256",
            AUTH_URL,
            CLIENT_ID,
            urlencoding::encode(&redirect_uri),
            urlencoding::encode(&SCOPES.join(" ")),
            state,
            code_challenge,
        );

        // Channel to receive the authorization code
        let (tx, rx) = oneshot::channel::<Result<String, String>>();
        let tx = Arc::new(std::sync::Mutex::new(Some(tx)));
        let expected_state = state.clone();

        // Start callback server
        let app = Router::new().route(
            "/auth/callback",
            get({
                let tx = Arc::clone(&tx);
                move |Query(params): Query<CallbackParams>| {
                    let tx = tx.clone();
                    async move {
                        let result = if params.state != expected_state {
                            Err("Invalid state parameter".to_string())
                        } else if let Some(error) = params.error {
                            Err(error)
                        } else if let Some(code) = params.code {
                            Ok(code)
                        } else {
                            Err("No authorization code received".to_string())
                        };

                        if let Some(tx) = tx.lock().unwrap().take() {
                            let _ = tx.send(result);
                        }

                        Html(r#"
                            <html>
                            <head><title>Authentication Complete</title></head>
                            <body>
                                <h1>Authentication successful!</h1>
                                <p>You can close this window and return to the terminal.</p>
                                <script>setTimeout(() => window.close(), 2000);</script>
                            </body>
                            </html>
                        "#)
                    }
                }
            }),
        );

        let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", REDIRECT_PORT)).await?;

        // Open browser
        println!("Opening browser for authentication...");
        println!("If browser doesn't open, visit:\n{}\n", auth_url);

        if let Err(e) = open::that(&auth_url) {
            tracing::warn!("Failed to open browser: {}", e);
        }

        // Spawn server task
        let server = tokio::spawn(async move {
            axum::serve(listener, app).await
        });

        // Wait for callback with timeout
        let code = tokio::time::timeout(
            std::time::Duration::from_secs(300),
            rx,
        ).await??
            .map_err(|e| anyhow::anyhow!("OAuth error: {}", e))?;

        // Stop server
        server.abort();

        // Exchange code for tokens
        Self::exchange_code(&code, &code_verifier, &redirect_uri).await
    }

    async fn exchange_code(code: &str, code_verifier: &str, redirect_uri: &str) -> anyhow::Result<Credentials> {
        let client = reqwest::Client::new();

        let response = client
            .post(TOKEN_URL)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&[
                ("grant_type", "authorization_code"),
                ("code", code),
                ("redirect_uri", redirect_uri),
                ("client_id", CLIENT_ID),
                ("code_verifier", code_verifier),
            ])
            .send()
            .await?;

        if !response.status().is_success() {
            let error = response.text().await?;
            anyhow::bail!("Token exchange failed: {}", error);
        }

        let token: TokenResponse = response.json().await?;

        let expires_at = Utc::now() + Duration::seconds(token.expires_in.unwrap_or(3600) as i64);

        Ok(Credentials::new(
            token.access_token,
            token.refresh_token.unwrap_or_default(),
            expires_at,
        ))
    }

    pub async fn refresh(refresh_token: &str) -> anyhow::Result<Credentials> {
        let client = reqwest::Client::new();

        let response = client
            .post(TOKEN_URL)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&[
                ("grant_type", "refresh_token"),
                ("refresh_token", refresh_token),
                ("client_id", CLIENT_ID),
            ])
            .send()
            .await?;

        if !response.status().is_success() {
            let error = response.text().await?;
            anyhow::bail!("Token refresh failed: {}", error);
        }

        let token: TokenResponse = response.json().await?;

        let expires_at = Utc::now() + Duration::seconds(token.expires_in.unwrap_or(3600) as i64);

        Ok(Credentials::new(
            token.access_token,
            token.refresh_token.unwrap_or_else(|| refresh_token.to_string()),
            expires_at,
        ))
    }
}

#[derive(Debug, Deserialize)]
struct CallbackParams {
    code: Option<String>,
    state: String,
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: Option<u64>,
}
