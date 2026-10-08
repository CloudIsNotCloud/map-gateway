use config_internal::LoggingConfig;
use std::{error::Error, fmt::Display, fs::OpenOptions, io, sync::OnceLock};
use tracing::level_filters::LevelFilter;
use tracing_reload::{Handle as ReloadHandle, Layer as ReloadLayer, ReloadSubscriber};
use tracing_subscriber::{
    Layer, Registry, fmt::writer::BoxMakeWriter, layer::SubscriberExt, util::SubscriberInitExt,
};

use crate::LogError::{ConfigFail, InitFail};

type BoxedLayer = Box<dyn Layer<ReloadSubscriber<Registry>> + Send + Sync + 'static>;
type MyHandle = ReloadHandle<BoxedLayer, Registry>;

static RELOAD_HANDLER: OnceLock<MyHandle> = OnceLock::new();

#[derive(Debug)]
pub enum LogError {
    InitFail(String),
    ConfigFail(String),
}

impl Display for LogError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InitFail(s) => write!(f, "Initial Failure: {s}"),
            ConfigFail(s) => write!(f, "Config Failure: {s}"),
        }
    }
}

impl Error for LogError {}

pub fn minimal_init() -> Result<(), LogError> {
    let console_layer = console_subscriber::spawn();

    let fmt_layer = tracing_subscriber::fmt::layer()
        .compact()
        .with_file(true)
        .with_level(true)
        .with_thread_ids(true)
        .with_thread_names(true);

    let (reloadable_fmt_layer, reload_handler) = ReloadLayer::new(fmt_layer.boxed());
    RELOAD_HANDLER
        .set(reload_handler)
        .map_err(|_| LogError::InitFail("Initial log".to_string()))?;

    tracing_subscriber::registry()
        .with(reloadable_fmt_layer)
        .with(console_layer)
        .init();

    Ok(())
}

pub fn config_layer(cfg: &LoggingConfig) -> Result<(), LogError> {
    use config_internal::Sink;
    if let Some(handler) = RELOAD_HANDLER.get() {
        let tracing_level = cfg.level.to_tracing_level();
        // Enable ansi while outputing to controller
        let mut enable_ansi = true;
        let output: BoxMakeWriter = match cfg.sink {
            Sink::Stdout => BoxMakeWriter::new(std::io::stdout),
            Sink::Stderr => BoxMakeWriter::new(std::io::stderr),
            Sink::File { ref path } => {
                enable_ansi = false;
                BoxMakeWriter::new(
                    OpenOptions::new()
                        .create(true) // 如果不存在则创建
                        .append(true) // 追加模式
                        .open(path)
                        .unwrap(),
                )
            }
            _ => BoxMakeWriter::new(io::sink),
        };

        use config_internal::LogFormat;
        let new_fmt_layer = match cfg.format {
            LogFormat::Json => tracing_subscriber::fmt::layer()
                .json()
                .with_ansi(enable_ansi)
                .with_writer(output)
                .with_file(true)
                .with_level(true)
                .with_thread_ids(true)
                .with_thread_names(true)
                .with_filter(LevelFilter::from_level(tracing_level))
                .boxed(),
            LogFormat::Compact => tracing_subscriber::fmt::layer()
                .compact()
                .with_ansi(enable_ansi)
                .with_writer(output)
                .with_file(true)
                .with_level(true)
                .with_thread_ids(true)
                .with_thread_names(true)
                .with_filter(LevelFilter::from_level(tracing_level))
                .boxed(),
        };

        handler
            .reload(new_fmt_layer)
            .map_err(|_| LogError::ConfigFail("Configuration".to_string()))?;
    }
    Ok(())
}
