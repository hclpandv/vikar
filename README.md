# Vikar

**Vikar** dispatches a workforce of substitute agents to do your work,
declared as data. Mix LLM calls, shell commands, and loops in one YAML
playbook, and run it as a single, dependency-free Rust binary — no Python
venv, no server, no infra.

> *"vikar"* (Norwegian, Bokmål) — a substitute or temp worker, sent to fill
> a role. That's exactly what an `ask` task is.

```yaml
models:
  claude-main:
    kind: anthropic
    model: claude-sonnet-4-5
    api_key_env: ANTHROPIC_API_KEY

tasks:
  - name: fetch_status
    kind: run
    command: "kubectl get pods --field-selector=status.phase=Failed"
    register: failures

  - name: assess
    kind: ask
    model: claude-main
    prompt: "Summarize what's wrong: {{ failures.stdout }}"
    register: summary

  - name: notify
    kind: run
    command: "echo '{{ summary }}' | mail -s 'Pod failures' oncall@example.com"
    allow_agent_output: true
```

```
vikar play playbook.yaml
```

## Try it now, no API key required

```
cargo install --path crates/vikar-cli
vikar init
vikar play .agents/playbooks/mock-demo.yaml
```

`vikar init` scaffolds a `.agents/` workspace: a gitignored `.env` for your
API keys, and two demo playbooks — `mock-demo.yaml` (deterministic,
offline, no key needed) and `limerick-nvidia.yaml` (a real hosted model,
once you've added `NVIDIA_API_KEY` to `.env`). It's meant to be a running
starting point, not a blank folder — add your own playbooks alongside the
demos.

Keys in `.agents/.env` are picked up automatically: `vikar` searches the
current directory and its parents for `.agents/.env`, then plain `.env`,
the same way `git` finds `.git`. A key already exported in your shell (or
injected by a CI runner) always takes priority over the same name in a
`.env` file.

Swap `kind: mock` for `kind: anthropic`, `kind: ollama` (a local server,
no key needed), or `kind: openai_compatible` (OpenAI itself, vLLM, LM
Studio, or anything else speaking the OpenAI chat schema) to go live.

## Why not LangGraph / CrewAI / AutoGen?

Those are mature, Python/JS-first multi-agent frameworks, and if you're
already in that ecosystem they're a fine choice. Vikar exists for a
different shape of problem: **you want to drop an agentic step into a
pipeline that has zero tolerance for infra** — a CI job, a cron on a bare
VM, a container with no Python runtime. Vikar is a single static binary
with no daemon, no database, and no server to stand up first.

## Core vocabulary

| Concept | Meaning |
|---|---|
| **Playbook** | the YAML file describing what to do |
| **Task** | one step — `kind: ask`, `kind: run`, or `kind: until` |
| **Model** | a named LLM backend under `models:`, referenced by an `ask` task |
| **Context** | the flat, shared state that `register:` writes into and `{{ }}` reads from |

Three task kinds in v0.1, deliberately:

- **`ask`** — calls an LLM (`model:`, `prompt:`, optional `register:`)
- **`run`** — runs a shell command (`command:`, optional `register:`,
  `allow_agent_output:` — see **Security** below)
- **`until`** — repeats a nested `tasks:` list until `condition:` (a bare
  Jinja expression, like Ansible's `when:`) is true, or `max_iterations:`
  is hit

No DAGs, no parallel execution, no conditionals outside `until` yet. See
[Scope](#scope--roadmap) below for what's deliberately left out of v0.1
and why.

## Security: agent output is untrusted by default

An LLM's output is not trusted input. If a `command:` template
interpolates a variable — regardless of where that variable came from —
Vikar refuses to run it unless the task sets `allow_agent_output: true`.
This is deny-by-default: an oversight in a playbook produces a clear error,
not a silent command injection.

```yaml
- kind: run
  command: "./remediate.sh '{{ summary.action }}'"
  allow_agent_output: true   # required — summary came from an `ask` task
```

There is no sandboxing of `run` tasks in v0.1 beyond this opt-in flag —
that's a real, planned feature (see Scope), not something to assume is
handled.

## CLI

```
vikar init                              # scaffold a .agents/ workspace with demo playbooks
vikar plan <playbook.yaml>              # validate + print the plan, no execution, no network
vikar play <playbook.yaml>              # execute
vikar play <playbook.yaml> --output-json  # machine-readable output for pipelines
```

`vikar play` exits `0` on success and non-zero on any task failure, an
unmet `until` condition, or a config error — safe to gate a CI step on.

## Architecture

Four abstractions, deliberately small, in `vikar-core` — everything else is
a plugin against one of them:

- **`Task`** — the unit of execution (`ask`, `run`, `until` all implement
  this; nothing else in the system needs to know which one it's holding)
- **`Model`** — the LLM provider boundary (Anthropic, OpenAI-compatible,
  Mock all implement this)
- **`Context`** — flat, shared state threaded through every task
- **`Runner`** — walks a linear list of `Task`s, stops on first error

```
crates/
├── vikar-core    # the four traits above — zero I/O, zero built-ins
├── vikar-config  # the YAML schema (serde structs) + validation — zero vikar-core dependency
└── vikar-cli     # built-in task kinds, built-in providers, the `vikar` binary
```

`vikar-core` never depends on `vikar-config` or `vikar-cli` — only the
reverse. That's what lets `cargo add vikar-core` pull in just the traits,
for anyone building a completely custom task/provider set without Vikar's
own built-ins.

Adding a new task kind or provider is additive: one new `TaskConfig` /
`ModelConfig` variant, one new `Task` / `Model` implementation, one new
arm in `vikar-cli`'s compiler/registry. See
[`CONTRIBUTING.md`](CONTRIBUTING.md).

## Scope & roadmap

v0.1 proves one sentence: *you can write a YAML playbook that mixes an LLM
call and a shell command, with a loop, and run it as a single binary with
zero setup.* Deliberately out of scope for now, roughly in the order
they'll likely come back:

- `kind: script` (a `run` variant with an explicit interpreter — mostly
  sugar over `run`)
- `when:` conditionals outside `until`
- Sandboxed/restricted execution for `run` tasks
- Streaming, tool-calling, multi-turn history in `ask`
- Parallel tasks / DAG dependencies (today: strictly linear)
- Retries / `continue_on_error:` per task
- Dynamic plugin loading (today: compile-time registry, matching the
  single-static-binary constraint)

## License

MIT OR Apache-2.0, at your option.
