use std::{ffi::OsStr, str::FromStr};

use tracing::Level;
use tracing_appender::rolling::Rotation;

use crate::error::Error;

pub fn get_log_level(level: &str) -> Result<Level, Error> {
    Level::from_str(level).map_err(|_| Error::InvalidLogLevelFormat)
}

pub fn get_rotation_file(rotation_file: &str) -> Result<Rotation, Error> {
    match rotation_file {
        "MINUTELY" => Ok(Rotation::MINUTELY),
        "HOURLY" => Ok(Rotation::HOURLY),
        "DAILY" => Ok(Rotation::DAILY),
        "NEVER" => Ok(Rotation::NEVER),
        _ => Err(Error::InvalidRotationFileFormat),
    }
}

pub fn get_bin_name() -> Result<String, Error> {
    let bin_path = std::env::current_exe().map_err(|e| Error::BinPathNotFound(e.to_string()))?;
    Ok(bin_path
        .file_name()
        .and_then(OsStr::to_str)
        .ok_or(Error::BinNameNotFound)?
        .to_string())
}
