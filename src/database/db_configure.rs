use serde::Deserialize;

#[derive(Deserialize)]
struct Config {
    database: DatabaseConfig,
}

#[derive(Deserialize)]
struct DatabaseConfig {
    connection: ConnectionConfig,
    schema: SchemaConfig,
    queries: UserDefinedQueries,
}

#[derive(Deserialize)]
struct ConnectionConfig {
    filepath: String,
}

#[derive(Deserialize)]
struct SchemaConfig {
    tables: Vec<TableConfig>,
}

#[derive(Deserialize)]
struct TableConfig {
    name: String,
    columns: Vec<ColumnConfig>,
}

#[derive(Deserialize)]
struct ColumnConfig {
    name: String,
    d_type_constraint: String,
}

#[derive(Deserialize)]
struct UserDefinedQueries {
    queries: Vec<QueryConfig>,
}

#[derive(Deserialize)]
struct QueryConfig {
    name: String,
    q: String,
}
