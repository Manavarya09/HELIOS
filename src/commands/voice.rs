#[derive(Debug, Clone)]
pub struct VoiceSystem {
    pub tts_enabled: bool,
    pub stt_enabled: bool,
    pub voice_rate: f32,
    pub voice_volume: f32,
    pub is_speaking: bool,
}

impl VoiceSystem {
    pub fn new() -> Self {
        Self {
            tts_enabled: false,
            stt_enabled: false,
            voice_rate: 1.0,
            voice_volume: 0.8,
            is_speaking: false,
        }
    }

    pub fn enable_tts(&mut self) {
        self.tts_enabled = true;
    }

    pub fn enable_stt(&mut self) {
        self.stt_enabled = true;
    }

    pub fn disable(&mut self) {
        self.tts_enabled = false;
        self.stt_enabled = false;
    }

    pub fn speak(&mut self, _text: &str) {
        if self.tts_enabled {
            self.is_speaking = true;
        }
    }

    pub fn set_rate(&mut self, rate: f32) {
        self.voice_rate = rate.clamp(0.5, 2.0);
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.voice_volume = volume.clamp(0.0, 1.0);
    }

    pub fn get_status(&self) -> String {
        if self.is_speaking {
            "[SPEAKING]".to_string()
        } else if self.tts_enabled {
            "[TTS ENABLED]".to_string()
        } else if self.stt_enabled {
            "[STT ENABLED]".to_string()
        } else {
            "[VOICE OFF]".to_string()
        }
    }
}

impl Default for VoiceSystem {
    fn default() -> Self {
        Self::new()
    }
}
