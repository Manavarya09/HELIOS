use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceConfig {
    pub enabled: bool,
    pub tts_enabled: bool,
    pub stt_enabled: bool,
    pub voice_rate: f32,
    pub voice_pitch: f32,
    pub voice_volume: f32,
    pub selected_voice: String,
}

impl Default for VoiceConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            tts_enabled: false,
            stt_enabled: false,
            voice_rate: 1.0,
            voice_pitch: 1.0,
            voice_volume: 0.8,
            selected_voice: "default".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct VoiceSystem {
    pub config: VoiceConfig,
    pub is_speaking: bool,
    pub transcription: String,
    pub audio_level: f32,
    pub language: String,
    pub mode: VoiceMode,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VoiceMode {
    Off,
    Listen,
    Speak,
    Continuous,
}

impl VoiceSystem {
    pub fn new() -> Self {
        Self {
            config: VoiceConfig::default(),
            is_speaking: false,
            transcription: String::new(),
            audio_level: 0.0,
            language: "en-US".to_string(),
            mode: VoiceMode::Off,
        }
    }

    pub fn enable_tts(&mut self) {
        self.config.tts_enabled = true;
        self.config.enabled = true;
        self.mode = VoiceMode::Speak;
    }

    pub fn enable_stt(&mut self) {
        self.config.stt_enabled = true;
        self.config.enabled = true;
        self.mode = VoiceMode::Listen;
    }

    pub fn disable(&mut self) {
        self.config.enabled = false;
        self.config.tts_enabled = false;
        self.config.stt_enabled = false;
        self.mode = VoiceMode::Off;
    }

    pub fn speak(&mut self, text: &str) {
        if self.config.tts_enabled {
            self.is_speaking = true;
        }
    }

    pub fn listen(&mut self) {
        if self.config.stt_enabled {
            self.mode = VoiceMode::Listen;
        }
    }

    pub fn set_rate(&mut self, rate: f32) {
        self.config.voice_rate = rate.clamp(0.5, 2.0);
    }

    pub fn set_pitch(&mut self, pitch: f32) {
        self.config.voice_pitch = pitch.clamp(0.5, 2.0);
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.config.voice_volume = volume.clamp(0.0, 1.0);
    }

    pub fn get_status(&self) -> String {
        if self.is_speaking {
            "[SPEAKING]".to_string()
        } else {
            match self.mode {
                VoiceMode::Off => "[VOICE OFF]".to_string(),
                VoiceMode::Listen => "[LISTENING]".to_string(),
                VoiceMode::Speak => "[READY TO SPEAK]".to_string(),
                VoiceMode::Continuous => "[CONTINUOUS]".to_string(),
            }
        }
    }
}

impl Default for VoiceSystem {
    fn default() -> Self {
        Self::new()
    }
}
