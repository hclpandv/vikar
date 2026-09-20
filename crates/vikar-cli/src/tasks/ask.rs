use std::sync::Arc;

use async_trait::async_trait;
use vikar_core::{CompletionRequest, Context, Model, StepError, Task, TaskOutput};

/// The `kind: ask` task — calls an LLM via a [`Model`].
pub struct AskTask {
    name: String,
    model: Arc<dyn Model>,
    prompt_template: String,
    register: Option<String>,
}

impl AskTask {
    pub fn new(
        name: String,
        model: Arc<dyn Model>,
        prompt_template: String,
        register: Option<String>,
    ) -> Self {
        Self {
            name,
            model,
            prompt_template,
            register,
        }
    }
}

#[async_trait]
impl Task for AskTask {
    async fn execute(&self, ctx: &mut Context) -> Result<TaskOutput, StepError> {
        let prompt = ctx
            .render(&self.prompt_template)
            .map_err(|source| StepError::Template {
                task: self.name.clone(),
                source,
            })?;

        tracing::info!(task = %self.name, model = self.model.name(), "asking model");

        let response = self
            .model
            .complete(CompletionRequest { prompt })
            .await
            .map_err(|source| StepError::Model {
                task: self.name.clone(),
                source,
            })?;

        let result = TaskOutput::text(response.text);

        // Same reasoning as `run`: print the model's answer to the
        // terminal as it happens, not just into `Context`. Otherwise an
        // `ask` task with no following `run`/echo appears to do nothing.
        if let Some(text) = &result.raw_text {
            println!("{text}");
        }

        if let Some(name) = &self.register {
            ctx.register(name.clone(), result.value.clone());
        }

        Ok(result)
    }

    fn name(&self) -> &str {
        &self.name
    }
}
