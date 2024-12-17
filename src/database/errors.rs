use csv;
use rusqlite::{Connection, Params, Result};
use toml;

///////////// ERROR DEFINITIONS ///////////////
#[derive(Debug)]
pub enum DataBaseError {
    CSVError(csv::Error),
    IoError(std::io::Error),
    TableNotFound(String),
    TomlError(toml::de::Error),
    SqliteError(rusqlite::Error),
    ConfigError(String),
}

impl From<String> for DataBaseError {
    fn from(err: String) -> Self {
        DataBaseError::ConfigError(err)
    }
}

impl From<std::io::Error> for DataBaseError {
    fn from(err: std::io::Error) -> Self {
        DataBaseError::IoError(err)
    }
}

impl From<toml::de::Error> for DataBaseError {
    fn from(err: toml::de::Error) -> Self {
        DataBaseError::TomlError(err)
    }
}

impl From<rusqlite::Error> for DataBaseError {
    fn from(err: rusqlite::Error) -> Self {
        DataBaseError::SqliteError(err)
    }
}

impl From<csv::Error> for DataBaseError {
    fn from(err: csv::Error) -> Self {
        DataBaseError::CSVError(err)
    }
}
