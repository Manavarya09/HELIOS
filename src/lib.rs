mod ai;
mod commands;
mod system;

use ai::{AiProvider, OllamaClient};
use commands::CommandInput;
use commands::{parse_file_command, parse_network_command, parse_system_command};
use eframe::egui;
use system::SystemStats;

pub struct HeliosApp {
    command_input: CommandInput,
    output_messages: Vec<String>,
    ollama: OllamaClient,
    system_stats: SystemStats,
    is_processing: bool,
    selected_category: usize,
    current_time: String,
}

impl Default for HeliosApp {
    fn default() -> Self {
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

        egui::CentralPanel::default().show_inside(ui, |ui| {
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
