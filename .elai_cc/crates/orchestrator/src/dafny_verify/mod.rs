//! C-52 — Dafny Formal Verification Spike (host-side scaffolding).
//!
//! Paper grounding: arXiv:2604.22601 (Md Erfan, Md Kamal Hossain
//! Chowdhury, Ahmed Ryan, Md Rayhanur Rahman 2026 — "From Natural
//! Language to Verified Code: Toward AI Assisted Problem-to-Code
//! Generation with Dafny-Based Formal Verification"). Verified via
//! arxiv.org/abs/2604.22601 + arxiv.org/html/2604.22601 + WebSearch
//! cross-confirmation on 2026-05-24. Interface map:
//! `workflow/papers/nl2vc-dafny-interface.md`.
//!
//! GROUNDING CAVEATS (mandatory — DO NOT misframe):
//!   - The paper describes a TIERED PROMPTING protocol with three
//!     tiers: contextless (RQ1), signature-guided (RQ2), and
//!     self-healing (RQ3). The signature tier is mandatory for any
//!     non-trivial result (Section 3, near-universal failure on
//!     contextless across 6 of 7 models).
//!   - The paper enforces a VACUOUS-VERIFICATION GUARD via uDebug
//!     functional tests: a Dafny spec that "verifies" trivially
//!     (e.g. constant return) is flagged as vacuous. Raw `dafny
//!     verify` exit code is INSUFFICIENT — uDebug functional tests
//!     MUST also pass before claiming a spec is verified.
//!   - Self-healing iteration is capped at 10 (paper Section 3).
//!     The paper does not characterise diminishing returns past 10.
//!   - Temperature sweep is {0.0, 0.2, 0.4, 0.6, 0.8} (paper
//!     Section 3); the optimum varies per model + per tier.
//!   - verify@k is reported for k in {1, 3, 5}; k=5 is the strongest
//!     signal in the paper's results tables.
//!   - The paper's top performer is Gemma 4-31B (RQ3a: 90.91%
//!     verify@5). ELAI MUST NOT integrate Gemma 4 per memory rule
//!     `feedback_no_gemma4.md`. Substitutes from the paper's
//!     evaluation set: Codestral-22B-v0.1 (63.64% RQ2 verify@5),
//!     GPT-OSS-120B (63.64% RQ2 + 81.82% RQ3b), Qwen 3 Coder 30B
//!     (45.45% RQ2). ELAI's current sidecar default (Qwen 2.5
//!     Coder 7B) is NOT in the paper's evaluation set; expect
//!     verify@5 below 45% on the signature tier and document the
//!     gap.
//!   - The local dev host has NO `dafny` binary on PATH (verified
//!     via `where.exe dafny` and `dafny --version` on 2026-05-24).
//!     Live `dafny verify` execution is `RUNPOD-PENDING:
//!     live_dafny_verification_with_non_gemma_substitution`. This
//!     module ships the host-side scaffolding (tier enum + prompt
//!     builder + CLI shellout wiring + heal-loop + vacuous-guard +
//!     types) fully; the wiring is dormant until a remote runner
//!     with Dafny installed is available.
//!
//! Module layout (sibling of `contract_coding/` per C-50 pattern):
//!   - [`tier`]          — `DafnyTier { Contextless, Signature,
//!                         SelfHealing }`, mirrors paper RQ1/RQ2/RQ3.
//!   - [`prompt`]        — `build_dafny_prompt(nl_problem, tier,
//!                         prior_attempts)` produces the per-tier
//!                         prompt body.
//!   - [`cli_shellout`]  — `invoke_dafny_cli(spec_path)` shells to
//!                         the local `dafny verify` binary; mirrors
//!                         the C-43 `archive_verify` sync entry +
//!                         the C-40 `github_pr/gh_cli.rs`
//!                         tokio::process::Command pattern.
//!   - [`heal_loop`]     — `self_heal_loop(...)` drives the
//!                         iterative repair loop with a hard cap of
//!                         10 iterations.
//!   - [`udebug_guard`]  — `check_vacuous(spec, functional_tests)`
//!                         returns `false` (NOT vacuous) iff the
//!                         spec passes Dafny AND every functional
//!                         test passes; returns `true` (vacuous)
//!                         when Dafny passes but tests fail.
//!   - [`types`]         — `VerifyAt { k_1, k_3, k_5 }` counters,
//!                         `TemperatureSweep` constants,
//!                         `DafnyVerifyResult`, `HealTrace`.

pub mod cli_shellout;
pub mod heal_loop;
pub mod prompt;
pub mod tier;
pub mod types;
pub mod udebug_guard;

pub use cli_shellout::{invoke_dafny_cli, DafnyCliError, DafnyVerifyResult};
pub use heal_loop::{self_heal_loop, HealError, HealTrace, MAX_HEAL_ITER};
pub use prompt::build_dafny_prompt;
pub use tier::DafnyTier;
pub use types::{
    TemperatureSweep, VerifyAt, ELAI_SUBSTITUTE_MODELS, GEMMA4_BLOCKED, TEMPERATURE_SWEEP,
    VERIFY_K_VALUES,
};
pub use udebug_guard::{check_vacuous, FunctionalTestResult};
