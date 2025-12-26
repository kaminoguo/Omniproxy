mod router;

use std::sync::Arc;

use axum::Router;
use tokio::net::TcpListener;

use crate::accounts::AccountManager;
use crate::config::Config;

pub struct Server {
    listener: TcpListener,
    router: Router,
}

impl Server {
    pub async fn new(host: &str, port: u16) -> anyhow::Result<Self> {
        let config = Config::load().await?;
        let account_manager = Arc::new(AccountManager::load().await?);

        if account_manager.is_empty().await {
            anyhow::bail!("No accounts configured. Use 'omniproxy account add <provider>' first.");
        }

        let router = router::create_router(account_manager, config);

        let addr = format!("{}:{}", host, port);
        let listener = TcpListener::bind(&addr).await?;

        Ok(Self { listener, router })
    }

    pub async fn run(self) -> anyhow::Result<()> {
        axum::serve(self.listener, self.router).await?;
        Ok(())
    }
}
