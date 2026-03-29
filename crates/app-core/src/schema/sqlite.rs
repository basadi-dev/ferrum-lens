use anyhow::Result;
use sqlx::{SqlitePool, Row};
use super::{DatabaseItem, TableMetadata};

pub async fn list_databases(_pool: &SqlitePool) -> Result<Vec<DatabaseItem>> {
    // SQLite doesn't have "multiple databases" on a server natively in the same way.
    // The "main" database is usually what you're connected to.
    Ok(vec![DatabaseItem { name: "main".to_string() }])
}

pub async fn list_tables(pool: &SqlitePool) -> Result<Vec<TableMetadata>> {
    let records = sqlx::query("SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%';")
    .fetch_all(pool)
    .await?;

    Ok(records.into_iter().map(|r| TableMetadata {
        name: r.try_get("name").unwrap_or_default(),
        schema: None,
    }).collect())
}

pub async fn list_columns(pool: &SqlitePool, table_name: &str) -> Result<Vec<crate::schema::ColumnMetadata>> {
    let query = format!("PRAGMA table_info('{}');", table_name.replace("'", "''")); // naive escape
    let records = sqlx::query(&query)
        .fetch_all(pool)
        .await?;
        
    Ok(records.into_iter().map(|r| crate::schema::ColumnMetadata {
        name: r.try_get("name").unwrap_or_default(),
        data_type: r.try_get("type").unwrap_or_default(),
        is_nullable: r.try_get::<i64, _>("notnull").unwrap_or(0) == 0,
        default_value: r.try_get("dflt_value").ok(),
        is_primary: r.try_get::<i64, _>("pk").unwrap_or(0) > 0,
    }).collect())
}
