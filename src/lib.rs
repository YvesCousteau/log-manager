use std::{fs, path::PathBuf};

use error::LogManagerError;
use tracing::Level;
use tracing_appender::{non_blocking::WorkerGuard, rolling::RollingFileAppender};
use tracing_subscriber::prelude::*;

mod error;
mod util;

pub struct LogManager {
    pub log_level: Level,
    pub guard: WorkerGuard,
}

impl LogManager {
    pub fn new(
        log_level: &str,
        rotation_file: &str,
        max_log_files: usize,
    ) -> Result<Self, LogManagerError> {
        let log_level = util::get_log_level(log_level)?;
        let rotation_file = util::get_rotation_file(rotation_file)?;
        let bin_name = util::get_bin_name()?;

        let mut path = PathBuf::from("/var/log");
        path.push(bin_name.clone());

        fs::create_dir_all(&path)
            .map_err(|e| LogManagerError::DirectoryCreationFailed(e.to_string()))?;

        // automatically creates new log files every hour
        let file_appender = RollingFileAppender::builder()
            .rotation(rotation_file)
            .filename_prefix("prefix")
            .filename_suffix(".log")
            .max_log_files(max_log_files)
            .build(path.clone())
            .map_err(|err| LogManagerError::RollingFileFailed(err.to_string()))?;

        // allows the application to continue running without waiting for I/O operations to complete
        let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

        // configures two logging layers with different outputs and formatting
        // for stdout and for file
        let subscriber = tracing_subscriber::registry()
            .with(
                tracing_subscriber::fmt::Layer::new()
                    .with_writer(std::io::stdout)
                    .with_ansi(true)
                    .with_filter(tracing_subscriber::filter::LevelFilter::from_level(
                        log_level,
                    )),
            )
            .with(
                tracing_subscriber::fmt::Layer::new()
                    .with_writer(non_blocking)
                    .with_ansi(false)
                    .with_filter(tracing_subscriber::filter::LevelFilter::from_level(
                        log_level,
                    )),
            );

        tracing::subscriber::set_global_default(subscriber)
            .map_err(|e| LogManagerError::LogSubscriberFailed(e.to_string()))?;
        tracing::info!("{} logging files are set at: {:?}", bin_name, path);
        Ok(Self { log_level, guard })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_manager() {
        let result = LogManager::new("INFO", "HOURLY", 10);
        assert!(result.is_ok());
        let result = LogManager::new("to_fail", "HOURLY", 1);
        assert!(result.is_err());
        let result = LogManager::new("INFO", "to_fail", 1);
        assert!(result.is_err());
    }
}
