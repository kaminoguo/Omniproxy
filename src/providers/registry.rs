use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::config::Config;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    #[serde(default)]
    pub reasoning_levels: Vec<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ModelRegistry {
    pub codex: Vec<ModelInfo>,
    pub claude: Vec<ModelInfo>,
    pub gemini: Vec<ModelInfo>,
}

impl ModelRegistry {
    fn path() -> anyhow::Result<PathBuf> {
        Ok(Config::dir()?.join("models.json"))
    }

    pub fn load() -> anyhow::Result<Self> {
        let path = Self::path()?;

        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            Ok(serde_json::from_str(&content)?)
        } else {
            Ok(Self::default_registry())
        }
    }

    pub async fn refresh() -> anyhow::Result<Self> {
        // For now, return default registry
        // In the future, could fetch from provider APIs
        let registry = Self::default_registry();
        registry.save()?;
        Ok(registry)
    }

    fn save(&self) -> anyhow::Result<()> {
        let path = Self::path()?;

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    fn default_registry() -> Self {
        Self {
            codex: vec![
                ModelInfo {
                    name: "gpt-4o".to_string(),
                    reasoning_levels: vec![],
                },
                ModelInfo {
                    name: "gpt-4o-mini".to_string(),
                    reasoning_levels: vec![],
                },
                ModelInfo {
                    name: "gpt-4-turbo".to_string(),
                    reasoning_levels: vec![],
                },
                ModelInfo {
                    name: "o1".to_string(),
                    reasoning_levels: vec!["low".to_string(), "medium".to_string(), "high".to_string()],
                },
                ModelInfo {
                    name: "o1-mini".to_string(),
                    reasoning_levels: vec!["low".to_string(), "medium".to_string(), "high".to_string()],
                },
                ModelInfo {
                    name: "o1-preview".to_string(),
                    reasoning_levels: vec!["low".to_string(), "medium".to_string(), "high".to_string()],
                },
                ModelInfo {
                    name: "o3-mini".to_string(),
                    reasoning_levels: vec!["low".to_string(), "medium".to_string(), "high".to_string()],
                },
            ],
            claude: vec![
                ModelInfo {
                    name: "claude-sonnet-4-20250514".to_string(),
                    reasoning_levels: vec![],
                },
                ModelInfo {
                    name: "claude-opus-4-20250514".to_string(),
                    reasoning_levels: vec![],
                },
                ModelInfo {
                    name: "claude-3-5-sonnet-20241022".to_string(),
                    reasoning_levels: vec![],
                },
                ModelInfo {
                    name: "claude-3-5-haiku-20241022".to_string(),
                    reasoning_levels: vec![],
                },
                ModelInfo {
                    name: "claude-3-opus-20240229".to_string(),
                    reasoning_levels: vec![],
                },
            ],
            gemini: vec![
                ModelInfo {
                    name: "gemini-2.0-flash".to_string(),
                    reasoning_levels: vec![],
                },
                ModelInfo {
                    name: "gemini-2.0-flash-thinking".to_string(),
                    reasoning_levels: vec![],
                },
                ModelInfo {
                    name: "gemini-1.5-pro".to_string(),
                    reasoning_levels: vec![],
                },
                ModelInfo {
                    name: "gemini-1.5-flash".to_string(),
                    reasoning_levels: vec![],
                },
            ],
        }
    }

    pub fn codex_models(&self) -> &[ModelInfo] {
        &self.codex
    }

    pub fn claude_models(&self) -> &[ModelInfo] {
        &self.claude
    }

    pub fn gemini_models(&self) -> &[ModelInfo] {
        &self.gemini
    }
}
