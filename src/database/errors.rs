#[derive(Debug)]
pub enum DataBaseError {
    IoError(std::io::Error),
    TableNotFound(String),
    JsonError(serde_json::Error),
    SqliteError(rusqlite::Error),
    ConfigError(String),
}

impl From<serde_json::Error> for DataBaseError {
    fn from(err: serde_json::Error) -> Self {
        DataBaseError::JsonError(err)
    }
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

impl From<rusqlite::Error> for DataBaseError {
    fn from(err: rusqlite::Error) -> Self {
        DataBaseError::SqliteError(err)
    }
}
