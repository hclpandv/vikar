use async_trait::async_trait;
use vikar_core::{CompletionRequest, CompletionResponse, Model, ModelError};

/// A deterministic, offline model. Echoes the prompt back with a prefix.
///
/// Exists so `vikar play examples/hello.yaml` works with zero setup — no
/// API key, no network access, nothing to configure. This is what a new
/// user should be able to run within the first minute of finding the
/// project.
pub struct MockModel {
    name: String,
    prefix: String,
}

impl MockModel {
    pub fn new(name: String, prefix: String) -> Self {
        Self { name, prefix }
    }
}

#[async_trait]
impl Model for MockModel {
    async fn complete(&self, req: CompletionRequest) -> Result<CompletionResponse, ModelError> {
        Ok(CompletionResponse {
            text: format!("{}{}", self.prefix, req.prompt),
        })
    }

    fn name(&self) -> &str {
        &self.name
    }
}
