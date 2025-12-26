mod codex;
mod claude;
mod gemini;
mod registry;

use axum::body::Body;
use axum::http::{Request, Response};

use crate::accounts::{Account, Provider};

pub use codex::CodexProvider;
pub use claude::ClaudeProvider;
pub use gemini::GeminiProvider;
pub use registry::ModelRegistry;

/// Proxy a request to the appropriate provider
pub async fn proxy_request(
    account: &Account,
    request: Request<Body>,
) -> anyhow::Result<Response<Body>> {
    match account.provider {
        Provider::Codex => CodexProvider::proxy(account, request).await,
        Provider::Claude => ClaudeProvider::proxy(account, request).await,
        Provider::Gemini => GeminiProvider::proxy(account, request).await,
    }
}
