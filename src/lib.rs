use std::{fs, path::PathBuf};

use chrono::Utc;
use error::LogManagerError;
use tracing_appender::{non_blocking::WorkerGuard, rolling::RollingFileAppender};
use tracing_subscriber::prelude::*;

pub mod error;
mod util;

/// Represents the log manager on which all log behavior operations will be setted.
///
/// # Fields
/// - `guard` : A guard that flushes spans/events associated to a `NonBlocking`.
/// - `path`  : A path where file is stored.
pub struct LogManager {
    pub path: PathBuf,
    pub guard: WorkerGuard,
}

impl LogManager {
    /// Initializes a new `LogManager` with the given log level, the rotation file and the max log
    /// files.
    ///
    /// # Arguments
    ///
    /// * `log_level`     - The log level.
    /// * `rotation_file` - The rotation file period.
    /// * `max_log_files` - The limit of kepted files.
    ///
    /// # Returns
    ///
    /// Returns a new `LogManager` instance if successful.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// * The log level is invalid.
    /// * The rotation file period is invalid.
    /// * The directory creation failed.
    /// * The generation of file appender failed.
    /// * The creation of log subscriber failed.
    pub fn new(
        log_level: &str,
        rotation_file: &str,
        max_log_files: usize,
    ) -> Result<Self, LogManagerError> {
        let log_level = util::get_log_level(log_level)?;
        let rotation_file = util::get_rotation_file(rotation_file)?;
        let bin_name = util::get_bin_name()?;

        let mut path =
            dirs_2::data_local_dir().ok_or(LogManagerError::DirectoryDataLocalNotFound)?;
        path.push(bin_name.clone());

        fs::create_dir_all(&path)
            .map_err(|e| LogManagerError::DirectoryCreationFailed(e.to_string()))?;

        // automatically creates new log files every hour
        let file_appender = RollingFileAppender::builder()
            .rotation(rotation_file)
            .filename_prefix(Utc::now().to_rfc3339())
            .filename_suffix("log")
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
        Ok(Self { path, guard })
    }
}

#[cfg(test)]
mod tests {
    use regex::Regex;
    use serial_test::serial;

    use super::*;

    #[serial]
    #[test]
    fn test_invalid_log_level() {
        let result = LogManager::new("to_fail", "HOURLY", 1);
        match result {
            Err(LogManagerError::InvalidLogLevelFormat) => (),
            _ => panic!("Expected InvalidLogLevelFormat error"),
        }
    }

    #[serial]
    #[test]
    fn test_invalid_rotation_file() {
        let result = LogManager::new("INFO", "to_fail", 1);
        match result {
            Err(LogManagerError::InvalidRotationFileFormat) => (),
            _ => panic!("Expected InvalidRotationFileFormat error"),
        }
    }

    #[serial]
    #[test]
    fn test_valid_log_manager_creation() {
        let result = LogManager::new("TRACE", "HOURLY", 10);
        assert!(result.is_ok());

        tracing::info!("test valid log");
        tracing::error!("test valid log");
        tracing::warn!("test valid log");
        tracing::debug!("test valid log");
        tracing::trace!("test valid log");

        let dir_path = result.unwrap().path;

        let mut entries: Vec<fs::DirEntry> = fs::read_dir(dir_path.clone())
            .expect("Failed to read directory")
            .filter_map(Result::ok)
            .collect();

        entries.sort_by(|a, b| {
            let a_metadata = a.metadata().expect("Failed to get file metadata");
            let b_metadata = b.metadata().expect("Failed to get file metadata");

            a_metadata
                .created()
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
                .cmp(
                    &b_metadata
                        .created()
                        .unwrap_or(std::time::SystemTime::UNIX_EPOCH),
                )
        });

        let contents = fs::read_to_string(entries.last().unwrap().path())
            .expect("Should have been able to read the file");

        let re = Regex::new(r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d+Z")
            .expect("Regex syntaxe is invalid");
        let cleaned_contents = re.replace_all(&contents, "").to_string();

        let dir_name = dir_path
            .as_path()
            .file_name()
            .expect("Failed to get file name")
            .to_str()
            .expect("Failed to parse path as string");

        assert_eq!(
            cleaned_contents.trim(),
            format!(
                "INFO log_manager: {dir_name} logging files are set at: {dir_path:?}\n  INFO log_manager::tests: test valid log\n ERROR log_manager::tests: test valid log\n  WARN log_manager::tests: test valid log\n DEBUG log_manager::tests: test valid log\n TRACE log_manager::tests: test valid log",
            ),
            "Stdout and file content do not match"
        );
    }
}
