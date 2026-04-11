mod ai;
mod commands;
mod config;
mod plugins;
mod system;
mod ui;

use ai::{AiProvider, OllamaClient};
use commands::CommandInput;
use commands::{parse_file_command, parse_network_command, parse_system_command};
use config::AppConfig;
use eframe::egui;
use plugins::{PluginRegistry, FileManagerPlugin, NetworkToolsPlugin, ProcessManagerPlugin};
use system::SystemStats;
use ui::{Theme, UiState};

pub struct HeliosApp {
    command_input: CommandInput,
    output_messages: Vec<String>,
    ollama: OllamaClient,
    system_stats: SystemStats,
    is_processing: bool,
    selected_category: usize,
    current_time: String,
    ui_state: UiState,
    config: AppConfig,
    plugin_registry: PluginRegistry,
}

impl Default for HeliosApp {
    fn default() -> Self {
        let config = AppConfig::load();
        let mut plugin_registry = PluginRegistry::new();
        
        let _ = plugin_registry.register(Box::new(FileManagerPlugin::new()));
        let _ = plugin_registry.register(Box::new(NetworkToolsPlugin::new()));
        let _ = plugin_registry.register(Box::new(ProcessManagerPlugin::new()));
        
        Self {
            command_input: CommandInput::default(),
            output_messages: vec![
                "HELIOS v0.1.0 Command System".to_string(),
                "Type 'help' for available commands".to_string(),
            ],
            ollama: OllamaClient::default(),
            system_stats: SystemStats::new(),
            is_processing: false,
            selected_category: 0,
            current_time: "00:00:00".to_string(),
            ui_state: UiState::default(),
            config: AppConfig::load(),
            plugin_registry,
        }
    }
}

impl HeliosApp {
    fn update_time(&mut self) {
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

        self.output_messages.push(format!("> {}", command));

        let parts: Vec<&str> = cmd.split_whitespace().collect();

        match parts.first() {
            Some(&"help") => {
                self.output_messages
                    .push("HELIOS v0.1.0 - Available Commands:".to_string());
                self.output_messages.push("".to_string());
                self.output_messages.push("=== General ===".to_string());
                self.output_messages
                    .push("  help - Show this help".to_string());
                self.output_messages
                    .push("  status - Show system status".to_string());
                self.output_messages
                    .push("  clear - Clear output".to_string());
                self.output_messages
                    .push("  stats - Show system stats".to_string());
                self.output_messages
                    .push("  time - Show current time".to_string());
                self.output_messages.push("".to_string());
                self.output_messages.push("=== AI ===".to_string());
                self.output_messages
                    .push("  ai <prompt> - Ask AI".to_string());
                self.output_messages
                    .push("  chat - Start AI chat mode".to_string());
                self.output_messages.push("".to_string());
                self.output_messages
                    .push("=== File Operations ===".to_string());
                self.output_messages
                    .push("  ls [path] - List directory".to_string());
                self.output_messages
                    .push("  cd <path> - Change directory".to_string());
                self.output_messages
                    .push("  pwd - Print working directory".to_string());
                self.output_messages
                    .push("  read <file> - Read file contents".to_string());
                self.output_messages
                    .push("  write <file> <content> - Write to file".to_string());
                self.output_messages
                    .push("  mkdir <dir> - Create directory".to_string());
                self.output_messages
                    .push("  delete <path> - Delete file/directory".to_string());
                self.output_messages.push("".to_string());
                self.output_messages.push("=== Network ===".to_string());
                self.output_messages
                    .push("  ping <host> [count] - Ping a host".to_string());
                self.output_messages
                    .push("  curl <url> - Fetch URL".to_string());
                self.output_messages
                    .push("  scan <host> [start] [end] - Scan ports".to_string());
                self.output_messages.push("".to_string());
                self.output_messages.push("=== System ===".to_string());
                self.output_messages
                    .push("  processes [count] - List processes".to_string());
                self.output_messages
                    .push("  kill <pid> - Kill process".to_string());
                self.output_messages
                    .push("  info <pid> - Process info".to_string());
                self.output_messages.push("".to_string());
                self.output_messages.push("=== UI/Theme ===".to_string());
                self.output_messages
                    .push("  theme list - List themes".to_string());
                self.output_messages
                    .push("  theme set <name> - Set theme".to_string());
                self.output_messages
                    .push("  theme next - Next theme".to_string());
                self.output_messages
                    .push("  shortcuts - Toggle shortcuts overlay".to_string());
                self.output_messages.push("".to_string());
                self.output_messages.push("=== Settings ===".to_string());
                self.output_messages
                    .push("  config list - List all settings".to_string());
                self.output_messages
                    .push("  config get <key> - Get setting value".to_string());
                self.output_messages
                    .push("  config set <key> <value> - Set setting".to_string());
                self.output_messages
                    .push("  config save - Save config to file".to_string());
                self.output_messages
                    .push("  config reset - Reset to defaults".to_string());
                self.output_messages.push("".to_string());
                self.output_messages
                    .push("Press ? for shortcuts, T to cycle theme".to_string());
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
                        .push("Usage: ai <prompt> | ai config <key> <value>".to_string());
                    self.output_messages
                        .push("  ai <prompt> - Ask AI".to_string());
                    self.output_messages
                        .push("  ai chat - Start chat mode".to_string());
                    self.output_messages
                        .push("  ai clear - Clear chat history".to_string());
                    self.output_messages
                        .push("  ai history - Show chat history".to_string());
                    self.output_messages
                        .push("  ai config - Show AI config".to_string());
                    self.output_messages.push(
                        "  ai provider <name> - Set provider (ollama/openai/anthropic)".to_string(),
                    );
                    self.output_messages
                        .push("  ai model <name> - Set model".to_string());
                } else if args[0] == "config" {
                    self.output_messages
                        .push(format!("AI Config: {:?}", self.ollama.config()));
                } else if args[0] == "provider" {
                    if args.len() < 2 {
                        self.output_messages
                            .push("Usage: ai provider <ollama|openai|anthropic>".to_string());
                    } else {
                        match args[1].to_lowercase().as_str() {
                            "ollama" => {
                                self.ollama.set_provider(AiProvider::Ollama);
                                self.output_messages
                                    .push("Provider set to Ollama".to_string());
                            }
                            "openai" => {
                                self.ollama.set_provider(AiProvider::OpenAI);
                                self.output_messages
                                    .push("Provider set to OpenAI".to_string());
                            }
                            "anthropic" => {
                                self.ollama.set_provider(AiProvider::Anthropic);
                                self.output_messages
                                    .push("Provider set to Anthropic".to_string());
                            }
                            _ => self.output_messages.push(
                                "Unknown provider. Use: ollama, openai, or anthropic".to_string(),
                            ),
                        }
                    }
                } else if args[0] == "model" {
                    if args.len() < 2 {
                        self.output_messages
                            .push("Usage: ai model <model_name>".to_string());
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
                    self.output_messages.push(
                        "Chat mode not yet implemented. Use 'ai <prompt>' for single queries."
                            .to_string(),
                    );
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
                            self.output_messages.push("AI:".to_string());
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
                        self.output_messages.push("  No plugins loaded.".to_string());
                    } else {
                        for p in plugins {
                            self.output_messages.push(format!("  {} v{} - {}", p.name, p.version, p.description));
                            self.output_messages.push(format!("    Commands: {:?}", p.commands));
                        }
                    }
                } else if args[0] == "info" {
                    if args.len() < 2 {
                        self.output_messages.push("Usage: plugin info <name>".to_string());
                    } else {
                        if let Some(p) = self.plugin_registry.get(args[1]) {
                            let info = p.info();
                            self.output_messages.push(format!("Plugin: {}", info.name));
                            self.output_messages.push(format!("Version: {}", info.version));
                            self.output_messages.push(format!("Description: {}", info.description));
                            self.output_messages.push(format!("Commands: {:?}", info.commands));
                        } else {
                            self.output_messages.push(format!("Plugin '{}' not found", args[1]));
                        }
                    }
                } else if args[0] == "run" {
                    if args.len() < 3 {
                        self.output_messages.push("Usage: plugin run <name> <command> [args...]".to_string());
                    } else {
                        let plugin_name = args[1];
                        let plugin_cmd = args[2];
                        let plugin_args: Vec<&str> = args[3..].iter().copied().collect();
                        match self.plugin_registry.execute(plugin_name, plugin_cmd, &plugin_args) {
                            Ok(result) => self.output_messages.push(result),
                            Err(e) => self.output_messages.push(format!("Error: {}", e)),
                        }
                    }
                } else if args[0] == "commands" {
                    self.output_messages.push("Available Plugin Commands:".to_string());
                    for (name, cmds) in self.plugin_registry.commands() {
                        for cmd in cmds {
                            self.output_messages.push(format!("  {} - {}", cmd, name));
                        }
                    }
                } else {
                    self.output_messages.push("Usage: plugin [list|info <name>|run <name> <command>|commands]".to_string());
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

            ui.columns(3, |columns| {
                columns[0].vertical(|ui| {
                    ui.add_space(10.0);
                    ui.heading("HELIOS");
                    ui.label("v0.1.0");
                    ui.separator();
                    ui.add_space(5.0);

                    let categories = ["System", "AI", "Files", "Network", "Settings"];
                    for (i, cat) in categories.iter().enumerate() {
                        if ui
                            .selectable_label(self.selected_category == i, *cat)
                            .clicked()
                        {
                            self.selected_category = i;
                        }
                    }
                });

                columns[1].vertical(|ui| {
                    ui.add_space(10.0);
                    ui.heading("COMMAND INPUT");
                    ui.separator();
                    ui.add_space(5.0);

                    ui.text_edit_singleline(&mut self.command_input.current);

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

                    let btn_text = if self.is_processing {
                        "PROCESSING..."
                    } else {
                        "EXECUTE"
                    };
                    if ui.button(btn_text).clicked() {
                        let cmd = self.command_input.current.clone();
                        if !cmd.is_empty() {
                            self.command_input.push_command(cmd.clone());
                            self.execute_command(&cmd);
                        }
                    }

                    ui.separator();
                    ui.heading("OUTPUT");
                    ui.separator();

                    egui::ScrollArea::vertical()
                        .stick_to_bottom(true)
                        .show(ui, |ui| {
                            for msg in &self.output_messages {
                                ui.label(msg);
                            }
                        });
                });

                columns[2].vertical(|ui| {
                    ui.add_space(10.0);
                    ui.heading("SYSTEM STATUS");
                    ui.separator();
                    ui.add_space(10.0);

                    self.system_stats.refresh();

                    ui.label(format!("CPU: {:.1}%", self.system_stats.cpu_usage()));
                    ui.label(format!(
                        "Memory: {} / {} MB",
                        self.system_stats.memory_used_mb(),
                        self.system_stats.memory_total_mb()
                    ));
                    ui.add(
                        egui::ProgressBar::new(self.system_stats.memory_percent() / 100.0)
                            .desired_width(150.0),
                    );

                    ui.separator();
                    ui.add_space(5.0);

                    ui.label("AI ENGINE:");
                    let ai_status = if self.ollama.is_available() {
                        "ONLINE"
                    } else {
                        "OFFLINE"
                    };
                    ui.label(ai_status);

                    ui.separator();
                    ui.add_space(5.0);

                    ui.label(format!("Host: {}", self.system_stats.hostname()));
                    ui.label(format!("Time: {}", self.current_time));
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
