use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: DateTime<Utc>,
    #[serde(default)]
    pub account_id: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
}

impl Credentials {
    pub fn new(access_token: String, refresh_token: String, expires_at: DateTime<Utc>) -> Self {
        Self {
            access_token,
            refresh_token,
            expires_at,
            account_id: None,
            email: None,
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() >= self.expires_at
    }

    pub fn is_valid(&self) -> bool {
        !self.access_token.is_empty() && !self.is_expired()
    }

    /// Check if token will expire within the given duration
    pub fn expires_within(&self, seconds: i64) -> bool {
        Utc::now() + chrono::Duration::seconds(seconds) >= self.expires_at
    }
}
