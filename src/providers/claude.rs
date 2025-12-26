use axum::body::Body;
use axum::http::{Request, Response, StatusCode};
use http_body_util::BodyExt;
use serde_json::Value;

use crate::accounts::Account;

const API_BASE: &str = "https://api.anthropic.com/v1";

pub struct ClaudeProvider;

impl ClaudeProvider {
    pub async fn proxy(account: &Account, request: Request<Body>) -> anyhow::Result<Response<Body>> {
        let client = reqwest::Client::new();

        // Get request body
        let (parts, body) = request.into_parts();
        let path = parts.uri.path().to_string();
        let body_bytes = body.collect().await?.to_bytes();

        // For Claude, we need to convert OpenAI format to Anthropic format
        // if the request is to /chat/completions
        let (url, body_bytes) = if path == "/v1/chat/completions" || path == "/chat/completions" || path == "chat/completions" {
            let body_json: Value = serde_json::from_slice(&body_bytes)?;
            let converted = Self::convert_request(body_json)?;
            (format!("{}/messages", API_BASE), serde_json::to_vec(&converted)?)
        } else {
            (format!("{}{}", API_BASE, path), body_bytes.to_vec())
        };

        // Build proxied request
        let mut req_builder = client
            .request(parts.method.clone(), &url)
            .header("x-api-key", &account.credentials.access_token)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json");

        // Copy relevant headers
        for (name, value) in parts.headers.iter() {
            if name != "host" && name != "authorization" && name != "content-length" {
                req_builder = req_builder.header(name.clone(), value.clone());
            }
        }

        let response = req_builder
            .body(body_bytes)
            .send()
            .await?;

        // Build response
        let status = StatusCode::from_u16(response.status().as_u16())?;
        let mut builder = Response::builder().status(status);

        for (name, value) in response.headers() {
            if name != "transfer-encoding" && name != "content-length" {
                builder = builder.header(name.clone(), value.clone());
            }
        }

        let body = response.bytes().await?;

        // Convert Anthropic response to OpenAI format
        let converted_body = if path == "/v1/chat/completions" || path == "/chat/completions" || path == "chat/completions" {
            if let Ok(anthropic_response) = serde_json::from_slice::<Value>(&body) {
                serde_json::to_vec(&Self::convert_response(anthropic_response)?)?
            } else {
                body.to_vec()
            }
        } else {
            body.to_vec()
        };

        let response = builder.body(Body::from(converted_body))?;

        Ok(response)
    }

    /// Convert OpenAI chat completion request to Anthropic messages format
    fn convert_request(openai_req: Value) -> anyhow::Result<Value> {
        let mut anthropic_req = serde_json::json!({});

        // Model
        if let Some(model) = openai_req.get("model") {
            anthropic_req["model"] = model.clone();
        }

        // Messages
        if let Some(messages) = openai_req.get("messages").and_then(|m| m.as_array()) {
            let mut anthropic_messages = Vec::new();
            let mut system_prompt = None;

            for msg in messages {
                let role = msg.get("role").and_then(|r| r.as_str()).unwrap_or("user");
                let content = msg.get("content").cloned().unwrap_or(Value::String(String::new()));

                if role == "system" {
                    system_prompt = Some(content);
                } else {
                    let anthropic_role = match role {
                        "assistant" => "assistant",
                        _ => "user",
                    };
                    anthropic_messages.push(serde_json::json!({
                        "role": anthropic_role,
                        "content": content,
                    }));
                }
            }

            anthropic_req["messages"] = Value::Array(anthropic_messages);

            if let Some(system) = system_prompt {
                anthropic_req["system"] = system;
            }
        }

        // Max tokens
        if let Some(max_tokens) = openai_req.get("max_tokens") {
            anthropic_req["max_tokens"] = max_tokens.clone();
        } else {
            anthropic_req["max_tokens"] = Value::Number(4096.into());
        }

        // Temperature
        if let Some(temp) = openai_req.get("temperature") {
            anthropic_req["temperature"] = temp.clone();
        }

        // Stream
        if let Some(stream) = openai_req.get("stream") {
            anthropic_req["stream"] = stream.clone();
        }

        Ok(anthropic_req)
    }

    /// Convert Anthropic response to OpenAI format
    fn convert_response(anthropic_resp: Value) -> anyhow::Result<Value> {
        let content = anthropic_resp
            .get("content")
            .and_then(|c| c.as_array())
            .and_then(|arr| arr.first())
            .and_then(|block| block.get("text"))
            .and_then(|t| t.as_str())
            .unwrap_or("");

        let model = anthropic_resp
            .get("model")
            .and_then(|m| m.as_str())
            .unwrap_or("claude");

        let id = anthropic_resp
            .get("id")
            .and_then(|i| i.as_str())
            .unwrap_or("msg_unknown");

        let usage = anthropic_resp.get("usage");
        let input_tokens = usage.and_then(|u| u.get("input_tokens")).and_then(|t| t.as_i64()).unwrap_or(0);
        let output_tokens = usage.and_then(|u| u.get("output_tokens")).and_then(|t| t.as_i64()).unwrap_or(0);

        let openai_response = serde_json::json!({
            "id": format!("chatcmpl-{}", id),
            "object": "chat.completion",
            "created": chrono::Utc::now().timestamp(),
            "model": model,
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": content,
                },
                "finish_reason": "stop",
            }],
            "usage": {
                "prompt_tokens": input_tokens,
                "completion_tokens": output_tokens,
                "total_tokens": input_tokens + output_tokens,
            }
        });

        Ok(openai_response)
    }
}
