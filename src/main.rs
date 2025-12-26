mod accounts;
mod auth;
mod cli;
mod config;
mod providers;
mod server;

use clap::Parser;
use cli::{Cli, Commands};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Account(cmd) => cli::account::handle(cmd).await?,
        Commands::Models(cmd) => cli::models::handle(cmd).await?,
        Commands::Serve(cmd) => cli::serve::handle(cmd).await?,
    }

    Ok(())
}
