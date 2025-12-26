use clap::Args;

use crate::providers::ModelRegistry;

#[derive(Args)]
pub struct ModelsCommand {
    /// Refresh model list from providers
    #[arg(long)]
    pub refresh: bool,
}

pub async fn handle(cmd: ModelsCommand) -> anyhow::Result<()> {
    let registry = if cmd.refresh {
        println!("Refreshing model list...");
        ModelRegistry::refresh().await?
    } else {
        ModelRegistry::load()?
    };

    println!("\nAvailable models:\n");

    println!("Codex (OpenAI):");
    for model in registry.codex_models() {
        println!("  - {}", model.name);
        if !model.reasoning_levels.is_empty() {
            println!("    reasoning: {}", model.reasoning_levels.join(", "));
        }
    }

    println!("\nClaude (Anthropic):");
    for model in registry.claude_models() {
        println!("  - {}", model.name);
    }

    println!("\nGemini (Google):");
    for model in registry.gemini_models() {
        println!("  - {}", model.name);
    }

    Ok(())
}
