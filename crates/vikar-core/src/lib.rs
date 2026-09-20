//! `vikar-core` — the load-bearing abstractions of Vikar.
//!
//! This crate defines four things and nothing else:
//!
//! 1. [`Task`] — the unit of execution. Every task kind (`ask`, `run`,
//!    `until`, and anything a third party adds later) implements this trait
//!    and nothing else in the system needs to know the difference.
//! 2. [`Model`] — the LLM boundary. An `ask` task holds an `Arc<dyn Model>`;
//!    it doesn't know or care which provider is behind it.
//! 3. [`Context`] — shared, flat, append-only state that flows between
//!    tasks. This is what makes `register:` and `{{ variable }}` templating
//!    work across task kinds.
//! 4. [`Runner`] — walks a linear list of tasks, threading `Context`
//!    through each one, and stops on the first error.
//!
//! `vikar-core` intentionally has **no** built-in task kinds and **no**
//! built-in providers. Those live in downstream crates (or third-party
//! crates) that depend on `vikar-core` — never the other way around. This
//! is the dependency direction that lets someone `cargo add vikar-core` and
//! build a completely custom task/provider set without pulling in Vikar's
//! own built-ins.

mod context;
mod error;
mod model;
mod runner;
mod task;

pub use context::Context;
pub use error::{ModelError, RunError, StepError, TemplateError};
pub use model::{CompletionRequest, CompletionResponse, Model};
pub use runner::Runner;
pub use task::{Task, TaskOutput};

/// Re-exported so downstream crates don't need to depend on `serde_json`
/// directly just to construct a [`TaskOutput`].
pub use serde_json::Value;
