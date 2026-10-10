use std::{path::PathBuf, time::Duration};

use notify::{Error, RecommendedWatcher, RecursiveMode};
use notify_debouncer_full::{DebounceEventResult, Debouncer, RecommendedCache, new_debouncer};
use tokio::sync::mpsc::{self, Receiver};

#[derive(Debug)]
pub(crate) struct ConfigFilesWatcher {
    _debouncer: Debouncer<RecommendedWatcher, RecommendedCache>,
    receiver: Receiver<DebounceEventResult>,
}

impl ConfigFilesWatcher {
    pub(crate) fn try_new(path: PathBuf) -> Result<Self, Error> {
        let (tx, rx) = mpsc::channel::<DebounceEventResult>(218);
        let mut debouncer = new_debouncer(
            Duration::from_secs(2),
            None,
            move |result: DebounceEventResult| {
                // OS thread of notify, do not panic
                if let Err(e) = tx.blocking_send(result) {
                    eprintln!("Send message error: {:?}", e)
                }
            },
        )?;

        debouncer.watch(path, RecursiveMode::NonRecursive)?;

        Ok(ConfigFilesWatcher {
            _debouncer: debouncer,
            receiver: rx,
        })
    }

    pub(crate) async fn recv(&mut self) -> Option<DebounceEventResult> {
        self.receiver.recv().await
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs::{self, OpenOptions},
        io::Write,
        path::Path,
    };

    use tokio::time::timeout;

    use super::*;

    type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;

    #[tokio::test]
    async fn test_watcher() -> Result<(), BoxError> {
        let path = Path::new("temp/config.toml");
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut config_file = OpenOptions::new().create(true).append(true).open(path)?;
        let mut watcher = ConfigFilesWatcher::try_new(PathBuf::from(path))?;

        config_file.write_all(b"[config]\n")?;
        config_file.write_all(b"fuck = \"you\"\n")?;
        config_file.sync_all()?;

        let res = timeout(Duration::from_secs(10), watcher.recv())
            .await
            .map_err(|_| "recv timed out: no event arrived within 10s")?;

        if res.is_some() {
            Ok(())
        } else {
            Err("watcher channel closed without any event".into())
        }
    }
}
