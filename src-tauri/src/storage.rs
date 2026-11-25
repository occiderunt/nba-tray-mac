use anyhow::Result;
use std::fs;
use std::path::PathBuf;

use crate::state::FollowedTeam;

pub struct Storage {
    config_path: PathBuf,
}

impl Storage {
    pub fn new() -> Result<Self> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("no config directory"))?
            .join("NBA Tracker");

        fs::create_dir_all(&config_dir)?;

        Ok(Self {
            config_path: config_dir.join("teams.json"),
        })
    }

    pub fn load_teams(&self) -> Result<Vec<FollowedTeam>> {
        if !self.config_path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&self.config_path)?;
        if content.trim().is_empty() {
            return Ok(Vec::new());
        }

        Ok(serde_json::from_str(&content)?)
    }

    pub fn save_teams(&self, teams: &[FollowedTeam]) -> Result<()> {
        let content = serde_json::to_string_pretty(teams)?;
        let tmp = self.config_path.with_extension("json.tmp");
        fs::write(&tmp, content)?;
        fs::rename(&tmp, &self.config_path)?;
        Ok(())
    }
}
