use argument_parser_internal::Cli;
use axum::{Router, routing::get};
use clap::Parser;
use server_internal::Server;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _cli = Cli::try_parse()?;
    log_internal::minimal_init()?;
    let app_cfg = config_internal::AppConfig::read();
    log_internal::config_layer(&app_cfg.logging)?;
    let app: Router = Router::new().route("/hello", get(hello));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:13000").await?;
    let mut server = Server::try_new(listener, app).await?;
    server
        .register_async_crontab("*/1 * * * * *", || Box::pin(cron_hello()))
        .await?;
    server.run().await?;

    Ok(())
}

#[tracing::instrument]
async fn hello() -> String {
    tracing::info!("This is hello function!");
    "Hello world".to_string()
}

#[tracing::instrument]
async fn cron_hello() {
    tracing::info!("This is hello crontab!");
}
