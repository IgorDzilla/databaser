use rusqlite::{Connection, Params, Result};
use toml;

#[derive(Debug, Clone)]
pub enum DataType {
    Int(i32),
    Float(f32),
    Bit(bool),
    Text(String),
    Param(String),
}

// data types storages
// не лучший вариант, но пусть пока будет так
pub const SQL_STR_DTYPES: [&str; 4] = ["INTEGER", "FLOAT", "BOOL", "TEXT"];

#[derive(Clone, Debug)]
pub struct Table {
    pub name: String,
    pub columns: Vec<(String, String)>, // name of each column and its data type
    pub primary_key: String,
}

pub struct DataBase {
    pub connection: Connection,
    pub tables: Vec<Table>,
}
