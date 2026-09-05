C-52 — Dafny Formal Verification Spike (host-side scaffolding)

Session: s-25120 session 2 of 4 (cluster with sessions 3/4 concurrent)
Paper grounding: arXiv:2604.22601 (NL2VC) verified 2026-05-24 via
arxiv.org/abs/2604.22601 + arxiv.org/html/2604.22601 + WebSearch.
Interface map: workflow/papers/nl2vc-dafny-interface.md.

Outcome path: SUCCESS-PATH (host-side scaffolding shipped; live
`dafny verify` execution is RUNPOD-PENDING). NOT infeasibility-note
path — the scaffolding (tier enum + prompt builder + CLI shellout +
heal loop + udebug guard + types + SSB cascade + CLI + contract
test) lands today.

DECISION 1 — Model substitution mandatory (Gemma 4 BLOCKED).

The paper's top performer is Gemma 4-31B (RQ3a 90.91% verify@5; RQ3b
tied at 81.82%). Per memory rule feedback_no_gemma4.md, ELAI MUST
NOT integrate Gemma 4 into the sidecar. The C-52 implementation cites
Gemma 4 results ONLY as paper-headline numbers; the substitute list
intentionally omits Gemma 4.

Substitutes (paper Table III, verify@5 RQ2 in parens):
  - Codestral-22B-v0.1 (63.64%)
  - GPT-OSS-120B (63.64% RQ2 + 81.82% RQ3b)
  - Qwen 3 Coder 30B (45.45%)
  - Qwen 3.5-9B (72.73%, T=0.8 — strongest non-Gemma RQ2)
  - Qwen 3.6-35B-A3B (63.64%) MoE
  - GPT-OSS-20B

Rationale: paper is comparative; it does not prescribe Gemma 4 for
downstream integration. Memory rule is HARD; substitution discipline
is enforced by const ELAI_SUBSTITUTE_MODELS + contract test D6 +
SSB payload docstring caveat + CLI surface caveat.

DECISION 2 — Crate location: orchestrator subdirectory.

Plan-as-written ADDed a new top-level workspace member crate
`crates/formal_verify/`. The session-prompt rebase note instructs
relocation to `crates/orchestrator/src/dafny_verify/` (sibling
pattern of contract_coding/ from C-50; github_pr/ from C-40;
mutation_testing.rs from C-28). The subdirectory pattern:
  - keeps Cargo.toml workspace members list stable (no new entry)
  - reuses existing tokio + serde + thiserror + tracing deps
  - matches C-40 / C-49 / C-50 sibling precedent
  - allows the SSB cascade to land in one crate (orchestrator) with
    no cross-crate import cycle

DECISION 3 — Local Dafny toolchain absent; live verification
RUNPOD-PENDING.

Per the C-52 cross-check report + context-compiler probe on
2026-05-24: `where.exe dafny` and `dafny --version` both return
"command not found" on the local dev host (F: drive). Live `dafny
verify` execution is RUNPOD-PENDING:
  RUNPOD-PENDING: live_dafny_verification_with_non_gemma_substitution

The host-side scaffolding ships fully:
  - tier enum, prompt builder, CLI shellout (tokio::process::Command),
    heal loop with MAX_HEAL_ITER=10, udebug vacuous-verification guard,
    constants (TEMPERATURE_SWEEP / VERIFY_K_VALUES / GEMMA4_BLOCKED /
    ELAI_SUBSTITUTE_MODELS), 1 SSB variant, CLI surface, contract test
  - the `invoke_dafny_cli` shellout RETURNS `DafnyCliError::ToolchainNotFound`
    on missing binary (not panic); the CLI `status` subcommand surfaces
    the absence as informative (exit 0)

Live RUNPOD pilot scope: 11-problem MBPP-DfyVerify subset (paper
ground-truth + uDebug functional tests). Expected outcomes per model:
  - Qwen 2.5 Coder 7B (current sidecar default): verify@5 BELOW 45%
    on the signature tier (NOT in paper eval set; lower-bound by
    smaller-model trend)
  - Codestral-22B-v0.1: target 63.64% RQ2 verify@5 (paper Table III)
  - Self-healing (RQ3) lifts above signature-tier baseline

DECISION 4 — Vacuous-verification guard MANDATORY (paper §3).

Raw `dafny verify` exit-zero is INSUFFICIENT. The paper documents
`vacuous_verification` as a failure mode: weak specs (e.g. constant
return, empty postcondition) pass Dafny trivially. The
`udebug_guard::check_vacuous` helper flags as VACUOUS:
  - empty functional-test suite (inadequate coverage), OR
  - ANY failing functional test

A spec is CLAIMED VERIFIED only when `dafny_pass && udebug_pass`.
The SSB payload `DafnyVerifyCompletedPayload` carries BOTH fields so
the dashboard / consumers can distinguish vacuous passes.

DECISION 5 — Tier ordering: contextless -> signature -> self_healing.

Paper RQ1 / RQ2 / RQ3 sequence. Contextless near-universal failure
(6 of 7 models at 0% verify@5) — included for paper-faithful
reproducibility, NOT recommended as the default tier. The CLI
default tier is `signature` (paper minimum-viable). Self-healing
(RQ3) is opt-in via the `--max-iter > 0` flag and is capped at 10.

DECISION 6 — SSB variant count: ONE (not three).

Plan-as-written proposed three variants (dafny_verify_run /
dafny_verify_pass / dafny_verify_fail). Session-prompt B2 instructs
ONE variant `DafnyVerifyCompleted` carrying the richer
{tier_hash, dafny_pass, udebug_pass, heal_iter, ts_ms} payload.
Rationale: one row per heal-loop termination captures the full
outcome (initial vs healed, pass vs fail, vacuous vs non-vacuous)
without the schema bloat of three near-duplicate rows. The
ContractCoding / Sycophancy / Mutation Testing precedents all use
single-completion rows, not pre-emit + post-emit pairs.

DECISION 7 — SCHEMA cascade: 125 -> 126 atomic.

Coordinated with session-3 SCHEMA head readout (125 post-C-50).
TOTAL_VARIANTS 350 -> 351 (+1 variant). The 8-site cascade:
  1. SsbActionType enum + doc-comment
  2. as_sql_str arm
  3. from_str arm
  4. SsbPayload enum
  5. action_type() impl arm
  6. Payload struct DafnyVerifyCompletedPayload
  7. test_payload helper arm
  8. all_test_payloads vector + len()=290 assert
  9. action_type_lock.rs ALL_VARIANTS + exhaustive_match_guard
  10. action_type_lock.golden (regen alphabetical; new entry at 0085
      between csq_evaluated and dag_repair_escalated)
  11. SQL CHECK constraint string

DECISION 8 — Citation discipline: 6th cite VALIDATED, NOT incident.

The mistakes.md cluster (5 prior bad-cite incidents from B-40 / B-42
/ C-43 / C-44 / C-50) had us pre-flag arXiv:2604.22601 as a potential
6th incident. The paper-ingestor verified the paper EXISTS, the
arXiv ID resolves, the title is "From Natural Language to Verified
Code...", Gemma 4-31B IS one of seven evaluated models, the 90.91%
figure is reported in Table III for RQ3a verify@5. The cite is
VALID and the topic matches. Recording this in mistakes.md as a
CITE-VALIDATION-PASS so the cluster pattern doesn't drift into "all
arXiv:26xx.xxxxx are suspect" — the verification protocol is
WORKING.

The ONE caveat: the paper compares models; it does NOT prescribe
Gemma 4 for downstream integration. ELAI's substitution is
PRINCIPLED REJECTION (memory rule), not paper misframing.

RUNPOD-PENDING list (1):
  1. live_dafny_verification_with_non_gemma_substitution
     - scope: 11-problem MBPP-DfyVerify subset
     - models: Codestral-22B-v0.1 (primary), GPT-OSS-120B (fallback),
       Qwen 3 Coder 30B (offline-friendly), Qwen 2.5 Coder 7B
       (current sidecar; expect below-45% verify@5 on signature)
     - infrastructure: remote runner with Dafny 4.11.0 + Z3 +
       Python 3.14.4 (uDebug harness)
     - blocker: local dev host (F:) has no Dafny binary on PATH

References:
  - C-52 plan: docs/thefinalglance/C-plan/C-52_dafny_formal_verification_spike.md
  - C-52 cross-check report: workflow/sessions/s-25120/cross-check/C-52_dafny_formal_verification_spike-report.md
  - Paper interface: workflow/papers/nl2vc-dafny-interface.md
  - Module: .elai_cc/crates/orchestrator/src/dafny_verify/{mod,tier,prompt,cli_shellout,heal_loop,udebug_guard,types}.rs
  - SSB: .elai_cc/crates/orchestrator/src/ssb.rs (DafnyVerifyCompleted + DafnyVerifyCompletedPayload, SCHEMA_VERSION 126)
  - CLI: .elai_cc/crates/cli/src/dafny_cmd.rs + .elai_cc/crates/cli/src/args.rs (DafnyAction)
  - Contract test: .elai_cc/crates/contracts/tests/c52_dafny_formal_verification_contract.rs
  - Memory rule: C:\Users\user\.claude\projects\f---ELAI-workflow\memory\feedback_no_gemma4.md
