// This part of program is responsible for creating and executing generic sql queries

use std::collections::HashMap;
use std::fs;
use std::io;

use rusqlite::ffi::sqlite3_column_database_name;
use rusqlite::ToSql;

use super::definitions::{DataType, Table, SQL_STR_DTYPES};

const CREATE_TABLE_TEMPLATE: &str = "CREATE TABLE IF NOT EXISTS %table_name% (%columns%);";
const INSERT_VALUES_TEMPLATE: &str = "INSERT INTO %table_name% (%columns%) VALUES (%params%);";
const COUNT_ROWS_TEMPLATE: &str = "SELECT COUNT(*) FROM %table_name%;";
const SELECT_COLUMNS_TEMPLATE: &str = "SELECT (%columns%) FROM %table_name%;";

pub enum Query {
    Select(fn(&Table, &Vec<&str>) -> String),
    Other(String),
}

pub fn generic_queries_init(tables: &Vec<Table>) -> HashMap<&str, String> {
    if tables.len() == 1 {
        generic_queries_one_table(&tables[0])
    } else {
        todo!("This function is not implemented yet");
    }
}

fn generic_queries_one_table(table: &Table) -> HashMap<&str, String> {
    let mut queries: HashMap<&str, String> = HashMap::new();

    queries.insert("create_table", create_query_maker(table));
    queries.insert("insert_values", insert_values_maker(table));
    queries.insert("count_rows", rows_count_maker(table));

    queries
}

// a little bit smarter version of std var
pub fn smart_generic_queries_init(tables: &Vec<Table>) -> HashMap<&str, Query> {
    if tables.len() == 1 {
        smart_generic_queries_one_table(&tables[0])
    } else {
        todo!("Not implemented yet!");
    }
}

fn smart_generic_queries_one_table(table: &Table) -> HashMap<&str, Query> {
    let mut queries: HashMap<&str, Query> = HashMap::new();

    queries.insert("create_table", Query::Other(create_query_maker(table)));
    queries.insert("insert_values", Query::Other(insert_values_maker(table)));
    queries.insert("count_rows", Query::Other(rows_count_maker(table)));
    queries.insert("select", Query::Select(select_rows_constructor));

    queries
}

// makes a generic creation query for a table
fn create_query_maker(table: &Table) -> String {
    let result = str::replace(CREATE_TABLE_TEMPLATE, "%table_name%", &table.name);

    let mut column_definitions = String::new();
    for (idx, (col, dtype)) in table.columns.iter().enumerate() {
        column_definitions.push_str(format!("{} {}", col, match_dtype(dtype)).as_str());

        if col == table.primary_key.as_str() {
            column_definitions.push_str(" PRIMARY KEY")
        }

        if idx != table.columns.len() - 1 {
            column_definitions.push_str(", ");
        }
    }

    result.replace("%columns%", column_definitions.as_str())
}

fn insert_values_maker(table: &Table) -> String {
    let mut result = str::replace(INSERT_VALUES_TEMPLATE, "%table_name%", &table.name);

    let mut columns = String::new();
    let mut params = String::new();

    for (idx, (col, _dtype)) in table.columns.iter().enumerate() {
        columns.push_str(col);
        params.push('?');

        if idx != table.columns.len() - 1 {
            columns.push_str(", ");
            params.push_str(", ");
        }
    }

    result = result.replace("%columns%", columns.as_str());
    result.replace("%params%", params.as_str())
}

fn rows_count_maker(table: &Table) -> String {
    str::replace(COUNT_ROWS_TEMPLATE, "%table_name%", &table.name)
}

// The only public function from its family as SELECT statement cannot be parameterized, thus this
// query cannot be constructed in advance and needs to be assembled for each specific case
pub fn select_rows_constructor(table: &Table, columns: &Vec<&str>) -> String {
    let result = str::replace(SELECT_COLUMNS_TEMPLATE, "%table_name%", table.name.as_str());

    let mut insert = String::new();

    for (idx, col) in columns.iter().enumerate() {
        insert.push_str(col);
        if idx != columns.len() - 1 {
            insert.push_str(", ");
        }
    }

    result.replace("%columns%", insert.as_str())
}

fn match_dtype(dtype: &DataType) -> &str {
    match dtype {
        DataType::Int => "INTEGER",
        DataType::Float => "FLOAT",
        DataType::Bool => "BOOL",
        DataType::String => "TEXT",
    }
}
