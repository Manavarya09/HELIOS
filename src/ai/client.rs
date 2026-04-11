use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AiProvider {
    Ollama,
    OpenAI,
    Anthropic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    pub provider: AiProvider,
    pub base_url: String,
    pub model: String,
    pub api_key: Option<String>,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            provider: AiProvider::Ollama,
            base_url: "http://localhost:11434".to_string(),
            model: "llama2".to_string(),
            api_key: None,
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
        self.config.provider = provider.clone();
        match provider {
            AiProvider::Ollama => {
                self.config.base_url = "http://localhost:11434".to_string();
                self.config.model = "llama2".to_string();
            }
            AiProvider::OpenAI => {
                self.config.base_url = "https://api.openai.com/v1".to_string();
                self.config.model = "gpt-3.5-turbo".to_string();
            }
            AiProvider::Anthropic => {
                self.config.base_url = "https://api.anthropic.com/v1".to_string();
                self.config.model = "claude-3-haiku-20240307".to_string();
            }
        }
    }

    pub fn set_model(&mut self, model: String) {
        self.config.model = model;
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
            AiProvider::Anthropic => self.generate_anthropic(prompt),
        }
    }

    fn generate_ollama(&self, prompt: String) -> Result<String, String> {
        let url = format!("{}/api/generate", self.config.base_url);

        let request = OllamaRequest {
            model: self.config.model.clone(),
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
            model: self.config.model.clone(),
            messages: vec![OpenAIMessage {
                role: "user".to_string(),
                content: prompt,
            }],
            temperature: 0.7,
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
            "model": self.config.model,
            "max_tokens": 1024,
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
