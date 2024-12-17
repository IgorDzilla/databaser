use crate::database::definitions::DataType;
use core::panic;

pub struct Select {
    table_name: String,
    columns: Option<Vec<String>>,
    condition: Option<String>,
}

pub struct CreateTable {
    table_name: String,
    cols_defs: Option<Vec<String>>,
}

pub struct Insert {
    table_name: String,
    columns: Option<Vec<String>>,
    vals: Option<Vec<DataType>>,
    parameterization: bool,
}

pub struct Count {
    table_name: String,
    column: Option<String>,
    distinct: bool,
    condition: Option<String>,
}

impl Select {
    pub fn new(table: &str) -> Self {
        Select {
            table_name: table.to_string(),
            columns: None,
            condition: None,
        }
    }

    pub fn columns(mut self, columns: &Vec<String>) -> Self {
        self.columns = Some(columns.to_owned());
        self
    }

    pub fn conditions(mut self, condition: &str) -> Self {
        self.condition = Some(condition.to_owned());
        self
    }
}

impl CreateTable {
    pub fn new(table: &str) -> Self {
        CreateTable {
            table_name: table.to_owned(),
            cols_defs: None,
        }
    }

    pub fn columns(mut self, col_definitions: &Vec<String>) -> Self {
        self.cols_defs = Some(col_definitions.to_owned());
        self
    }
}

impl Insert {
    pub fn new(table_name: &str) -> Self {
        Insert {
            table_name: table_name.to_owned(),
            columns: None,
            vals: None,
            parameterization: false,
        }
    }

    pub fn columns(mut self, columns: &Vec<String>) -> Self {
        self.columns = Some(columns.to_owned());
        self
    }

    fn parameterize(mut self) -> Self {
        let length = match &self.columns {
            Some(col) => col.len(),
            None => panic!("No columns found"),
        };

        self.vals = Some(vec![DataType::Param("?".to_string()); length]);
        self.parameterization = true;
        self
    }

    pub fn values(mut self, vals: Option<Vec<DataType>>) -> Self {
        let cols_len = match &self.columns {
            Some(col) => col.len(),
            None => panic!("Insert: No columns found"),
        };

        match vals {
            Some(ref l) => {
                if l.len() != cols_len {
                    panic!("Insert: Number of columns and values do not match");
                } else {
                    self.vals = vals.clone();
                    return self;
                }
            }
            None => self.parameterize(),
        }
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

    pub fn column(mut self, column: &str) -> Self {
        self.column = Some(column.to_string());
        self
    }

    pub fn distinct(mut self) -> Self {
        self.distinct = true;
        self
    }

    pub fn condition(mut self, condition: &str) -> Self {
        self.condition = Some(condition.to_string());
        self
    }
}

// converts vector of Strings into String, where each string is separated
// with comma to make an insert for querry
fn cols_to_insert(columns: &Vec<String>) -> String {
    let mut result = String::new();

    for (idx, col) in columns.iter().enumerate() {
        result.push_str(col);
        if idx != columns.len() - 1 {
            result.push_str(", ");
        }
    }

    result
}

pub trait ToQuery {
    fn to_query(&self) -> String;
}

impl ToQuery for Select {
    fn to_query(&self) -> String {
        let columns_part = match &self.columns {
            Some(cols) => cols.join(", "),
            None => "*".to_string(), // Default to all columns if none are specified
        };

        let condition_part = match &self.condition {
            Some(cond) => format!(" WHERE {}", cond),
            None => String::new(),
        };

        format!(
            "SELECT {} FROM {}{};",
            columns_part, self.table_name, condition_part
        )
    }
}

impl ToQuery for CreateTable {
    fn to_query(&self) -> String {
        let cols_part = match &self.cols_defs {
            Some(cols) => cols.join(", "),
            None => panic!("No column definitions provided for table creation"),
        };

        format!(
            "CREATE TABLE IF NOT EXISTS {} ({});",
            self.table_name, cols_part
        )
    }
}

impl ToQuery for Insert {
    fn to_query(&self) -> String {
        if self.columns.is_none() {
            panic!("No columns provided for INSERT");
        } else if self.vals.is_none() {
            panic!("No values provided for INSERT");
        }

        let mut cols_insert = String::new();
        let mut vals_insert = String::new();
        if let Some(cols_defs) = &self.columns {
            for (idx, col_def) in cols_defs.iter().enumerate() {
                cols_insert.push_str(col_def.as_str());
                if idx != cols_defs.len() - 1 {
                    cols_insert.push_str(", ");
                }
            }
        }

        if let Some(vals) = &self.vals {
            for (idx, val) in vals.iter().enumerate() {
                match val {
                    DataType::Int(v) => vals_insert.push_str(&v.to_string()),
                    DataType::Float(v) => vals_insert.push_str(&v.to_string()),
                    DataType::Bit(v) => {
                        vals_insert.push_str(&format!("{}", if *v { 1 } else { 0 }))
                    }
                    DataType::Text(v) => vals_insert.push_str(&format!("\"{}\"", v)),
                    DataType::Param(v) => vals_insert.push_str(v),
                }

                if idx != vals.len() - 1 {
                    vals_insert.push_str(", ");
                }
            }
        }

        format!(
            "INSERT INTO {} ({}) VALUES ({});",
            self.table_name, cols_insert, vals_insert
        )
    }
}

impl ToQuery for Count {
    fn to_query(&self) -> String {
        format!(
            "COUNT ({}{}) FROM {}{};",
            if self.distinct { "DISTINCT " } else { "" },
            self.column.clone().unwrap_or("*".to_string()),
            self.table_name,
            self.condition.clone().unwrap_or("".to_string())
        )
    }
}
