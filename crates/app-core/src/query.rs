use anyhow::Result;
use sqlx::{Column, Row, TypeInfo, ValueRef};
use crate::db_pool::DbPool;
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub execution_time_ms: u64,
    pub rows_affected: Option<u64>,
}

/// Returns true if the SQL statement is a read query that returns rows.
fn is_read_query(sql: &str) -> bool {
    let trimmed = sql.trim_start().to_uppercase();
    trimmed.starts_with("SELECT")
        || trimmed.starts_with("WITH")
        || trimmed.starts_with("SHOW")
        || trimmed.starts_with("EXPLAIN")
        || trimmed.starts_with("DESCRIBE")
        || trimmed.starts_with("DESC")
        || trimmed.starts_with("PRAGMA")
        || trimmed.starts_with("TABLE")
        || trimmed.starts_with("VALUES")
}

pub async fn execute_query(pool: &DbPool, sql: &str) -> Result<QueryResult> {
    let start_time = Instant::now();

    if !is_read_query(sql) {
        return execute_write(pool, sql, start_time).await;
    }

    let mut cols = Vec::new();
    let mut mapped_rows = Vec::new();

    match pool {
        DbPool::Postgres(pg) => {
            let res = sqlx::query(sql).fetch_all(pg).await?;
            if let Some(first_row) = res.first() {
                for col in first_row.columns() {
                    cols.push(col.name().to_string());
                }
            }
            for row in res {
                let mut row_data = Vec::with_capacity(cols.len());
                for (i, _) in cols.iter().enumerate() {
                    let val = match row.try_get_raw(i) {
                        Ok(v) if v.is_null() => "NULL".to_string(),
                        Ok(_) => {
                            let type_name = row.column(i).type_info().name().to_uppercase();
                            match type_name.as_str() {
                                "INT2" | "INT4" | "INT8" => row.try_get::<i64, _>(i).map(|v| format!("{}", v)).unwrap_or_default(),
                                "FLOAT4" | "FLOAT8" => row.try_get::<f64, _>(i).map(|v| format!("{}", v)).unwrap_or_default(),
                                "BOOL" => row.try_get::<bool, _>(i).map(|v| format!("{}", v)).unwrap_or_default(),
                                "VARCHAR" | "TEXT" | "CHAR" | "NAME" => row.try_get::<String, _>(i).unwrap_or_default(),
                                "UUID" => row.try_get::<uuid::Uuid, _>(i).map(|v| format!("{}", v)).unwrap_or_default(),
                                "TIMESTAMPTZ" | "TIMESTAMP" => {
                                    row.try_get::<chrono::NaiveDateTime, _>(i)
                                        .map(|v| format!("{}", v))
                                        .or_else(|_| row.try_get::<chrono::DateTime<chrono::Utc>, _>(i).map(|v| format!("{}", v)))
                                        .unwrap_or_default()
                                }
                                "JSON" | "JSONB" => row.try_get::<serde_json::Value, _>(i).map(|v| format!("{}", v)).unwrap_or_default(),
                                _ => "[Binary/Unknown]".to_string()
                            }
                        }
                        Err(_) => "ERR".to_string(),
                    };
                    row_data.push(val);
                }
                mapped_rows.push(row_data);
            }
        }
        DbPool::MySql(my) => {
            let res = sqlx::query(sql).fetch_all(my).await?;
            if let Some(first_row) = res.first() {
                for col in first_row.columns() {
                    cols.push(col.name().to_string());
                }
            }
            for row in res {
                let mut row_data = Vec::with_capacity(cols.len());
                for (i, _) in cols.iter().enumerate() {
                    let val = match row.try_get_raw(i) {
                        Ok(v) if v.is_null() => "NULL".to_string(),
                        Ok(_) => {
                            let type_name = row.column(i).type_info().name().to_uppercase();
                            match type_name.as_str() {
                                "TINYINT" | "SMALLINT" | "INT" | "BIGINT" => row.try_get::<i64, _>(i).map(|v| format!("{}", v)).unwrap_or_default(),
                                "FLOAT" | "DOUBLE" => row.try_get::<f64, _>(i).map(|v| format!("{}", v)).unwrap_or_default(),
                                "VARCHAR" | "TEXT" | "CHAR" => row.try_get::<String, _>(i).unwrap_or_default(),
                                "DATETIME" | "TIMESTAMP" => row.try_get::<chrono::NaiveDateTime, _>(i).map(|v| format!("{}", v)).unwrap_or_default(),
                                _ => "[Binary/Unknown]".to_string()
                            }
                        }
                        Err(_) => "ERR".to_string(),
                    };
                    row_data.push(val);
                }
                mapped_rows.push(row_data);
            }
        }
        DbPool::Sqlite(sq) => {
            let res = sqlx::query(sql).fetch_all(sq).await?;
            if let Some(first_row) = res.first() {
                for col in first_row.columns() {
                    cols.push(col.name().to_string());
                }
            }
            for row in res {
                let mut row_data = Vec::with_capacity(cols.len());
                for (i, _) in cols.iter().enumerate() {
                    let val = match row.try_get_raw(i) {
                        Ok(v) if v.is_null() => "NULL".to_string(),
                        Ok(_) => {
                            let type_name = row.column(i).type_info().name().to_uppercase();
                            match type_name.as_str() {
                                "INTEGER" | "INT" => row.try_get::<i64, _>(i).map(|v| format!("{}", v)).unwrap_or_default(),
                                "REAL" | "FLOAT" | "DOUBLE" => row.try_get::<f64, _>(i).map(|v| format!("{}", v)).unwrap_or_default(),
                                "TEXT" | "VARCHAR" => row.try_get::<String, _>(i).unwrap_or_default(),
                                "BOOLEAN" | "BOOL" => row.try_get::<bool, _>(i).map(|v| format!("{}", v)).unwrap_or_default(),
                                _ => row.try_get::<String, _>(i).unwrap_or_else(|_| "[Binary/Unknown]".into())
                            }
                        }
                        Err(_) => "ERR".to_string(),
                    };
                    row_data.push(val);
                }
                mapped_rows.push(row_data);
            }
        }
    }

    Ok(QueryResult {
        columns: cols,
        rows: mapped_rows,
        execution_time_ms: start_time.elapsed().as_millis() as u64,
        rows_affected: None,
    })
}

/// Execute a write/DDL statement and return a synthetic result with rows_affected info.
async fn execute_write(pool: &DbPool, sql: &str, start_time: Instant) -> Result<QueryResult> {
    let rows_affected = match pool {
        DbPool::Postgres(pg) => sqlx::query(sql).execute(pg).await?.rows_affected(),
        DbPool::MySql(my) => sqlx::query(sql).execute(my).await?.rows_affected(),
        DbPool::Sqlite(sq) => sqlx::query(sql).execute(sq).await?.rows_affected(),
    };

    Ok(QueryResult {
        columns: vec!["Result".to_string()],
        rows: vec![vec![format!("{} row(s) affected", rows_affected)]],
        execution_time_ms: start_time.elapsed().as_millis() as u64,
        rows_affected: Some(rows_affected),
    })
}

