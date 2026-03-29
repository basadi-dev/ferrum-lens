use crate::models::connection::{ConnectionConfig, DatabaseEngine};
use anyhow::Result;
use sqlx::{mysql::MySqlPool, postgres::PgPool, sqlite::SqlitePool};

#[derive(Clone)]
pub enum DbPool {
    Postgres(PgPool),
    MySql(MySqlPool),
    Sqlite(SqlitePool),
}

impl DbPool {
    pub async fn connect(config: &ConnectionConfig) -> Result<Self> {
        match config.engine {
            DatabaseEngine::PostgreSql => {
                let url = format!(
                    "postgres://{}:{}@{}:{}/{}",
                    config.username,
                    config.password.as_deref().unwrap_or(""),
                    config.host,
                    config.port.unwrap_or(5432),
                    config.database
                );
                let pool = PgPool::connect(&url).await?;
                Ok(Self::Postgres(pool))
            }
            DatabaseEngine::MySql => {
                let url = format!(
                    "mysql://{}:{}@{}:{}/{}",
                    config.username,
                    config.password.as_deref().unwrap_or(""),
                    config.host,
                    config.port.unwrap_or(3306),
                    config.database
                );
                let pool = MySqlPool::connect(&url).await?;
                Ok(Self::MySql(pool))
            }
            DatabaseEngine::Sqlite => {
                // SQLite uses path instead of host
                let url = format!("sqlite://{}", config.database);
                let pool = SqlitePool::connect(&url).await?;
                Ok(Self::Sqlite(pool))
            }
        }
    }

    pub async fn test_connection(config: &ConnectionConfig) -> Result<()> {
        let _pool = Self::connect(config).await?;
        // Once connected, we assume success for the test
        // Ideally we execute a `SELECT 1` for validity, but connection is usually proof.
        Ok(())
    }
}
