use std::sync::Arc;

use axum::{
    body::Body,
    extract::State,
    http::{Request, Response, StatusCode},
    routing::{get, post},
    Json, Router,
};
use http_body_util::BodyExt;
use serde_json::{json, Value};

use crate::accounts::{AccountManager, Provider};
use crate::config::Config;
use crate::providers;

#[derive(Clone)]
struct AppState {
    account_manager: Arc<AccountManager>,
    #[allow(dead_code)]
    config: Config,
}

pub fn create_router(account_manager: Arc<AccountManager>, config: Config) -> Router {
    let state = AppState {
        account_manager,
        config,
    };

    Router::new()
        .route("/v1/chat/completions", post(chat_completions))
        .route("/chat/completions", post(chat_completions))
        .route("/v1/models", get(list_models))
        .route("/models", get(list_models))
        .route("/health", get(health))
        .with_state(state)
}

async fn health() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}

async fn list_models(State(state): State<AppState>) -> Json<Value> {
    let mut models = Vec::new();

    // Add models based on available accounts
    if state.account_manager.count(&Provider::Codex).await > 0 {
        models.extend([
            "gpt-4o", "gpt-4o-mini", "gpt-4-turbo", "gpt-4",
            "o1", "o1-mini", "o1-preview", "o3-mini",
        ]);
    }

    if state.account_manager.count(&Provider::Claude).await > 0 {
        models.extend([
            "claude-sonnet-4-20250514", "claude-opus-4-20250514",
            "claude-3-5-sonnet-20241022", "claude-3-5-haiku-20241022",
            "claude-3-opus-20240229",
        ]);
    }

    if state.account_manager.count(&Provider::Gemini).await > 0 {
        models.extend([
            "gemini-2.0-flash", "gemini-2.0-flash-thinking",
            "gemini-1.5-pro", "gemini-1.5-flash",
        ]);
    }

    let data: Vec<Value> = models
        .iter()
        .map(|m| {
            json!({
                "id": m,
                "object": "model",
                "owned_by": "omniproxy",
            })
        })
        .collect();

    Json(json!({
        "object": "list",
        "data": data,
    }))
}

async fn chat_completions(
    State(state): State<AppState>,
    request: Request<Body>,
) -> Result<Response<Body>, (StatusCode, Json<Value>)> {
    // Read body to extract model
    let (parts, body) = request.into_parts();
    let body_bytes = body
        .collect()
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": format!("Failed to read body: {}", e) })),
            )
        })?
        .to_bytes();

    let body_json: Value = serde_json::from_slice(&body_bytes).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": format!("Invalid JSON: {}", e) })),
        )
    })?;

    let model = body_json
        .get("model")
        .and_then(|m| m.as_str())
        .ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Missing 'model' field" })),
            )
        })?;

    // Determine provider from model
    let provider = determine_provider(model).ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": format!("Unknown model: {}", model) })),
        )
    })?;

    // Get next account using round-robin
    let account = state
        .account_manager
        .next_account(&provider)
        .await
        .ok_or_else(|| {
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({ "error": format!("No valid accounts for provider: {}", provider) })),
            )
        })?;

    tracing::info!(
        "Routing request for model '{}' to {} account '{}'",
        model,
        provider,
        account.name
    );

    // Reconstruct request
    let request = Request::from_parts(parts, Body::from(body_bytes.to_vec()));

    // Proxy to provider
    providers::proxy_request(&account, request).await.map_err(|e| {
        tracing::error!("Proxy error: {}", e);
        (
            StatusCode::BAD_GATEWAY,
            Json(json!({ "error": format!("Proxy error: {}", e) })),
        )
    })
}

fn determine_provider(model: &str) -> Option<Provider> {
    if Provider::Codex.matches_model(model) {
        Some(Provider::Codex)
    } else if Provider::Claude.matches_model(model) {
        Some(Provider::Claude)
    } else if Provider::Gemini.matches_model(model) {
        Some(Provider::Gemini)
    } else {
        None
    }
}
