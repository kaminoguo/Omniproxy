use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use super::{Credentials, Provider};
use crate::config::Config;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub name: String,
    pub provider: Provider,
    pub credentials: Credentials,
}

impl Account {
    pub fn is_valid(&self) -> bool {
        self.credentials.is_valid()
    }

    pub fn expires_at(&self) -> String {
        self.credentials.expires_at.format("%Y-%m-%d %H:%M").to_string()
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct AccountsData {
    accounts: Vec<Account>,
}

pub struct AccountManager {
    data: Arc<RwLock<AccountsData>>,
    path: PathBuf,
    // Round-robin counters per provider
    counters: HashMap<Provider, AtomicUsize>,
}

impl AccountManager {
    pub async fn load() -> anyhow::Result<Self> {
        let path = Config::accounts_path()?;

        let data = if path.exists() {
            let content = tokio::fs::read_to_string(&path).await?;
            serde_json::from_str(&content)?
        } else {
            AccountsData::default()
        };

        let mut counters = HashMap::new();
        counters.insert(Provider::Codex, AtomicUsize::new(0));
        counters.insert(Provider::Claude, AtomicUsize::new(0));
        counters.insert(Provider::Gemini, AtomicUsize::new(0));

        Ok(Self {
            data: Arc::new(RwLock::new(data)),
            path,
            counters,
        })
    }

    pub async fn save(&self) -> anyhow::Result<()> {
        let data = self.data.read().await;
        let content = serde_json::to_string_pretty(&*data)?;

        // Ensure parent directory exists
        if let Some(parent) = self.path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        tokio::fs::write(&self.path, content).await?;
        Ok(())
    }

    pub async fn add(&mut self, provider: Provider, name: &str, credentials: Credentials) -> anyhow::Result<()> {
        let mut data = self.data.write().await;

        // Check for duplicate name
        if data.accounts.iter().any(|a| a.provider == provider && a.name == name) {
            anyhow::bail!("Account already exists: {}:{}", provider, name);
        }

        data.accounts.push(Account {
            name: name.to_string(),
            provider,
            credentials,
        });

        Ok(())
    }

    pub async fn remove(&mut self, provider: &Provider, name: &str) -> anyhow::Result<()> {
        let mut data = self.data.write().await;

        let idx = data.accounts.iter()
            .position(|a| a.provider == *provider && a.name == name)
            .ok_or_else(|| anyhow::anyhow!("Account not found: {}:{}", provider, name))?;

        data.accounts.remove(idx);
        Ok(())
    }

    pub async fn count(&self, provider: &Provider) -> usize {
        let data = self.data.read().await;
        data.accounts.iter().filter(|a| a.provider == *provider).count()
    }

    pub async fn is_empty(&self) -> bool {
        let data = self.data.read().await;
        data.accounts.is_empty()
    }

    pub async fn list(&self, provider: &Provider) -> Vec<Account> {
        let data = self.data.read().await;
        data.accounts.iter()
            .filter(|a| a.provider == *provider)
            .cloned()
            .collect()
    }

    /// Get the next account for a provider using round-robin
    pub async fn next_account(&self, provider: &Provider) -> Option<Account> {
        let data = self.data.read().await;
        let accounts: Vec<_> = data.accounts.iter()
            .filter(|a| a.provider == *provider && a.is_valid())
            .collect();

        if accounts.is_empty() {
            return None;
        }

        let counter = self.counters.get(provider)?;
        let idx = counter.fetch_add(1, Ordering::Relaxed) % accounts.len();

        accounts.get(idx).map(|a| (*a).clone())
    }

    /// Update credentials for an account
    pub async fn update_credentials(&self, provider: &Provider, name: &str, credentials: Credentials) -> anyhow::Result<()> {
        let mut data = self.data.write().await;

        let account = data.accounts.iter_mut()
            .find(|a| a.provider == *provider && a.name == name)
            .ok_or_else(|| anyhow::anyhow!("Account not found: {}:{}", provider, name))?;

        account.credentials = credentials;
        Ok(())
    }
}
