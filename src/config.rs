use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Per-area threshold configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdConfig {
    /// Warning threshold value (percent for memory_load, MB for others).
    pub warning: f64,
    /// Action to take when threshold is exceeded.
    pub action: ThresholdAction,
}

/// Action to perform when a threshold is crossed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThresholdAction {
    None,
    Notify,
    Purge,
}

/// All application settings, persisted as JSON.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub memory_load: ThresholdConfig,
    pub modified_list: ThresholdConfig,
    pub standby_list: ThresholdConfig,
    pub available_memory: ThresholdConfig,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            memory_load: ThresholdConfig {
                warning: 85.0,
                action: ThresholdAction::Notify,
            },
            modified_list: ThresholdConfig {
                warning: 1024.0,
                action: ThresholdAction::Notify,
            },
            standby_list: ThresholdConfig {
                warning: 2048.0,
                action: ThresholdAction::Notify,
            },
            available_memory: ThresholdConfig {
                warning: 4096.0,
                action: ThresholdAction::Notify,
            },
        }
    }
}

impl Settings {
    /// Path to the config file alongside the executable.
    fn config_path() -> PathBuf {
        std::env::current_exe()
            .unwrap_or_else(|_| PathBuf::from("mpa.exe"))
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."))
            .join("mpa-settings.json")
    }

    /// Load settings from disk, falling back to defaults on any error.
    pub fn load() -> Self {
        let path = Self::config_path();
        match fs::read_to_string(&path) {
            Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    /// Save settings to disk alongside the executable.
    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_path();
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(&path, json).map_err(|e| format!("Failed to write {}: {e}", path.display()))
    }
}
