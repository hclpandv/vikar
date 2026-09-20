use async_trait::async_trait;
use vikar_core::{Context, StepError, Task, TaskOutput};

/// The `kind: run` task — executes a shell command.
///
/// # Security
///
/// `allow_agent_output` guards whether a previously `register`ed value that
/// came from an `ask` task is allowed to be interpolated into `command:`.
/// Deny-by-default: a playbook that pipes untrusted model output straight
/// into a shell command without opting in gets a clear error instead of
/// silently running whatever the model said. This crate doesn't yet track
/// *provenance* of each variable (i.e. "did this specific value come from
/// an `ask` task?") — v0.1 conservatively requires the flag whenever the
/// rendered command differs from the literal template, which is stricter
/// than necessary but never silently unsafe. Precise provenance tracking is
/// a natural v0.2 improvement once real playbooks show what's too strict.
pub struct RunTask {
    name: String,
    command_template: String,
    register: Option<String>,
    allow_agent_output: bool,
}

impl RunTask {
    pub fn new(
        name: String,
        command_template: String,
        register: Option<String>,
        allow_agent_output: bool,
    ) -> Self {
        Self {
            name,
            command_template,
            register,
            allow_agent_output,
        }
    }
}

#[async_trait]
impl Task for RunTask {
    async fn execute(&self, ctx: &mut Context) -> Result<TaskOutput, StepError> {
        let rendered =
            ctx.render(&self.command_template)
                .map_err(|source| StepError::Template {
                    task: self.name.clone(),
                    source,
                })?;

        if rendered != self.command_template && !self.allow_agent_output {
            // The template contained at least one `{{ ... }}` that got
            // substituted, and the author hasn't opted in. Refuse to run
            // it rather than guess whether the substituted value was safe.
            return Err(StepError::Failed {
                task: self.name.clone(),
                message: "command contains interpolated variables; set `allow_agent_output: true` on this task to permit it".to_string(),
            });
        }

        tracing::info!(task = %self.name, command = %rendered, "running shell command");

        let output = tokio::process::Command::new("sh")
            .arg("-c")
            .arg(&rendered)
            .output()
            .await
            .map_err(|e| StepError::Failed {
                task: self.name.clone(),
                message: format!("failed to spawn command: {e}"),
            })?;

        let stdout = String::from_utf8_lossy(&output.stdout)
            .trim_end()
            .to_string();
        let stderr = String::from_utf8_lossy(&output.stderr)
            .trim_end()
            .to_string();
        let status = output.status.code().unwrap_or(-1);

        // Print captured output back to the terminal — a `run` task that
        // silently swallows its command's stdout/stderr is surprising to
        // watch; `Context` still gets a captured copy below (that's what
        // `register:` reuses), but the person watching `vikar play` run
        // should see what the shell actually printed, same as running it
        // directly would.
        if !stdout.is_empty() {
            println!("{stdout}");
        }
        if !stderr.is_empty() {
            eprintln!("{stderr}");
        }

        if !output.status.success() {
            return Err(StepError::Failed {
                task: self.name.clone(),
                message: format!("command exited with status {status}: {stderr}"),
            });
        }

        let value = serde_json::json!({
            "stdout": stdout,
            "stderr": stderr,
            "status": status,
        });
        let result = TaskOutput {
            value,
            raw_text: Some(stdout),
        };

        if let Some(name) = &self.register {
            ctx.register(name.clone(), result.value.clone());
        }

        Ok(result)
    }

    fn name(&self) -> &str {
        &self.name
    }
}
