use axum::body::Body;
use axum::http::{Request, Response, StatusCode};
use http_body_util::BodyExt;

use crate::accounts::Account;

const API_BASE: &str = "https://api.openai.com/v1";

pub struct CodexProvider;

impl CodexProvider {
    pub async fn proxy(account: &Account, request: Request<Body>) -> anyhow::Result<Response<Body>> {
        let client = reqwest::Client::new();

        let path = request.uri().path();
        let url = format!("{}{}", API_BASE, path);

        // Get request body
        let (parts, body) = request.into_parts();
        let body_bytes = body.collect().await?.to_bytes();

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
            .body(body_bytes.to_vec())
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
        let response = builder.body(Body::from(body))?;

        Ok(response)
    }
}
