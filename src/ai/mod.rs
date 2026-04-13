pub mod client;

pub use client::AiClient;

pub type OllamaClient = AiClient;

#[derive(Debug, Clone)]
pub struct HolographicAI {
    pub active: bool,
    pub projection_mode: String,
    pub voice_enabled: bool,
    pub emotion_state: String,
    pub processing_speed: f32,
    pub neural_links: usize,
    pub quantum_state: String,
}

impl HolographicAI {
    pub fn new() -> Self {
        Self {
            active: true,
            projection_mode: "STANDARD".to_string(),
            voice_enabled: false,
            emotion_state: "CALM".to_string(),
            processing_speed: 100.0,
            neural_links: 0,
            quantum_state: "COHERENT".to_string(),
        }
    }

    pub fn update(&mut self) {
        use std::time::SystemTime;
        let time = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as f32;

        self.processing_speed = 80.0 + (time.sin().abs() * 20.0);
        self.neural_links = ((time / 1000.0).sin().abs() * 50.0) as usize;
    }

    pub fn set_emotion(&mut self, emotion: &str) {
        self.emotion_state = emotion.to_string();
    }

    pub fn activate_voice(&mut self) {
        self.voice_enabled = true;
    }

    pub fn deactivate_voice(&mut self) {
        self.voice_enabled = false;
    }

    pub fn get_status_indicator(&self) -> &'static str {
        if self.active {
            "[AI ONLINE]"
        } else {
            "[AI OFFLINE]"
        }
    }
}

impl Default for HolographicAI {
    fn default() -> Self {
        Self::new()
    }
}
