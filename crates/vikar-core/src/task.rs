use async_trait::async_trait;
use serde_json::Value;

use crate::context::Context;
use crate::error::StepError;

/// The result of executing one [`Task`].
///
/// `value` is always structured (`serde_json::Value`) so downstream tasks
/// can address fields with `{{ name.field }}`; a plain string result is
/// just `Value::String`. `raw_text` preserves the original, unstructured
/// output for display/logging even when `value` has been reshaped.
#[derive(Debug, Clone)]
pub struct TaskOutput {
    pub value: Value,
    pub raw_text: Option<String>,
}

impl TaskOutput {
    /// Convenience constructor for the common case: a task that produces
    /// plain text (a shell command's stdout, a model's completion, etc).
    pub fn text(text: impl Into<String>) -> Self {
        let text = text.into();
        Self {
            value: Value::String(text.clone()),
            raw_text: Some(text),
        }
    }

    /// Convenience constructor for a task with no meaningful output
    /// (e.g. a loop wrapper — its own result isn't registered, only its
    /// body's).
    pub fn none() -> Self {
        Self {
            value: Value::Null,
            raw_text: None,
        }
    }
}

/// The unit of execution. Every task kind — `ask`, `run`, `until`, and
/// anything added later by Vikar or a third party — implements this trait,
/// and nothing else in the system needs to know which one it's holding.
///
/// # Keep `execute`'s signature boring
///
/// Every task kind ever written has to implement this method. Adding a
/// parameter here (even an `Option`) is a tax on every implementor,
/// built-in or third-party. If a future feature needs more context than
/// `&mut Context` provides, prefer putting it *in* `Context` over widening
/// this signature.
#[async_trait]
pub trait Task: Send + Sync {
    /// Execute this task against the given context, returning its output
    /// or an error. Implementations that produce a value meant for later
    /// tasks are responsible for calling [`Context::register`] themselves,
    /// under the `register:` name given at configuration time — `Runner`
    /// does not do this on a task's behalf, since not every task wants its
    /// raw output registered verbatim (e.g. `until` registers its body's
    /// output, not its own).
    async fn execute(&self, ctx: &mut Context) -> Result<TaskOutput, StepError>;

    /// A short, human-readable name for this task instance, used in
    /// progress output and error messages (the playbook's `name:` field,
    /// or a generated fallback like `"task 3"` if none was given).
    fn name(&self) -> &str;
}
