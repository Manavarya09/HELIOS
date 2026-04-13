use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AiProvider {
    Ollama,
    OpenAI,
    Anthropic,
    Claude,
    Gemini,
    DeepSeek,
}

impl Default for AiProvider {
    fn default() -> Self {
        AiProvider::Ollama
    }
}

impl std::fmt::Display for AiProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AiProvider::Ollama => write!(f, "OLLAMA"),
            AiProvider::OpenAI => write!(f, "OPENAI"),
            AiProvider::Anthropic => write!(f, "ANTHROPIC"),
            AiProvider::Claude => write!(f, "CLAUDE"),
            AiProvider::Gemini => write!(f, "GEMINI"),
            AiProvider::DeepSeek => write!(f, "DEEPSEEK"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AiModel {
    Llama2,
    Llama3,
    Mistral,
    Codellama,
    Gpt35Turbo,
    Gpt4,
    Gpt4Turbo,
    Claude2,
    Claude3,
    GeminiPro,
    DeepSeekChat,
}

impl Default for AiModel {
    fn default() -> Self {
        AiModel::Llama2
    }
}

impl AiModel {
    pub fn name(&self) -> &'static str {
        match self {
            AiModel::Llama2 => "llama2",
            AiModel::Llama3 => "llama3",
            AiModel::Mistral => "mistral",
            AiModel::Codellama => "codellama",
            AiModel::Gpt35Turbo => "gpt-3.5-turbo",
            AiModel::Gpt4 => "gpt-4",
            AiModel::Gpt4Turbo => "gpt-4-turbo",
            AiModel::Claude2 => "claude-2",
            AiModel::Claude3 => "claude-3-opus",
            AiModel::GeminiPro => "gemini-pro",
            AiModel::DeepSeekChat => "deepseek-chat",
        }
    }

    pub fn all() -> Vec<AiModel> {
        vec![
            AiModel::Llama2,
            AiModel::Llama3,
            AiModel::Mistral,
            AiModel::Codellama,
            AiModel::Gpt35Turbo,
            AiModel::Gpt4,
            AiModel::Gpt4Turbo,
            AiModel::Claude2,
            AiModel::Claude3,
            AiModel::GeminiPro,
            AiModel::DeepSeekChat,
        ]
    }
}

impl std::fmt::Display for AiModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    pub provider: AiProvider,
    pub model: AiModel,
    pub base_url: String,
    pub api_key: Option<String>,
    pub temperature: f32,
    pub max_tokens: usize,
    pub thinking_enabled: bool,
    pub context_window: usize,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            provider: AiProvider::Ollama,
            base_url: "http://localhost:11434".to_string(),
            model: AiModel::Llama2,
            api_key: None,
            temperature: 0.7,
            max_tokens: 2048,
            thinking_enabled: false,
            context_window: 4096,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    pub timestamp: std::time::SystemTime,
}

impl ChatMessage {
    pub fn user(content: String) -> Self {
        Self {
            role: "user".to_string(),
            content,
            timestamp: std::time::SystemTime::now(),
        }
    }

    pub fn assistant(content: String) -> Self {
        Self {
            role: "assistant".to_string(),
            content,
            timestamp: std::time::SystemTime::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaRequest {
    pub model: String,
    pub prompt: String,
    pub stream: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaResponse {
    pub response: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIRequest {
    pub model: String,
    pub messages: Vec<OpenAIMessage>,
    pub temperature: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIResponse {
    pub choices: Vec<OpenAIChoice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIChoice {
    pub message: OpenAIMessage,
}

pub struct AiClient {
    config: AiConfig,
    client: reqwest::blocking::Client,
    chat_history: Vec<ChatMessage>,
}

impl AiClient {
    pub fn new(config: AiConfig) -> Self {
        Self {
            config,
            client: reqwest::blocking::Client::new(),
            chat_history: Vec::new(),
        }
    }

    pub fn with_defaults() -> Self {
        Self::new(AiConfig::default())
    }

    pub fn set_provider(&mut self, provider: AiProvider) {
        self.config.provider = provider;
        match self.config.provider {
            AiProvider::Ollama => {
                self.config.base_url = "http://localhost:11434".to_string();
                self.config.model = AiModel::Llama2;
            }
            AiProvider::OpenAI => {
                self.config.base_url = "https://api.openai.com/v1".to_string();
                self.config.model = AiModel::Gpt35Turbo;
            }
            AiProvider::Anthropic | AiProvider::Claude => {
                self.config.base_url = "https://api.anthropic.com/v1".to_string();
                self.config.model = AiModel::Claude3;
            }
            AiProvider::Gemini => {
                self.config.base_url = "https://generativelanguage.googleapis.com/v1".to_string();
                self.config.model = AiModel::GeminiPro;
            }
            AiProvider::DeepSeek => {
                self.config.base_url = "https://api.deepseek.com/v1".to_string();
                self.config.model = AiModel::DeepSeekChat;
            }
        }
    }

    pub fn set_model(&mut self, model: String) {
        let ai_model = match model.to_lowercase().as_str() {
            "llama2" => AiModel::Llama2,
            "llama3" => AiModel::Llama3,
            "mistral" => AiModel::Mistral,
            "codellama" => AiModel::Codellama,
            "gpt-3.5-turbo" | "gpt3.5" | "gpt-3.5" => AiModel::Gpt35Turbo,
            "gpt-4" | "gpt4" => AiModel::Gpt4,
            "gpt-4-turbo" | "gpt4turbo" => AiModel::Gpt4Turbo,
            "claude-2" | "claude2" => AiModel::Claude2,
            "claude-3" | "claude3" => AiModel::Claude3,
            "gemini-pro" | "geminipro" => AiModel::GeminiPro,
            "deepseek-chat" | "deepseek" => AiModel::DeepSeekChat,
            _ => AiModel::Llama2,
        };
        self.config.model = ai_model;
    }

    pub fn set_api_key(&mut self, key: String) {
        self.config.api_key = Some(key);
    }

    pub fn set_base_url(&mut self, url: String) {
        self.config.base_url = url;
    }

    pub fn generate(&self, prompt: String) -> Result<String, String> {
        match self.config.provider {
            AiProvider::Ollama => self.generate_ollama(prompt),
            AiProvider::OpenAI => self.generate_openai(prompt),
            AiProvider::Anthropic | AiProvider::Claude => self.generate_anthropic(prompt),
            AiProvider::Gemini => self.generate_ollama(prompt), // Fallback
            AiProvider::DeepSeek => self.generate_ollama(prompt), // Fallback
        }
    }

    fn generate_ollama(&self, prompt: String) -> Result<String, String> {
        let url = format!("{}/api/generate", self.config.base_url);

        let request = OllamaRequest {
            model: self.config.model.name().to_string(),
            prompt,
            stream: false,
        };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .map_err(|e| format!("Request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Ollama error: {}", response.status()));
        }

        let ollama_response: OllamaResponse = response
            .json()
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        Ok(ollama_response.response)
    }

    fn generate_openai(&self, prompt: String) -> Result<String, String> {
        let api_key = self
            .config
            .api_key
            .as_ref()
            .ok_or("OpenAI requires an API key")?;

        let url = format!("{}/chat/completions", self.config.base_url);

        let request = OpenAIRequest {
            model: self.config.model.name().to_string(),
            messages: vec![OpenAIMessage {
                role: "user".to_string(),
                content: prompt,
            }],
            temperature: self.config.temperature,
        };

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .map_err(|e| format!("Request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("OpenAI error: {}", response.status()));
        }

        let openai_response: OpenAIResponse = response
            .json()
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        openai_response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .ok_or_else(|| "No response from OpenAI".to_string())
    }

    fn generate_anthropic(&self, prompt: String) -> Result<String, String> {
        let api_key = self
            .config
            .api_key
            .as_ref()
            .ok_or("Anthropic requires an API key")?;

        let url = format!("{}/messages", self.config.base_url);

        let body = serde_json::json!({
            "model": self.config.model.name(),
            "max_tokens": self.config.max_tokens,
            "messages": [{
                "role": "user",
                "content": prompt
            }]
        });

        let response = self
            .client
            .post(&url)
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .map_err(|e| format!("Request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Anthropic error: {}", response.status()));
        }

        let resp_text = response
            .text()
            .map_err(|e| format!("Failed to read response: {}", e))?;

        let resp_json: serde_json::Value = serde_json::from_str(&resp_text)
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        resp_json["content"]
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|msg| msg.get("text"))
            .and_then(|t| t.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| "No response from Anthropic".to_string())
    }

    pub fn chat(&mut self, message: String) -> Result<String, String> {
        self.chat_history.push(ChatMessage::user(message.clone()));

        let context: String = self
            .chat_history
            .iter()
            .map(|m| format!("{}: {}", m.role, m.content))
            .collect::<Vec<_>>()
            .join("\n");

        let prompt = if self.chat_history.len() > 1 {
            format!("Conversation history:\n{}\n\nUser: {}", context, message)
        } else {
            message.clone()
        };

        let response = self.generate(prompt)?;

        self.chat_history
            .push(ChatMessage::assistant(response.clone()));

        Ok(response)
    }

    pub fn clear_history(&mut self) {
        self.chat_history.clear();
    }

    pub fn history(&self) -> &[ChatMessage] {
        &self.chat_history
    }

    pub fn is_available(&self) -> bool {
        !self.config.base_url.is_empty()
    }

    pub fn config(&self) -> &AiConfig {
        &self.config
    }

    pub fn check_connection(&self) -> bool {
        if let Ok(response) = self.client.get(&self.config.base_url).send() {
            response.status().is_success()
        } else {
            false
        }
    }
}

impl Default for AiClient {
    fn default() -> Self {
        Self::with_defaults()
    }
}

impl Clone for AiClient {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            client: reqwest::blocking::Client::new(),
            chat_history: self.chat_history.clone(),
        }
    }
}
