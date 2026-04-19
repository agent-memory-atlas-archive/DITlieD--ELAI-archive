//! Batch 2C — Svelte 5 rune-purity contract.
//!
//! Asserts that every `.svelte` file under the dashboard frontend uses
//! Svelte 5 idioms only. The banned Svelte-4 patterns (`on:click`,
//! `on:submit`, `on:change`, `createEventDispatcher`, `export let`) MUST
//! not appear in any Svelte source file committed to main.
//!
//! Grounding: frontend root is
//!   crates/dashboard/src-frontend/src/
//! We read every `*.svelte` file under that directory and grep for the
//! banned patterns. One match in ANY file fails the test with a precise
//! location report so the offender is identifiable.
//!
//! This test is a build-time gate: a future refactor that accidentally
//! reintroduces Svelte-4 syntax triggers a CI failure.

use std::fs;
use std::path::{Path, PathBuf};

fn frontend_src_dir() -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent()
        .expect("crates dir")
        .join("dashboard")
        .join("src-frontend")
        .join("src")
}

/// Recursively collect `*.svelte` files under `dir`. We own the walk so the
/// contracts crate doesn't take on a new `walkdir` dependency.
fn collect_svelte_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_svelte_files(&path, out);
        } else if path.extension().and_then(|s| s.to_str()) == Some("svelte") {
            out.push(path);
        }
    }
}

/// A single banned pattern + the human-readable explanation of why.
struct BannedPattern {
    needle: &'static str,
    reason: &'static str,
}

const BANNED: &[BannedPattern] = &[
    BannedPattern {
        needle: "createEventDispatcher",
        reason: "Svelte 5 uses callback props (e.g. `onclick={fn}`) instead",
    },
    BannedPattern {
        needle: "on:click",
        reason: "Svelte 5 uses `onclick` not `on:click`",
    },
    BannedPattern {
        needle: "on:submit",
        reason: "Svelte 5 uses `onsubmit` not `on:submit`",
    },
    BannedPattern {
        needle: "on:change",
        reason: "Svelte 5 uses `onchange` not `on:change`",
    },
    BannedPattern {
        needle: "on:input",
        reason: "Svelte 5 uses `oninput` not `on:input`",
    },
    BannedPattern {
        needle: "on:keydown",
        reason: "Svelte 5 uses `onkeydown` not `on:keydown`",
    },
    BannedPattern {
        needle: "export let ",
        reason: "Svelte 5 uses `let { foo }: Props = $props()` instead of `export let`",
    },
];

// ---------------------------------------------------------------------------
// Contract 1: no banned Svelte-4 pattern in any .svelte file.
// ---------------------------------------------------------------------------

#[test]
fn contract_no_svelte_4_patterns() {
    let root = frontend_src_dir();
    assert!(
        root.is_dir(),
        "frontend src dir missing at {}",
        root.display()
    );
    let mut files: Vec<PathBuf> = Vec::new();
    collect_svelte_files(&root, &mut files);
    assert!(!files.is_empty(), "no .svelte files found under {}", root.display());

    let mut violations: Vec<String> = Vec::new();
    for path in &files {
        let body = match fs::read_to_string(path) {
            Ok(b) => b,
            Err(e) => {
                violations.push(format!("{}: read error {e}", path.display()));
                continue;
            }
        };
        for banned in BANNED {
            // Walk line by line so we can report file:line for every match.
            for (lineno, line) in body.lines().enumerate() {
                // Ignore comment lines (// ... and /* ... */ and <!-- ... -->)
                // so a rule-reference in a comment stays legal. We only skip
                // lines whose FIRST non-whitespace chars indicate a comment —
                // real code calling the banned API never fits that shape.
                let trimmed = line.trim_start();
                if trimmed.starts_with("//")
                    || trimmed.starts_with("*")
                    || trimmed.starts_with("/*")
                    || trimmed.starts_with("<!--")
                {
                    continue;
                }
                if line.contains(banned.needle) {
                    violations.push(format!(
                        "{}:{} — `{}` ({})",
                        path.display(),
                        lineno + 1,
                        banned.needle,
                        banned.reason
                    ));
                }
            }
        }
    }

    assert!(
        violations.is_empty(),
        "Svelte 4 idioms found in frontend sources:\n  {}",
        violations.join("\n  ")
    );
}

// ---------------------------------------------------------------------------
// Contract 1b: main.ts bootstrap uses Svelte 5 `mount()` — NOT Svelte 4
// `new App({ target })`. The class-style constructor throws "App is not a
// constructor" at runtime in Svelte 5 and leaves `#app` empty. Contract 1
// only scans .svelte files, so this separate check covers the TS entry point.
// Grounding: blank-dashboard regression debugged live (2026-04-19); class
// mount leaked past Contract 1 because main.ts is TypeScript not Svelte.
// ---------------------------------------------------------------------------

#[test]
fn contract_main_ts_uses_svelte5_mount() {
    let root = frontend_src_dir();
    let main_ts = root.join("main.ts");
    let body = fs::read_to_string(&main_ts)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", main_ts.display()));

    // Banned: `new App(` or `new \w+\({ target` — Svelte 4 class bootstrap.
    let banned_patterns = ["new App(", "new App ({", "new App({"];
    for pat in banned_patterns {
        assert!(
            !body.contains(pat),
            "{} contains banned Svelte-4 pattern `{}` — use Svelte 5 `mount(App, {{ target }})` instead",
            main_ts.display(),
            pat
        );
    }

    // Required: either `mount(App, ...)` or `hydrate(App, ...)` from svelte.
    let has_mount = body.contains("mount(App") || body.contains("hydrate(App");
    assert!(
        has_mount,
        "{} must bootstrap via Svelte 5 `mount(App, {{ target }})` (import from 'svelte')",
        main_ts.display()
    );

    // Required: the `mount` (or `hydrate`) symbol must be imported from "svelte".
    let has_import = body.contains("from \"svelte\"") || body.contains("from 'svelte'");
    assert!(
        has_import,
        "{} must import `mount` (or `hydrate`) from \"svelte\"",
        main_ts.display()
    );
}

// ---------------------------------------------------------------------------
// Contract 2: every script block (lang="ts") uses `$props()` OR is stateless.
// We assert by counting that the TOTAL appearance of `$props(`, `$state(`,
// or `$derived(` across all files is greater than zero — i.e. the dashboard
// is actually using runes — and that no `.svelte` file mixes Svelte-4
// stores (`writable(` / `readable(`) with imports from svelte/store.
// ---------------------------------------------------------------------------

#[test]
fn contract_runes_in_use() {
    let root = frontend_src_dir();
    let mut files: Vec<PathBuf> = Vec::new();
    collect_svelte_files(&root, &mut files);
    let mut runes_uses = 0usize;
    let mut store_imports: Vec<String> = Vec::new();
    for path in &files {
        let body = match fs::read_to_string(path) {
            Ok(b) => b,
            Err(_) => continue,
        };
        runes_uses += body.matches("$props(").count();
        runes_uses += body.matches("$state(").count();
        runes_uses += body.matches("$derived(").count();
        // svelte/store imports are allowed only when paired with a runes
        // file (none exist currently). We flag every raw `svelte/store`
        // import so a future regression is caught.
        for (lineno, line) in body.lines().enumerate() {
            if line.contains("from \"svelte/store\"")
                || line.contains("from 'svelte/store'")
            {
                store_imports.push(format!("{}:{}", path.display(), lineno + 1));
            }
        }
    }
    assert!(
        runes_uses > 0,
        "no rune usages ($props/$state/$derived) found under {} — either migration regressed or walker broke",
        root.display()
    );
    assert!(
        store_imports.is_empty(),
        "svelte/store imports detected — Batch 2C migrated everything to runes:\n  {}",
        store_imports.join("\n  ")
    );
}

// ---------------------------------------------------------------------------
// Contract 3: the key migrated files actually use runes-native surfaces.
// If ActivityEntry/StatusBar/App/Workspace/etc. slide back to Svelte 4, this
// flags it even if the banned list above is circumvented.
// ---------------------------------------------------------------------------

#[test]
fn contract_key_files_use_props_rune() {
    let root = frontend_src_dir();
    let expected = [
        "ActivityEntry.svelte",
        "ActivityFeed.svelte",
        "StatusBar.svelte",
        "App.svelte",
        "Workspace.svelte",
        "ProjectSelector.svelte",
        "TemplatePicker.svelte",
        "TaskInput.svelte",
        "InterventionCard.svelte",
        "DiffViewer.svelte",
        "PlanTree.svelte",
        "DiffHunk.svelte",
    ];
    let mut missing: Vec<String> = Vec::new();
    for name in expected {
        let p = root.join(name);
        let Ok(body) = fs::read_to_string(&p) else {
            missing.push(format!("{name} (read error)"));
            continue;
        };
        // App.svelte has no props, only state — accept $state too.
        let has_props = body.contains("$props(")
            || body.contains("$state(")
            || body.contains("$derived(");
        if !has_props {
            missing.push(format!("{name} (no $props/$state/$derived)"));
        }
    }
    assert!(
        missing.is_empty(),
        "key files not using runes: {missing:?}"
    );
}
