use thiserror::Error;

/// An error from rendering a `{{ template }}` string against [`crate::Context`].
#[derive(Debug, Error)]
#[error("template error: {0}")]
pub struct TemplateError(#[from] pub minijinja::Error);

/// An error from a single [`crate::Model`] completion call.
#[derive(Debug, Error)]
pub enum ModelError {
    #[error("model '{model}' request failed: {message}")]
    Request { model: String, message: String },

    #[error("model '{model}' returned a response Vikar could not parse: {message}")]
    Response { model: String, message: String },
}

/// An error from executing a single [`crate::Task`].
///
/// This is intentionally a small, closed set. Task implementations should
/// map their own failure modes onto these variants rather than growing the
/// enum — every downstream `match` on `StepError` (in the CLI's exit-code
/// logic, for instance) has to stay exhaustive.
#[derive(Debug, Error)]
pub enum StepError {
    #[error("task '{task}' failed: {message}")]
    Failed { task: String, message: String },

    #[error("task '{task}' template error: {source}")]
    Template {
        task: String,
        #[source]
        source: TemplateError,
    },

    #[error("task '{task}' model error: {source}")]
    Model {
        task: String,
        #[source]
        source: ModelError,
    },

    #[error("task '{task}' timed out")]
    Timeout { task: String },
}

/// An error from running an entire [`crate::Runner`] — currently just a
/// wrapper around the first [`StepError`] encountered, but kept as its own
/// type so a distinct exit code / JSON shape can be attached later without
/// changing `Runner::run`'s signature.
#[derive(Debug, Error)]
pub enum RunError {
    #[error(transparent)]
    Step(#[from] StepError),
}
