mod anthropic;
mod mock;
mod ollama;
mod openai_compatible;

use std::collections::HashMap;
use std::sync::Arc;

use vikar_config::ModelConfig;
use vikar_core::Model;

/// Build every `models:` entry in a playbook into a live `Arc<dyn Model>`,
/// keyed by the same name used in `ask` tasks' `model:` field.
///
/// This is the provider-side counterpart of `compile::compile_tasks` — the
/// one place a `ModelConfig` variant maps onto a concrete `Model`
/// implementation. Adding a fourth built-in provider means adding one arm
/// here and one new type under `providers/`.
pub fn build_models(
    configs: &HashMap<String, ModelConfig>,
) -> anyhow::Result<HashMap<String, Arc<dyn Model>>> {
    let mut out = HashMap::with_capacity(configs.len());
    for (name, config) in configs {
        out.insert(name.clone(), build_model(name, config)?);
    }
    Ok(out)
}

fn build_model(name: &str, config: &ModelConfig) -> anyhow::Result<Arc<dyn Model>> {
    match config {
        ModelConfig::Anthropic {
            model,
            api_key_env,
            base_url,
        } => {
            let api_key = std::env::var(api_key_env).map_err(|_| {
                anyhow::anyhow!("model '{name}': environment variable `{api_key_env}` is not set")
            })?;
            Ok(Arc::new(anthropic::AnthropicModel::new(
                name.to_string(),
                model.clone(),
                api_key,
                base_url.clone(),
            )))
        }
        ModelConfig::OpenaiCompatible {
            model,
            base_url,
            api_key_env,
        } => {
            let api_key = match api_key_env {
                Some(env_var) => Some(std::env::var(env_var).map_err(|_| {
                    anyhow::anyhow!("model '{name}': environment variable `{env_var}` is not set")
                })?),
                None => None,
            };
            Ok(Arc::new(openai_compatible::OpenAiCompatibleModel::new(
                name.to_string(),
                model.clone(),
                base_url.clone(),
                api_key,
            )))
        }
        ModelConfig::Ollama { model, base_url } => Ok(Arc::new(ollama::OllamaModel::new(
            name.to_string(),
            model.clone(),
            base_url.clone(),
        ))),
        ModelConfig::Mock { prefix } => Ok(Arc::new(mock::MockModel::new(
            name.to_string(),
            prefix.clone(),
        ))),
    }
}
