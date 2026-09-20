//! `vikar-config` — the Vikar playbook YAML schema.
//!
//! This crate is pure data: `serde`-derived structs, a loader, and
//! cross-reference validation (does every `model:` a task names actually
//! exist under `models:`?). It has **no dependency on `vikar-core`** and
//! knows nothing about execution — turning a validated [`Playbook`] into
//! something runnable is `vikar-cli`'s job (see its `compile` module),
//! precisely so that the schema can be reused by anything that wants to
//! read or write Vikar playbooks without pulling in an async runtime.

mod error;
mod model;
mod task;

pub use error::ConfigError;
pub use model::ModelConfig;
pub use task::TaskConfig;

use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

/// A fully parsed (but not yet validated) Vikar playbook: the top-level
/// shape of a `playbook.yaml` file.
#[derive(Debug, Deserialize)]
pub struct Playbook {
    /// Named LLM backends, referenced by `model:` in `ask` tasks.
    #[serde(default)]
    pub models: HashMap<String, ModelConfig>,

    /// The ordered list of top-level tasks to run.
    pub tasks: Vec<TaskConfig>,
}

impl Playbook {
    /// Parse a playbook from a YAML string. Does **not** validate
    /// cross-references — call [`Playbook::validate`] afterwards.
    pub fn from_yaml(yaml: &str) -> Result<Self, ConfigError> {
        serde_yaml::from_str(yaml).map_err(ConfigError::Parse)
    }

    /// Load and parse a playbook from a file on disk.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let path = path.as_ref();
        let yaml = std::fs::read_to_string(path).map_err(|source| ConfigError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        Self::from_yaml(&yaml)
    }

    /// Check that every `model:` a task refers to (including tasks nested
    /// inside `until:` blocks) exists in `models:`. Call this immediately
    /// after loading, before compiling the playbook into runnable tasks —
    /// it's what lets Vikar fail fast on a typo'd model name instead of
    /// discovering it mid-run, possibly after side effects have already
    /// happened.
    pub fn validate(&self) -> Result<(), ConfigError> {
        Self::validate_tasks(&self.tasks, &self.models)
    }

    fn validate_tasks(
        tasks: &[TaskConfig],
        models: &HashMap<String, ModelConfig>,
    ) -> Result<(), ConfigError> {
        for task in tasks {
            match task {
                TaskConfig::Ask { model, .. } => {
                    if !models.contains_key(model) {
                        return Err(ConfigError::UnknownModel {
                            name: model.clone(),
                        });
                    }
                }
                TaskConfig::Run { .. } => {}
                TaskConfig::Until { tasks, .. } => {
                    Self::validate_tasks(tasks, models)?;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_minimal_playbook() {
        let yaml = r#"
models:
  demo:
    kind: mock
tasks:
  - kind: run
    command: "echo hi"
"#;
        let pb = Playbook::from_yaml(yaml).unwrap();
        assert_eq!(pb.tasks.len(), 1);
        assert!(pb.validate().is_ok());
    }

    #[test]
    fn validate_rejects_unknown_model() {
        let yaml = r#"
models:
  demo:
    kind: mock
tasks:
  - kind: ask
    model: typo
    prompt: "hi"
"#;
        let pb = Playbook::from_yaml(yaml).unwrap();
        let err = pb.validate().unwrap_err();
        assert!(matches!(err, ConfigError::UnknownModel { name } if name == "typo"));
    }

    #[test]
    fn validate_recurses_into_until_tasks() {
        let yaml = r#"
models:
  demo:
    kind: mock
tasks:
  - kind: until
    condition: "true"
    tasks:
      - kind: ask
        model: typo
        prompt: "hi"
"#;
        let pb = Playbook::from_yaml(yaml).unwrap();
        assert!(pb.validate().is_err());
    }

    #[test]
    fn until_defaults_max_iterations() {
        let yaml = r#"
tasks:
  - kind: until
    condition: "true"
    tasks:
      - kind: run
        command: "echo hi"
"#;
        let pb = Playbook::from_yaml(yaml).unwrap();
        match &pb.tasks[0] {
            TaskConfig::Until { max_iterations, .. } => assert_eq!(*max_iterations, 10),
            _ => panic!("expected Until"),
        }
    }
}
