use serde::Deserialize;

/// Configuration for one entry under `models:` in a playbook.
///
/// Tagged on `kind`, mirroring how `TaskConfig` is tagged on `kind` — the
/// same pattern everywhere in the schema so a reader only has to learn it
/// once. Adding a new provider kind (e.g. `bedrock`) means adding one
/// variant here and one matching implementation in `vikar-cli`'s provider
/// registry; nothing else in this crate changes.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ModelConfig {
    /// Anthropic's native API.
    Anthropic {
        model: String,
        api_key_env: String,
        /// Optional override, mainly useful for testing against a proxy.
        #[serde(default)]
        base_url: Option<String>,
    },

    /// Anything speaking the OpenAI chat-completions schema — OpenAI
    /// itself, Ollama, vLLM, LM Studio, etc. Covering this family with one
    /// variant (rather than one per product) is what makes "provider
    /// agnostic" a real claim instead of a promise.
    OpenaiCompatible {
        model: String,
        base_url: String,
        /// Optional: many local servers (Ollama, etc.) don't require a key.
        #[serde(default)]
        api_key_env: Option<String>,
    },

    /// A local Ollama server, via its native `/api/chat` endpoint. No key
    /// needed by default — Ollama's OpenAI-compatible endpoint also works
    /// fine through `kind: openai_compatible`, but a dedicated variant
    /// means playbook authors don't have to know Ollama exposes both APIs
    /// or spell out `base_url: http://localhost:11434/v1` themselves.
    Ollama {
        model: String,
        #[serde(default = "default_ollama_base_url")]
        base_url: String,
    },

    /// A deterministic, offline model that needs no API key and no
    /// network access. Exists so `vikar plan`/`vikar play` on the example
    /// playbook works out of the box — nobody should have to get an API
    /// key just to see Vikar run once.
    Mock {
        /// The mock model echoes the prompt back, optionally prefixed with
        /// this string, so playbook authors can distinguish mock output
        /// from a real model's in a demo.
        #[serde(default = "default_mock_prefix")]
        prefix: String,
    },
}

fn default_mock_prefix() -> String {
    "[mock] ".to_string()
}

fn default_ollama_base_url() -> String {
    "http://localhost:11434".to_string()
}
