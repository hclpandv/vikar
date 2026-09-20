# Contributing to Vikar

Thanks for considering it. The two highest-value, lowest-risk contributions
are **a new task kind** and **a new model provider** — both are additive
and self-contained. Here's exactly how to add each.

## Adding a new task kind

Say you want `kind: http` (make an HTTP request as a task).

1. **Schema** — add a variant to `TaskConfig` in
   `crates/vikar-config/src/task.rs`:
   ```rust
   Http {
       #[serde(default)]
       name: Option<String>,
       url: String,
       #[serde(default)]
       register: Option<String>,
   },
   ```
2. **Implementation** — add `crates/vikar-cli/src/tasks/http.rs` implementing
   `vikar_core::Task` (look at `tasks/run.rs` for the shape: render any
   templated fields via `ctx.render(...)`, do the work, `ctx.register(...)`
   the result if `register:` was set, return a `TaskOutput`).
3. **Wire it up** — add one arm to `compile_task` in
   `crates/vikar-cli/src/compile.rs`, and export the new type from
   `crates/vikar-cli/src/tasks/mod.rs`.
4. **Validate cross-references, if any** — if your task kind references
   something by name (like `ask` references a model), add a check in
   `Playbook::validate` in `crates/vikar-config/src/lib.rs`.
5. **Example + test** — add a minimal playbook under `examples/` that
   exercises it, and confirm both `vikar plan` and `vikar play` handle it.

Nothing in `vikar-core` changes. That's the point — `Runner` and `Task`
don't know or care how many task kinds exist.

## Adding a new model provider

Say you want `kind: bedrock`.

1. **Schema** — add a variant to `ModelConfig` in
   `crates/vikar-config/src/model.rs`.
2. **Implementation** — add `crates/vikar-cli/src/providers/bedrock.rs`
   implementing `vikar_core::Model` (one method: `complete`). Look at
   `providers/anthropic.rs` for the shape.
3. **Wire it up** — add one arm to `build_model` in
   `crates/vikar-cli/src/providers/mod.rs`.

Keep the `Model` trait itself untouched. If your provider needs something
the trait doesn't expose (streaming, tool schemas), raise an issue first —
every method added to `Model` is a tax on every other provider
implementation, built-in or third-party.

## Design constraints worth knowing before you dive in

These are documented in code comments where they matter, but the short
version:

- **`Context` is flat and single-threaded in v0.1.** No nested scoping, no
  concurrent access. Don't build around an assumption that this changes
  soon unless you're the one proposing the design for why it should.
- **`Runner` is strictly linear.** No DAGs, no parallelism. A loop
  (`until`) is just a `Task` that owns its own nested `Runner` — the
  orchestrator itself has no special case for it.
- **Task-kind and provider registration is compile-time**, matching the
  "single static binary, zero infra" goal. Dynamic plugin loading (`.so`/
  WASM) is a real future feature, not a v0.1 goal.
- **`run` tasks are unsandboxed** beyond the `allow_agent_output` opt-in
  guard. Don't assume more safety than that exists yet.

## Before opening a PR

- `cargo build` and `cargo test` clean across the workspace
- `cargo fmt` and `cargo clippy` with no new warnings
- If you touched the schema, update `README.md`'s task-kind / provider
  tables
