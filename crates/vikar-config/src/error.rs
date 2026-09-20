use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("could not read playbook at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("could not parse playbook YAML: {0}")]
    Parse(#[source] serde_yaml::Error),

    #[error("task refers to unknown model '{name}' (not declared under `models:`)")]
    UnknownModel { name: String },
}
