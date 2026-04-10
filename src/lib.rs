mod commands;
mod ai;
mod system;

use commands::CommandInput;
use ai::OllamaClient;
use system::SystemStats;
use eframe::egui;

pub struct HeliosApp {
    command_input: CommandInput,
    output_messages: Vec<String>,
    ollama: OllamaClient,
    system_stats: SystemStats,
    is_processing: bool,
    selected_category: usize,
}

impl Default for HeliosApp {
    fn default() -> Self {
        Self {
            command_input: CommandInput::default(),
            output_messages: vec!["HELIOS v0.1.0 initialized.".to_string()],
            ollama: OllamaClient::default(),
            system_stats: SystemStats::new(),
            is_processing: false,
            selected_category: 0,
        }
    }
}

impl HeliosApp {
    fn execute_command(&mut self, command: &str) {
        let cmd = command.trim().to_lowercase();
        
        if cmd.is_empty() {
            return;
        }

        self.output_messages.push(format!("> {}", command));

        match cmd.split_whitespace().next() {
            Some("help") => {
                self.output_messages.push("Available commands:".to_string());
                self.output_messages.push("  help - Show this help".to_string());
                self.output_messages.push("  status - Show system status".to_string());
                self.output_messages.push("  clear - Clear output".to_string());
                self.output_messages.push("  ai <prompt> - Ask AI".to_string());
                self.output_messages.push("  stats - Show system stats".to_string());
            }
            Some("status") => {
                self.system_stats.refresh();
                let status = self.system_stats.summary();
                self.output_messages.push(format!("System: {}", status));
                self.output_messages.push(format!("AI: {}", if self.ollama.is_available() { "Ready" } else { "Unavailable" }));
            }
            Some("clear") => {
                self.output_messages.clear();
                self.output_messages.push("Output cleared.".to_string());
            }
            Some("stats") => {
                self.system_stats.refresh();
                self.output_messages.push(self.system_stats.summary());
            }
            Some("ai") => {
                let prompt = command[2..].trim();
                if prompt.is_empty() {
                    self.output_messages.push("Usage: ai <prompt>".to_string());
                } else {
                    self.output_messages.push("Processing...".to_string());
                    self.is_processing = true;
                    match self.ollama.generate(prompt.to_string()) {
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
            _ => {
                self.output_messages.push(format!("Unknown command: {}. Type 'help' for available commands.", cmd));
            }
        }
    }
}

impl eframe::App for HeliosApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.columns(3, |columns| {
                columns[0].vertical(|ui| {
                    ui.heading("COMMANDS");
                    ui.separator();
                    let categories = ["System", "AI", "Files", "Network", "Settings"];
                    for (i, cat) in categories.iter().enumerate() {
                        if ui.selectable_label(self.selected_category == i, *cat).clicked() {
                            self.selected_category = i;
                        }
                    }
                });

                columns[1].vertical(|ui| {
                    ui.heading("INPUT");
                    ui.separator();
                    
                    let text_edit = ui.text_edit_singleline(&mut self.command_input.current);
                    if text_edit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
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
                    
                    if ui.input(|i| i.key_pressed(egui::Key::Tab)) {
                        self.command_input.apply_selected_suggestion();
                    }

                    if !self.command_input.suggestions.is_empty() {
                        ui.separator();
                        let suggestions = self.command_input.suggestions.clone();
                        let selected = self.command_input.selected_suggestion;
                        for (i, suggestion) in suggestions.iter().enumerate() {
                            let is_selected = Some(i) == selected;
                            let label = if is_selected {
                                ui.label(format!("> {}", suggestion))
                            } else {
                                ui.label(suggestion.clone())
                            };
                            if label.interact(egui::Sense::click()).clicked() {
                                self.command_input.current = suggestion.clone();
                                self.command_input.suggestions.clear();
                            }
                        }
                    }

                    if ui.button("EXECUTE").clicked() {
                        let cmd = self.command_input.current.clone();
                        if !cmd.is_empty() {
                            self.command_input.push_command(cmd.clone());
                            self.execute_command(&cmd);
                        }
                    }
                    
                    ui.separator();
                    ui.heading("OUTPUT");
                    ui.separator();
                    
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for msg in &self.output_messages {
                            ui.label(msg);
                        }
                    });
                });

                columns[2].vertical(|ui| {
                    ui.heading("STATUS");
                    ui.separator();
                    self.system_stats.refresh();
                    ui.label(format!("CPU: {:.1}%", self.system_stats.cpu_usage()));
                    ui.label(format!("RAM: {}MB / {}MB", self.system_stats.memory_used_mb(), self.system_stats.memory_total_mb()));
                    ui.label(format!("AI: {}", if self.ollama.is_available() { "Ready" } else { "Offline" }));
                    ui.label(format!("Host: {}", self.system_stats.hostname()));
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