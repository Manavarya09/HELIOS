use eframe::egui;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum Theme {
    Dark,
    Light,
    Cyberpunk,
    Ocean,
    Forest,
    Neon,
    Matrix,
}

impl Default for Theme {
    fn default() -> Self {
        Theme::Neon
    }
}

impl Theme {
    pub fn all() -> &'static [Theme] {
        &[
            Theme::Dark,
            Theme::Light,
            Theme::Cyberpunk,
            Theme::Ocean,
            Theme::Forest,
            Theme::Neon,
            Theme::Matrix,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            Theme::Dark => "Dark",
            Theme::Light => "Light",
            Theme::Cyberpunk => "Cyberpunk",
            Theme::Ocean => "Ocean",
            Theme::Forest => "Forest",
            Theme::Neon => "Neon",
            Theme::Matrix => "Matrix",
        }
    }

    pub fn apply(&self, ctx: &egui::Context) {
        let visuals = match self {
            Theme::Dark => egui::Visuals::dark(),
            Theme::Light => egui::Visuals::light(),
            Theme::Cyberpunk => {
                let mut visuals = egui::Visuals::dark();
                visuals_override_cyberpunk(&mut visuals);
                visuals
            }
            Theme::Ocean => {
                let mut visuals = egui::Visuals::dark();
                visuals_override_ocean(&mut visuals);
                visuals
            }
            Theme::Forest => {
                let mut visuals = egui::Visuals::dark();
                visuals_override_forest(&mut visuals);
                visuals
            }
            Theme::Neon => {
                let mut visuals = egui::Visuals::dark();
                visuals_override_neon(&mut visuals);
                visuals
            }
            Theme::Matrix => {
                let mut visuals = egui::Visuals::dark();
                visuals_override_matrix(&mut visuals);
                visuals
            }
        };
        ctx.set_visuals(visuals);
    }
}

fn visuals_override_cyberpunk(visuals: &mut egui::Visuals) {
    visuals.widgets.noninteractive.bg_fill = egui::Color32::from_gray(20);
    visuals.widgets.noninteractive.bg_stroke =
        egui::Stroke::new(1.0, egui::Color32::from_rgb(255, 0, 128));
    visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(40, 0, 60);
    visuals.widgets.active.bg_fill = egui::Color32::from_rgb(60, 0, 80);
    visuals.selection.bg_fill = egui::Color32::from_rgb(255, 0, 128);
    visuals.selection.stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(255, 0, 255));
}

fn visuals_override_ocean(visuals: &mut egui::Visuals) {
    visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(10, 30, 50);
    visuals.widgets.noninteractive.bg_stroke =
        egui::Stroke::new(1.0, egui::Color32::from_rgb(0, 150, 200));
    visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(20, 60, 90);
    visuals.widgets.active.bg_fill = egui::Color32::from_rgb(30, 80, 120);
    visuals.selection.bg_fill = egui::Color32::from_rgb(0, 150, 200);
    visuals.selection.stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(100, 200, 255));
}

fn visuals_override_forest(visuals: &mut egui::Visuals) {
    visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(20, 40, 20);
    visuals.widgets.noninteractive.bg_stroke =
        egui::Stroke::new(1.0, egui::Color32::from_rgb(50, 150, 50));
    visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(30, 60, 30);
    visuals.widgets.active.bg_fill = egui::Color32::from_rgb(40, 80, 40);
    visuals.selection.bg_fill = egui::Color32::from_rgb(50, 150, 50);
    visuals.selection.stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(100, 200, 100));
}

fn visuals_override_neon(visuals: &mut egui::Visuals) {
    visuals.panel_fill = egui::Color32::from_rgb(10, 10, 20);
    visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(15, 15, 30);
    visuals.widgets.noninteractive.bg_stroke =
        egui::Stroke::new(1.5, egui::Color32::from_rgb(0, 255, 200));
    visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(0, 60, 50);
    visuals.widgets.active.bg_fill = egui::Color32::from_rgb(0, 100, 80);
    visuals.selection.bg_fill = egui::Color32::from_rgb(0, 255, 200);
    visuals.selection.stroke = egui::Stroke::new(2.0, egui::Color32::from_rgb(0, 255, 255));
    visuals.faint_bg_color = egui::Color32::from_rgb(20, 20, 40);
    visuals.override_text_color = Some(egui::Color32::from_rgb(200, 255, 230));
}

fn visuals_override_matrix(visuals: &mut egui::Visuals) {
    visuals.panel_fill = egui::Color32::from_rgb(0, 10, 0);
    visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(0, 20, 0);
    visuals.widgets.noninteractive.bg_stroke =
        egui::Stroke::new(1.0, egui::Color32::from_rgb(0, 200, 0));
    visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(0, 40, 0);
    visuals.widgets.active.bg_fill = egui::Color32::from_rgb(0, 60, 0);
    visuals.selection.bg_fill = egui::Color32::from_rgb(0, 180, 0);
    visuals.selection.stroke = egui::Stroke::new(1.5, egui::Color32::from_rgb(100, 255, 100));
    visuals.override_text_color = Some(egui::Color32::from_rgb(150, 255, 150));
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiState {
    pub theme: Theme,
    pub show_shortcuts: bool,
    pub panel_widths: [f32; 3],
    pub output_wrap: bool,
    pub timestamp_format: bool,
    pub animations_enabled: bool,
    pub glow_effects: bool,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            theme: Theme::Neon,
            show_shortcuts: false,
            panel_widths: [150.0, 400.0, 200.0],
            output_wrap: true,
            timestamp_format: true,
            animations_enabled: true,
            glow_effects: true,
        }
    }
}
