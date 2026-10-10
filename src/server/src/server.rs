use std::pin::Pin;

use axum::Router;
use tokio::net::TcpListener;
use tokio_cron_scheduler::{Job, JobScheduler};
use tokio_util::sync::CancellationToken;
use tracing::{error, info};

pub struct Server {
    listener: TcpListener,
    app: Router,
    scheduler: JobScheduler,
    token: CancellationToken,
}

// Capture shutdown signal
async fn shutdown_signal() {
    let signal_ctrl_c = tokio::signal::ctrl_c();

    #[cfg(unix)]
    let signal_term = async {
        let mut unix_shutdown =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                .expect("SIGTERM registered error");
        unix_shutdown.recv().await
    };

    #[cfg(not(unix))]
    let signal_term = async {
        let mut windows_shutdown =
            tokio::signal::windows::ctrl_shutdown().expect("Ctrl shutdown register error");
        windows_shutdown.recv().await
    };

    tokio::select! {
        _ = signal_ctrl_c => {},
        _ = signal_term => {},
    };
}

impl Server {
    pub async fn try_new(listener: TcpListener, app: Router) -> anyhow::Result<Self> {
        let scheduler = JobScheduler::new().await?;
        let token = CancellationToken::new();
        Ok(Self {
            listener,
            app,
            scheduler,
            token,
        })
    }

    pub async fn register_async_crontab<T>(
        &mut self,
        cron_expression: &str,
        task: T,
    ) -> anyhow::Result<()>
    where
        T: Fn() -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync + 'static,
    {
        let token = self.token.clone();
        let job = Job::new_async(cron_expression, move |_uuid, _lock| {
            let token = token.clone();
            let fut = task();

            Box::pin(async move {
                tokio::select! {
                    _ = token.cancelled() => {}
                    _ = fut => {}
                }
            })
        })?;
        self.scheduler.add(job).await?;
        Ok(())
    }

    pub async fn run(mut self) -> anyhow::Result<()> {
        self.scheduler.start().await?;
        info!("Job Scheduler Started...");

        let shutdown_token = self.token.clone();
        tokio::spawn(async move {
            shutdown_signal().await;
            info!("Shutdown signal received");
            shutdown_token.cancel();
        });

        info!("Web Server Started...");
        axum::serve(self.listener, self.app)
            .with_graceful_shutdown(async move {
                let token = self.token.clone();
                token.cancelled().await;
            })
            .await?;

        info!("Stopping scheduler...");
        if let Err(e) = self.scheduler.shutdown().await {
            error!(error = ?e, "Failed to shutdown scheduler");
        }

        info!("Server shut down!");
        Ok(())
    }
}

pub async fn run(listener: TcpListener, app: Router) -> anyhow::Result<()> {
    let server = Server::try_new(listener, app).await?;
    server.run().await?;
    Ok(())
}
