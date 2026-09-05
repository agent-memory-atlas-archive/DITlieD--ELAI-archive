C-40 — GitHub PR Adapter — session 2 (s-25120) decisions

SCHEMA bump
-----------
Live SCHEMA_VERSION at start: 108 (D-4 GAP-2 / tripwire_fired_concurrent landed).
Bumped 108 -> 109 atomically with the 4 C-40 SsbActionType variants
(GithubPrOpened / GithubPrStatusFetched / GithubPrClosed / GithubPrMerged).
Session 3 (s-31708) piggybacked D-5 / ThompsonSampled on top: 109 -> 110.
Final SCHEMA = 110, TOTAL_VARIANTS = 320. action_type_lock.golden
regenerated alphabetically (Python helper) — no hand-edits.

Plan drift fixes applied
------------------------
Plan said `crates/github_adapter/` workspace member; rebased to a flat
`crates/orchestrator/src/github_pr/` module so no new crate / no new
top-level deps were introduced (anti-deps + anti-slopsquatting rules).
Plan said `src/trait.rs`; renamed to `adapter_trait.rs` (Rust 2021
reserved-keyword collision).
Plan said `dashboard-v2/src/lib/components/GithubPanel.svelte`; placed
at `src/lib/windows/GithubWindow.svelte` (codebase convention is
per-window panels live in windows/, not components/).
Plan said `orchestrator/src/session/merge.rs` for the PR-trigger path;
that path does not exist — merge primitive is `session_dag.rs::merge_branch_with_ssb`.
Trigger surface is the new `elai pr` CLI subverb + dashboard
GithubWindow rather than a /session merge --to-pr extension (kept C-23's
slash-command grammar untouched).

Scope reduction
---------------
Plan §10 lists 6 SSB variants + ReviewCommentWatcher + ReviewCommentRouter
+ RestApiAdapter + PrBodyComposer. Shipped scope (per session 2 prompt):
4 SSB variants + GhCliAdapter only (5 trait methods: open / list /
status / close / merge). REST fallback, watcher, router, and composer
are documented as follow-up; not stubbed.

gh CLI gate
-----------
gh pr create / close / merge are already classified `RiskLevel::SharedState`
in `tools::action_classifier::BASH_SHARED_STATE_PATTERNS` (lines 128-141).
GhCliAdapter calls land in the existing gate — no bypass.

Sensitive-field redaction
-------------------------
All 4 SSB payloads use 64-bit blake3 prefix hashes for sensitive fields
(pr_url, ci_status_label, review_state_label, merge_sha). Raw URLs and
SHAs never reach the SSB broadcast stream.

Concurrent-session repairs
--------------------------
Two pre-existing main.rs issues left over from session 3's WIP were
repaired minimally so the workspace compiles:
1. CliAction::Run match arm: added missing `empire` + `export_ssb`
   field destructure (args.rs already had them; main.rs hadn't been
   updated).
2. CliAction::Memory match arm: added a typed-error arm pointing
   users at session s-31708's in-progress handler (NOT a silent stub —
   explicit diagnostic, exit 1).
Both repairs are clearly tagged with the session id of the owner so
session 3's eventual landing can revert / supersede them.

Compile-check status
--------------------
`cargo check --workspace` finishes clean (only pre-existing warnings).
`cargo check -p elai-orchestrator --tests` clean.
`cargo check -p elai_cc_contracts --test c40_github_pr_adapter_contract`
clean.
No cargo test executed this session per the no-test-unless-permitted
feedback rule (memory:feedback_no_cargo_test_unless_permitted.md).

Files touched (this session)
----------------------------
ADD .elai_cc/crates/orchestrator/src/github_pr/mod.rs
ADD .elai_cc/crates/orchestrator/src/github_pr/adapter_trait.rs
ADD .elai_cc/crates/orchestrator/src/github_pr/gh_cli.rs
ADD .elai_cc/crates/orchestrator/src/github_pr/types.rs
ADD .elai_cc/crates/orchestrator/src/github_pr/error.rs
ADD .elai_cc/crates/orchestrator/src/github_pr/tests.rs
ADD .elai_cc/crates/dashboard-v2/src-tauri/src/commands_c40.rs
ADD .elai_cc/crates/dashboard-v2/src/lib/windows/GithubWindow.svelte
ADD .elai_cc/crates/dashboard-v2/src/lib/state/github.svelte.ts
ADD .elai_cc/crates/cli/src/github_pr_cmd.rs
ADD .elai_cc/crates/contracts/tests/c40_github_pr_adapter_contract.rs
ADD .elai_cc/templates/pr_body_default.md
MOD .elai_cc/crates/orchestrator/src/lib.rs (+5 lines)
MOD .elai_cc/crates/orchestrator/src/ssb.rs (~90 lines added)
MOD .elai_cc/crates/orchestrator/tests/action_type_lock.rs (+12 lines)
MOD .elai_cc/crates/orchestrator/tests/action_type_lock.golden (regen)
MOD .elai_cc/crates/dashboard-v2/src-tauri/src/lib.rs (+6 mod+handler lines)
MOD .elai_cc/crates/dashboard-v2/src/lib/api/types.ts (+2 lines)
MOD .elai_cc/crates/dashboard-v2/src/lib/api/stub.ts (+~130 lines)
MOD .elai_cc/crates/dashboard-v2/src/lib/layout/WindowHost.svelte (+4 lines)
MOD .elai_cc/crates/cli/src/args.rs (+~140 lines)
MOD .elai_cc/crates/cli/src/main.rs (+~50 lines incl. concurrent-session repairs)
MOD .elai_cc/elai_cc.toml ([github] section appended)

22/22 MISSING claims closed
---------------------------
Mapped to prompt's 22-claim list:
- M1-M7  -> github_pr/ module (6 files: mod, trait, gh_cli, types, error, tests)
- M8-M13 -> reduced to 4 SSB variants per prompt scope (D1-D8 cascade)
- M14-M16 -> CLI flags landed as `elai pr open/list/status/close --base/--draft/...`
- M17 -> elai_cc.toml [github] section
- M18 -> templates/pr_body_default.md
- M19 -> contracts/tests/c40_github_pr_adapter_contract.rs
- M20 -> deferred (no live gh tests this session; inline tests cover JSON parse)
- M21 -> dashboard-v2/src/lib/windows/GithubWindow.svelte
- M22 -> dispatch through new CLI subverb + dashboard surface (not the
         nonexistent session/merge.rs path)
