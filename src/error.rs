use thiserror::Error;

#[derive(Error, Debug)]
pub enum LogManagerError {
    #[error("Log level format is invalid")]
    InvalidLogLevelFormat,
    #[error("Rotation file format is invalid")]
    InvalidRotationFileFormat,
    #[error("Bin path not found: {0}")]
    BinPathNotFound(String),
    #[error("Bin name not found")]
    BinNameNotFound,
    #[error("Directory data local not found")]
    DirectoryDataLocalNotFound,
    #[error("Directory creation failed: {0}")]
    DirectoryCreationFailed(String),
    #[error("Log subscriber failed: {0}")]
    LogSubscriberFailed(String),
    #[error("Rolling file failed: {0}")]
    RollingFileFailed(String),
}
