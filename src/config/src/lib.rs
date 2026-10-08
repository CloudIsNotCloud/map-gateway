mod setting;

pub use crate::setting::DatabaseConfig;
pub use crate::setting::LogFormat;
pub use crate::setting::LogLevel;
pub use crate::setting::LoggingConfig;
pub use crate::setting::ServerConfig;
pub use crate::setting::Settings;
pub use crate::setting::Sink;

mod singleton;

pub use crate::singleton::AppConfig;

mod watcher;
