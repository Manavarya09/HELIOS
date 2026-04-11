use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub ai: AiSettings,
    pub ui: UiSettings,
    pub general: GeneralSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiSettings {
    pub provider: String,
    pub model: String,
    pub base_url: String,
    pub api_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiSettings {
    pub theme: String,
    pub show_shortcuts: bool,
    pub output_wrap: bool,
    pub timestamp: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralSettings {
    pub auto_save: bool,
    pub log_level: String,
    pub max_history: usize,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            ai: AiSettings {
                provider: "ollama".to_string(),
                model: "llama2".to_string(),
                base_url: "http://localhost:11434".to_string(),
                api_key: None,
            },
            ui: UiSettings {
                theme: "Dark".to_string(),
                show_shortcuts: false,
                output_wrap: true,
                timestamp: true,
            },
            general: GeneralSettings {
                auto_save: true,
                log_level: "info".to_string(),
                max_history: 100,
            },
        }
    }
}

impl AppConfig {
    pub fn config_path() -> PathBuf {
        let mut path = dirs_config();
        path.push("helios");
        path.push("config.json");
        path
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(content) => match serde_json::from_str(&content) {
                    Ok(config) => return config,
                    Err(e) => eprintln!("Failed to parse config: {}", e),
                },
                Err(e) => eprintln!("Failed to read config file: {}", e),
            }
        }
        Self::default()
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
        }
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;
        fs::write(&path, content).map_err(|e| format!("Failed to write config file: {}", e))?;
        Ok(())
    }

    pub fn get(&self, key: &str) -> Option<String> {
        let parts: Vec<&str> = key.split('.').collect();
        match parts.as_slice() {
            ["ai", sub] => match *sub {
                "provider" => Some(self.ai.provider.clone()),
                "model" => Some(self.ai.model.clone()),
                "base_url" => Some(self.ai.base_url.clone()),
                "api_key" => self.ai.api_key.clone(),
                _ => None,
            },
            ["ui", sub] => match *sub {
                "theme" => Some(self.ui.theme.clone()),
                "show_shortcuts" => Some(self.ui.show_shortcuts.to_string()),
                "output_wrap" => Some(self.ui.output_wrap.to_string()),
                "timestamp" => Some(self.ui.timestamp.to_string()),
                _ => None,
            },
            ["general", sub] => match *sub {
                "auto_save" => Some(self.general.auto_save.to_string()),
                "log_level" => Some(self.general.log_level.clone()),
                "max_history" => Some(self.general.max_history.to_string()),
                _ => None,
            },
            _ => None,
        }
    }

    pub fn set(&mut self, key: &str, value: &str) -> Result<(), String> {
        let parts: Vec<&str> = key.split('.').collect();
        match parts.as_slice() {
            ["ai", sub] => match *sub {
                "provider" => self.ai.provider = value.to_string(),
                "model" => self.ai.model = value.to_string(),
                "base_url" => self.ai.base_url = value.to_string(),
                "api_key" => self.ai.api_key = Some(value.to_string()),
                _ => return Err(format!("Unknown ai setting: {}", sub)),
            },
            ["ui", sub] => match *sub {
                "theme" => self.ui.theme = value.to_string(),
                "show_shortcuts" => self.ui.show_shortcuts = value.parse().unwrap_or(false),
                "output_wrap" => self.ui.output_wrap = value.parse().unwrap_or(true),
                "timestamp" => self.ui.timestamp = value.parse().unwrap_or(true),
                _ => return Err(format!("Unknown ui setting: {}", sub)),
            },
            ["general", sub] => match *sub {
                "auto_save" => self.general.auto_save = value.parse().unwrap_or(true),
                "log_level" => self.general.log_level = value.to_string(),
                "max_history" => self.general.max_history = value.parse().unwrap_or(100),
                _ => return Err(format!("Unknown general setting: {}", sub)),
            },
            _ => return Err(format!("Unknown setting: {}", key)),
        }
        Ok(())
    }

    pub fn list_all(&self) -> Vec<(String, String)> {
        vec![
            (format!("ai.provider"), self.ai.provider.clone()),
            (format!("ai.model"), self.ai.model.clone()),
            (format!("ai.base_url"), self.ai.base_url.clone()),
            (
                format!("ai.api_key"),
                self.ai.api_key.clone().unwrap_or_default(),
            ),
            (format!("ui.theme"), self.ui.theme.clone()),
            (
                format!("ui.show_shortcuts"),
                self.ui.show_shortcuts.to_string(),
            ),
            (format!("ui.output_wrap"), self.ui.output_wrap.to_string()),
            (format!("ui.timestamp"), self.ui.timestamp.to_string()),
            (
                format!("general.auto_save"),
                self.general.auto_save.to_string(),
            ),
            (format!("general.log_level"), self.general.log_level.clone()),
            (
                format!("general.max_history"),
                self.general.max_history.to_string(),
            ),
        ]
    }
}

fn dirs_config() -> PathBuf {
    if let Some(data_dir) = std::env::var_os("APPDATA") {
        PathBuf::from(data_dir)
    } else if let Some(home) = std::env::var_os("USERPROFILE") {
        PathBuf::from(home).join("AppData").join("Roaming")
    } else {
        PathBuf::from(".")
    }
}
