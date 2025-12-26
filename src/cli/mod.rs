pub mod account;
pub mod models;
pub mod serve;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "omniproxy")]
#[command(about = "Unified API gateway for AI model subscriptions")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Manage accounts
    Account(account::AccountCommand),
    /// List available models
    Models(models::ModelsCommand),
    /// Start the API server
    Serve(serve::ServeCommand),
}
