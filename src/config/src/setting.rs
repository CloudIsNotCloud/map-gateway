use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use std::{env, path::PathBuf};

#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Settings {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub logging: LoggingConfig,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            server: ServerConfig::default(),
            database: DatabaseConfig::default(),
            logging: LoggingConfig::default(),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
#[cfg_attr(test, derive(PartialEq))]
pub struct ServerConfig {
    pub host: String,
    pub port: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig {
            host: "127.0.0.1".into(),
            port: "8080".into(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(test, derive(PartialEq))]
pub enum LogLevel {
    Off,
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl LogLevel {
    pub fn to_tracing_level(&self) -> tracing::Level {
        match self {
            LogLevel::Off => tracing::Level::ERROR,
            LogLevel::Error => tracing::Level::ERROR,
            LogLevel::Warn => tracing::Level::WARN,
            LogLevel::Info => tracing::Level::INFO,
            LogLevel::Debug => tracing::Level::DEBUG,
            LogLevel::Trace => tracing::Level::TRACE,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
#[cfg_attr(test, derive(PartialEq))]
pub enum Sink {
    Stdout,
    Stderr,
    File { path: PathBuf },
    MultiFiles { paths: Vec<PathBuf> },
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum LogFormat {
    Json,
    Compact,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
#[cfg_attr(test, derive(PartialEq))]
pub struct LoggingConfig {
    pub level: LogLevel,
    pub sink: Sink,
    pub format: LogFormat,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        LoggingConfig {
            level: LogLevel::Info,
            sink: Sink::Stdout,
            format: LogFormat::Compact,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
#[cfg_attr(test, derive(PartialEq))]
pub struct DatabaseConfig {
    pub host: String,
    pub port: String,
    pub user: String,
    pub max_connections: u32,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        DatabaseConfig {
            host: "127.0.0.1".into(),
            port: "5432".into(),
            user: "postgres".into(),
            max_connections: 20,
        }
    }
}

impl Settings {
    pub(crate) fn try_new() -> Result<Self, ConfigError> {
        // Determine Run Mode
        let run_mode = env::var("APP_RUN_MODE").unwrap_or_else(|_| "development".to_string());

        // Join path and filename
        let config_path = Self::get_config_path();
        let config_file = config_path.join("config");
        let run_mode_file = config_path.join(run_mode);

        // Convert path to filename
        let config_file_name = config_file.to_string_lossy();
        let run_mode_file_name = run_mode_file.to_string_lossy();

        let s = Config::builder()
            .add_source(File::with_name(&config_file_name).required(false))
            .add_source(File::with_name(&run_mode_file_name).required(false))
            .add_source(Environment::with_prefix("APP_CFG"))
            .build()?;

        s.try_deserialize()
    }

    pub(crate) fn get_config_path() -> PathBuf {
        let config_path = env::var("APP_CONFIG_PATH").unwrap_or_else(|_| "./".to_string());
        PathBuf::from(config_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_file_load() {
        let settings = Settings::try_new().unwrap();
        let default = Settings::default();
        println!("{:?}", settings);
        assert_eq!(settings.server.host, default.server.host);
    }
}
