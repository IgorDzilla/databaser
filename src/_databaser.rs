use rusqlite::{Connection, Params, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use toml;

pub const CONFIG_PATH: &str = "config.toml";

///////////// ERROR DEFINITIONS ///////////////
#[derive(Debug)]
pub enum DataBaseError {
    IoError(std::io::Error),
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
///////////////////////////////

/////////// CONFIG PARSING ///////////
#[derive(Debug, Deserialize)]
pub struct Config {
    pub database: DatabaseConfig,
    pub tables: Vec<TableConfig>,
}

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub path: String,
}

#[derive(Debug, Deserialize)]
pub struct TableConfig {
    pub name: String,
    pub columns: Vec<String>,
    pub column_dtypes: Vec<String>,
    pub primary_key: String,
}

pub fn print_config(config_path: &str) {
    // Read the TOML configuration file to a string
    let toml_content = fs::read_to_string(config_path).expect("Failed to read config file");

    // Parse the TOML content
    let config: Config = toml::from_str(&toml_content).expect("Failed to parse TOML");

    // Print the configuration details
    println!("Database Configuration:");
    println!("  Path: {}", config.database.path);

    println!("\nTables:");
    for (i, table) in config.tables.iter().enumerate() {
        println!("\nTable {}: {}", i + 1, table.name);
        println!("  Primary Key: {}", table.primary_key);
        println!("  Columns:");
        for (j, column) in table.columns.iter().enumerate() {
            let column_type = &table.column_dtypes[j];
            println!("    {}: {}", column, column_type);
        }
    }
}
/////////////////////////////////////

/// DB contents info
#[derive(Debug)]
pub struct EquipmentInfo {
    pub name: String,
    pub serial_number: u32,
    pub unit: String,
    pub inventory_number: u32,
    pub supply_date: String,
    pub supply_number: u32,
    pub supply_doc_number: u32,
}

#[derive(Debug, Clone)]
pub enum DataType {
    Int = 0,
    Float = 1,
    Bool = 2,
    String = 3,
}

// data types storages
// не лучший вариант, но пусть пока будет так
pub const SQL_STR_DTYPES: [&str; 4] = ["INTEGER", "FLOAT", "BOOL", "TEXT"];
pub const RUST_DTYPES: [DataType; 4] = [
    DataType::Int,
    DataType::Float,
    DataType::Bool,
    DataType::String,
];

#[derive(Clone, Debug)]
pub struct Table {
    name: String,
    columns: Vec<(String, DataType)>, // name of each column and its data type
    primary_key: String,
}

pub struct DataBase {
    pub connection: Connection,
    pub tables: Vec<Table>,
    pub querry_templates: Vec<String>,
}

impl DataBase {
    pub fn from_config() -> Result<Self, DataBaseError> {
        // read config to string
        let toml_content = fs::read_to_string(CONFIG_PATH).map_err(DataBaseError::from)?;
        // parse it
        let config: Config = toml::from_str(&toml_content).map_err(DataBaseError::from)?;

        let connection: Connection = Connection::open(config.database.path.clone())?;
        let tables: Vec<Table> = parse_tables_configs(&config)?;
        let querry_templates: Vec<String> = Vec::new();

        Ok(DataBase {
            connection,
            tables,
            querry_templates,
        })
    }

    pub fn show_contents(&self) {
        println!("Database contains {} tables:", self.tables.len());

        // Display each table's details
        for table in &self.tables {
            println!("Table: {}", table.name);
            println!("  Primary Key: {}", table.primary_key);
            println!("  Columns:");

            for (column_name, column_type) in &table.columns {
                let column_type_str = match column_type {
                    DataType::Int => "INT",
                    DataType::String => "TEXT",
                    DataType::Float => "FLOAT",
                    DataType::Bool => "BOOL",
                };
                println!("    {}: {}", column_name, column_type_str);
            }
            println!();
        }
    }
}

//////// AUXILLARY FUNCTIONS ////////
fn parse_tables_configs(config: &Config) -> Result<Vec<Table>, DataBaseError> {
    let mut tables: Vec<Table> = Vec::new();

    for table_config in &config.tables {
        let mut columns: Vec<(String, DataType)> =
            parse_columns(&table_config).map_err(DataBaseError::from)?;
        tables.push(Table {
            name: table_config.name.clone(),
            primary_key: table_config.primary_key.clone(),
            columns,
        });
    }

    Ok(tables)
}

fn parse_columns(table_config: &TableConfig) -> Result<Vec<(String, DataType)>, DataBaseError> {
    if table_config.columns.len() != table_config.column_dtypes.len() {
        return Err(DataBaseError::ConfigError(format!(
            "Mismatch between number of columns ({}) and data types ({})",
            table_config.columns.len(),
            table_config.column_dtypes.len()
        )));
    }

    let mut columns_wdtypes: Vec<(String, DataType)> = Vec::new();
    for i in 0..table_config.columns.len() {
        columns_wdtypes.push((
            String::from(table_config.columns[i].clone()),
            match table_config.column_dtypes[i].as_str() {
                "INTEGER" => DataType::Int,
                "FLOAT" => DataType::Float,
                "BOOL" => DataType::Bool,
                "TEXT" => DataType::String,
                _ => {
                    return Err(DataBaseError::ConfigError(format!(
                        "Error parsing config: uknown data type \"{}\" in table \"{}\"",
                        table_config.column_dtypes[i], table_config.name
                    )))
                }
            },
        ))
    }

    return Ok(columns_wdtypes);
}
////////////////////////
