use serde::{Deserialize, Serialize};
use std::fs;
use toml;

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
    pub columns_dtypes: Vec<Vec<String>>,
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
        println!("  Columns:");
        for col_info in table.columns_dtypes.iter() {
            println!("    {}: {}", col_info[0], col_info[1]);
        }
    }
}
