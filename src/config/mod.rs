use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const KUREN_DIR: &str = ".kuren";
const CONFIG_FILE: &str = "config.toml";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    pub server_url: Option<String>,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub token_expires_at: Option<i64>,
    pub handle: Option<String>,
}

impl Config {
    /// Get the path to the kuren config directory (~/.kuren)
    pub fn dir() -> Result<PathBuf> {
        let home = dirs::home_dir().context("Could not find home directory")?;
        Ok(home.join(KUREN_DIR))
    }

    /// Get the path to the config file (~/.kuren/config.toml)
    pub fn path() -> Result<PathBuf> {
        Ok(Self::dir()?.join(CONFIG_FILE))
    }

    /// Load config from disk, or return default if not found
    pub fn load() -> Result<Self> {
        let path = Self::path()?;
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;
        toml::from_str(&content).context("Failed to parse config file")
    }

    /// Save config to disk
    pub fn save(&self) -> Result<()> {
        let dir = Self::dir()?;
        std::fs::create_dir_all(&dir)
            .with_context(|| format!("Failed to create config directory: {}", dir.display()))?;

        let path = Self::path()?;
        let content = toml::to_string_pretty(self).context("Failed to serialize config")?;
        std::fs::write(&path, content)
            .with_context(|| format!("Failed to write config file: {}", path.display()))?;

        // Set restrictive permissions on config file (contains tokens)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
        }

        Ok(())
    }

    /// Get server URL with default
    pub fn server_url(&self) -> String {
        self.server_url
            .clone()
            .unwrap_or_else(|| "https://kya.kuren.ai".to_string())
    }

    /// Clear tokens (logout)
    pub fn clear_tokens(&mut self) {
        self.access_token = None;
        self.refresh_token = None;
        self.token_expires_at = None;
    }

    /// Check if logged in (has either a valid access token or a refresh token)
    pub fn is_logged_in(&self) -> bool {
        self.access_token.is_some() || self.refresh_token.is_some()
    }
}
