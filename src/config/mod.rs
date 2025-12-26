use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub rotation: RotationConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotationConfig {
    #[serde(default = "default_strategy")]
    pub strategy: String,
}

fn default_host() -> String {
    "127.0.0.1".to_string()
}

fn default_port() -> u16 {
    8000
}

fn default_strategy() -> String {
    "round-robin".to_string()
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
        }
    }
}

impl Default for RotationConfig {
    fn default() -> Self {
        Self {
            strategy: default_strategy(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            rotation: RotationConfig::default(),
        }
    }
}

impl Config {
    /// Get the config directory path (~/.omniproxy)
    pub fn dir() -> anyhow::Result<PathBuf> {
        let home = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
        Ok(home.join(".omniproxy"))
    }

    /// Get the config file path (~/.omniproxy/config.toml)
    pub fn path() -> anyhow::Result<PathBuf> {
        Ok(Self::dir()?.join("config.toml"))
    }

    /// Get the accounts file path (~/.omniproxy/accounts.json)
    pub fn accounts_path() -> anyhow::Result<PathBuf> {
        Ok(Self::dir()?.join("accounts.json"))
    }

    /// Load config from file
    pub async fn load() -> anyhow::Result<Self> {
        let path = Self::path()?;

        if !path.exists() {
            return Ok(Self::default());
        }

        let content = tokio::fs::read_to_string(&path).await?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    /// Save config to file
    pub async fn save(&self) -> anyhow::Result<()> {
        let path = Self::path()?;

        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let content = toml::to_string_pretty(self)?;
        tokio::fs::write(&path, content).await?;
        Ok(())
    }
}
