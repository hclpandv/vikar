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

> 💼 **Real-world example:** a playbook that audits an **Azure subscription for cost waste** (unattached disks, orphaned public IPs, stopped-but-billing VMs). It queries Azure Resource Graph, has an LLM write a prioritized fix list with exact `az` commands, and saves a Markdown report. [View the playbook →](examples/azure-finops-audit.yaml)

---

## 🚀 Try It Now (No API Key Needed)

Vikar isn't on crates.io yet, so you build it straight from Git. You only need the [Rust toolchain](https://rustup.rs/).

**Option A: install directly from GitHub**

```bash
cargo install --git https://github.com/hclpandv/vikar vikar-cli
```

**Option B: clone and build**

```bash
git clone https://github.com/hclpandv/vikar.git
cd vikar
cargo install --path crates/vikar-cli
```

Then:

```bash
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

**Go live** by swapping `kind: mock` for `anthropic`, `ollama` (local, no key), or `openai_compatible` (OpenAI, vLLM, LM Studio, or anything
