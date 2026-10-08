<div align="center">

# 🦀 Vikar

### A workforce of substitute agents, declared as data.
Mix **LLM calls**, **shell commands** and **loops** in one YAML playbook.
Run it as a **single Rust binary**: no Python venv, no server, no infra.

![Rust](https://img.shields.io/badge/Rust-Single_Binary-DEA584?logo=rust&logoColor=black)
![YAML](https://img.shields.io/badge/Playbooks-YAML-CB171E?logo=yaml&logoColor=white)
![License](https://img.shields.io/badge/License-MIT_OR_Apache--2.0-blue)
![Version](https://img.shields.io/badge/Version-v0.1-orange)
![No Infra](https://img.shields.io/badge/Infra-None-success)

</div>

---

> 🇳🇴 ***"vikar"*** (Norwegian, Bokmål): a substitute or temp worker, sent to fill a role.
> That's exactly what an `ask` task is.

## ⚡ See It in 10 Lines

```yaml
models:
  helper:
    kind: ollama            # or anthropic, openai_compatible, mock
    model: llama3

tasks:
  - name: disk_usage
    kind: run
    command: "df -h /"
    register: disk
    allow_agent_output: true

  - name: explain
    kind: ask
    model: helper
    prompt: "In two sentences, is this disk healthy? {{ disk.stdout }}"
```

```bash
vikar play playbook.yaml
```

> 💼 **Real-world example:** the repo's flagship playbook audits an **Azure subscription for cost waste** (unattached disks, orphaned public IPs, stopped-but-billing VMs). It queries Azure Resource Graph, has an LLM write a prioritized fix list with exact `az` commands, and saves a Markdown report. [See the full playbook ↓](#-full-example-azure-finops-audit)

---

## 🚀 Try It Now (No API Key Needed)

```bash
cargo install --path crates/vikar-cli
vikar init
vikar play .agents/playbooks/mock-demo.yaml
```

`vikar init` scaffolds a ready-to-run `.agents/` workspace:

| File | Purpose |
|---|---|
| `.env` | 🔑 Gitignored home for your API keys |
| `mock-demo.yaml` | 🧪 Deterministic, offline demo, no key needed |
| `limerick-nvidia.yaml` | 🌐 Real hosted model (add `NVIDIA_API_KEY` to `.env`) |

Keys are found the way `git` finds `.git`: Vikar searches the current directory and its parents for `.agents/.env`, then `.env`. A key already exported in your shell or CI runner always wins.

**Go live** by swapping `kind: mock` for `anthropic`, `ollama` (local, no key), or `openai_compatible` (OpenAI, vLLM, LM Studio, or anything speaking the OpenAI chat schema).

---

## 🤔 Why Not LangGraph / CrewAI / AutoGen?

Those are mature, Python/JS-first frameworks, and if you're already in that ecosystem they're a fine choice. Vikar is for a different shape of problem:

> **You want to drop an agentic step into a pipeline with zero tolerance for infra.**
> A CI job. A cron on a bare VM. A container with no Python runtime.

| | Vikar |
|---|---|
| 📦 **Distribution** | One static binary |
| 🐍 **Runtime** | No Python, no Node |
| 🖥️ **Server / daemon** | None |
| 🗄️ **Database** | None |
| 📝 **Workflow format** | Declarative YAML |
| 🚦 **CI-friendly** | Non-zero exit on failure, `--output-json` for pipelines |

---

## 🧩 Core Vocabulary

| Concept | Meaning |
|---|---|
| 📜 **Playbook** | The YAML file describing what to do |
| 🔧 **Task** | One step: `ask`, `run` or `until` |
| 🤖 **Model** | A named LLM backend under `models:` |
| 🧠 **Context** | Flat, shared state: `register:` writes it, `{{ }}` reads it |

### Three task kinds (v0.1, deliberately)

| Kind | What it does |
|---|---|
| 🗣️ **`ask`** | Calls an LLM (`model:`, `prompt:`, optional `register:`) |
| 💻 **`run`** | Runs a shell command (`command:`, optional `register:`, `allow_agent_output:`) |
| 🔁 **`until`** | Repeats nested `tasks:` until `condition:` is true or `max_iterations:` is hit |

---

## 🛡️ Security: Agent Output Is Untrusted by Default

An LLM's output is not trusted input. If a `command:` template interpolates **any** variable, Vikar refuses to run it unless the task sets `allow_agent_output: true`.

```yaml
- kind: run
  command: "./remediate.sh '{{ summary.action }}'"
  allow_agent_output: true   # required, since summary came from an `ask` task
```

**Deny-by-default:** a playbook oversight produces a clear error, not a silent command injection.

> ⚠️ `run` tasks are **not sandboxed** in v0.1 beyond this opt-in flag. Sandboxing is planned, but don't assume it today.

---

## 🎛️ CLI

```bash
vikar init                                # scaffold a .agents/ workspace with demo playbooks
vikar plan <playbook.yaml>                # validate + print the plan (no execution, no network)
vikar play <playbook.yaml>                # execute
vikar play <playbook.yaml> --output-json  # machine-readable output for pipelines
```

`vikar play` exits `0` on success and non-zero on any task failure, unmet `until` condition or config error, so it is safe to gate a CI step on.

---

## 🏗️ Architecture

Four small abstractions in `vikar-core`. Everything else is a plugin against one of them.

| Abstraction | Role |
|---|---|
| **`Task`** | The unit of execution (`ask`, `run`, `until`) |
| **`Model`** | The LLM provider boundary (Anthropic, OpenAI-compatible, Mock) |
| **`Context`** | Flat shared state threaded through every task |
| **`Runner`** | Walks a linear list of tasks, stops on first error |

```
crates/
├── vikar-core    # the four traits: zero I/O, zero built-ins
├── vikar-config  # YAML schema (serde) + validation, no vikar-core dependency
└── vikar-cli     # built-in task kinds, providers, and the `vikar` binary
```

`vikar-core` never depends on the other crates, so `cargo add vikar-core` gives you just the traits for a fully custom task/provider set.

**Extending is additive:** one new config variant, one new `Task`/`Model` implementation, one new registry arm. See [CONTRIBUTING.md](CONTRIBUTING.md).

---

## 🗺️ Scope & Roadmap

v0.1 proves one sentence:

> *You can write a YAML playbook that mixes an LLM call and a shell command, with a loop, and run it as a single binary with zero setup.*

Deliberately out of scope for now, roughly in the order they'll likely return:

- [ ] `kind: script` (a `run` variant with an explicit interpreter)
- [ ] `when:` conditionals outside `until`
- [ ] 🔒 Sandboxed / restricted execution for `run` tasks
- [ ] Streaming, tool-calling and multi-turn history in `ask`
- [ ] ⚡ Parallel tasks and DAG dependencies (today: strictly linear)
- [ ] Retries and per-task `continue_on_error:`
- [ ] Dynamic plugin loading (today: compile-time registry, matching the single-binary constraint)

---

## 💼 Full Example: Azure FinOps Audit

<details>
<summary><b>Click to expand the full playbook</b></summary>

```yaml
models:
  auditor:
    kind: openai_compatible
    model: nvidia/nemotron-3-ultra-550b-a55b
    base_url: https://integrate.api.nvidia.com/v1
    api_key_env: NVIDIA_API_KEY

tasks:
  - name: whoami
    kind: run
    command: "az account show --query '{subscriptionId:id, name:name, user:user.name}' -o json"
    register: account
    allow_agent_output: true

  - name: ensure_resource_graph_extension
    kind: run
    command: "az extension add --name resource-graph --only-show-errors -y || true"

  - name: unattached_disks
    kind: run
    command: >
      az graph query -q
      "Resources
      | where type =~ 'microsoft.compute/disks'
      | where properties.diskState =~ 'Unattached'
      | project name, resourceGroup, sizeGB=properties.diskSizeGB, sku=sku.name, location"
      --query "data" -o json
    register: disks
    allow_agent_output: true

  - name: unused_public_ips
    kind: run
    command: >
      az graph query -q
      "Resources
      | where type =~ 'microsoft.network/publicipaddresses'
      | where isnull(properties.ipConfiguration)
      | project name, resourceGroup, sku=sku.name, location"
      --query "data" -o json
    register: unused_ips
    allow_agent_output: true

  - name: stopped_not_deallocated_vms
    kind: run
    command: >
      az graph query -q
      "Resources
      | where type =~ 'microsoft.compute/virtualmachines'
      | extend powerState = tostring(properties.extended.instanceView.powerState.code)
      | where powerState == 'PowerState/stopped'
      | project name, resourceGroup, size=tostring(properties.hardwareProfile.vmSize), powerState"
      --query "data" -o json
    register: stopped_vms
    allow_agent_output: true

  - name: analyze
    kind: ask
    model: auditor
    prompt: |
      You are a FinOps assistant reviewing an Azure subscription for cost
      waste. Given the JSON below, produce a prioritized action list
      (5 items max), each with a one-line reason and the exact `az`
      command to fix it. Keep the whole response under 200 words. Do not
      invent resources that aren't in the data.

      Subscription: {{ account.stdout }}

      Unattached managed disks (still billed, attached to nothing):
      {{ disks.stdout }}

      Public IPs with no attached NIC (still billed, doing nothing):
      {{ unused_ips.stdout }}

      Stopped VMs that are NOT deallocated (still billing for compute):
      {{ stopped_vms.stdout }}
    register: report

  - name: write_report
    kind: run
    command: |
      mkdir -p reports
      cat > "reports/azure-cost-audit-$(date +%Y%m%d-%H%M%S).md" << 'REPORT_EOF'
      # Azure Cost Audit
      Generated by Vikar

      {{ report }}
      REPORT_EOF
      echo "Report written to reports/"
    allow_agent_output: true
```

</details>

---

## 🤝 Contributing

New task kinds, providers and ideas are welcome. See [CONTRIBUTING.md](CONTRIBUTING.md).

## 📄 License

Dual-licensed under **MIT OR Apache-2.0**, at your option.

<div align="center">

**If Vikar saves you from standing up infra, give it a ⭐**

</div>
