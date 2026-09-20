use async_trait::async_trait;
use serde::Deserialize;
use vikar_core::{CompletionRequest, CompletionResponse, Model, ModelError};

/// A local Ollama server, via its native `/api/chat` endpoint (not the
/// OpenAI-compatible one — see the doc comment on `ModelConfig::Ollama`
/// for why this has its own implementation instead of reusing
/// `OpenAiCompatibleModel`).
pub struct OllamaModel {
    name: String,
    model: String,
    base_url: String,
    client: reqwest::Client,
}

impl OllamaModel {
    pub fn new(name: String, model: String, base_url: String) -> Self {
        Self {
            name,
            model,
            base_url,
            client: reqwest::Client::new(),
        }
    }
}

#[derive(Deserialize)]
struct ChatResponse {
    message: ChatMessage,
}

#[derive(Deserialize)]
struct ChatMessage {
    content: String,
}

#[async_trait]
impl Model for OllamaModel {
    async fn complete(&self, req: CompletionRequest) -> Result<CompletionResponse, ModelError> {
        let url = format!("{}/api/chat", self.base_url.trim_end_matches('/'));
        let body = serde_json::json!({
            "model": self.model,
            "messages": [{ "role": "user", "content": req.prompt }],
            // Ollama defaults to streaming NDJSON; Vikar's `ask` task does
            // one-shot completions in v0.1 (see the project roadmap), so
            // request the single-JSON-object form explicitly.
            "stream": false,
        });

        let resp = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| ModelError::Request {
                model: self.name.clone(),
                message: format!(
                    "{e} (is Ollama running at {}? try `ollama serve`)",
                    self.base_url
                ),
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

        Ok(CompletionResponse {
            text: parsed.message.content,
        })
    }

    fn name(&self) -> &str {
        &self.name
    }
}
