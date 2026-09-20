use async_trait::async_trait;
use serde::Deserialize;
use vikar_core::{CompletionRequest, CompletionResponse, Model, ModelError};

/// Anything speaking the OpenAI `/chat/completions` schema. Covers OpenAI
/// itself as well as local servers (Ollama, vLLM, LM Studio, ...) that
/// implement the same API shape — one implementation, many backends.
pub struct OpenAiCompatibleModel {
    name: String,
    model: String,
    base_url: String,
    api_key: Option<String>,
    client: reqwest::Client,
}

impl OpenAiCompatibleModel {
    pub fn new(name: String, model: String, base_url: String, api_key: Option<String>) -> Self {
        Self {
            name,
            model,
            base_url,
            api_key,
            client: reqwest::Client::new(),
        }
    }
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: ChatMessage,
}

#[derive(Deserialize)]
struct ChatMessage {
    content: String,
}

#[async_trait]
impl Model for OpenAiCompatibleModel {
    async fn complete(&self, req: CompletionRequest) -> Result<CompletionResponse, ModelError> {
        let url = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));
        let body = serde_json::json!({
            "model": self.model,
            "messages": [{ "role": "user", "content": req.prompt }],
        });

        let mut request = self.client.post(&url).json(&body);
        if let Some(key) = &self.api_key {
            request = request.bearer_auth(key);
        }

        let resp = request.send().await.map_err(|e| ModelError::Request {
            model: self.name.clone(),
            message: e.to_string(),
        })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(ModelError::Request {
                model: self.name.clone(),
                message: format!("HTTP {status}: {text}"),
            });
        }

        let parsed: ChatResponse = resp.json().await.map_err(|e| ModelError::Response {
            model: self.name.clone(),
            message: e.to_string(),
        })?;

        let text = parsed
            .choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .unwrap_or_default();

        Ok(CompletionResponse { text })
    }

    fn name(&self) -> &str {
        &self.name
    }
}
