use crate::database::definitions::{Column, SqlConstraints};
use rusqlite::types::{Type, Value};
use std::error;
use std::fmt;

#[derive(Debug, Clone)]
struct  SqlBuilderError {
    msg: String,
}

impl From<String> for SqlBuilderError {
    fn from(value: String) -> Self {
        SqlBuilderError {
            msg: value
        }
    }
}

impl From<&str> for SqlBuilderError {
    fn from(value: &str) -> Self {
        SqlBuilderError{
            msg: value.to_string()
        }
    }
}

pub struct CreateTable {
    table_name: String,
    columns: Option<Vec<Column>>,
    if_not_exists: bool,
}

pub struct Select {
    table_name: String,
    columns: Option<Vec<Column>>,
    condition: Option<String>,
}

pub struct Insert {
    table_name: String,
    columns: Option<Vec<Column>>,
    vals: Option<Vec<rusqlite::types::Value>>,
}

pub struct Update {}

pub struct Delete {}

pub struct Count {
    table_name: String,
    column: Option<Column>,
    distinct: bool,
    condition: Option<String>,
}

pub trait ToQuery {
    fn to_query(self) -> Result<String, SqlBuilderError>;
}
impl CreateTable {
    pub fn new(table_name: &str) -> Self {
        CreateTable {
            table_name: table_name.to_string(),
            columns: None,
            if_not_exists: true,
        }
    }

    pub fn columns(mut self, columns: &Vec<Column>) -> Self {
        self.columns = Some(columns.to_owned());
        self
    }

    pub fn if_not_exists(mut self, state: bool) -> Self {
        self.if_not_exists = state;
        self
    }
}

impl ToQuery for CreateTable {
    fn to_query(self) -> Result<String, SqlBuilderError>{
        if self.columns.is_none() {
            return Err(SqlBuilderError::from("Can not create query with no columns".to_string()));
        }

        let cols = self.columns.unwrap();

        let mut pk_found = false;

        let mut columns_insert = String::new();
        for (idx, col) in cols.iter().enumerate() {
            columns_insert.push_str(&col.name);

            match &col.d_type {
                Type::Integer => columns_insert.push_str(" INTEGER"),
                Type::Real => columns_insert.push_str(" REAL"),
                Type::Text => columns_insert.push_str(" TEXT"),
                Type::Blob => columns_insert.push_str(" BLOB"),
                Type::Null => return Err(SqlBuilderError::from("ToQuery: CreateTable: column type can't be Null")),
            }

            match &col.constraint {
                Some(c) => match c {
                    SqlConstraints::PrimaryKey => {
                        if !pk_found {
                            columns_insert.push_str(" PRIMARY KEY");
                            pk_found = true;
                        } else {
                            return Err(SqlBuilderError::from("ToQuery: CreateTable: table can't have two primary keys"));
                        }
                    }
                    SqlConstraints::Autoincrement => columns_insert.push_str(" AUTOINCREMENT"),
                    SqlConstraints::Default(val) => {
                        columns_insert.push_str(&format!(" DEFAULT \'{}\'", val))
                    }
                    SqlConstraints::NotNull => columns_insert.push_str(" NOT NULL"),
                    SqlConstraints::Unique => columns_insert.push_str(" UNIQUE"),
                },
                None => {}
            }

            if idx != cols.len() - 1 {
                columns_insert.push_str(", ");
            }
        }

        Ok(format!(
            "CREATE TABLE {}{} ({});",
            if self.if_not_exists {
                "IF NOT EXISTS "
            } else {
                ""
            },
            self.table_name,
            columns_insert
        ))
    }
}

impl Select {
    pub fn new(table_name: &str) -> Self {
        Select {
            table_name: table_name.to_string(),
            columns: None,
            condition: None,
        }
    }

    pub fn columns(mut self, columns: &Vec<Column>) -> Self {
        self.columns = Some(columns.to_owned());
        self
    }

    pub fn condition(mut self, condition: &str) -> Self {
        self.condition = Some(condition.to_string());
        self
    }
}

impl ToQuery for Select {
    fn to_query(self) -> Result<String, SqlBuilderError> {
        let columns_insert = match &self.columns {
            Some(columns) => {
                let mut s = String::new();
                for (idx, col) in columns.iter().enumerate() {
                    s.push_str(col.name.as_str());
                    if idx != columns.len() - 1 {
                        s.push_str(", ");
                    }
                }
                s
            }
            None => "*".to_string(),
        };

        let conditions_insert = match &self.condition {
            Some(conditions) => format!(" WHERE {}", conditions),
            None => "".to_string(),
        };

        Ok(format!(
            "SELECT {} FROM {}{};",
            columns_insert, self.table_name, conditions_insert,
        ))
    }
}

impl Insert {
    pub fn new(table_name: &str) -> Self {
        Insert {
            table_name: table_name.to_string(),
            columns: None,
            vals: None,
        }
    }

    pub fn columns(mut self, columns: &Vec<Column>) -> Self {
        self.columns = Some(columns.to_owned());
        self
    }

    pub fn vals(mut self, vals: &Vec<rusqlite::types::Value>) -> Self {
        self.vals = Some(vals.to_owned());
        self
    }
}

impl ToQuery for Insert {
    fn to_query(self) -> Result<String, SqlBuilderError>{
        let columns_insert = match &self.columns {
            Some(columns) => {
                let mut s = String::new();
                let l = columns.len();
                for (idx, col) in columns.iter().enumerate() {
                    s.push_str(col.name.as_str());
                    if idx != l - 1 {
                        s.push_str(", ");
                    }
                }
                (s, l)
            }
            None => return Err(SqlBuilderError::from("ToQuery: Insert: columns can't be None.")),
        };

        let vals_insert = match &self.vals {
            Some(vals) => {
                if vals.len() != columns_insert.1 {
                    return Err(SqlBuilderError::from("ToQuery: Insert: columns' values' sizes do not match."));
                }
                let mut s = String::new();
                for (idx, val) in vals.iter().enumerate() {
                    match val {
                        Value::Text(t) => s.push_str(&format!("\'{}\'", t)),
                        Value::Blob(_b) => return Err(SqlBuilderError::from("ToQuery: Insert: BLOB must be parameterized.")),
                        Value::Integer(i) => s.push_str(i.to_string().as_str()),
                        Value::Real(r) => s.push_str(r.to_string().as_str()),
                        Value::Null => s.push_str("NULL"),
                    }
                    if idx != vals.len() - 1 {
                        s.push_str(", ");
                    }
                }
                s
            }
            None => {
                let mut s = String::new();
                for idx in 0..columns_insert.1 {
                    s.push_str("?");
                    if idx != columns_insert.1 - 1 {
                        s.push_str(", ");
                    }
                }
                s
            }
        };

        Ok(format!(
            "INSERT INTO {} ({}) VALUES ({});",
            self.table_name, columns_insert.0, vals_insert
        ))
    }
}

impl Count {
    pub fn new(table_name: &str) -> Self {
        Count {
            table_name: table_name.to_string(),
            column: None,
            distinct: false,
            condition: None,
        }
    }

    pub fn column(mut self, column: &Column) -> Self {
        self.column = Some(column.clone());
        self
    }

    pub fn condition(mut self, condition: &str) -> Self {
        self.condition = Some(condition.to_string());
        self
    }

    pub fn distinct(mut self, state: bool) -> Self {
        self.distinct = state;
        self
    }
}

impl ToQuery for Count {
    fn to_query(self) -> Result<String, SqlBuilderError>{
        let column_insert = match &self.column {
            Some(column) => column.name.clone(),
            None => {
                if self.distinct {
                    return Err(SqlBuilderError::from("ToQuery: Count: DISTINCT can be applied only to columns."));
                }
                "*".to_string()
            }
        };

        let condition_insert = match &self.condition {
            Some(conditions) => format!(" WHERE {}", conditions),
            None => "".to_string(),
        };

        Ok(format!(
            "SELECT COUNT({}{}) FROM {} {};",
            if self.distinct { "DISTINCT " } else { "" },
            column_insert,
            self.table_name,
            condition_insert,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::definitions::Table;
    use lazy_static::lazy_static;
    lazy_static! {
        static ref cols: Vec<Column> = vec![
            Column {
                name: "column1".to_string(),
                d_type: Type::Integer,
                constraint: Some(SqlConstraints::PrimaryKey),
            },
            Column {
                name: "column2".to_string(),
                d_type: Type::Real,
                constraint: None,
            },
            Column {
                name: "column3".to_string(),
                d_type: Type::Integer,
                constraint: Some(SqlConstraints::NotNull),
            },
            Column {
                name: "column4".to_string(),
                d_type: Type::Text,
                constraint: None,
            },
        ];
    }
    #[test]
    fn test_count() -> Result<(), SqlBuilderError>{
        let column: Column = Column {
            name: "test_col".to_string(),
            d_type: Type::Text,
            constraint: None,
        };

        let mut q = Count::new("test_table").column(&column);
        println!("\nCount query test");
        println!("{}", q.to_query()?);
        q = Count::new("test_table")
            .column(&column)
            .distinct(true)
            .condition("test_val > 20");
        println!("{}", q.to_query()?);

        Ok(())
    }

    #[test]
    fn test_insert() -> Result<(), SqlBuilderError> {
        let table = Table {
            name: "test_table".to_string(),
            columns: cols.clone(),
        };

        println!("\nInsert query test");
        let mut q = Insert::new("test_table")
            .columns(&table.columns)
            .vals(&vec![
                Value::Integer(1),
                Value::Real(2.0),
                Value::Integer(3),
                Value::Text("four".to_string()),
            ])
            .to_query();
        println!("{}", q?);
        q = Insert::new(&table.name).columns(&table.columns).to_query();
        println!("{}", q?);

        Ok(())
    }

    #[test]
    fn test_create_table() -> Result<(), SqlBuilderError>{
        println!("\nCreate table test");
        let q = CreateTable::new("test_table")
            .if_not_exists(true)
            .columns(&cols)
            .to_query();
        println!("{}", q?);

        Ok(())
    }

    #[test]
    fn test_select() -> Result<(), SqlBuilderError> {
        println!("\nSelect test");
        let mut q = Select::new("test_table")
            .columns(&cols)
            .condition("column1 > column2");
        println!("{}", q.to_query()?);
        q = Select::new("test_table");
        println!("{}", q.to_query()?);
        q = Select::new("test_table").columns(&cols);
        println!("{}", q.to_query()?);

        Ok(())
    }
}
