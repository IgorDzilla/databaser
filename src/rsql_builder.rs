use rusqlite::types;

pub enum SqlConstraint {
    PrimaryKey,
    NotNull,
    Unique,
    Default(String),
    Autoincrement,
}

pub struct Select {
    table_name: String,
    columns: Option<Vec<(String, types::Type)>>,
    condition: Option<String>,
}

pub struct CreateTable {
    table_name: String,
    columns: Option<Vec<(String, types::Type, SqlConstraint)>>,
    if_not_exists: bool,
}

pub struct Insert {
    table_name: String,
    columns: Option<Vec<(String, types::Type)>>,
    vals: Option<Vec<types::Value>>,
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

    pub fn columns(mut self, columns: &Vec<(String, types::Type)>) -> Self {
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
            columns: None,
            if_not_exists: false,
        }
    }

    pub fn columns(mut self, col_definitions: &Vec<(String, types::Type)>) -> Self {
        self.columns = Some(col_definitions.to_owned());
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

    pub fn columns(mut self, columns: &Vec<(String, types::Type)>) -> Self {
        self.columns = Some(columns.to_owned());
        self
    }

    fn parameterize(mut self) -> Self {
        if self.columns.is_none() {
            panic!("Insert: parameterize called with None columns");
        }

        self.parameterization = true;
        self
    }

    pub fn values(mut self, vals: Option<Vec<types::Value>>) -> Self {
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

pub trait ToQuery {
    fn to_query(&self) -> String;
}

impl ToQuery for Select {
    fn to_query(&self) -> String {
        let columns_insert: String = match &self.columns {
            Some(cols) => {
                let mut s = String::new();
                for (idx, (col, t)) in cols.iter().enumerate() {
                    s.push_str(col);
                    if idx != cols.len() - 1 {
                        s.push_str(", ");
                    }
                }
                s
            }
            None => "*".to_string(),
        };

        let condition_insert: String = match &self.condition {
            Some(condition) => format!(" WHERE {}", condition),
            None => "".to_string(),
        };

        format!(
            "SELECT {} FROM {}{};",
            columns_insert, self.table_name, condition_insert
        )
    }
}

impl ToQuery for CreateTable {
    fn to_query(&self) -> String {}
}
