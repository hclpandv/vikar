use std::path::Path;

/// Find and load an `.env` file so keys placed there (e.g. by `vikar init`)
/// are visible to `std::env::var` when providers are built.
///
/// Search order, starting from the current directory and walking upward
/// through parents (the same pattern `git` uses to find `.git`):
///
/// 1. `.agents/.env` — where `vikar init` puts it
/// 2. `.env` — the common convention, for anyone not using `vikar init`
///
/// Loading is non-destructive: an already-set environment variable (e.g.
/// exported in the shell, or injected by a CI runner) always wins over the
/// same key in a `.env` file — this matters for the "zero infra" pipeline
/// use case, where secrets come from the CI platform, not a checked-in
/// file. Missing or unreadable `.env` files are silently ignored; there is
/// nothing wrong with not having one.
pub fn load() {
    let Ok(start) = std::env::current_dir() else {
        return;
    };

    for dir in start.ancestors() {
        if try_load(&dir.join(".agents").join(".env")) {
            return;
        }
        if try_load(&dir.join(".env")) {
            return;
        }
    }
}

fn try_load(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }
    // `from_path` never overrides variables already set in the process
    // environment — see the doc comment above on why that matters.
    match dotenvy::from_path(path) {
        Ok(()) => {
            tracing::debug!(path = %path.display(), "loaded .env file");
            true
        }
        Err(e) => {
            tracing::warn!(path = %path.display(), error = %e, "found .env file but could not parse it");
            true // still stop searching — an unparsable .env shouldn't fall through to a parent dir's
        }
    }
}
