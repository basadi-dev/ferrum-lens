use anyhow::Result;
use crate::db_pool::DbPool;

pub mod postgres;
pub mod mysql;
pub mod sqlite;

#[derive(Debug, Clone)]
pub struct DatabaseItem {
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct TableMetadata {
    pub name: String,
    pub schema: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ColumnMetadata {
    pub name: String,
    pub data_type: String,
    pub is_nullable: bool,
    pub default_value: Option<String>,
    pub is_primary: bool,
}

pub async fn list_databases(pool: &DbPool) -> Result<Vec<DatabaseItem>> {
    match pool {
        DbPool::Postgres(p) => postgres::list_databases(p).await,
        DbPool::MySql(p) => mysql::list_databases(p).await,
        DbPool::Sqlite(p) => sqlite::list_databases(p).await,
    }
}

pub async fn list_tables(pool: &DbPool, db_name: &str) -> Result<Vec<TableMetadata>> {
    match pool {
        DbPool::Postgres(p) => postgres::list_tables(p, db_name).await,
        DbPool::MySql(p) => mysql::list_tables(p, db_name).await,
        DbPool::Sqlite(p) => sqlite::list_tables(p).await,
    }
}

pub async fn list_columns(pool: &DbPool, db_name: &str, table_name: &str) -> Result<Vec<ColumnMetadata>> {
    match pool {
        DbPool::Postgres(p) => postgres::list_columns(p, table_name).await,
        DbPool::MySql(p) => mysql::list_columns(p, db_name, table_name).await,
        DbPool::Sqlite(p) => sqlite::list_columns(p, table_name).await,
    }
}
