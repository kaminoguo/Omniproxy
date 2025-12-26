use clap::Args;

use crate::server::Server;

#[derive(Args)]
pub struct ServeCommand {
    /// Host to bind to
    #[arg(long, default_value = "127.0.0.1")]
    pub host: String,

    /// Port to listen on
    #[arg(long, short, default_value = "8000")]
    pub port: u16,
}

pub async fn handle(cmd: ServeCommand) -> anyhow::Result<()> {
    let server = Server::new(&cmd.host, cmd.port).await?;

    println!("Omniproxy starting on http://{}:{}", cmd.host, cmd.port);
    println!("Press Ctrl+C to stop\n");

    server.run().await?;

    Ok(())
}
