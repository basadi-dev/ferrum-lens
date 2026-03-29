use anyhow::Result;
use sqlx::{MySqlPool, Row};
use super::{DatabaseItem, TableMetadata};

pub async fn list_databases(pool: &MySqlPool) -> Result<Vec<DatabaseItem>> {
    let records = sqlx::query("SHOW DATABASES")
        .fetch_all(pool)
        .await?;
        
    Ok(records.into_iter().map(|r| DatabaseItem { 
        name: r.try_get("Database").unwrap_or_default() 
    }).collect())
}

pub async fn list_tables(pool: &MySqlPool, db_name: &str) -> Result<Vec<TableMetadata>> {
    let records = sqlx::query(
        "SELECT TABLE_NAME FROM information_schema.tables WHERE TABLE_SCHEMA = ?"
    )
    .bind(db_name)
    .fetch_all(pool)
    .await?;

    Ok(records.into_iter().map(|r| TableMetadata {
        name: r.try_get("TABLE_NAME").unwrap_or_default(),
        schema: Some(db_name.to_string()),
    }).collect())
}

pub async fn list_columns(pool: &MySqlPool, db_name: &str, table_name: &str) -> Result<Vec<crate::schema::ColumnMetadata>> {
    let query = "
        SELECT COLUMN_NAME, DATA_TYPE, IS_NULLABLE, COLUMN_DEFAULT, COLUMN_KEY 
        FROM information_schema.columns 
        WHERE TABLE_SCHEMA = ? AND TABLE_NAME = ?
    ";
    let records = sqlx::query(query)
        .bind(db_name)
        .bind(table_name)
        .fetch_all(pool)
        .await?;
        
    Ok(records.into_iter().map(|r| crate::schema::ColumnMetadata {
        name: r.try_get("COLUMN_NAME").unwrap_or_default(),
        data_type: r.try_get("DATA_TYPE").unwrap_or_default(),
        is_nullable: r.try_get::<String, _>("IS_NULLABLE").unwrap_or_default() == "YES",
        default_value: r.try_get("COLUMN_DEFAULT").ok(),
        is_primary: r.try_get::<String, _>("COLUMN_KEY").unwrap_or_default() == "PRI",
    }).collect())
}
