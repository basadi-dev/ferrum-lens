use crate::models::connection::ConnectionConfig;
use anyhow::{Context, Result};
use directories::ProjectDirs;
use std::fs;
use std::path::PathBuf;

#[derive(Clone)]
pub struct ConfigStore {
    config_dir: PathBuf,
}

impl ConfigStore {
    pub fn new() -> Result<Self> {
        let proj_dirs = ProjectDirs::from("com", "ferrum-lens", "ferrum-lens")
            .context("Could not determine config directory")?;
        
        let config_dir = proj_dirs.config_dir().to_path_buf();
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)?;
        }
        
        Ok(Self { config_dir })
    }
    
    fn connections_file(&self) -> PathBuf {
        self.config_dir.join("connections.json")
    }

    pub fn load_connections(&self) -> Result<Vec<ConnectionConfig>> {
        let path = self.connections_file();
        if !path.exists() {
            return Ok(Vec::new());
        }
        
        let data = fs::read_to_string(&path)?;
        let connections: Vec<ConnectionConfig> = serde_json::from_str(&data)?;
        Ok(connections)
    }

    pub fn save_connections(&self, connections: &[ConnectionConfig]) -> Result<()> {
        let data = serde_json::to_string_pretty(connections)?;
        fs::write(self.connections_file(), data)?;
        Ok(())
    }
}
