pub mod configuration;
pub mod definitions;
pub mod errors;

use super::query_builder as qr;
use super::query_builder::ToQuery;

use configuration::*;
use definitions::*;
use errors::*;

use csv::{ReaderBuilder, WriterBuilder};
use rusqlite::{params, Connection, Params, Result};
use std::error::Error;
use std::fs;
use std::fs::File;
use std::path::Path;

pub const STD_CONFIG_PATH: &str = "config.toml";

impl DataBase {
    pub fn from_config(config_path: &str) -> Result<Self, DataBaseError> {
        // read config to string
        let toml_content = fs::read_to_string(config_path).map_err(DataBaseError::from)?;
        // parse it
        let config: Config = toml::from_str(&toml_content).map_err(DataBaseError::from)?;

        let connection: Connection = Connection::open(config.database.path.clone())?;
        let tables: Vec<Table> = parse_tables_configs(&config)?;

        Ok(DataBase { connection, tables })
    }

    pub fn from_file(db_path: &str) -> Result<Self, DataBaseError> {
        let connection = Connection::open(db_path)?;
        let tables: Vec<Table> = get_tables_from_file(&connection)?;

        Ok(DataBase { connection, tables })
    }

    pub fn show_structure(&self) {
        println!("Database contains {} tables:", self.tables.len());
        for (i, tbl) in self.tables.iter().enumerate() {
            println!("\t{} TABLE {}", i, tbl.name);
            for col in &tbl.columns {
                println!("\t\t{} {}", col.0, col.1);
            }
        }
    }

    pub fn table_shape(self: &Self, table_name: &str) -> Result<(usize, usize), DataBaseError> {
        if let Some(table_idx) = self.tables.iter().position(|x| x.name == table_name) {
            let count = self
                .connection
                .query_row(&qr::Count::new(table_name).to_query(), [], |row| row.get(0))
                .map_err(DataBaseError::from)?;
            return Ok((self.tables[table_idx].columns.len(), count));
        }
        Err(DataBaseError::TableNotFound(format!(
            "Could not find table \"{}\"",
            table_name
        )))
    }

    pub fn create_table(
        self: &Self,
        table_name: &str,
        columns: &Vec<(String, String)>,
    ) -> Result<(), DataBaseError> {
        self.connection
            .execute(
                qr::CreateTable::new(table_name)
                    .columns(&get_col_names(columns))
                    .to_query()
                    .as_str(),
                params![],
            )
            .map_err(DataBaseError::from)?;

        Ok(())
    }

    pub fn insert(
        self: &Self,
        table_name: &str,
        vals: Option<&Vec<DataType>>,
    ) -> Result<(), DataBaseError> {
        if let Some(table_idx) = self.tables.iter().position(|x| x.name == table_name) {
            let q = qr::Insert::new(table_name)
                .columns(&get_col_names(&self.tables[table_idx].columns))
                .values(vals.cloned())
                .to_query();
            println!("{}", q);
            self.connection
                .execute(q.as_str(), params![])
                .map_err(DataBaseError::from)?;

            return Ok(());
        } else {
            return Err(DataBaseError::TableNotFound(format!(
                "No table \"{}\" found in database ",
                table_name
            )));
        }
    }

    pub fn construct_tables(self: &Self) -> Result<(), DataBaseError> {
        for table in &self.tables {
            self.create_table(&table.name, &table.columns)?;
        }
        Ok(())
    }
    
    /*
    pub fn create_from_csv(self: &mut Self, path: &str) -> Result<(), DataBaseError> {
        let file = File::open(path).map_err(|e| DataBaseError::IoError(e))?;
        let mut rdr = ReaderBuilder::new().has_headers(true).from_reader(file);

        // Read headers to match with table columns
        let headers = rdr.headers().map_err(DataBaseError::from)?;
        let columns: Vec<String> = headers.iter().map(|s| s.to_string()).collect();
        let mut col_defs: Vec<(String, String)> = Vec::new();
        for col in columns {
            col_defs.push((col, "TEXT".to_string()));
        }

        // Ensure the table exists, create it if needed
        let table_name = &path[0..path.find(".").unwrap()];
        self.tables.push(Table {
            columns: col_defs.clone(),
            primary_key: "".to_string(),
            name: table_name.to_string(),
        });

        self.create_table(table_name, &col_defs)?;

        // For each record in the CSV, insert the data into the database
        for result in rdr.records() {
            let record = result.map_err(DataBaseError::from)?;
            let values: Vec<String> = record.iter().map(|r| r.to_string()).collect();
            self.insert(table_name, Some(&values))?;
        }

        Ok(())
    }

    pub fn to_csv(&self, path: &str, table_name: &str) -> Result<(), DataBaseError> {
        // Ensure the table exists
        if !self.tables.iter().any(|table| table.name == table_name) {
            return Err(DataBaseError::TableNotFound(format!(
                "Table '{}' not found.",
                table_name
            )));
        }

        // Get the data from the table using the select method
        let query = format!("SELECT * FROM {}", table_name);
        let mut stmt = self
            .connection
            .prepare(&query)
            .map_err(DataBaseError::from)?;

        // Get column names from the query result
        let mut column_names: Vec<String> = Vec::new();
        for col in stmt.column_names() {
            column_names.push(col.to_string());
        }

        // Create a CSV file and write the column names as headers
        let file = File::create(path).map_err(DataBaseError::from)?;
        let mut wtr = WriterBuilder::new().has_headers(true).from_writer(file);
        wtr.write_record(column_names.clone())
            .map_err(DataBaseError::from)?;

        let rows = stmt
            .query_map([], |row| {
                let mut record = Vec::new();
                // Use `&column_names` to avoid moving the vector
                for col in &column_names {
                    record.push(row.get::<_, String>(col.as_str())?);
                }
                Ok(record)
            })
            .map_err(DataBaseError::from)?;

        // Write each row into the CSV
        //
        for row_result in rows {
            let row = row_result.map_err(DataBaseError::from)?;
            wtr.write_record(row).map_err(DataBaseError::from)?;
        }

        // Ensure all data is written to the file
        wtr.flush().map_err(DataBaseError::from)?;

        Ok(())
    }

    */
    pub fn show_all_data(&self) -> Result<(), DataBaseError> {
        // Iterate over all tables in the database
        for table in &self.tables {
            println!("Table: {}", table.name);
    
            // Execute a SELECT query to get all rows from the current table
            let query = format!("SELECT * FROM {}", table.name);
            let mut stmt = self
                .connection
                .prepare(&query)
                .map_err(DataBaseError::from)?;
            let column_names = stmt.column_names().to_vec(); // Get column names for display
    
            let rows = stmt
                .query_map([], |row| {
                    let mut row_data = Vec::new();
                    for (col_idx, _) in &table.columns {
                        let value = row.get::<usize, rusqlite::types::Value>(*col_idx); // Use rusqlite's Value type
                        match value {
                            Ok(val) => {
                                // Convert the value to a human-readable string representation
                                row_data.push(match val {
                                    rusqlite::types::Value::Null => "NULL".to_string(),
                                    rusqlite::types::Value::Integer(i) => i.to_string(),
                                    rusqlite::types::Value::Real(f) => f.to_string(),
                                    rusqlite::types::Value::Text(s) => s,
                                    rusqlite::types::Value::Blob(_) => "[BLOB]".to_string(),
                                });
                            }
                            Err(e) => return Err(e), // Convert rusqlite::Error into DataBaseError
                        }
                    }
                    Ok(row_data)
                })
                .map_err(DataBaseError::from)?;
    
            // Print the column names
            println!("{:?}", column_names);
    
            // Iterate through the rows and print them
            for row_result in rows {
                match row_result {
                    Ok(row) => {
                        println!("{:?}", row); // Print each row (formatted)
                    }
                    Err(e) => {
                        eprintln!("Error reading row: {}", e);
                    }
                }
            }
            println!("-----------------------------------------"); // Separator for readability
        }
        Ok(())
    }
    
}

//////// AUXILLARY FUNCTIONS ////////
fn parse_tables_configs(config: &Config) -> Result<Vec<Table>, DataBaseError> {
    let mut tables: Vec<Table> = Vec::new();

    for table_config in &config.tables {
        let (columns, pkey) = parse_columns(&table_config).map_err(DataBaseError::from)?;
        tables.push(Table {
            name: table_config.name.clone(),
            primary_key: pkey,
            columns,
        });
    }

    Ok(tables)
}

fn parse_columns(
    table_config: &TableConfig,
) -> Result<(Vec<(String, String)>, String), DataBaseError> {
    let mut cols_defs: Vec<(String, String)> = Vec::new();
    let mut pkey = String::new();
    for col_def in &table_config.columns_dtypes {
        if col_def.len() != 2 {
            return Err(DataBaseError::ConfigError(
                "Syntax error at definitions of columns".to_string(),
            ));
        }
        // add checks of types
        if col_def[1].contains("PRIMARY KEY") {
            pkey = col_def[0].clone();
        }

        cols_defs.push((col_def[0].clone(), col_def[1].clone()));
    }

    Ok((cols_defs, pkey))
}

fn get_col_names(cols_defs: &Vec<(String, String)>) -> Vec<String> {
    let mut cols = Vec::new();
    for col_def in cols_defs.iter() {
        cols.push(col_def.0.to_string());
    }

    cols
}

fn get_table_schema(
    conn: &Connection,
    table_name: &str,
) -> Result<(Vec<(String, String)>, String)> {
    let query = format!("PRAGMA table_info('{}');", table_name);
    let mut stmt = conn.prepare(&query)?;
    let schema_iter = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(1)?,  // Column name
            row.get::<_, String>(2)?,  // Column type
            row.get::<_, i32>(5)? > 0, // Primary key flag (pk > 0 means it’s a primary key)
        ))
    })?;

    let mut schema = Vec::new();
    let mut pk = String::new();
    for column in schema_iter {
        let (col_name, col_type, pk_status) = column?;

        schema.push((col_name.clone(), col_type.clone()));
        if pk_status {
            pk = col_name.clone();
        }
    }

    Ok((schema, pk))
}

fn get_tables_from_file(connection: &Connection) -> Result<Vec<Table>> {
    let mut query = connection.prepare("SELECT name FROM sqlite_master WHERE type='table';")?;
    let table_iter: Result<Vec<String>> = query
        .query_map([], |row| Ok(row.get::<_, String>(0)?))?
        .collect();

    let mut tables: Vec<Table> = Vec::new();

    let tables_names: Vec<String> = table_iter?;
    for tab_name in tables_names {
        let (cols, pk) = get_table_schema(&connection, &tab_name)?;
        tables.push(Table {
            name: tab_name,
            columns: cols,
            primary_key: pk,
        })
    }

    Ok(tables)
}
