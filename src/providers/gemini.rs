use axum::body::Body;
use axum::http::{Request, Response, StatusCode};
use http_body_util::BodyExt;
use serde_json::Value;

use crate::accounts::Account;

const API_BASE: &str = "https://generativelanguage.googleapis.com/v1beta";

pub struct GeminiProvider;

impl GeminiProvider {
    pub async fn proxy(account: &Account, request: Request<Body>) -> anyhow::Result<Response<Body>> {
        let client = reqwest::Client::new();

        // Get request body
        let (parts, body) = request.into_parts();
        let path = parts.uri.path().to_string();
        let body_bytes = body.collect().await?.to_bytes();

        // Parse the OpenAI request to get the model
        let body_json: Value = serde_json::from_slice(&body_bytes)?;
        let model = body_json
            .get("model")
            .and_then(|m| m.as_str())
            .unwrap_or("gemini-2.0-flash")
            .to_string();

        // Convert model name if needed
        let gemini_model = Self::map_model(&model);

        // For Gemini, convert OpenAI format to Gemini format
        let (url, body_bytes) = if path == "/v1/chat/completions" || path == "/chat/completions" || path == "chat/completions" {
            let converted = Self::convert_request(body_json)?;
            let url = format!("{}/models/{}:generateContent", API_BASE, gemini_model);
            (url, serde_json::to_vec(&converted)?)
        } else {
            (format!("{}{}", API_BASE, path), body_bytes.to_vec())
        };

        // Build proxied request
        let mut req_builder = client
            .request(parts.method.clone(), &url)
            .header("Authorization", format!("Bearer {}", account.credentials.access_token))
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

        // Convert Gemini response to OpenAI format
        let converted_body = if path == "/v1/chat/completions" || path == "/chat/completions" || path == "chat/completions" {
            if let Ok(gemini_response) = serde_json::from_slice::<Value>(&body) {
                serde_json::to_vec(&Self::convert_response(gemini_response, &model)?)?
            } else {
                body.to_vec()
            }
        } else {
            body.to_vec()
        };

        let response = builder.body(Body::from(converted_body))?;

        Ok(response)
    }

    /// Map OpenAI-style model names to Gemini model names
    fn map_model(model: &str) -> &str {
        let model_lower = model.to_lowercase();
        if model_lower.contains("gemini") {
            model
        } else if model_lower.contains("flash") {
            "gemini-2.0-flash"
        } else if model_lower.contains("pro") {
            "gemini-1.5-pro"
        } else {
            "gemini-2.0-flash"
        }
    }

    /// Convert OpenAI chat completion request to Gemini format
    fn convert_request(openai_req: Value) -> anyhow::Result<Value> {
        let mut contents = Vec::new();
        let mut system_instruction = None;

        if let Some(messages) = openai_req.get("messages").and_then(|m| m.as_array()) {
            for msg in messages {
                let role = msg.get("role").and_then(|r| r.as_str()).unwrap_or("user");
                let content = msg.get("content").and_then(|c| c.as_str()).unwrap_or("");

                if role == "system" {
                    system_instruction = Some(serde_json::json!({
                        "parts": [{"text": content}]
                    }));
                } else {
                    let gemini_role = match role {
                        "assistant" => "model",
                        _ => "user",
                    };
                    contents.push(serde_json::json!({
                        "role": gemini_role,
                        "parts": [{"text": content}]
                    }));
                }
            }
        }

        let mut gemini_req = serde_json::json!({
            "contents": contents,
        });

        if let Some(system) = system_instruction {
            gemini_req["systemInstruction"] = system;
        }

        // Generation config
        let mut generation_config = serde_json::json!({});

        if let Some(max_tokens) = openai_req.get("max_tokens") {
            generation_config["maxOutputTokens"] = max_tokens.clone();
        }

        if let Some(temp) = openai_req.get("temperature") {
            generation_config["temperature"] = temp.clone();
        }

        if generation_config.as_object().map(|o| !o.is_empty()).unwrap_or(false) {
            gemini_req["generationConfig"] = generation_config;
        }

        Ok(gemini_req)
    }

    /// Convert Gemini response to OpenAI format
    fn convert_response(gemini_resp: Value, model: &str) -> anyhow::Result<Value> {
        let content = gemini_resp
            .get("candidates")
            .and_then(|c| c.as_array())
            .and_then(|arr| arr.first())
            .and_then(|candidate| candidate.get("content"))
            .and_then(|content| content.get("parts"))
            .and_then(|parts| parts.as_array())
            .and_then(|parts| parts.first())
            .and_then(|part| part.get("text"))
            .and_then(|t| t.as_str())
            .unwrap_or("");

        let usage = gemini_resp.get("usageMetadata");
        let prompt_tokens = usage.and_then(|u| u.get("promptTokenCount")).and_then(|t| t.as_i64()).unwrap_or(0);
        let completion_tokens = usage.and_then(|u| u.get("candidatesTokenCount")).and_then(|t| t.as_i64()).unwrap_or(0);

        let openai_response = serde_json::json!({
            "id": format!("chatcmpl-{}", uuid::Uuid::new_v4()),
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
                "prompt_tokens": prompt_tokens,
                "completion_tokens": completion_tokens,
                "total_tokens": prompt_tokens + completion_tokens,
            }
        });

        Ok(openai_response)
    }
}
