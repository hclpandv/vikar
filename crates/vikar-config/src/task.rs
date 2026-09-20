use serde::Deserialize;

/// Configuration for one entry under `tasks:` in a playbook.
///
/// Tagged on `kind`. Exactly three variants in v0.1, on purpose — see the
/// crate-level scope notes in the project README. Adding a fourth (`script`,
/// `http`, `condition`, ...) is additive: one new variant here, one new
/// `Task` implementation in `vikar-cli`, one new arm in the compiler. No
/// existing playbook or task kind has to change.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TaskConfig {
    /// Calls an LLM. `model` must name an entry under the playbook's
    /// top-level `models:` map — checked by [`crate::Playbook::validate`].
    Ask {
        /// Optional display name; falls back to a generated one if absent.
        #[serde(default)]
        name: Option<String>,
        model: String,
        prompt: String,
        /// If set, the model's response text is stored in `Context` under
        /// this name, available to later tasks as `{{ name }}`.
        #[serde(default)]
        register: Option<String>,
    },

    /// Runs a shell command.
    Run {
        #[serde(default)]
        name: Option<String>,
        command: String,
        #[serde(default)]
        register: Option<String>,
        /// Deny-by-default guard: an agent's own output must not be
        /// interpolated into `command:` unless a playbook author opts in
        /// explicitly. See the security notes in the project README.
        #[serde(default)]
        allow_agent_output: bool,
    },

    /// Repeats a nested list of tasks until `condition` evaluates true, or
    /// `max_iterations` is reached (whichever comes first).
    Until {
        #[serde(default)]
        name: Option<String>,
        condition: String,
        #[serde(default = "default_max_iterations")]
        max_iterations: u32,
        tasks: Vec<TaskConfig>,
    },
}

fn default_max_iterations() -> u32 {
    10
}

impl TaskConfig {
    /// The task's declared `name:`, or a stable fallback derived from its
    /// kind — used anywhere a name is needed for display before the task
    /// has been compiled into a runtime [`crate::TaskConfig`] wrapper.
    pub fn display_name(&self) -> String {
        match self {
            TaskConfig::Ask { name, .. } => name.clone().unwrap_or_else(|| "ask".to_string()),
            TaskConfig::Run { name, .. } => name.clone().unwrap_or_else(|| "run".to_string()),
            TaskConfig::Until { name, .. } => name.clone().unwrap_or_else(|| "until".to_string()),
        }
    }
}
