// B-15 Tranche O / T11 COHORT 3: constants + type-alias block extracted from
// main.rs to bound main.rs size. All items are pub(crate) to match their
// original visibility. main.rs re-exports `pub(crate) use crate::cli_consts::*;`
// alongside the existing cli_helpers glob so every consumer that reaches these
// via `use crate::*;` continues to compile byte-for-byte.
//
// RuntimePluginStateBuildOutput references live_cli::RuntimeMcpState, which
// still lives in live_cli.rs. A later tranche will move RuntimeMcpState and
// add a re-export here; at that point the `use crate::live_cli` below stays
// valid through the re-export chain.

#![allow(dead_code)]

use std::collections::BTreeSet;
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub(crate) const DEFAULT_MODEL: &str = "claude-opus-4-6";

pub(crate) fn max_tokens_for_model(model: &str) -> u32 {
    if model.contains("opus") {
        32_000
    } else {
        64_000
    }
}

pub(crate) const TOKEN_CHARS_PER_TOKEN: usize = 4;
pub(crate) const BUDGET_OUTPUT_RESERVE: usize = 4096;

pub(crate) const FENCE_LANG_MAP: &[(&str, &str)] = &[
    ("rust", "rust"), ("rs", "rust"),
    ("python", "python"), ("py", "python"),
    ("typescript", "typescript"), ("ts", "typescript"),
    ("go", "go"), ("golang", "go"),
    ("java", "java"),
    ("csharp", "c_sharp"), ("cs", "c_sharp"), ("c#", "c_sharp"),
    ("ruby", "ruby"), ("rb", "ruby"),
];

pub(crate) const DEFAULT_DATE: &str = "2026-03-31";
pub(crate) const DEFAULT_OAUTH_CALLBACK_PORT: u16 = 4545;
pub(crate) const VERSION: &str = env!("CARGO_PKG_VERSION");
pub(crate) const BUILD_TARGET: Option<&str> = option_env!("TARGET");
pub(crate) const GIT_SHA: Option<&str> = option_env!("GIT_SHA");
pub(crate) const INTERNAL_PROGRESS_HEARTBEAT_INTERVAL: Duration = Duration::from_secs(3);
pub(crate) const SESSION_HANDOFF_THRESHOLD: f64 = 0.85;
pub(crate) const PLANS_DIR: &str = "plans";
pub(crate) const PRIMARY_SESSION_EXTENSION: &str = "jsonl";
pub(crate) const OLLAMA_DEFAULT_URL: &str = "http://localhost:11434";
pub(crate) const DEGRADED_PROMPT_PREFIX: &str = "[degraded] using ollama fallback";
pub(crate) const OLLAMA_PROBE_TIMEOUT: Duration = Duration::from_secs(2);
pub(crate) const KNOWLEDGE_TOP_K: usize = 5;
pub(crate) const EMPTY_TIER_TOML: &str = "\
[tier1]\ndomains = []\npatterns = []\n\
[tier2]\ndomains = []\nmin_so_score = 5\n\
[tier4_blocklist]\ndomains = []\npatterns = []\n";
pub(crate) const LEGACY_SESSION_EXTENSION: &str = "json";
pub(crate) const LATEST_SESSION_REFERENCE: &str = "latest";
pub(crate) const SESSION_REFERENCE_ALIASES: &[&str] =
    &[LATEST_SESSION_REFERENCE, "last", "recent"];
pub(crate) const CLI_OPTION_SUGGESTIONS: &[&str] = &[
    "--help",
    "-h",
    "--version",
    "-V",
    "--model",
    "--output-format",
    "--permission-mode",
    "--dangerously-skip-permissions",
    "--allowedTools",
    "--allowed-tools",
    "--resume",
    "--print",
    "-p",
];

pub(crate) type AllowedToolSet = BTreeSet<String>;
pub(crate) type RuntimePluginStateBuildOutput = (
    Option<Arc<Mutex<crate::live_cli::RuntimeMcpState>>>,
    Vec<elai_tools::RuntimeToolDefinition>,
);
