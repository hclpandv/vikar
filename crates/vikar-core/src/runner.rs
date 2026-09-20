use crate::context::Context;
use crate::error::RunError;
use crate::task::Task;

/// Walks a linear list of [`Task`]s, threading one [`Context`] through all
/// of them, and stops on the first error.
///
/// Deliberately dumb: no DAG, no scheduler, no concurrency. A `Playbook`
/// (defined in `vikar-config`) compiles down to exactly this — a
/// `Vec<Box<dyn Task>>` — regardless of how expressive the YAML looks.
/// A `Task::execute` implementation for `until` recurses into its own
/// nested list of tasks; `Runner` itself never needs to know loops exist.
pub struct Runner {
    tasks: Vec<Box<dyn Task>>,
}

impl Runner {
    pub fn new(tasks: Vec<Box<dyn Task>>) -> Self {
        Self { tasks }
    }

    /// Run every task in order against a fresh [`Context`]. Returns the
    /// final context (so a caller — the CLI, or an embedder — can inspect
    /// what was registered) on success, or the first [`RunError`]
    /// encountered.
    pub async fn run(&self) -> Result<Context, RunError> {
        let mut ctx = Context::new();
        self.run_with_context(&mut ctx).await?;
        Ok(ctx)
    }

    /// Like [`Runner::run`], but against a caller-supplied context. Used by
    /// `until` tasks to run their body against the *same* context as their
    /// parent, rather than a fresh one.
    pub async fn run_with_context(&self, ctx: &mut Context) -> Result<(), RunError> {
        for task in &self.tasks {
            tracing::info!(task = task.name(), "running task");
            task.execute(ctx).await?;
        }
        Ok(())
    }
}
