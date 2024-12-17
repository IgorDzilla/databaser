mod database;
mod query_builder;
mod readers;

use crate::query_builder::{Insert, ToQuery};
use database::configuration::print_config;
use database::definitions::*;
use database::errors::DataBaseError;
use database::STD_CONFIG_PATH;

use clap::{ArgGroup, Parser};
use rusqlite::{params, Connection, Result};
use std::collections::HashMap;

/// CLI args parser
#[derive(Parser, Debug)]
#[command(name = "lab_db")]
#[command(about = "An example CLI application", long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long)]
    name: String,
}

fn main() -> Result<(), DataBaseError> {
    let db = DataBase::from_config(STD_CONFIG_PATH)?;
    db.show_structure();
    db.construct_tables()?;
    db.insert(
        db.tables[0].name.as_str(),
        Some(&vec![
            DataType::Int(1),
            DataType::Int(2),
            DataType::Text("lab".to_string()),
            DataType::Text("12.12.24".to_string()),
            DataType::Int(3),
            DataType::Int(4),
        ]),
    )?;

    db.show_all_data()?;

    Ok(())
}
