use anyhow::Result;
use sqlx::{PgPool, Row};
use super::{DatabaseItem, TableMetadata};

pub async fn list_databases(pool: &PgPool) -> Result<Vec<DatabaseItem>> {
    let records = sqlx::query("SELECT datname FROM pg_database WHERE datistemplate = false;")
        .fetch_all(pool)
        .await?;
        
    Ok(records.into_iter().map(|r| DatabaseItem { 
        name: r.try_get("datname").unwrap_or_default() 
    }).collect())
}

pub async fn list_tables(pool: &PgPool, _db_name: &str) -> Result<Vec<TableMetadata>> {
    let records = sqlx::query(
        "SELECT table_name, table_schema FROM information_schema.tables WHERE table_schema NOT IN ('information_schema', 'pg_catalog')"
    )
    .fetch_all(pool)
    .await?;

    Ok(records.into_iter().map(|r| TableMetadata {
        name: r.try_get("table_name").unwrap_or_default(),
        schema: r.try_get("table_schema").ok(),
    }).collect())
}

pub async fn list_columns(pool: &PgPool, table_name: &str) -> Result<Vec<crate::schema::ColumnMetadata>> {
    // simplified lookup mostly for demo 
    let query = "
        SELECT column_name, data_type, is_nullable, column_default 
        FROM information_schema.columns 
        WHERE table_name = $1
    ";
    let records = sqlx::query(query)
        .bind(table_name)
        .fetch_all(pool)
        .await?;
        
    Ok(records.into_iter().map(|r| crate::schema::ColumnMetadata {
        name: r.try_get("column_name").unwrap_or_default(),
        data_type: r.try_get("data_type").unwrap_or_default(),
        is_nullable: r.try_get::<String, _>("is_nullable").unwrap_or_default() == "YES",
        default_value: r.try_get("column_default").ok(),
        is_primary: false, // would require probing pg_index, skip for demo logic
    }).collect())
}
