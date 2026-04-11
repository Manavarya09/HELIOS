use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub commands: Vec<String>,
}

pub trait Plugin: Send + Sync {
    fn info(&self) -> PluginInfo;
    fn execute(&self, command: &str, args: &[&str]) -> Result<String, String>;
    fn on_load(&self) -> Result<(), String> {
        Ok(())
    }
    fn on_unload(&self) -> Result<(), String> {
        Ok(())
    }
}

pub struct PluginRegistry {
    plugins: HashMap<String, Box<dyn Plugin>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }

    pub fn register(&mut self, plugin: Box<dyn Plugin>) -> Result<(), String> {
        let info = plugin.info();
        if self.plugins.contains_key(&info.name) {
            return Err(format!("Plugin '{}' is already registered", info.name));
        }
        plugin.on_load()?;
        self.plugins.insert(info.name.clone(), plugin);
        Ok(())
    }

    pub fn unregister(&mut self, name: &str) -> Result<(), String> {
        if let Some(plugin) = self.plugins.remove(name) {
            plugin.on_unload()?;
            Ok(())
        } else {
            Err(format!("Plugin '{}' not found", name))
        }
    }

    pub fn get(&self, name: &str) -> Option<&dyn Plugin> {
        self.plugins.get(name).map(|p| p.as_ref())
    }

    pub fn execute(&self, name: &str, command: &str, args: &[&str]) -> Result<String, String> {
        self.plugins
            .get(name)
            .ok_or_else(|| format!("Plugin '{}' not found", name))?
            .execute(command, args)
    }

    pub fn list(&self) -> Vec<PluginInfo> {
        self.plugins.values().map(|p| p.info()).collect()
    }

    pub fn commands(&self) -> Vec<(String, Vec<String>)> {
        self.plugins
            .iter()
            .map(|(name, p)| (name.clone(), p.info().commands))
            .collect()
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

pub struct FileManagerPlugin;

impl FileManagerPlugin {
    pub fn new() -> Self {
        Self
    }
}

impl Plugin for FileManagerPlugin {
    fn info(&self) -> PluginInfo {
        PluginInfo {
            name: "file_manager".to_string(),
            version: "1.0.0".to_string(),
            description: "Advanced file operations plugin".to_string(),
            commands: vec![
                "files list".to_string(),
                "files search <pattern>".to_string(),
                "files size <path>".to_string(),
                "files tree <path>".to_string(),
            ],
        }
    }

    fn execute(&self, command: &str, args: &[&str]) -> Result<String, String> {
        match command {
            "list" => {
                let path = args
                    .first()
                    .map(|s| std::path::PathBuf::from(*s))
                    .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
                if !path.exists() {
                    return Err(format!("Path does not exist: {}", path.display()));
                }
                let entries = std::fs::read_dir(&path)
                    .map_err(|e| format!("Failed to read directory: {}", e))?;
                let mut output = Vec::new();
                output.push(format!("Contents of {}:", path.display()));
                for entry in entries.flatten() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    let is_dir = entry.path().is_dir();
                    output.push(format!(
                        "{} {}",
                        if is_dir { "[DIR]" } else { "[FILE]" },
                        name
                    ));
                }
                Ok(output.join("\n"))
            }
            "search" => {
                if args.is_empty() {
                    return Err("Usage: files search <pattern>".to_string());
                }
                Ok(format!("Search functionality - searching for: {}", args[0]))
            }
            "size" => {
                let path = args
                    .first()
                    .map(|s| std::path::PathBuf::from(*s))
                    .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
                if !path.exists() {
                    return Err(format!("Path does not exist: {}", path.display()));
                }
                Ok(format!("Size analysis for: {}", path.display()))
            }
            "tree" => {
                let path = args
                    .first()
                    .map(|s| std::path::PathBuf::from(*s))
                    .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
                if !path.exists() {
                    return Err(format!("Path does not exist: {}", path.display()));
                }
                Ok(format!("Tree view for: {}", path.display()))
            }
            _ => Err(format!(
                "Unknown command: {}. Use: list, search, size, tree",
                command
            )),
        }
    }
}

pub struct NetworkToolsPlugin;

impl NetworkToolsPlugin {
    pub fn new() -> Self {
        Self
    }
}

impl Plugin for NetworkToolsPlugin {
    fn info(&self) -> PluginInfo {
        PluginInfo {
            name: "network_tools".to_string(),
            version: "1.0.0".to_string(),
            description: "Network diagnostic and utility tools".to_string(),
            commands: vec![
                "net info".to_string(),
                "net interfaces".to_string(),
                "net connections".to_string(),
            ],
        }
    }

    fn execute(&self, command: &str, _args: &[&str]) -> Result<String, String> {
        match command {
            "info" => Ok(
                "Network Tools Plugin v1.0.0\nAvailable commands: info, interfaces, connections"
                    .to_string(),
            ),
            "interfaces" => Ok("Network interfaces: (use 'net info' for details)".to_string()),
            "connections" => Ok("Active connections: (use 'net info' for details)".to_string()),
            _ => Err(format!("Unknown command: {}", command)),
        }
    }
}

pub struct ProcessManagerPlugin;

impl ProcessManagerPlugin {
    pub fn new() -> Self {
        Self
    }
}

impl Plugin for ProcessManagerPlugin {
    fn info(&self) -> PluginInfo {
        PluginInfo {
            name: "process_manager".to_string(),
            version: "1.0.0".to_string(),
            description: "Advanced process management tools".to_string(),
            commands: vec![
                "proc tree".to_string(),
                "proc find <name>".to_string(),
                "proc stats".to_string(),
            ],
        }
    }

    fn execute(&self, command: &str, _args: &[&str]) -> Result<String, String> {
        match command {
            "tree" => Ok("Process tree view (use 'processes' command)".to_string()),
            "find" => Ok("Process finder (use 'processes' command)".to_string()),
            "stats" => Ok("Process statistics (use 'stats' command)".to_string()),
            _ => Err(format!("Unknown command: {}", command)),
        }
    }
}
