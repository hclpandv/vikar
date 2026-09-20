use async_trait::async_trait;

use crate::error::ModelError;

/// A single request to a [`Model`]. Deliberately minimal for v0.1: one
/// prompt in, one completion out. No tool-calling, no streaming, no
/// multi-turn history — those are real features, added when an `ask` task
/// actually needs them, not speculatively here.
#[derive(Debug, Clone)]
pub struct CompletionRequest {
    pub prompt: String,
}

/// A single response from a [`Model`].
#[derive(Debug, Clone)]
pub struct CompletionResponse {
    pub text: String,
}

/// The LLM provider boundary.
///
/// An `ask` task holds an `Arc<dyn Model>` and calls [`Model::complete`].
/// It does not know or care whether that's Anthropic, an OpenAI-compatible
/// endpoint, or a local model — that's the entire point of this trait.
///
/// # Keep this trait small
///
/// Every method added here is a method every third-party `Model`
/// implementation has to keep up with. Resist adding parameters
/// (tool schemas, streaming callbacks, etc.) until an actual task kind
/// needs them — grow this by necessity, not anticipation.
#[async_trait]
pub trait Model: Send + Sync {
    /// Run one completion request against this model.
    async fn complete(&self, req: CompletionRequest) -> Result<CompletionResponse, ModelError>;

    /// A short, stable identifier for this model instance, used in error
    /// messages and logs (e.g. `"claude-main"`, matching the key under
    /// `models:` in the playbook).
    fn name(&self) -> &str;
}
