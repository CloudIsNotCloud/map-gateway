use tracing::info;

use crate::{setting::Settings, watcher::ConfigFilesWatcher};
use std::sync::{OnceLock, RwLock};

// Singleton Patterns
#[derive(Debug)]
pub struct AppConfig {
    settings: Settings,
    watcher: ConfigFilesWatcher,
}
// Use `RwLock` to implement Hot Update
static APP_CONFIG: OnceLock<RwLock<AppConfig>> = OnceLock::new();

impl AppConfig {
    #[tracing::instrument]
    fn get() -> &'static RwLock<Self> {
        APP_CONFIG.get_or_init(|| {
            info!("Config initialization...");
            let settings = Settings::try_new().expect("Loading Config Failed");
            let config_path = Settings::get_config_path();
            let watcher =
                ConfigFilesWatcher::try_new(config_path).expect("Initialize watcher fail");
            RwLock::new(AppConfig { settings, watcher })
        })
    }

    pub fn read() -> Settings {
        Self::get().read().unwrap().settings.clone()
    }

    // TODO specify change behaviour... DOES IT DESERVE?
    #[tracing::instrument]
    pub async fn changed(&mut self) {
        info!("Configuration changed, reloading...");
        if let Some(res) = self.watcher.recv().await {
            match res {
                Ok(_envents) => {}
                Err(_errors) => {}
            }
        }
    }
}
