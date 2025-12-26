use clap::{Args, Subcommand};

use crate::accounts::{AccountManager, Provider};
use crate::auth;

#[derive(Args)]
pub struct AccountCommand {
    #[command(subcommand)]
    pub action: AccountAction,
}

#[derive(Subcommand)]
pub enum AccountAction {
    /// Add a new account
    Add {
        /// Provider (codex, claude, gemini)
        provider: String,
        /// Account name (optional)
        #[arg(long)]
        name: Option<String>,
    },
    /// List all accounts
    List,
    /// Remove an account
    Remove {
        /// Account ID (provider:name)
        id: String,
    },
}

pub async fn handle(cmd: AccountCommand) -> anyhow::Result<()> {
    let manager = AccountManager::load().await?;

    match cmd.action {
        AccountAction::Add { provider, name } => {
            let provider = Provider::from_str(&provider)?;
            let name = name.unwrap_or_else(|| format!("{}-{}", provider.as_str(), 1));

            println!("Adding {} account: {}", provider.as_str(), name);
            println!("Opening browser for OAuth login...");

            let credentials = auth::oauth_login(&provider).await?;

            let mut manager = manager;
            manager.add(provider, &name, credentials).await?;
            manager.save().await?;

            println!("Account added: {}:{}", provider.as_str(), name);
        }
        AccountAction::List => {
            if manager.is_empty().await {
                println!("No accounts configured.");
                println!("Add one with: omniproxy account add <provider>");
                return Ok(());
            }

            for provider in [Provider::Codex, Provider::Claude, Provider::Gemini] {
                let accounts = manager.list(&provider).await;
                if !accounts.is_empty() {
                    println!("{}:", provider.as_str());
                    for acc in accounts {
                        let status = if acc.is_valid() { "✓" } else { "✗" };
                        println!("  {} {} (expires: {})", status, acc.name, acc.expires_at());
                    }
                }
            }
        }
        AccountAction::Remove { id } => {
            let parts: Vec<&str> = id.split(':').collect();
            if parts.len() != 2 {
                anyhow::bail!("Invalid ID format. Use: provider:name");
            }

            let provider = Provider::from_str(parts[0])?;
            let name = parts[1];

            let mut manager = manager;
            manager.remove(&provider, name).await?;
            manager.save().await?;

            println!("Account removed: {}", id);
        }
    }

    Ok(())
}
