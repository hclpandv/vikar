use std::fs;
use std::path::Path;

const ENV_TEMPLATE: &str = r#"# Vikar workspace environment — fill in the keys for the providers you
# plan to use. This file is gitignored (see .gitignore in this folder);
# never commit real keys.
#
# Ollama needs no key at all — see playbooks/mock-demo.yaml's sibling
# comments if you'd rather start there than with a cloud provider.

ANTHROPIC_API_KEY=
NVIDIA_API_KEY=
OPENAI_API_KEY=
"#;

const GITIGNORE_TEMPLATE: &str = ".env\n";

const MOCK_DEMO: &str = r#"# Runs with zero setup — no API key, no network access.
# Try it:  vikar play .agents/playbooks/mock-demo.yaml

models:
  demo:
    kind: mock
    prefix: "[demo model] "

tasks:
  - name: greet
    kind: ask
    model: demo
    prompt: "Hello from Vikar!"
    register: greeting
"#;

const LIMERICK_NVIDIA: &str = r#"# Calls NVIDIA's hosted API (OpenAI-compatible). Set NVIDIA_API_KEY in
# .env first — get a key at https://build.nvidia.com.
# Try it:  vikar play .agents/playbooks/limerick-nvidia.yaml

models:
  nemotron:
    kind: openai_compatible
    model: nvidia/nemotron-3-ultra-550b-a55b
    base_url: https://integrate.api.nvidia.com/v1
    api_key_env: NVIDIA_API_KEY

tasks:
  - name: limerick
    kind: ask
    model: nemotron
    prompt: "Write a limerick about the wonders of GPU computing."
    register: poem
"#;

/// `vikar init` — scaffold a `.agents/` workspace: `.env` (gitignored),
/// `.gitignore`, and two demo playbooks (`mock-demo.yaml`, which needs no
/// key, and `limerick-nvidia.yaml`, which does) — a runnable starting
/// point the user then adds their own flows to, rather than a blank
/// directory and a README to read first.
pub fn run(force: bool) -> anyhow::Result<()> {
    let root = Path::new(".agents");
    let playbooks = root.join("playbooks");

    fs::create_dir_all(&playbooks)?;

    let mut created = Vec::new();
    let mut skipped = Vec::new();

    for (path, contents) in [
        (root.join(".env"), ENV_TEMPLATE),
        (root.join(".gitignore"), GITIGNORE_TEMPLATE),
        (playbooks.join("mock-demo.yaml"), MOCK_DEMO),
        (playbooks.join("limerick-nvidia.yaml"), LIMERICK_NVIDIA),
    ] {
        if path.exists() && !force {
            skipped.push(path);
            continue;
        }
        fs::write(&path, contents)?;
        created.push(path);
    }

    println!("Vikar workspace: .agents/\n");
    println!(".agents/");
    println!("├── .env                       (add your API keys here — gitignored)");
    println!("├── .gitignore");
    println!("└── playbooks/");
    println!("    ├── mock-demo.yaml          (zero setup, no API key needed)");
    println!("    └── limerick-nvidia.yaml    (needs NVIDIA_API_KEY)");

    if !skipped.is_empty() {
        println!("\nAlready present, left untouched (use --force to overwrite):");
        for path in &skipped {
            println!("  {}", path.display());
        }
    }

    println!("\nNext steps:");
    println!("  1. vikar play .agents/playbooks/mock-demo.yaml       # try it now, no key needed");
    println!("  2. Add your NVIDIA_API_KEY to .agents/.env");
    println!("  3. vikar play .agents/playbooks/limerick-nvidia.yaml # once your key is set");

    Ok(())
}
