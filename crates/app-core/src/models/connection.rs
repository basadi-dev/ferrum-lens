use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DatabaseEngine {
    PostgreSql,
    MySql,
    Sqlite,
}

impl std::str::FromStr for DatabaseEngine {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "postgresql" | "postgres" => Ok(DatabaseEngine::PostgreSql),
            "mysql" => Ok(DatabaseEngine::MySql),
            "sqlite" => Ok(DatabaseEngine::Sqlite),
            _ => Err(format!("Unknown database engine: {}", s)),
        }
    }
}

impl std::fmt::Display for DatabaseEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DatabaseEngine::PostgreSql => write!(f, "PostgreSQL"),
            DatabaseEngine::MySql => write!(f, "MySQL"),
            DatabaseEngine::Sqlite => write!(f, "SQLite"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    pub id: Uuid,
    pub name: String,
    pub engine: DatabaseEngine,
    pub host: String,
    pub port: Option<u16>,
    pub database: String,
    pub username: String,
    pub password: Option<String>,
    pub use_ssl: bool,
}

impl ConnectionConfig {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: String,
        engine: DatabaseEngine,
        host: String,
        port: Option<u16>,
        database: String,
        username: String,
        password: Option<String>,
        use_ssl: bool,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            engine,
            host,
            port,
            database,
            username,
            password,
            use_ssl,
        }
    }
}
