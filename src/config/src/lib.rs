mod setting;

pub use crate::setting::DatabaseConfig;
pub use crate::setting::ServerConfig;
pub use crate::setting::Settings;

mod singleton;

pub use crate::singleton::AppConfig;

mod watcher;
