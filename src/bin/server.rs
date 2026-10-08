use axum::{Router, routing::get};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    log_internal::minimal_init()?;
    let app_cfg = config_internal::AppConfig::read();
    log_internal::config_layer(&app_cfg.logging)?;
    let app = Router::new().route("/hello", get(hello));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:13000").await?;

    axum::serve(listener, app).await.unwrap();
    Ok(())
}

#[tracing::instrument]
async fn hello() -> String {
    tracing::info!("This is hello function!");
    "Hello world".to_string()
}
