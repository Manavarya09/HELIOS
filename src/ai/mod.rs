use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct OllamaRequest {
    pub model: String,
    pub prompt: String,
    pub stream: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OllamaResponse {
    pub response: String,
}

pub struct OllamaClient {
    base_url: String,
    model: String,
    client: reqwest::blocking::Client,
}

impl OllamaClient {
    pub fn new(base_url: String, model: String) -> Self {
        Self {
            base_url,
            model,
            client: reqwest::blocking::Client::new(),
        }
    }

    pub fn generate(&self, prompt: String) -> Result<String, String> {
        let url = format!("{}/api/generate", self.base_url);
        
        let request = OllamaRequest {
            model: self.model.clone(),
            prompt,
            stream: false,
        };

        let response = self.client
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

    pub fn is_available(&self) -> bool {
        !self.base_url.is_empty()
    }

    pub fn check_connection(&self) -> bool {
        if let Ok(response) = self.client.get(&self.base_url).send() {
            response.status().is_success()
        } else {
            false
        }
    }
}

impl Default for OllamaClient {
    fn default() -> Self {
        Self::new("http://localhost:11434".to_string(), "llama2".to_string())
    }
}

impl Clone for OllamaClient {
    fn clone(&self) -> Self {
        Self {
            base_url: self.base_url.clone(),
            model: self.model.clone(),
            client: reqwest::blocking::Client::new(),
        }
    }
}