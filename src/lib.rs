mod ai;
mod commands;
mod config;
mod jarvis;
mod output;
mod plugins;
mod system;
mod ui;

use ai::{AiProvider, HolographicAI, OllamaClient};
use commands::{parse_file_command, parse_network_command, parse_system_command};
use commands::{AliasManager, Calculator, CommandInput, EnvManager, NotesManager, TodoManager};
use config::AppConfig;
use eframe::egui;
use jarvis::JarviState;
use output::OutputFormatter;
use plugins::{FileManagerPlugin, NetworkToolsPlugin, PluginRegistry, ProcessManagerPlugin};
use system::SystemStats;
use ui::{AlertManager, RealtimeGraph, Theme, UiState};

pub struct HeliosApp {
    command_input: CommandInput,
    output_messages: Vec<String>,
    ollama: OllamaClient,
    holographic_ai: HolographicAI,
    system_stats: SystemStats,
    is_processing: bool,
    selected_category: usize,
    current_time: String,
    ui_state: UiState,
    config: AppConfig,
    plugin_registry: PluginRegistry,
    alias_manager: AliasManager,
    env_manager: EnvManager,
    output_formatter: OutputFormatter,
    notes_manager: NotesManager,
    todo_manager: TodoManager,
    calculator: Calculator,
    jarvis: JarviState,
    realtime_graphs: RealtimeGraph,
    alert_manager: AlertManager,
    ai_chat_mode: bool,
    current_ai_prompt: String,
}

impl Default for HeliosApp {
    fn default() -> Self {
        let _config = AppConfig::load();
        let mut plugin_registry = PluginRegistry::new();
        let alias_manager = AliasManager::new();
        let env_manager = EnvManager::new();
        let notes_manager = NotesManager::new();
        let todo_manager = TodoManager::new();
        let calculator = Calculator::new();
        let jarvis = JarviState::new();
        let realtime_graphs = RealtimeGraph::new();
        let alert_manager = AlertManager::new();
        let holographic_ai = HolographicAI::new();

        let _ = plugin_registry.register(Box::new(FileManagerPlugin::new()));
        let _ = plugin_registry.register(Box::new(NetworkToolsPlugin::new()));
        let _ = plugin_registry.register(Box::new(ProcessManagerPlugin::new()));

        Self {
            command_input: CommandInput::default(),
            output_messages: vec![
                "HELIOS v0.7.0 Advanced AI Command System".to_string(),
                "Type 'help' for command reference".to_string(),
                "Use 'ai chat' for holographic AI mode".to_string(),
            ],
            ollama: OllamaClient::default(),
            holographic_ai,
            system_stats: SystemStats::new(),
            is_processing: false,
            selected_category: 0,
            current_time: "00:00:00".to_string(),
            ui_state: UiState::default(),
            config: AppConfig::load(),
            plugin_registry,
            alias_manager,
            env_manager,
            output_formatter: OutputFormatter::new(),
            notes_manager,
            todo_manager,
            calculator,
            jarvis,
            realtime_graphs,
            alert_manager,
            ai_chat_mode: false,
            current_ai_prompt: String::new(),
        }
    }
}

impl HeliosApp {
    fn update_time(&mut self) {
        self.jarvis.update();
        self.holographic_ai.update();

        // Update realtime graphs
        self.realtime_graphs.update(
            self.system_stats.cpu_usage(),
            self.system_stats.memory_percent(),
            self.jarvis.network_activity,
            self.jarvis.power_consumption,
            self.jarvis.core_temp,
            self.jarvis.processing_load,
        );

        use std::time::SystemTime;
        let now = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let hours = (now / 3600) % 24;
        let mins = (now / 60) % 60;
        let secs = now % 60;
        self.current_time = format!("{:02}:{:02}:{:02}", hours, mins, secs);
    }

    fn execute_command(&mut self, command: &str) {
        let cmd = command.trim().to_lowercase();

        if cmd.is_empty() {
            return;
        }

        let parts: Vec<&str> = cmd.split_whitespace().collect();
        let first_word = parts.first().unwrap_or(&"");

        if let Some(resolved) = self.alias_manager.resolve(first_word) {
            let rest = if parts.len() > 1 {
                format!("{} {}", resolved, parts[1..].join(" "))
            } else {
                resolved
            };
            self.execute_command(&rest);
            return;
        }

        self.output_messages.push(format!("> {}", command));

        match parts.first() {
            Some(&"help") => {
                self.output_messages
                    .push("HELIOS v0.2.0 - Command Reference".to_string());
                self.output_messages.push("".to_string());
                self.output_messages
                    .push("=== SYSTEM COMMANDS ===".to_string());
                self.output_messages
                    .push("  help - Display command reference".to_string());
                self.output_messages
                    .push("  status - Display system status".to_string());
                self.output_messages
                    .push("  clear - Clear terminal output".to_string());
                self.output_messages
                    .push("  stats - Display detailed system statistics".to_string());
                self.output_messages
                    .push("  time - Display current system time".to_string());
                self.output_messages
                    .push("  history [cmd] - Command history management".to_string());
                self.output_messages
                    .push("  alias [cmd] - Alias management".to_string());
                self.output_messages
                    .push("  env [cmd] - Environment variable management".to_string());
                self.output_messages.push("".to_string());
                self.output_messages.push("=== AI ENGINE ===".to_string());
                self.output_messages
                    .push("  ai <prompt> - Query AI assistant".to_string());
                self.output_messages
                    .push("  ai provider <name> - Set AI provider".to_string());
                self.output_messages
                    .push("  ai model <name> - Set AI model".to_string());
                self.output_messages.push("".to_string());
                self.output_messages.push("=== FILE SYSTEM ===".to_string());
                self.output_messages
                    .push("  ls [path] - List directory contents".to_string());
                self.output_messages
                    .push("  cd <path> - Change working directory".to_string());
                self.output_messages
                    .push("  pwd - Print working directory".to_string());
                self.output_messages
                    .push("  read <file> - Read file contents".to_string());
                self.output_messages
                    .push("  write <file> <content> - Write to file".to_string());
                self.output_messages
                    .push("  mkdir <dir> - Create directory".to_string());
                self.output_messages
                    .push("  delete <path> - Delete file or directory".to_string());
                self.output_messages.push("".to_string());
                self.output_messages.push("=== NETWORK ===".to_string());
                self.output_messages
                    .push("  ping <host> [count] - ICMP ping utility".to_string());
                self.output_messages
                    .push("  curl <url> - HTTP request client".to_string());
                self.output_messages
                    .push("  scan <host> [start] [end] - Port scanner".to_string());
                self.output_messages.push("".to_string());
                self.output_messages
                    .push("=== PROCESS MANAGEMENT ===".to_string());
                self.output_messages
                    .push("  processes [count] - List running processes".to_string());
                self.output_messages
                    .push("  kill <pid> - Terminate process by ID".to_string());
                self.output_messages
                    .push("  info <pid> - Display process information".to_string());
                self.output_messages.push("".to_string());
                self.output_messages.push("=== APPEARANCE ===".to_string());
                self.output_messages
                    .push("  theme list - List available themes".to_string());
                self.output_messages
                    .push("  theme set <name> - Apply theme".to_string());
                self.output_messages
                    .push("  theme next - Cycle to next theme".to_string());
                self.output_messages
                    .push("  shortcuts - Toggle keyboard shortcuts".to_string());
                self.output_messages.push("".to_string());
                self.output_messages
                    .push("=== CONFIGURATION ===".to_string());
                self.output_messages
                    .push("  config list - List all settings".to_string());
                self.output_messages
                    .push("  config get <key> - Get setting value".to_string());
                self.output_messages
                    .push("  config set <key> <value> - Set configuration".to_string());
                self.output_messages
                    .push("  config save - Persist configuration".to_string());
                self.output_messages
                    .push("  config reset - Reset to defaults".to_string());
                self.output_messages.push("".to_string());
                self.output_messages.push("=== OUTPUT ===".to_string());
                self.output_messages
                    .push("  format show - Display output format".to_string());
                self.output_messages
                    .push("  format set <type> - Set output format".to_string());
                self.output_messages
                    .push("  format demo - Demonstrate formats".to_string());
                self.output_messages.push("".to_string());
                self.output_messages
                    .push("Keyboard: ?=shortcuts T=theme Esc=close".to_string());
            }
            Some(&"status") => {
                self.system_stats.refresh();
                let status = self.system_stats.summary();
                self.output_messages.push(format!("System: {}", status));
                self.output_messages.push(format!(
                    "AI: {}",
                    if self.ollama.is_available() {
                        "Ready"
                    } else {
                        "Unavailable"
                    }
                ));
                self.output_messages
                    .push(format!("Time: {}", self.current_time));
            }
            Some(&"clear") => {
                self.output_messages.clear();
                self.output_messages.push("Output cleared.".to_string());
            }
            Some(&"stats") => {
                self.system_stats.refresh();
                self.output_messages.push(self.system_stats.summary());
            }
            Some(&"time") => {
                self.output_messages
                    .push(format!("Current time: {}", self.current_time));
            }
            Some(&"ai") => {
                let args: Vec<&str> = parts[1..].iter().copied().collect();
                if args.is_empty() {
                    self.output_messages
                        .push("=== HOLOGRAPHIC AI SYSTEM ===".to_string());
                    self.output_messages.push(format!(
                        "Status: {}",
                        self.holographic_ai.get_status_indicator()
                    ));
                    self.output_messages
                        .push(format!("Emotion: {}", self.holographic_ai.emotion_state));
                    self.output_messages.push(format!(
                        "Processing Speed: {:.0}%",
                        self.holographic_ai.processing_speed
                    ));
                    self.output_messages.push(format!(
                        "Neural Links: {}",
                        self.holographic_ai.neural_links
                    ));
                    self.output_messages.push(format!(
                        "Quantum State: {}",
                        self.holographic_ai.quantum_state
                    ));
                    self.output_messages.push("".to_string());
                    self.output_messages
                        .push("Usage: ai <prompt> | ai <subcommand>".to_string());
                    self.output_messages
                        .push("  ai <prompt> - Ask AI".to_string());
                    self.output_messages
                        .push("  ai chat - Toggle chat mode".to_string());
                    self.output_messages
                        .push("  ai clear - Clear chat history".to_string());
                    self.output_messages
                        .push("  ai history - Show chat history".to_string());
                    self.output_messages
                        .push("  ai config - Show AI config".to_string());
                    self.output_messages
                        .push("  ai provider <name> - Set provider".to_string());
                    self.output_messages
                        .push("  ai model <name> - Set model".to_string());
                    self.output_messages
                        .push("  ai emotion <state> - Set AI emotion".to_string());
                    self.output_messages
                        .push("  ai voice on/off - Toggle voice".to_string());
                } else if args[0] == "config" {
                    self.output_messages
                        .push(format!("AI Config: {:?}", self.ollama.config()));
                } else if args[0] == "provider" {
                    if args.len() < 2 {
                        self.output_messages.push(
                            "Usage: ai provider <ollama|openai|anthropic|claude|gemini|deepseek>"
                                .to_string(),
                        );
                    } else {
                        match args[1].to_lowercase().as_str() {
                            "ollama" => {
                                self.ollama.set_provider(AiProvider::Ollama);
                                self.output_messages
                                    .push("Provider set to OLLAMA".to_string());
                            }
                            "openai" => {
                                self.ollama.set_provider(AiProvider::OpenAI);
                                self.output_messages
                                    .push("Provider set to OPENAI".to_string());
                            }
                            "anthropic" | "claude" => {
                                self.ollama.set_provider(AiProvider::Anthropic);
                                self.output_messages
                                    .push("Provider set to ANTHROPIC".to_string());
                            }
                            "gemini" => {
                                self.ollama.set_provider(AiProvider::Gemini);
                                self.output_messages
                                    .push("Provider set to GEMINI".to_string());
                            }
                            "deepseek" => {
                                self.ollama.set_provider(AiProvider::DeepSeek);
                                self.output_messages
                                    .push("Provider set to DEEPSEEK".to_string());
                            }
                            _ => self.output_messages.push(
                                "Unknown provider. Use: ollama, openai, anthropic, claude, gemini, deepseek".to_string(),
                            ),
                        }
                    }
                } else if args[0] == "model" {
                    if args.len() < 2 {
                        self.output_messages
                            .push("Usage: ai model <model_name>".to_string());
                        self.output_messages
                            .push("Available: llama2, llama3, mistral, codellama, gpt-3.5, gpt-4, claude-3, gemini-pro, deepseek-chat".to_string());
                    } else {
                        self.ollama.set_model(args[1].to_string());
                        self.output_messages
                            .push(format!("Model set to: {}", args[1]));
                    }
                } else if args[0] == "apikey" {
                    if args.len() < 2 {
                        self.output_messages
                            .push("Usage: ai apikey <key>".to_string());
                    } else {
                        self.ollama.set_api_key(args[1].to_string());
                        self.output_messages.push("API key set".to_string());
                    }
                } else if args[0] == "chat" {
                    self.ai_chat_mode = !self.ai_chat_mode;
                    if self.ai_chat_mode {
                        self.output_messages
                            .push(">>> HOLOGRAPHIC CHAT MODE ACTIVATED <<<".to_string());
                        self.output_messages
                            .push("Type your message and press Enter to chat with AI".to_string());
                    } else {
                        self.output_messages
                            .push("Holographic chat mode deactivated".to_string());
                    }
                } else if args[0] == "emotion" {
                    if args.len() < 2 {
                        self.output_messages.push(
                            "Usage: ai emotion <calm|happy|focused|alert|thinking>".to_string(),
                        );
                    } else {
                        let emotion = args[1].to_uppercase();
                        self.holographic_ai.set_emotion(&emotion);
                        self.output_messages
                            .push(format!("AI emotion set to: {}", emotion));
                    }
                } else if args[0] == "voice" {
                    if args.len() < 2 {
                        self.output_messages
                            .push("Usage: ai voice <on|off>".to_string());
                    } else if args[1] == "on" {
                        self.holographic_ai.activate_voice();
                        self.output_messages
                            .push("Voice synthesis activated".to_string());
                    } else if args[1] == "off" {
                        self.holographic_ai.deactivate_voice();
                        self.output_messages
                            .push("Voice synthesis deactivated".to_string());
                    }
                } else if args[0] == "clear" {
                    self.ollama.clear_history();
                    self.output_messages
                        .push("Chat history cleared.".to_string());
                } else if args[0] == "history" {
                    let history = self.ollama.history();
                    if history.is_empty() {
                        self.output_messages.push("No chat history.".to_string());
                    } else {
                        self.output_messages.push("Chat History:".to_string());
                        for msg in history {
                            self.output_messages
                                .push(format!("[{}] {}", msg.role, msg.content));
                        }
                    }
                } else {
                    let prompt = args.join(" ");
                    self.output_messages.push("Processing...".to_string());
                    self.is_processing = true;
                    match self.ollama.generate(prompt) {
                        Ok(response) => {
                            self.output_messages.push("HOLOGRAPHIC AI:".to_string());
                            self.output_messages.push(response);
                        }
                        Err(e) => {
                            self.output_messages.push(format!("Error: {}", e));
                        }
                    }
                    self.is_processing = false;
                }
            }
            Some(&"ls") | Some(&"cd") | Some(&"pwd") | Some(&"read") | Some(&"write")
            | Some(&"mkdir") | Some(&"delete") | Some(&"rm") => {
                let args: Vec<&str> = parts[1..].iter().copied().collect();
                match parse_file_command(&args) {
                    Ok(op) => match op.execute() {
                        Ok(result) => self.output_messages.push(result),
                        Err(e) => self.output_messages.push(format!("Error: {}", e)),
                    },
                    Err(e) => self.output_messages.push(e),
                }
            }
            Some(&"ping") | Some(&"curl") | Some(&"scan") => {
                let args: Vec<&str> = parts[1..].iter().copied().collect();
                match parse_network_command(&args) {
                    Ok(op) => {
                        self.output_messages.push("Processing...".to_string());
                        match op.execute() {
                            Ok(result) => self.output_messages.push(result),
                            Err(e) => self.output_messages.push(format!("Error: {}", e)),
                        }
                    }
                    Err(e) => self.output_messages.push(e),
                }
            }
            Some(&"processes") | Some(&"ps") | Some(&"kill") | Some(&"info") => {
                let args: Vec<&str> = parts[1..].iter().copied().collect();
                match parse_system_command(&args) {
                    Ok(cmd) => match cmd.execute(&mut self.system_stats.system) {
                        Ok(result) => self.output_messages.push(result),
                        Err(e) => self.output_messages.push(format!("Error: {}", e)),
                    },
                    Err(e) => self.output_messages.push(e),
                }
            }
            Some(&"theme") => {
                let args: Vec<&str> = parts[1..].iter().copied().collect();
                if args.is_empty() || args[0] == "list" {
                    self.output_messages.push("Available themes:".to_string());
                    for t in Theme::all() {
                        let current = if self.ui_state.theme == *t {
                            " (current)"
                        } else {
                            ""
                        };
                        self.output_messages
                            .push(format!("  {}{}", t.name(), current));
                    }
                } else if args[0] == "set" {
                    if args.len() < 2 {
                        self.output_messages
                            .push("Usage: theme set <name>".to_string());
                    } else {
                        let theme_name = args[1].to_lowercase();
                        for t in Theme::all() {
                            if t.name().to_lowercase() == theme_name {
                                self.ui_state.theme = *t;
                                self.output_messages
                                    .push(format!("Theme set to: {}", t.name()));
                                break;
                            }
                        }
                    }
                } else if args[0] == "next" {
                    let themes = Theme::all();
                    let current_idx = themes
                        .iter()
                        .position(|t| *t == self.ui_state.theme)
                        .unwrap_or(0);
                    let next_idx = (current_idx + 1) % themes.len();
                    self.ui_state.theme = themes[next_idx].clone();
                    self.output_messages
                        .push(format!("Theme: {}", self.ui_state.theme.name()));
                } else {
                    self.output_messages
                        .push("Usage: theme [list|set <name>|next]".to_string());
                }
            }
            Some(&"shortcuts") => {
                self.ui_state.show_shortcuts = !self.ui_state.show_shortcuts;
                self.output_messages.push(if self.ui_state.show_shortcuts {
                    "Shortcuts overlay enabled. Press ? to toggle.".to_string()
                } else {
                    "Shortcuts overlay disabled.".to_string()
                });
            }
            Some(&"config") => {
                let args: Vec<&str> = parts[1..].iter().copied().collect();
                if args.is_empty() || args[0] == "list" {
                    self.output_messages.push("Current Settings:".to_string());
                    for (key, value) in self.config.list_all() {
                        self.output_messages.push(format!("  {} = {}", key, value));
                    }
                } else if args[0] == "get" {
                    if args.len() < 2 {
                        self.output_messages
                            .push("Usage: config get <key>".to_string());
                    } else {
                        match self.config.get(args[1]) {
                            Some(value) => self
                                .output_messages
                                .push(format!("{} = {}", args[1], value)),
                            None => self
                                .output_messages
                                .push(format!("Unknown key: {}", args[1])),
                        }
                    }
                } else if args[0] == "set" {
                    if args.len() < 3 {
                        self.output_messages
                            .push("Usage: config set <key> <value>".to_string());
                    } else {
                        let key = args[1];
                        let value = args[2..].join(" ");
                        match self.config.set(key, &value) {
                            Ok(()) => {
                                self.output_messages
                                    .push(format!("Set {} = {}", key, value));
                                if self.config.general.auto_save {
                                    if let Err(e) = self.config.save() {
                                        self.output_messages
                                            .push(format!("Warning: Failed to save config: {}", e));
                                    } else {
                                        self.output_messages.push("Config saved.".to_string());
                                    }
                                }
                            }
                            Err(e) => self.output_messages.push(format!("Error: {}", e)),
                        }
                    }
                } else if args[0] == "save" {
                    match self.config.save() {
                        Ok(()) => self
                            .output_messages
                            .push("Config saved successfully.".to_string()),
                        Err(e) => self
                            .output_messages
                            .push(format!("Error saving config: {}", e)),
                    }
                } else if args[0] == "reset" {
                    self.config = AppConfig::default();
                    match self.config.save() {
                        Ok(()) => self
                            .output_messages
                            .push("Config reset to defaults and saved.".to_string()),
                        Err(e) => self
                            .output_messages
                            .push(format!("Error saving config: {}", e)),
                    }
                } else {
                    self.output_messages.push(
                        "Usage: config [list|get <key>|set <key> <value>|save|reset]".to_string(),
                    );
                }
            }
            Some(&"plugins") | Some(&"plugin") => {
                let args: Vec<&str> = parts[1..].iter().copied().collect();
                if args.is_empty() || args[0] == "list" {
                    self.output_messages.push("Loaded Plugins:".to_string());
                    let plugins = self.plugin_registry.list();
                    if plugins.is_empty() {
                        self.output_messages
                            .push("  No plugins loaded.".to_string());
                    } else {
                        for p in plugins {
                            self.output_messages
                                .push(format!("  {} v{} - {}", p.name, p.version, p.description));
                            self.output_messages
                                .push(format!("    Commands: {:?}", p.commands));
                        }
                    }
                } else if args[0] == "info" {
                    if args.len() < 2 {
                        self.output_messages
                            .push("Usage: plugin info <name>".to_string());
                    } else {
                        if let Some(p) = self.plugin_registry.get(args[1]) {
                            let info = p.info();
                            self.output_messages.push(format!("Plugin: {}", info.name));
                            self.output_messages
                                .push(format!("Version: {}", info.version));
                            self.output_messages
                                .push(format!("Description: {}", info.description));
                            self.output_messages
                                .push(format!("Commands: {:?}", info.commands));
                        } else {
                            self.output_messages
                                .push(format!("Plugin '{}' not found", args[1]));
                        }
                    }
                } else if args[0] == "run" {
                    if args.len() < 3 {
                        self.output_messages
                            .push("Usage: plugin run <name> <command> [args...]".to_string());
                    } else {
                        let plugin_name = args[1];
                        let plugin_cmd = args[2];
                        let plugin_args: Vec<&str> = args[3..].iter().copied().collect();
                        match self
                            .plugin_registry
                            .execute(plugin_name, plugin_cmd, &plugin_args)
                        {
                            Ok(result) => self.output_messages.push(result),
                            Err(e) => self.output_messages.push(format!("Error: {}", e)),
                        }
                    }
                } else if args[0] == "commands" {
                    self.output_messages
                        .push("Available Plugin Commands:".to_string());
                    for (name, cmds) in self.plugin_registry.commands() {
                        for cmd in cmds {
                            self.output_messages.push(format!("  {} - {}", cmd, name));
                        }
                    }
                } else {
                    self.output_messages.push(
                        "Usage: plugin [list|info <name>|run <name> <command>|commands]"
                            .to_string(),
                    );
                }
            }
            Some(&"history") | Some(&"hist") => {
                let args: Vec<&str> = parts[1..].iter().copied().collect();

                if args.is_empty() || args[0] == "list" {
                    let history = self.command_input.get_history_list(Some(20));
                    if history.is_empty() {
                        self.output_messages.push("No command history.".to_string());
                    } else {
                        self.output_messages
                            .push("Command History (last 20):".to_string());
                        for (i, (cmd, time)) in history.iter().enumerate() {
                            self.output_messages
                                .push(format!("  {}  [{}]  {}", i + 1, time, cmd));
                        }
                    }
                } else if args[0] == "search" {
                    if args.len() < 2 {
                        self.output_messages
                            .push("Usage: history search <query>".to_string());
                    } else {
                        let query = args[1..].join(" ");
                        self.command_input.search_history(&query);
                        let count = self.command_input.get_search_results_count();
                        if count == 0 {
                            self.output_messages
                                .push(format!("No matches found for '{}'", query));
                        } else {
                            self.output_messages
                                .push(format!("Found {} matches for '{}':", count, query));
                            for i in 0..count.min(10) {
                                if let Some(idx) = self.command_input.search_results.get(i) {
                                    if let Some(entry) = self.command_input.history.get(*idx) {
                                        self.output_messages.push(format!(
                                            "  {}. {}",
                                            i + 1,
                                            entry.command
                                        ));
                                    }
                                }
                            }
                        }
                    }
                } else if args[0] == "clear" {
                    self.command_input.clear_history();
                    self.output_messages
                        .push("Command history cleared.".to_string());
                } else if args[0] == "category" {
                    if args.len() < 2 {
                        self.output_messages
                            .push("Usage: history category <name>".to_string());
                        self.output_messages.push(
                            "Categories: general, files, network, system, ai, settings, plugins"
                                .to_string(),
                        );
                    } else {
                        let commands = self.command_input.get_history_by_category(args[1]);
                        if commands.is_empty() {
                            self.output_messages
                                .push(format!("No commands in category '{}'.", args[1]));
                        } else {
                            self.output_messages
                                .push(format!("Commands in '{}':", args[1]));
                            for cmd in commands.iter().take(20) {
                                self.output_messages.push(format!("  {}", cmd));
                            }
                        }
                    }
                } else if args[0] == "stat" {
                    self.output_messages.push("History Statistics:".to_string());
                    self.output_messages.push(format!(
                        "  Total commands: {}",
                        self.command_input.history.len()
                    ));
                    self.output_messages
                        .push(format!("  Max size: {}", self.command_input.max_history));

                    let categories = [
                        "general", "files", "network", "system", "ai", "settings", "plugins",
                    ];
                    for cat in categories {
                        let count = self.command_input.get_history_by_category(cat).len();
                        if count > 0 {
                            self.output_messages.push(format!("  {}: {}", cat, count));
                        }
                    }
                } else {
                    self.output_messages.push(
                        "Usage: history [list|search <query>|clear|category <name>|stat]"
                            .to_string(),
                    );
                }
            }
            Some(&"alias") => {
                let args: Vec<&str> = parts[1..].iter().copied().collect();

                if args.is_empty() || args[0] == "list" {
                    let aliases = self.alias_manager.list();
                    if aliases.is_empty() {
                        self.output_messages.push(
                            "No aliases defined. Use 'alias create <name> <command>' to add."
                                .to_string(),
                        );
                    } else {
                        self.output_messages.push("Defined Aliases:".to_string());
                        for a in aliases {
                            self.output_messages
                                .push(format!("  {} -> {}", a.name, a.command));
                        }
                    }
                } else if args[0] == "create" {
                    if args.len() < 3 {
                        self.output_messages
                            .push("Usage: alias create <name> <command>".to_string());
                    } else {
                        let name = args[1].to_string();
                        let command = args[2..].join(" ");
                        match self.alias_manager.create(name.clone(), command, None) {
                            Ok(()) => self
                                .output_messages
                                .push(format!("Alias '{}' created.", name)),
                            Err(e) => self.output_messages.push(format!("Error: {}", e)),
                        }
                    }
                } else if args[0] == "delete" || args[0] == "remove" {
                    if args.len() < 2 {
                        self.output_messages
                            .push("Usage: alias delete <name>".to_string());
                    } else {
                        match self.alias_manager.delete(args[1]) {
                            Ok(()) => self
                                .output_messages
                                .push(format!("Alias '{}' deleted.", args[1])),
                            Err(e) => self.output_messages.push(format!("Error: {}", e)),
                        }
                    }
                } else if args[0] == "info" {
                    if args.len() < 2 {
                        self.output_messages
                            .push("Usage: alias info <name>".to_string());
                    } else {
                        match self.alias_manager.get(args[1]) {
                            Some(a) => {
                                self.output_messages.push(format!("Alias: {}", a.name));
                                self.output_messages
                                    .push(format!("  Command: {}", a.command));
                                self.output_messages
                                    .push(format!("  Uses: {}", a.use_count));
                            }
                            None => self
                                .output_messages
                                .push(format!("Alias '{}' not found.", args[1])),
                        }
                    }
                } else if args[0] == "search" {
                    if args.len() < 2 {
                        self.output_messages
                            .push("Usage: alias search <prefix>".to_string());
                    } else {
                        let matches = self.alias_manager.find_by_prefix(args[1]);
                        if matches.is_empty() {
                            self.output_messages
                                .push(format!("No aliases starting with '{}'.", args[1]));
                        } else {
                            self.output_messages
                                .push(format!("Aliases starting with '{}':", args[1]));
                            for a in matches {
                                self.output_messages
                                    .push(format!("  {} -> {}", a.name, a.command));
                            }
                        }
                    }
                } else if args[0] == "stat" {
                    let (total, has_most_used, names) = self.alias_manager.get_stats();
                    self.output_messages.push("Alias Statistics:".to_string());
                    self.output_messages
                        .push(format!("  Total aliases: {}", total));
                    self.output_messages.push(format!(
                        "  All aliases: {}",
                        if names.is_empty() {
                            "none".to_string()
                        } else {
                            names
                        }
                    ));
                } else {
                    self.output_messages.push("Usage: alias [list|create <name> <cmd>|delete <name>|info <name>|search <prefix>|stat]".to_string());
                }
            }
            Some(&"env") | Some(&"set") | Some(&"unset") => {
                let cmd_name = parts[0];

                if cmd_name == "set" && (parts.len() == 1 || (parts.len() == 2 && parts[1] == "")) {
                    self.output_messages
                        .push("Usage: set VAR=value or set VAR value".to_string());
                    return;
                }

                if cmd_name == "unset" && parts.len() < 2 {
                    self.output_messages
                        .push("Usage: unset <variable>".to_string());
                    return;
                }

                let is_set_cmd = cmd_name == "set";
                let is_unset_cmd = cmd_name == "unset";

                if is_unset_cmd {
                    if let Some(val) = self.env_manager.unset(parts[1]) {
                        self.output_messages
                            .push(format!("Unset: {}={}", parts[1], val));
                    } else {
                        self.output_messages
                            .push(format!("Variable '{}' not found", parts[1]));
                    }
                } else if is_set_cmd || cmd_name == "env" {
                    let args: Vec<&str> = parts[1..].iter().copied().collect();

                    if args.is_empty() || (args.len() == 1 && args[0] == "list") {
                        let env_vars = self.env_manager.list();
                        self.output_messages
                            .push("Environment Variables:".to_string());
                        let limit = 30.min(env_vars.len());
                        for (key, value) in env_vars.iter().take(limit) {
                            let display_value = if value.len() > 50 {
                                format!("{}...", &value[..47])
                            } else {
                                value.clone()
                            };
                            self.output_messages
                                .push(format!("  {}={}", key, display_value));
                        }
                        if env_vars.len() > 30 {
                            self.output_messages
                                .push(format!("  ... and {} more", env_vars.len() - 30));
                        }
                    } else if args[0] == "get" {
                        if args.len() < 2 {
                            self.output_messages
                                .push("Usage: env get <variable>".to_string());
                        } else {
                            match self.env_manager.get(args[1]) {
                                Some(value) => {
                                    self.output_messages.push(format!("{}={}", args[1], value))
                                }
                                None => self
                                    .output_messages
                                    .push(format!("Variable '{}' not found", args[1])),
                            }
                        }
                    } else if args[0] == "set" {
                        if args.len() < 3 {
                            self.output_messages
                                .push("Usage: env set VAR value".to_string());
                        } else {
                            let var_name = args[1].to_string();
                            let var_value = args[2..].join(" ");
                            self.env_manager.set(var_name.clone(), var_value.clone());
                            self.output_messages
                                .push(format!("Set: {}={}", var_name, var_value));
                        }
                    } else if args[0] == "unset" {
                        if args.len() < 2 {
                            self.output_messages
                                .push("Usage: env unset <variable>".to_string());
                        } else {
                            if let Some(val) = self.env_manager.unset(args[1]) {
                                self.output_messages
                                    .push(format!("Unset: {}={}", args[1], val));
                            } else {
                                self.output_messages
                                    .push(format!("Variable '{}' not found", args[1]));
                            }
                        }
                    } else if args[0] == "search" {
                        if args.len() < 2 {
                            self.output_messages
                                .push("Usage: env search <pattern>".to_string());
                        } else {
                            let results = self.env_manager.list_filtered(args[1]);
                            if results.is_empty() {
                                self.output_messages
                                    .push(format!("No variables matching '{}'", args[1]));
                            } else {
                                self.output_messages
                                    .push(format!("Matching variables for '{}':", args[1]));
                                for (key, value) in results.iter().take(20) {
                                    self.output_messages.push(format!("  {}={}", key, value));
                                }
                            }
                        }
                    } else if args[0] == "path" {
                        let path_entries = self.env_manager.get_path();
                        self.output_messages.push("PATH entries:".to_string());
                        for (i, p) in path_entries.iter().enumerate() {
                            self.output_messages.push(format!("  {}: {}", i + 1, p));
                        }
                    } else if args[0] == "export" {
                        self.env_manager.export_to_env();
                        self.output_messages
                            .push("Environment exported to system.".to_string());
                    } else if args[0] == "expand" {
                        if args.len() < 2 {
                            self.output_messages
                                .push("Usage: env expand <text>".to_string());
                        } else {
                            let expanded = self.env_manager.expand(args[1]);
                            self.output_messages.push(format!("Expanded: {}", expanded));
                        }
                    } else if is_set_cmd {
                        let var_name = args[0].to_string();
                        let var_value = args[1..].join(" ");
                        self.env_manager.set(var_name, var_value);
                    } else {
                        self.output_messages.push("Usage: env [list|get <var>|set <var> <val>|unset <var>|search <pattern>|path|expand <text>]".to_string());
                    }
                }
            }
            Some(&"format") | Some(&"output") => {
                let args: Vec<&str> = parts[1..].iter().copied().collect();

                if args.is_empty() || args[0] == "show" {
                    self.output_messages.push(format!(
                        "Current format: {:?}",
                        self.output_formatter.format
                    ));
                } else if args[0] == "set" {
                    if args.len() < 2 {
                        self.output_messages
                            .push("Usage: format set <plain|json|table|markdown>".to_string());
                    } else {
                        if let Some(fmt) = output::OutputFormat::from_str(args[1]) {
                            self.output_formatter.set_format(fmt.clone());
                            self.output_messages
                                .push(format!("Output format set to: {:?}", fmt));
                        } else {
                            self.output_messages.push(
                                "Invalid format. Use: plain, json, table, markdown".to_string(),
                            );
                        }
                    }
                } else if args[0] == "demo" {
                    self.output_messages.push("Format Demo:".to_string());
                    let test_data = r#"{"name": "HELIOS", "version": "0.2.0", "features": ["AI", "Plugins", "Themes"]}"#;
                    self.output_messages.push(format!("Plain: {}", test_data));
                    self.output_messages.push("".to_string());
                    self.output_messages.push("JSON:".to_string());
                    self.output_messages
                        .push(self.output_formatter.format_output(test_data));
                    self.output_messages.push("".to_string());
                    self.output_messages.push("Table:".to_string());
                    self.output_messages.push(
                        self.output_formatter
                            .format_output("item1: value1\nitem2: value2\nitem3: value3"),
                    );
                } else if args[0] == "color" {
                    if args.len() < 2 {
                        self.output_messages
                            .push("Usage: format color <on|off>".to_string());
                    } else if args[1] == "on" {
                        self.output_formatter.color_enabled = true;
                        self.output_messages
                            .push("Color output enabled.".to_string());
                    } else if args[1] == "off" {
                        self.output_formatter.color_enabled = false;
                        self.output_messages
                            .push("Color output disabled.".to_string());
                    } else {
                        self.output_messages
                            .push("Usage: format color <on|off>".to_string());
                    }
                } else {
                    self.output_messages
                        .push("Usage: format [show|set <type>|demo|color <on|off>]".to_string());
                }
            }
            Some(&"calc") | Some(&"calculate") => {
                let args: Vec<&str> = parts[1..].iter().copied().collect();
                if args.is_empty() {
                    self.output_messages
                        .push("Usage: calc <expression> (e.g., calc 5+3*2)".to_string());
                    self.output_messages
                        .push("Operations: + - * / ^ %".to_string());
                } else {
                    let expr = args.join(" ");
                    match self.calculator.evaluate(&expr) {
                        Ok(result) => {
                            self.output_messages.push(format!("Result: {}", result));
                        }
                        Err(e) => self.output_messages.push(format!("Error: {}", e)),
                    }
                }
            }
            Some(&"note") | Some(&"notes") => {
                let args: Vec<&str> = parts[1..].iter().copied().collect();

                if args.is_empty() || args[0] == "list" {
                    let notes = self.notes_manager.list();
                    if notes.is_empty() {
                        self.output_messages.push("No notes found.".to_string());
                    } else {
                        self.output_messages
                            .push(format!("Notes ({} total):", notes.len()));
                        for n in notes.iter().take(20) {
                            self.output_messages
                                .push(format!("  [{}] {}", n.id, n.title));
                        }
                    }
                } else if args[0] == "add" {
                    if args.len() < 3 {
                        self.output_messages
                            .push("Usage: note add <title> <content>".to_string());
                    } else {
                        let title = args[1].to_string();
                        let content = args[2..].join(" ");
                        let id = self.notes_manager.add(title, content);
                        self.output_messages
                            .push(format!("Note created with ID: {}", id));
                    }
                } else if args[0] == "get" {
                    if args.len() < 2 {
                        self.output_messages
                            .push("Usage: note get <id>".to_string());
                    } else if let Ok(id) = args[1].parse::<usize>() {
                        if let Some(note) = self.notes_manager.get(id) {
                            self.output_messages.push(format!("Title: {}", note.title));
                            self.output_messages
                                .push(format!("Content: {}", note.content));
                        } else {
                            self.output_messages.push(format!("Note {} not found", id));
                        }
                    }
                } else if args[0] == "delete" || args[0] == "del" {
                    if args.len() < 2 {
                        self.output_messages
                            .push("Usage: note delete <id>".to_string());
                    } else if let Ok(id) = args[1].parse::<usize>() {
                        if self.notes_manager.delete(id) {
                            self.output_messages.push(format!("Note {} deleted", id));
                        } else {
                            self.output_messages.push(format!("Note {} not found", id));
                        }
                    }
                } else if args[0] == "search" {
                    if args.len() < 2 {
                        self.output_messages
                            .push("Usage: note search <query>".to_string());
                    } else {
                        let results = self.notes_manager.search(args[1]);
                        if results.is_empty() {
                            self.output_messages
                                .push(format!("No notes found matching '{}'", args[1]));
                        } else {
                            self.output_messages
                                .push(format!("Found {} notes:", results.len()));
                            for n in results {
                                self.output_messages
                                    .push(format!("  [{}] {}", n.id, n.title));
                            }
                        }
                    }
                } else if args[0] == "count" {
                    self.output_messages
                        .push(format!("Total notes: {}", self.notes_manager.count()));
                } else {
                    self.output_messages.push("Usage: note [list|add <title> <content>|get <id>|delete <id>|search <query>|count]".to_string());
                }
            }
            Some(&"todo") => {
                let args: Vec<&str> = parts[1..].iter().copied().collect();

                if args.is_empty() || args[0] == "list" {
                    let items = self.todo_manager.list(true);
                    if items.is_empty() {
                        self.output_messages.push("No todo items.".to_string());
                    } else {
                        self.output_messages.push(format!(
                            "Todo List ({} pending, {} completed):",
                            self.todo_manager.get_pending_count(),
                            self.todo_manager.get_completed_count()
                        ));
                        for item in items.iter().take(30) {
                            let status = if item.completed { "[x]" } else { "[ ]" };
                            let priority = "*".repeat(item.priority as usize);
                            self.output_messages.push(format!(
                                "  {} {} {} - {}",
                                status, priority, item.id, item.title
                            ));
                        }
                    }
                } else if args[0] == "add" {
                    if args.len() < 2 {
                        self.output_messages
                            .push("Usage: todo add <title> [priority 1-5]".to_string());
                    } else {
                        let priority: u8 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(1);
                        let title = args[1].to_string();
                        let description = args.get(3).map(|s| s.to_string()).unwrap_or_default();
                        let id = self.todo_manager.add(title, description, priority);
                        self.output_messages
                            .push(format!("Todo added with ID: {}", id));
                    }
                } else if args[0] == "done" || args[0] == "complete" {
                    if args.len() < 2 {
                        self.output_messages
                            .push("Usage: todo done <id>".to_string());
                    } else if let Ok(id) = args[1].parse::<usize>() {
                        if self.todo_manager.complete(id) {
                            self.output_messages
                                .push(format!("Todo {} marked complete", id));
                        } else {
                            self.output_messages.push(format!("Todo {} not found", id));
                        }
                    }
                } else if args[0] == "undo" {
                    if args.len() < 2 {
                        self.output_messages
                            .push("Usage: todo undo <id>".to_string());
                    } else if let Ok(id) = args[1].parse::<usize>() {
                        if self.todo_manager.uncomplete(id) {
                            self.output_messages
                                .push(format!("Todo {} marked pending", id));
                        } else {
                            self.output_messages.push(format!("Todo {} not found", id));
                        }
                    }
                } else if args[0] == "delete" || args[0] == "del" {
                    if args.len() < 2 {
                        self.output_messages
                            .push("Usage: todo delete <id>".to_string());
                    } else if let Ok(id) = args[1].parse::<usize>() {
                        if self.todo_manager.delete(id) {
                            self.output_messages.push(format!("Todo {} deleted", id));
                        } else {
                            self.output_messages.push(format!("Todo {} not found", id));
                        }
                    }
                } else if args[0] == "clear" {
                    self.todo_manager.clear_completed();
                    self.output_messages
                        .push("Completed todos cleared.".to_string());
                } else if args[0] == "pending" {
                    let items = self.todo_manager.list(false);
                    self.output_messages
                        .push(format!("Pending todos ({}):", items.len()));
                    for item in items {
                        let priority = "*".repeat(item.priority as usize);
                        self.output_messages
                            .push(format!("  [{}] {} - {}", priority, item.id, item.title));
                    }
                } else {
                    self.output_messages.push("Usage: todo [list|add <title> [priority]|done <id>|undo <id>|delete <id>|clear|pending]".to_string());
                }
            }
            _ => {
                self.output_messages.push(format!(
                    "Unknown command: {}. Type 'help' for available commands.",
                    cmd
                ));
            }
        }
    }
}

impl eframe::App for HeliosApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.update_time();
        self.ui_state.theme.apply(ui.ctx());

        if ui.input(|i| i.key_pressed(egui::Key::Questionmark)) {
            self.ui_state.show_shortcuts = !self.ui_state.show_shortcuts;
        }
        if ui.input(|i| i.key_pressed(egui::Key::T))
            && !self.command_input.current.contains("theme")
        {
            let themes = Theme::all();
            let current_idx = themes
                .iter()
                .position(|t| *t == self.ui_state.theme)
                .unwrap_or(0);
            let next_idx = (current_idx + 1) % themes.len();
            self.ui_state.theme = themes[next_idx].clone();
        }
        if ui.input(|i| i.key_pressed(egui::Key::T)) && self.command_input.current.is_empty() {
            let themes = Theme::all();
            let current_idx = themes
                .iter()
                .position(|t| *t == self.ui_state.theme)
                .unwrap_or(0);
            let next_idx = (current_idx + 1) % themes.len();
            self.ui_state.theme = themes[next_idx].clone();
        }
        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.ui_state.show_shortcuts = false;
        }

        egui::CentralPanel::default().show_inside(ui, |ui| {
            if self.ui_state.show_shortcuts {
                egui::Area::new(egui::Id::new("shortcuts_overlay"))
                    .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                    .show(ui.ctx(), |ui| {
                        egui::Frame::default()
                            .fill(egui::Color32::from_gray(20))
                            .stroke(egui::Stroke::new(
                                2.0,
                                egui::Color32::from_rgb(100, 150, 255),
                            ))
                            .show(ui, |ui| {
                                ui.heading("Keyboard Shortcuts");
                                ui.separator();
                                let shortcuts = ui::get_shortcuts();
                                for shortcut in shortcuts {
                                    ui.horizontal(|ui| {
                                        ui.label(format!("{}", shortcut.key));
                                        ui.label(shortcut.description);
                                    });
                                }
                                ui.separator();
                                ui.label("Press ? or Esc to close");
                            });
                    });
            }

            ui.columns(4, |columns| {
                // PANEL 1 - Navigation & Quick Stats
                columns[0].vertical(|ui| {
                    ui.add_space(5.0);

                    // Logo Area
                    ui.heading("HELIOS");
                    ui.label("v0.3.0 | Jarvis Core");
                    ui.separator();

                    // Navigation
                    let categories = ["MAIN", "AI", "FILES", "NETWORK", "SYSTEM", "CONFIG"];
                    for (i, cat) in categories.iter().enumerate() {
                        let selected = self.selected_category == i;
                        ui.add_space(3.0);
                        if ui.selectable_label(selected, *cat).clicked() {
                            self.selected_category = i;
                        }
                    }

                    ui.add_space(15.0);
                    ui.separator();

                    // Quick Stats
                    ui.label("QUICK STATS");
                    ui.add_space(5.0);
                    self.system_stats.refresh();
                    ui.label(format!("CPU: {:.0}%", self.system_stats.cpu_usage()));
                    ui.label(format!("MEM: {:.0}%", self.system_stats.memory_percent()));
                    ui.label(format!(
                        "PROCS: {}",
                        self.system_stats.system.processes().len()
                    ));
                    ui.label(format!("UPTIME: {}", self.system_stats.uptime()));
                });

                // PANEL 2 - Realtime Graphs
                columns[1].vertical(|ui| {
                    ui.add_space(5.0);
                    ui.heading("REAL-TIME METRICS");
                    ui.separator();
                    ui.add_space(5.0);

                    // CPU Graph
                    ui.label("CPU LOAD");
                    let cpu_data = self.realtime_graphs.cpu.get_normalized();
                    for (i, &val) in cpu_data.iter().take(30).enumerate() {
                        let bar_height = (val * 20.0) as usize;
                        let color = if val > 0.8 {
                            egui::Color32::RED
                        } else if val > 0.5 {
                            egui::Color32::YELLOW
                        } else {
                            egui::Color32::from_rgb(0, 200, 150)
                        };
                        ui.colored_label(color, "█".repeat(bar_height + 1));
                    }
                    ui.label(format!("{:.1}%", self.jarvis.processing_load));

                    ui.add_space(10.0);

                    // Memory Graph
                    ui.label("MEMORY USAGE");
                    let mem_data = self.realtime_graphs.memory.get_normalized();
                    for (i, &val) in mem_data.iter().take(30).enumerate() {
                        let bar_height = (val * 20.0) as usize;
                        let color = if val > 0.9 {
                            egui::Color32::RED
                        } else if val > 0.7 {
                            egui::Color32::YELLOW
                        } else {
                            egui::Color32::from_rgb(100, 100, 255)
                        };
                        ui.colored_label(color, "█".repeat(bar_height + 1));
                    }
                    ui.label(format!("{:.1}%", self.jarvis.memory_usage));

                    ui.add_space(10.0);

                    // Network Graph
                    ui.label("NETWORK ACTIVITY");
                    let net_data = self.realtime_graphs.network.get_normalized();
                    for (i, &val) in net_data.iter().take(30).enumerate() {
                        let bar_height = (val * 20.0) as usize;
                        ui.colored_label(
                            egui::Color32::from_rgb(255, 100, 50),
                            "█".repeat(bar_height + 1),
                        );
                    }
                    ui.label(format!("{:.1}%", self.jarvis.network_activity));

                    ui.add_space(10.0);

                    // Power Graph
                    ui.label("POWER CONSUMPTION");
                    let power_data = self.realtime_graphs.power.get_normalized();
                    for (i, &val) in power_data.iter().take(30).enumerate() {
                        let bar_height = (val * 20.0) as usize;
                        ui.colored_label(
                            egui::Color32::from_rgb(255, 200, 0),
                            "█".repeat(bar_height + 1),
                        );
                    }
                    ui.label(format!("{:.1}%", self.jarvis.power_consumption));
                });

                // PANEL 3 - System Status & JARVIS
                columns[2].vertical(|ui| {
                    ui.add_space(5.0);
                    ui.heading("SYSTEM STATUS");
                    ui.separator();
                    ui.add_space(5.0);

                    // JARVIS Status
                    ui.colored_label(egui::Color32::from_rgb(0, 255, 200), "● JARVIS ACTIVE");
                    ui.add_space(3.0);
                    ui.label(format!("MODE: {}", self.jarvis.system_mode));
                    ui.label(format!("THREAT: {}", self.jarvis.threat_level));
                    ui.label(format!("SECURITY: {}", self.jarvis.security_status));

                    ui.separator();
                    ui.add_space(5.0);

                    // Gauges
                    ui.label("CORE TEMPERATURE");
                    ui.add(
                        egui::ProgressBar::new(self.jarvis.core_temp / 100.0).desired_width(180.0),
                    );
                    ui.label(format!("{:.1}°C", self.jarvis.core_temp));

                    ui.add_space(8.0);
                    ui.label("PROCESSING LOAD");
                    ui.add(
                        egui::ProgressBar::new(self.jarvis.processing_load / 100.0)
                            .desired_width(180.0),
                    );
                    ui.label(format!("{:.1}%", self.jarvis.processing_load));

                    ui.add_space(8.0);
                    ui.label("AI ENGINE");
                    let ai_online = self.ollama.is_available();
                    ui.colored_label(
                        if ai_online {
                            egui::Color32::GREEN
                        } else {
                            egui::Color32::RED
                        },
                        if ai_online {
                            "● ONLINE"
                        } else {
                            "○ OFFLINE"
                        },
                    );

                    ui.separator();
                    ui.add_space(5.0);
                    ui.label("ALERTS");
                    if self.alert_manager.alerts.is_empty() {
                        ui.label("No active alerts");
                    } else {
                        for alert in self.alert_manager.alerts.iter().take(5) {
                            let color = match alert.level {
                                ui::AlertLevel::Critical => egui::Color32::RED,
                                ui::AlertLevel::Warning => egui::Color32::YELLOW,
                                ui::AlertLevel::Info => egui::Color32::LIGHT_BLUE,
                            };
                            ui.colored_label(color, &alert.message);
                        }
                    }
                });

                // PANEL 4 - Command Input
                columns[3].vertical(|ui| {
                    ui.add_space(5.0);
                    ui.heading("COMMAND CONSOLE");
                    ui.separator();
                    ui.add_space(5.0);

                    ui.horizontal(|ui| {
                        ui.label(">");
                        ui.text_edit_singleline(&mut self.command_input.current);
                    });

                    if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        let cmd = self.command_input.current.clone();
                        if !cmd.is_empty() {
                            self.command_input.push_command(cmd.clone());
                            self.execute_command(&cmd);
                        }
                    }

                    if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                        self.command_input.navigate_history_up();
                    }
                    if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                        self.command_input.navigate_history_down();
                    }

                    ui.add_space(5.0);
                    if ui.button("EXECUTE").clicked() {
                        let cmd = self.command_input.current.clone();
                        if !cmd.is_empty() {
                            self.command_input.push_command(cmd.clone());
                            self.execute_command(&cmd);
                        }
                    }

                    ui.separator();
                    egui::ScrollArea::vertical()
                        .stick_to_bottom(true)
                        .show(ui, |ui| {
                            for msg in &self.output_messages {
                                ui.label(msg);
                            }
                        });
                });
            });
        });
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_fullsize_content_view(true)
            .with_decorations(false)
            .with_transparent(true),
        ..Default::default()
    };

    eframe::run_native(
        "HELIOS",
        native_options,
        Box::new(|_cc| Ok(Box::new(HeliosApp::default()))),
    )
}
