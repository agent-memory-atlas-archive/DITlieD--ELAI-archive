//! C-40: GitHub PR adapter — orchestrator-side surface for shelling out
//! to the `gh` CLI (the team's recommended path) so /session merge can
//! complete the loop from local commit to upstream review surface.
//!
//! Design notes (all grounded in the existing codebase patterns):
//!
//! - **Native async fn in traits** (no `async-trait` crate, no boxed
//!   futures): the surface is consumed from a single async dispatch site
//!   (`commands_c40` Tauri handler + REPL slash-command), so trait
//!   objects (`Arc<dyn GithubPrAdapter>`) are not required. Concrete
//!   type wiring keeps the trait clean per CLAUDE.md anti-stale-training
//!   rule.
//! - **`gh` shell-out is the sole transport** in this slice. A REST
//!   fallback is sketched in the plan but deferred — adding `reqwest` to
//!   the orchestrator crate breaks the anti-deps rule without explicit
//!   user confirmation, and the `gh pr *` commands are already
//!   classified `RiskLevel::SharedState` by `tools::action_classifier`
//!   (lines 128-141), so the existing gate covers this surface for
//!   free.
//! - **SSB emission is mandatory** per the wiring contract. Every
//!   successful public method writes exactly one C-40 SSB row via
//!   [`crate::ssb::Ssb::write_payload`]. Hashed-prefix fields keep the
//!   raw URL / SHA out of the broadcast stream.
//! - **Error surface uses thiserror at the lib boundary** (no
//!   `Box<dyn Error>`, no `anyhow` re-export). `anyhow` lives only
//!   inside the gh-cli adapter impl for IO/parse plumbing, mirroring
//!   the `wip_squash.rs` precedent.
//!
//! This file is the module root; submodules carry the concrete types:
//!
//! - [`adapter_trait`] — [`GithubPrAdapter`] trait (5 async methods).
//! - [`gh_cli`]        — [`GhCliAdapter`] concrete impl over `gh` CLI.
//! - [`types`]         — wire DTOs ([`PrSummary`], [`PrStatus`],
//!   [`PrOpenRequest`], [`PrCheckRun`]).
//! - [`error`]         — [`GithubPrError`] thiserror enum.
//!
//! Cross-language pair: dashboard-v2/src-tauri/src/commands_c40.rs +
//! dashboard-v2/src/lib/windows/GithubWindow.svelte +
//! dashboard-v2/src/lib/stores/githubStore.ts.

pub mod adapter_trait;
pub mod auto_merge_guard;
pub mod body_composer;
pub(crate) mod clock;
pub mod codeowners_gen;
pub mod comment_watcher;
pub mod evidence_emitter;
pub mod error;
pub mod gh_cli;
pub mod git_assist;
pub mod package_verify;
pub mod review;
pub mod semgrep_gate;
pub mod types;

pub use adapter_trait::GithubPrAdapter;
pub use auto_merge_guard::{
    apply_branch_protection, assess_merge_safety, classify as classify_merge_safety, MergePolicy,
    MergeVerdict, PrShape,
};
pub use body_composer::{compose_pr_body, PrContext};
pub use codeowners_gen::{generate_codeowners, render as render_codeowners, CodeownersEntry};
pub use comment_watcher::{watch_comments, NewComment};
pub use error::GithubPrError;
pub use gh_cli::GhCliAdapter;
pub use git_assist::{detect_force_push, suggest_squash_message};
pub use package_verify::{
    extract_dep_candidates, flag_candidate, verify_pr_deps_no_slopsquat, SlopsquatReport,
};
pub use review::{build_review_argv, post_review_comment, ReviewCommentKind};
pub use semgrep_gate::{
    parse_semgrep_findings, run_semgrep_gate, SemgrepResult, SemgrepSeverity,
};
pub use types::{PrCheckRun, PrOpenRequest, PrStatus, PrSummary, PrState};

#[cfg(test)]
mod tests;
