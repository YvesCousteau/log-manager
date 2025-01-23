use std::{ffi::OsStr, str::FromStr};

use tracing::Level;
use tracing_appender::rolling::Rotation;

use crate::error::LogManagerError;

pub fn get_log_level(level: &str) -> Result<Level, LogManagerError> {
    Level::from_str(level).map_err(|_| LogManagerError::InvalidLogLevelFormat)
}

pub fn get_rotation_file(rotation_file: &str) -> Result<Rotation, LogManagerError> {
    match rotation_file {
        "MINUTELY" => Ok(Rotation::MINUTELY),
        "HOURLY" => Ok(Rotation::HOURLY),
        "DAILY" => Ok(Rotation::DAILY),
        "NEVER" => Ok(Rotation::NEVER),
        _ => Err(LogManagerError::InvalidRotationFileFormat),
    }
}

pub fn get_bin_name() -> Result<String, LogManagerError> {
    let bin_path =
        std::env::current_exe().map_err(|e| LogManagerError::BinPathNotFound(e.to_string()))?;
    Ok(bin_path
        .file_name()
        .and_then(OsStr::to_str)
        .ok_or(LogManagerError::BinNameNotFound)?
        .to_string())
}
