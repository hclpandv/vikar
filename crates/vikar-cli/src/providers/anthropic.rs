use async_trait::async_trait;
use serde::Deserialize;
use vikar_core::{CompletionRequest, CompletionResponse, Model, ModelError};

const DEFAULT_BASE_URL: &str = "https://api.anthropic.com";
const ANTHROPIC_VERSION: &str = "2023-06-01";
const MAX_TOKENS: u32 = 4096;

/// Anthropic's native Messages API.
pub struct AnthropicModel {
    name: String,
    model: String,
    api_key: String,
    base_url: String,
    client: reqwest::Client,
}

impl AnthropicModel {
    pub fn new(name: String, model: String, api_key: String, base_url: Option<String>) -> Self {
        Self {
            name,
            model,
            api_key,
            base_url: base_url.unwrap_or_else(|| DEFAULT_BASE_URL.to_string()),
            client: reqwest::Client::new(),
        }
    }
}

#[derive(Deserialize)]
struct MessagesResponse {
    content: Vec<ContentBlock>,
}

#[derive(Deserialize)]
struct ContentBlock {
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    text: String,
}

#[async_trait]
impl Model for AnthropicModel {
    async fn complete(&self, req: CompletionRequest) -> Result<CompletionResponse, ModelError> {
        let url = format!("{}/v1/messages", self.base_url);
        let body = serde_json::json!({
            "model": self.model,
            "max_tokens": MAX_TOKENS,
            "messages": [{ "role": "user", "content": req.prompt }],
        });

        let resp = self
            .client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", ANTHROPIC_VERSION)
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| ModelError::Request {
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

        let parsed: MessagesResponse = resp.json().await.map_err(|e| ModelError::Response {
            model: self.name.clone(),
            message: e.to_string(),
        })?;

        let text = parsed
            .content
            .into_iter()
            .filter(|b| b.kind == "text")
            .map(|b| b.text)
            .collect::<Vec<_>>()
            .join("");

        Ok(CompletionResponse { text })
    }

    fn name(&self) -> &str {
        &self.name
    }
}
