use rusqlite;

#[derive(Debug, Clone)]
pub enum SqlConstraints {
    PrimaryKey,
    NotNull,
    Unique,
    Default(String),
    Autoincrement,
}

#[derive(Debug, Clone)]
pub struct Column {
    pub name: String,
    pub d_type: rusqlite::types::Type,
    pub constraint: Option<SqlConstraints>,
}

pub struct Table {
    pub name: String,
    pub columns: Vec<Column>,
}

pub struct DataBase {
    pub file_path: Option<String>,
    pub connection: rusqlite::Connection,
    pub tables: Vec<Table>,
}
