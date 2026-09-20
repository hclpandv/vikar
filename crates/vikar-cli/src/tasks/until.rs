use async_trait::async_trait;
use vikar_core::{Context, Runner, StepError, Task, TaskOutput};

/// The `kind: until` task — repeats a nested list of tasks against the
/// *same* context (see [`Runner::run_with_context`]) until `condition`
/// evaluates true, or `max_iterations` is reached.
///
/// `Runner` itself has no idea loops exist; `UntilTask` simply owns its own
/// nested `Runner` and calls it in a loop. That's the payoff of keeping
/// `Task` and `Runner`'s contracts small — a loop is just another `Task`
/// implementation, not a special case the orchestrator has to know about.
pub struct UntilTask {
    name: String,
    condition: String,
    max_iterations: u32,
    body: Runner,
}

impl UntilTask {
    pub fn new(
        name: String,
        condition: String,
        max_iterations: u32,
        body_tasks: Vec<Box<dyn Task>>,
    ) -> Self {
        Self {
            name,
            condition,
            max_iterations,
            body: Runner::new(body_tasks),
        }
    }
}

#[async_trait]
impl Task for UntilTask {
    async fn execute(&self, ctx: &mut Context) -> Result<TaskOutput, StepError> {
        for iteration in 1..=self.max_iterations {
            tracing::info!(task = %self.name, iteration, "loop iteration");

            self.body.run_with_context(ctx).await.map_err(|e| match e {
                vikar_core::RunError::Step(step_err) => step_err,
            })?;

            let done = ctx
                .eval_bool(&self.condition)
                .map_err(|source| StepError::Template {
                    task: self.name.clone(),
                    source,
                })?;

            if done {
                tracing::info!(task = %self.name, iteration, "condition met, exiting loop");
                return Ok(TaskOutput::none());
            }
        }

        Err(StepError::Failed {
            task: self.name.clone(),
            message: format!(
                "condition `{}` not met after {} iteration(s)",
                self.condition, self.max_iterations
            ),
        })
    }

    fn name(&self) -> &str {
        &self.name
    }
}
