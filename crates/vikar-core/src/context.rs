use std::collections::HashMap;

use serde_json::Value;

use crate::error::TemplateError;

/// Shared state that flows between tasks in a playbook.
///
/// # Design constraints (deliberate, for v0.1)
///
/// - **Flat.** There is no nested scoping, no per-task-private namespace.
///   Every `register:` writes into one global map.
/// - **Append-only in spirit, overwrite-in-practice.** A task that
///   `register:`s a name that already exists simply overwrites it. This
///   matches Ansible's `register` semantics and is exactly what a loop body
///   wants (each iteration overwrites the same key).
/// - **Single-threaded.** `Context` is threaded through the [`crate::Runner`]
///   by value/`&mut` reference, not shared behind a lock. There is no
///   concurrent task execution in v0.1, so there is nothing to protect.
///
/// These constraints are intentional, not oversights — see the crate-level
/// docs. Lexical scoping and concurrent access are real features, planned
/// for once there's a concrete need (sub-playbooks, parallel tasks) driving
/// their design, rather than speculative infrastructure now.
#[derive(Debug, Default, Clone)]
pub struct Context {
    vars: HashMap<String, Value>,
    env: HashMap<String, String>,
}

impl Context {
    /// Create an empty context, seeded with the current process environment.
    /// Playbooks read env vars (e.g. `api_key_env:`) through this, never
    /// through `std::env` directly, so the whole system stays testable.
    pub fn new() -> Self {
        Self {
            vars: HashMap::new(),
            env: std::env::vars().collect(),
        }
    }

    /// Create an empty context with an explicit, injected environment map.
    /// Used by tests and by anything that wants Vikar to see a controlled
    /// environment rather than the real process environment.
    pub fn with_env(env: HashMap<String, String>) -> Self {
        Self {
            vars: HashMap::new(),
            env,
        }
    }

    /// Store a value under `name`, overwriting any existing value.
    /// This is what a task's `register:` key writes into.
    pub fn register(&mut self, name: impl Into<String>, value: Value) {
        self.vars.insert(name.into(), value);
    }

    /// Look up a previously `register`ed value by name.
    pub fn get(&self, name: &str) -> Option<&Value> {
        self.vars.get(name)
    }

    /// Look up an environment variable by name (e.g. for `api_key_env:`).
    pub fn env(&self, name: &str) -> Option<&str> {
        self.env.get(name).map(String::as_str)
    }

    /// Render a `{{ ... }}` template string against the current variables.
    ///
    /// This is the single seam every task kind uses to resolve
    /// `{{ logs.stdout }}`-style expressions — `ShellStep`/`RunTask`,
    /// `AskTask`, and `UntilTask`'s `condition:` all call through here so
    /// template syntax and semantics never drift between task kinds.
    pub fn render(&self, template: &str) -> Result<String, TemplateError> {
        let env = minijinja::Environment::new();
        let ctx = serde_json::Value::Object(self.vars.clone().into_iter().collect());
        env.render_str(template, ctx).map_err(TemplateError)
    }

    /// Dump all registered variables as a single JSON object — used by
    /// `vikar play --output-json` so pipeline steps downstream can parse
    /// the result without scraping human-readable stdout.
    pub fn as_json(&self) -> Value {
        Value::Object(self.vars.clone().into_iter().collect())
    }

    /// Evaluate a boolean Jinja expression, e.g. an `until:` task's
    /// `condition:`. Non-boolean results are coerced with minijinja's
    /// truthiness rules.
    ///
    /// Unlike [`Context::render`], this takes a *bare* expression —
    /// `loop_iterations is defined`, not `{{ loop_iterations is defined }}`
    /// — matching Ansible's `when:` convention, since `condition:` is
    /// always evaluated as a whole rather than interpolated into a larger
    /// string. `{{ }}` wrapping is stripped if present, so either form
    /// works and a playbook author's habit from `command:`/`prompt:`
    /// templates doesn't produce a confusing parse error here.
    pub fn eval_bool(&self, expr: &str) -> Result<bool, TemplateError> {
        let expr = expr.trim();
        let expr = expr
            .strip_prefix("{{")
            .and_then(|s| s.strip_suffix("}}"))
            .map(str::trim)
            .unwrap_or(expr);

        let env = minijinja::Environment::new();
        let ctx = serde_json::Value::Object(self.vars.clone().into_iter().collect());
        let value = env.compile_expression(expr).and_then(|e| e.eval(ctx));
        value.map(|v| v.is_true()).map_err(TemplateError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_then_render() {
        let mut ctx = Context::with_env(HashMap::new());
        ctx.register("logs", serde_json::json!({ "stdout": "all good" }));
        assert_eq!(
            ctx.render("Summary: {{ logs.stdout }}").unwrap(),
            "Summary: all good"
        );
    }

    #[test]
    fn register_overwrites_previous_value() {
        let mut ctx = Context::with_env(HashMap::new());
        ctx.register("x", serde_json::json!(1));
        ctx.register("x", serde_json::json!(2));
        assert_eq!(ctx.get("x"), Some(&serde_json::json!(2)));
    }

    #[test]
    fn eval_bool_accepts_bare_expression() {
        let mut ctx = Context::with_env(HashMap::new());
        ctx.register("done", serde_json::json!(true));
        assert!(ctx.eval_bool("done").unwrap());
        assert!(ctx.eval_bool("done == true").unwrap());
    }

    #[test]
    fn eval_bool_tolerates_double_brace_wrapping() {
        let mut ctx = Context::with_env(HashMap::new());
        ctx.register("done", serde_json::json!(true));
        assert!(ctx.eval_bool("{{ done }}").unwrap());
    }

    #[test]
    fn eval_bool_is_defined_check() {
        let ctx = Context::with_env(HashMap::new());
        assert!(!ctx.eval_bool("missing is defined").unwrap());
    }

    #[test]
    fn env_lookup_uses_injected_map_not_real_process_env() {
        let mut env = HashMap::new();
        env.insert("MY_KEY".to_string(), "secret".to_string());
        let ctx = Context::with_env(env);
        assert_eq!(ctx.env("MY_KEY"), Some("secret"));
        assert_eq!(ctx.env("NOT_SET"), None);
    }
}
