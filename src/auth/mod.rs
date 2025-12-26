mod codex;
mod claude;
mod gemini;
mod pkce;

use crate::accounts::{Credentials, Provider};

pub use codex::CodexAuth;
pub use claude::ClaudeAuth;
pub use gemini::GeminiAuth;

/// Perform OAuth login for a provider
pub async fn oauth_login(provider: &Provider) -> anyhow::Result<Credentials> {
    match provider {
        Provider::Codex => CodexAuth::login().await,
        Provider::Claude => ClaudeAuth::login().await,
        Provider::Gemini => GeminiAuth::login().await,
    }
}

/// Refresh access token for a provider
pub async fn refresh_token(provider: &Provider, refresh_token: &str) -> anyhow::Result<Credentials> {
    match provider {
        Provider::Codex => CodexAuth::refresh(refresh_token).await,
        Provider::Claude => ClaudeAuth::refresh(refresh_token).await,
        Provider::Gemini => GeminiAuth::refresh(refresh_token).await,
    }
}
