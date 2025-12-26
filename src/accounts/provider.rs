use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    Codex,
    Claude,
    Gemini,
}

impl Provider {
    pub fn from_str(s: &str) -> anyhow::Result<Self> {
        match s.to_lowercase().as_str() {
            "codex" | "openai" | "gpt" | "chatgpt" => Ok(Provider::Codex),
            "claude" | "anthropic" => Ok(Provider::Claude),
            "gemini" | "google" => Ok(Provider::Gemini),
            _ => anyhow::bail!("Unknown provider: {}. Use: codex, claude, or gemini", s),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Provider::Codex => "codex",
            Provider::Claude => "claude",
            Provider::Gemini => "gemini",
        }
    }

    /// Check if a model name belongs to this provider
    pub fn matches_model(&self, model: &str) -> bool {
        let model_lower = model.to_lowercase();
        match self {
            Provider::Codex => {
                model_lower.contains("gpt") ||
                model_lower.contains("codex") ||
                model_lower.starts_with("o1") ||
                model_lower.starts_with("o3")
            }
            Provider::Claude => {
                model_lower.contains("claude") ||
                model_lower.contains("opus") ||
                model_lower.contains("sonnet") ||
                model_lower.contains("haiku")
            }
            Provider::Gemini => {
                model_lower.contains("gemini") ||
                model_lower.contains("flash") ||
                model_lower.contains("pro") && !model_lower.contains("gpt")
            }
        }
    }
}

impl std::fmt::Display for Provider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
