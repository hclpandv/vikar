use std::collections::HashMap;
use std::sync::Arc;

use vikar_config::TaskConfig;
use vikar_core::{Model, Task};

use crate::tasks::{AskTask, RunTask, UntilTask};

/// Turn a playbook's task list into a `Vec<Box<dyn Task>>` the
/// [`vikar_core::Runner`] can execute.
///
/// This is the one place that maps a `TaskConfig` variant onto a concrete
/// `Task` implementation. Adding a fourth built-in task kind means adding
/// one arm here (and one new type under `tasks/`) — nothing in
/// `vikar-core` or `vikar-config` changes.
pub fn compile_tasks(
    configs: &[TaskConfig],
    models: &HashMap<String, Arc<dyn Model>>,
) -> anyhow::Result<Vec<Box<dyn Task>>> {
    let mut out: Vec<Box<dyn Task>> = Vec::with_capacity(configs.len());
    for config in configs {
        out.push(compile_task(config, models)?);
    }
    Ok(out)
}

fn compile_task(
    config: &TaskConfig,
    models: &HashMap<String, Arc<dyn Model>>,
) -> anyhow::Result<Box<dyn Task>> {
    match config {
        TaskConfig::Ask {
            model,
            prompt,
            register,
            ..
        } => {
            let model_ref = models
                .get(model)
                .ok_or_else(|| {
                    anyhow::anyhow!("unknown model '{model}' (validate() should have caught this)")
                })?
                .clone();
            Ok(Box::new(AskTask::new(
                config.display_name(),
                model_ref,
                prompt.clone(),
                register.clone(),
            )))
        }
        TaskConfig::Run {
            command,
            register,
            allow_agent_output,
            ..
        } => Ok(Box::new(RunTask::new(
            config.display_name(),
            command.clone(),
            register.clone(),
            *allow_agent_output,
        ))),
        TaskConfig::Until {
            condition,
            max_iterations,
            tasks,
            ..
        } => {
            let body = compile_tasks(tasks, models)?;
            Ok(Box::new(UntilTask::new(
                config.display_name(),
                condition.clone(),
                *max_iterations,
                body,
            )))
        }
    }
}
