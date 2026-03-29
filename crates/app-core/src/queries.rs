use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedQuery {
    pub id: String,
    pub name: String,
    pub content: String,
    pub created_at: std::time::SystemTime,
}

#[derive(Clone)]
pub struct SavedQueryStore {
    config_dir: PathBuf,
}

impl SavedQueryStore {
    pub fn new() -> Result<Self> {
        let proj_dirs = ProjectDirs::from("com", "ferrum-lens", "ferrum-lens")
            .context("Could not determine config directory")?;
        
        let config_dir = proj_dirs.config_dir().to_path_buf();
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)?;
        }
        
        Ok(Self { config_dir })
    }
    
    fn queries_file(&self) -> PathBuf {
        self.config_dir.join("queries.json")
    }

    pub fn load_queries(&self) -> Result<Vec<SavedQuery>> {
        let path = self.queries_file();
        if !path.exists() {
            return Ok(Vec::new());
        }
        
        let data = fs::read_to_string(&path)?;
        let queries: Vec<SavedQuery> = serde_json::from_str(&data).unwrap_or_default();
        Ok(queries)
    }

    pub fn save_queries(&self, queries: &[SavedQuery]) -> Result<()> {
        let data = serde_json::to_string_pretty(queries)?;
        fs::write(self.queries_file(), data)?;
        Ok(())
    }
}
