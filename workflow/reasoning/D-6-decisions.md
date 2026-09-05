D-6 closure decisions — held-out task store + 9-gram contamination filter

Session: s-25120 (D-6 completion-drive recheck closure).
Plan: docs/thefinalglance/D-plan/D-6_held_out_task_store.md.
Prior ship: session s-62376 (initial held_out.rs at SCHEMA 73).

DECISION-1 — SCHEMA bump 111 -> 112 (plan's 51 -> 52 stale)

The plan body called for SCHEMA_VERSION 51 -> 52. Real ssb.rs head at the time of this closure is 111 (post-D-5 ThompsonSampled + C-41 SkillTriggered/SkillExecuted). The closure bumps to 112 atomically with the two new D-6 variants HeldOutPromotionGate + HeldOutRefresh. This is the quadruple-confirmed plan-stale-vs-actual rule per D-2/D-3/D-4/D-5 sessions: always read FRESH SCHEMA_VERSION before bumping; never trust the plan body's number.

The bump is ATOMIC: SCHEMA const, SQL CHECK allowlist, SsbActionType enum, as_sql_str arm, ssb_action_type_from_str arm, SsbPayload enum, SsbPayload::action_type() arm, all_test_payloads helper, action_type_lock.rs TOTAL_VARIANTS / ALL_VARIANTS / exhaustive_match_guard, and action_type_lock.golden file — all in the same edit pass. Per C-25 drop+recreate rule.

DECISION-2 — SQLite store kept (plan wanted per-task TOML; trade-off documented)

The plan §1-2 + §8 + observable_effect contract called for `.elai_cc/held-out/{date}/{task_id}.toml` per-task TOML files. The shipped implementation (s-62376) is a single SQLite store at `.elai_cc/held-out/store.sqlite` with the rusqlite-backed `HeldOutStore` mirroring `crates/memory/src/store.rs`.

Trade-off:
  + SQLite gives atomic insert + n-gram filter scan in a single transaction, file-locking-free, query-able via indexes.
  - Loses the git-diff-friendly per-task TOML the plan called for.

Closure pass KEEPS the SQLite store (out-of-scope per the executor prompt: "SQLite store kept... do NOT switch to per-task TOML"). The 25-seed-task layout (plan §8) is reclassified as SPEC_DRIFT per the closure's `.elai_cc/held-out/2026-05/` JSONL bundle layout from s-62376.

DECISION-3 — GateResult renamed to PromotionGateResult (collision avoidance)

The plan §5 called for `held_out::gate_promotion -> GateResult { allow, reason }`. `crate::gate::GateResult { gate_name, passed, detail }` already exists (evidence-gate for FSM transitions). Closure renames the held-out variant to `PromotionGateResult` to avoid the collision; pubic API surface, all call sites referencing the new helper use the new name.

The plan body still says "GateResult"; the rename is documented here so a future reviewer cross-referencing the plan doesn't search for a non-existent symbol.

DECISION-4 — Hardcoded n=9 / threshold=0.3 re-cited as ELAI-empirical (NOT LiveCodeBench-specified)

The shipped `DEFAULT_NGRAM_SIZE = 9` + `DEFAULT_JACCARD_THRESHOLD = 0.3` docstrings (held_out.rs:55-64) cited "LiveCodeBench §3.2 published 9 as the contamination-detection sweet spot" + "LiveCodeBench §3.2 reports 0.3 as the false-positive-balanced operating point". paper-ingestor sweep on arxiv:2403.07974 (workflow/papers/livecodebench-2403.07974-interface.md) confirms the paper has NO n-gram methodology section — LiveCodeBench's contamination defense is purely temporal date-windowing (§S3.SS1 + §S6). The n-gram technique itself is from Shi et al. 2023 (arXiv:2303.10130) + Riddell et al. 2024, cited as EXTERNAL prior work in §S6.SS3.

Closure rewrites the docstrings:
  - n=9 is ELAI-empirical: large enough to avoid common-bigram false positives, small enough that paraphrase still trips the filter.
  - threshold=0.3 is ELAI-empirical: calibrated against the ELAI baseline corpus.

Override via the new `[held_out]` section of elai_cc.toml (`ngram_n`, `ngram_threshold`). The contract test `default_ngram_size_matches_elai_empirical_value` carries the re-citation in its assertion message so a future re-grounding pass can quickly find the citation chain.

DECISION-5 — 6-month retirement window is ELAI policy (NOT LiveCodeBench-specified)

The plan §4 + §11 called for "task-retirement at 6 months uniformly-passing" as a LiveCodeBench-style defense. paper-ingestor confirms LiveCodeBench has no uniformly-passing retirement policy — the paper retires by date alone (release_date older than next data refresh). The 6-month + uniformly-passing-100 cutoff is ELAI-novel.

The closure tags the policy in code comments (`held_out.rs::check_for_auto_retire`) AND in the new `[held_out]` section of elai_cc.toml (with the ELAI-policy comment). `RetireReason::UniformlyPassing { passes_in_window, window_months }` carries the policy values into the SSB audit trail so the choice is queryable post-hoc.

DECISION-6 — Soft Contamination does NOT extend LiveCodeBench; layering documented

The plan §1 introduction framed arxiv:2602.12413 (Soft Contamination) as "extends LiveCodeBench n-gram". paper-ingestor confirms the paper does NOT cite LiveCodeBench; its central argument is that n-gram matching is INSUFFICIENT (false-negatives on near-duplicates) and proposes cosine + LLM annotation as a FULL REPLACEMENT, not an n-gram extension.

The closure integrates the two with explicit layering:
  - Temporal date-windowing (LiveCodeBench S3.SS1) is the PRIMARY eligibility predicate via `is_task_after_cutoff(task, cutoff)`.
  - 9-gram Jaccard fingerprint (Shi 2023 / Riddell 2024) is a defense-in-depth layer on top — ELAI-bolted, not LiveCodeBench-specified.
  - Soft-contamination cosine + LLM (arxiv:2602.12413) is a SEPARATE follow-up filter, reserved for the sidecar-embedded fingerprint pass. The `ContaminationSource::SoftContamination { cosine, threshold }` variant lands now so the retire-reason serde surface is stable.

DECISION-7 — BAI consumer routes through agent_pool::sample_held_out_task

The plan §5 called for `held_out_regression` + `gate_promotion` to live in held_out.rs and feed the BAI promotion gate directly. The shipped wiring routes the BAI consumer through `agent_pool::sample_held_out_task` which yields the next held-out task (oldest-first deterministic pick). Closure adds a thin wrapper `gate_promotion_via_held_out` in held_out.rs that bundles the regression + gate computation against pre-collected eval histories — keeping the call-site readable from the plan's perspective while preserving the shipped agent_pool layering.

The layering is:
  1. `agent_pool::sample_held_out_task(store, now_ms)` -> next task envelope.
  2. BAI eval loop runs against the sampled task and collects eval snapshots.
  3. `held_out::gate_promotion_via_held_out(baseline, candidate, threshold)` -> PromotionGateResult.
  4. BAI emits `HeldOutPromotionGate` SSB row with the gate outcome.

DECISION-8 — ts-rs not adopted; hand-authored TS mirror for new types; chrono not adopted

D-5 DECISION-7 established: no ts-rs in the dashboard-v2 crate; cross-language type pairs are hand-typed and verified by `.claude/tools/cross_lang_diff.py`. D-6 closure preserves this: the new SSB payload types (`HeldOutPromotionGatePayload`, `HeldOutRefreshPayload`) are Rust-only for now; dashboard surface extension is deferred (the existing HeldOutPanel already renders `held_out_task_added` / `held_out_task_used`; the closure does not extend the panel — the new SSB rows are observable via raw SSB-tail queries pending a follow-up panel pass).

chrono is also NOT adopted. The plan called for `DateTime<Utc>` on the `fetched_at_ms` field; closure uses `i64` UNIX-ms consistent with the rest of the orchestrator codebase (every existing `*_ts_ms` field). chrono is not in the workspace and adding it requires explicit user confirmation per CLAUDE.md no-new-deps rule.

DECISION-9 — TaskSource additive: ArxivRepro + HumanCurated only; existing variants untouched

The plan §1 called for adding `fetched_at` to ALL TaskSource variants. Doing so would break 15+ struct-literal construction sites across canary.rs, held_out.rs (internal), and the smoke tests — an additive field break on a closed enum literal. Closure adds the field ONLY on the two new variants (ArxivRepro + HumanCurated) where the field is load-bearing. A normalized accessor `TaskSource::fetched_at_ms() -> Option<i64>` returns `Some` for the two new variants and `None` for the three pre-existing variants (where the surrounding `HeldOutTask.ingested_ts_ms` already carries the wall-clock dimension).

The `#[serde(default)]` on the field keeps deserialization wire-compatible with pre-D-6-closure SQLite rows.

DECISION-10 — ExpectedOutcome 3-variant collapse intentionally preserved

The plan §1 called for a 5-variant `ExpectedOutcome` (Code{test_cmd}, Math{sympy_expr}, Prose{rubric}, Planning{final_state}, KnowledgeQa{reference_answer}). The shipped enum is 3 single-string variants (Code(String), Math(String), Prose(String)). Closure EXPLICITLY preserves the 3-variant shape per the executor prompt's out-of-scope rule ("The shipped ExpectedOutcome 3-variant collapse... do NOT re-expand"). Planning + KnowledgeQa are deferred to a follow-up plan (likely D-3 or D-15 where the verifier-consensus path needs Planning state).

Files touched (D-6 closure)

ADD:
  - .elai_cc/crates/contracts/tests/held_out_contract.rs (10 tests)
  - workflow/reasoning/D-6-decisions.md (this file)

MODIFY:
  - .elai_cc/crates/orchestrator/src/held_out.rs
      * docstring re-citation (DEFAULT_NGRAM_SIZE + DEFAULT_JACCARD_THRESHOLD)
      * TaskSource +2 variants (ArxivRepro + HumanCurated) + fetched_at_ms accessor
      * +HeldOutStatus enum (5 variants)
      * +ContaminationSource enum (3 variants)
      * +RetireReason enum (3 variants, incl. nested ContaminationSource)
      * +AutoRefreshPolicy enum (4 variants)
      * +ModelTrainingCutoff struct + is_task_after_cutoff fn
      * +EvalHistorySnapshot struct + pass_rate method
      * +check_for_auto_retire fn (with min_evals guard)
      * +PromotionGateResult struct
      * +held_out_regression fn
      * +gate_promotion fn
      * +gate_promotion_via_held_out fn
      * +10 new unit tests
  - .elai_cc/crates/orchestrator/src/ssb.rs
      * SCHEMA_VERSION 111 -> 112
      * SQL CHECK allowlist +2 entries (held_out_promotion_gate + held_out_refresh)
      * SsbActionType +2 variants
      * as_sql_str +2 arms; ssb_action_type_from_str +2 arms
      * SsbPayload +2 variants
      * SsbPayload::action_type() +2 arms
      * +HeldOutPromotionGatePayload struct
      * +HeldOutRefreshPayload struct
      * test-payload helper +2 explicit arms
      * all_test_payloads +2 entries (count bump 261 -> 263)
  - .elai_cc/crates/orchestrator/tests/action_type_lock.rs
      * TOTAL_VARIANTS 322 -> 324
      * ALL_VARIANTS +2 (HeldOutPromotionGate + HeldOutRefresh)
      * exhaustive_match_guard +2 arms
  - .elai_cc/crates/orchestrator/tests/action_type_lock.golden
      * +2 entries; ordinals shifted at insertion point (alphabetical sort)
  - .elai_cc/elai_cc.toml
      * +[held_out] section (6 tunables; ELAI-empirical defaults documented)

Out-of-scope (deferred to follow-ups)

  - HeldOutPanel.svelte extension to render HeldOutPromotionGate +
    HeldOutRefresh counters (the existing panel renders task_added /
    task_used; the closure does not extend it).
  - Per-task TOML on-disk layout (SQLite preserved).
  - 25 seed held-out tasks at .elai_cc/held-out/2026-05-19/ (existing
    JSONL bundles preserved at .elai_cc/held-out/2026-05/).
  - ExpectedOutcome expansion from 3 to 5 variants.
  - Soft-contamination cosine fingerprint wire (variant lands, producer
    deferred to the sidecar-embedded fingerprint pass).
