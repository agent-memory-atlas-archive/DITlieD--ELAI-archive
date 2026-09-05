# K-Plan Grounding Map — FUTURE-PROOF / WEDGE EXECUTION (W-7 source for all K detail files)
Authored: 2026-06-12. Run: s-kplan-author-2026-06-12 (main LLM in-band, paper-to-plan PHASE 5 shape; no fleet).
Source brief: docs/thefinalglance/K-MASTER-BRIEF-2026-06-12.md + workflow/reasoning/
ELAI-FUTUREPROOF-POST-RESEARCH-REASONING-2026-06-12.md (the round-2 scorecard + kill-switches).

This map carries the LIVE-grep-confirmed anchors every K-N detail file cites instead of re-asserting
facts, the binding corrections where the master brief went stale (it warned it would), the dated web
verification of every external number K-1/K-7 use, and the live SSB baseline. Detail-file authors and
the wiring-contract validator MUST use the CORRECTED values here, not the brief's snapshot (W-7 is
enforced twice: at authoring and at validation). Re-grep every KG anchor at edit (W-9) — the SSB
baseline goes stale within days (the J-plan logged six values in one week).

================================================================================
CANONICAL CONSTANTS (live grep 2026-06-12)
================================================================================
SCHEMA_VERSION = 236 — the I-43 ClaimScopeMeasured bump (ssb.rs comment block at :3103-3105, :4987-4990,
  :9120-9127). NEVER hard-code; re-read the `pub const SCHEMA_VERSION` at edit (W-9).
TOTAL_VARIANTS = 545 — `const TOTAL_VARIANTS: usize = 545;` at
  crates/orchestrator/tests/action_type_lock.rs:685 (ALL_VARIANTS array length must match, :694/:2649).
The master brief's "234/543" (its own stated bait) is DEAD. K SSB deltas bump SEQUENTIALLY from 236/545,
one at a time, re-grep ssb.rs + action_type_lock.rs before each (9-tuple golden lockstep; a partial add
breaks the workspace — the 91-error incident).

================================================================================
KG — K GROUND-TRUTH ANCHORS (live grep-confirmed 2026-06-12)
================================================================================
KG-1  I-43 claim-scoper is SHIPPED, not PLANNED. SSB variant `ClaimScopeMeasured(ClaimScopeMeasuredKind)`
      live at ssb.rs:9127 (3-arm: PilotDelta / CrossLaneDivergence / ClaimScoped); action_type string
      'claim_scope_measured' (ssb.rs:9971, :10717); golden row ssb.rs:4990. CLI entrypoint live:
      `elai bench claim-scope [--suite <SUITE>] [--seed <u64>] [--json]` (args_bench.rs:70, parser
      :1011-1063, CliAction::BenchClaimScope :1063). Emitter `cli::bench_claim_scope::run_inner`
      (ssb.rs:3105). Fixture contract crates/contracts/tests/i43_claim_scope_fixture_contract.rs.
      => A-1 is INVOKE-the-shipped-gate, not build-it (KC-1).
KG-2  github_pr is a FULL module (C-40/E-41 WIRED), not a dark/renamed crate:
      crates/orchestrator/src/github_pr/{review.rs, body_composer.rs, comment_watcher.rs,
      adapter_trait.rs, gh_cli.rs, semgrep_gate.rs, auto_merge_guard.rs, package_verify.rs,
      git_assist.rs, clock.rs, types.rs, tests.rs}; CLI crates/cli/src/{pr_review_cmd.rs,
      github_pr_cmd.rs}; Tauri crates/dashboard-v2/src-tauri/src/commands_c40.rs; frontend
      crates/dashboard-v2/src/lib/state/github.svelte.ts; contract e41_ci_cd_pr_review_contract.rs +
      c40_github_pr_adapter_contract.rs. => K-3 renderer rides a real adapter (KC-2).
KG-3  review surface present (A-5 / K-3 evidence source): crates/orchestrator/src/review_surface.rs;
      CLI crates/cli/src/review_cmd.rs; G-5 contracts g5_dsr_recompute_contract.rs +
      g5_silent_by_default_contract.rs; H-52 anchor contract crates/contracts/tests/
      h52_finding_anchor_contract.rs. (score_review_surface / review_anchor exact symbols are
      PLAN-asserted at function granularity; executor re-greps the symbol names per W-7.)
KG-4  SAFEDELTA substrate present (A-3): crates/orchestrator/src/benchmarks/safedelta_runner.rs;
      CLI crates/cli/src/bench_safedelta.rs; contracts g6_safedelta_static_contract.rs +
      g6_anti_gaming_contract.rs.
KG-5  QDELTA substrate present (K-4 composes I-38): crates/cli/src/bench_qdelta.rs +
      benchmarks/mod.rs registration. K-4's gate-tax runner MIRRORS this runner shape, shares its corpus.
KG-6  SeCodePLT substrate present (A-2 F-2b): crates/cli/src/bench_seccodeplt.rs; smokes
      crates/orchestrator/tests/{f2_seccodeplt_fixture_smoke.rs, f2c_seccodeplt_baseline_smoke.rs}.
      The LIVE local-model run (W-10 RunPod checkbox) is the still-missing leg, not the harness.
KG-7  RRA CLI present (G-2; A-4 / K-6 precondition): crates/cli/src/rra_cmd.rs; verify path referenced
      from run_subcommand.rs + bench_cmd.rs. `elai verify-attestation` referenced in K-6's offline-verify.
KG-8  wire-dark-allowlist.toml = 105 entries at .elai_cc/wire-dark-allowlist.toml (grep -cE of the row
      markers). This is K-4's gate-inventory source (the default-ON gate set the composed tax measures)
      and K-5's policy-table input (which legs exist to tier).
KG-9  bench sub-command dispatcher live at args_bench.rs:75 — known subs: prmbench | mode-routing |
      conformal | arc | rag | sycophancy | canary-regression | tamper | seccodeplt | safedelta |
      staleness | mcp-schema-budget | crane | qdelta | claim-scope. K-2 (routing-economics) and K-4
      (gate-tax) ADD subs here, mirroring the existing arm shape (KG-5).

================================================================================
KC — BINDING CORRECTIONS (where the master brief / feature map went stale; authors MUST use these)
================================================================================
KC-1  A-1 STATUS FLIP. Brief §6.3 said "I-43's file (bench_claim_scope) exists as PLANNED — confirm no
      code landed." It HAS landed (KG-1, full CLI + SSB + contract). A-1 in K-0 is therefore framed
      "INVOKE the shipped `elai bench claim-scope` kill-gate and read its go/no-go verdict," and K-1's
      first measured number is harvested from a real run of it, not from building it. This ACCELERATES
      K-T1 (the cheap falsifier already exists).
KC-2  github_pr IS WIRED. Brief §6.3 warned "the rename caused stale FAIL verdicts; grep the module not
      the old crate." Grepped: the module is present and substantial (KG-2). K-3 is a RENDERER over this
      adapter + the G-5/H-52 evidence surfaces; it re-implements neither.
KC-3  A-8 (J-28 token ledger) is NOT shipped. No `UsageTurnEvent` in ssb.rs (live grep). J-28 remains
      PLANNED. CONSEQUENCE: K-2's honest per-lane $/verified-task capture DEPENDS on J-28 landing first
      (J-28 fixes the streaming-local input_tokens=0 capture bug + the sonnet-rate mispricing the routing
      number needs). K-2's live cost-delta leg is GATED on J-28; its deterministic fixture leg is not.
      This is the single most important sequencing fact K-2 carries.
KC-4  A-4 (I-40 `elai wrap` + .rra spec) is PLANNED. No wrap_cmd / trust_wrapper CLI present (live grep).
      A-4 stays the wedge ARTIFACT to build; G-9 redaction is its publication precondition (per I-40's
      own file); K-3 (renderer) does NOT wait on A-4 — it renders ELAI's own-run evidence first, the
      wrapped-foreign-agent evidence when A-4 lands.
KC-5  SSB baseline 236/545 (CANONICAL CONSTANTS), not 234/543. Every K +1 sequences from 236/545.

================================================================================
KW — WEB VERIFICATION of external numbers (dated 2026-06-12; KD-1 mandates these before any external use)
================================================================================
Every figure below is VERIFIED against the primary source named. They are EXTERNAL evidence for the
problem K solves; per KD-1 they are tagged [EXTERNAL] in every plan file and NEVER claimed as an ELAI
measured number. The dossier-build session (K-1 execution) re-verifies each before it ships in copy.
KW-1  Package hallucination. Spracklen et al., USENIX Security 2025, "We Have a Package for You!"
      (usenix.org/conference/usenixsecurity25/presentation/spracklen; PDF usenixsecurity25-spracklen).
      576,000 code samples, 16 LLMs, 2 languages; hallucinated-package rate >=5.2% commercial /
      21.7% open-source; 205,474 unique fake names; recurrence: 58% recur across 10 iterations,
      43% recur in ALL 10 of 10 repeated queries (the brief's "43%" is the stricter all-10 sub-metric;
      both true, record BOTH with the distinction). Composition: 38% conflations / 13% typos / 51%
      fabrications. Grounds ELAI F-9 + the K-7 AppSec brief.
KW-2  Patch insecurity. "How Safe Are AI-Generated Patches?", arXiv:2507.02976. Standalone Llama-3.3-70B
      introduced 135 verified new vulns vs 12 in developer patches (= 11x) over 20,000+ SWE-bench issues;
      novel CWE-95 (eval-injection) / CWE-502 (deserialization) distinct from developers' CWE-732/377;
      agentic frameworks (OpenHands/AutoCodeRover/HoneyComb) on a 2,200+ subset ALSO introduce vulns,
      highest count under the most-autonomy framework (more autonomy = more vulns). Grounds K-7 vs F-2.
KW-3  Self-improvement reward hacking. "Reward Hacking in Self-Improving Code Agents", OpenReview
      ikrQWGgxYg. 73.8% of Kernel-Bench optimizations and 46.8% of ALE-Bench (algorithmic) optimizations
      show proxy gains without real-task gains; the proxy-reality gap widens 26.4% -> 57.8% from 10 to
      100 optimization steps. Grounds K's "visible-pass != trust" thesis + the self-improvement guard.
KW-4  Orchestration overhead. Microsoft Agent Framework, BUILD 2026 "CodeAct" (devblogs.microsoft.com/
      agent-framework/codeact-with-hyperlight + microsoft-agent-framework-at-build-2026-announce).
      Collapsing the tool-call loop into one executed code block cuts end-to-end latency ~50% and token
      usage >60% in representative tool-heavy workloads; "many agents are bottlenecked by orchestration
      overhead, not model quality." Grounds K-4 (gate-tax is real) + K-5 (collapse-where-safe).
KW-5  Visible-vs-hidden suite gap. SpecBench, arXiv:2605.21384 ("Measuring Reward Hacking in Long-Horizon
      Coding Agents"). The brief's "+28pp per 10x code size" figure is attributed here; PLAN-asserted at
      the exact-number granularity — the dossier-build session quotes the precise table value + page
      anchor before any external use (KD-1; the B-40/B-42 bad-cite house rule).
KW-6  PENDING-VERIFY at dossier build (Perplexity-report-sourced, NOT yet primary-source-confirmed; tag
      [EXTERNAL, unverified] until K-1 execution): DORA 2026 (+25% AI use ~ -1.5% throughput / -7.2%
      delivery stability) + GitClear 211M-line duplication trend; METR (~half benchmark-passing PRs not
      merged by maintainers; median 4 tests/problem); vendor telemetry Uplevel +41% bugs-with-Copilot,
      CodeRabbit 1.7x issues in AI-majority PRs (vendor-telemetry tier, weakest — label as such). These
      do NOT enter any plan file as asserted numbers; they enter the dossier only after K-1 re-verifies.

================================================================================
KS — SOURCE DOCS (the K tracks' provenance)
================================================================================
KS-1  K-MASTER-BRIEF-2026-06-12.md — the comprehensive handoff; Sections 3.1 (Section-A pointers),
      3.2 (the K-1..K-10 specs), 3.4 (KD-1..KD-5), 4 (non-goals), 5 (kill-switches). Authoritative for
      SCOPE; superseded on ANCHORS by this map (KC-1..KC-5).
KS-2  ELAI-FUTUREPROOF-POST-RESEARCH-REASONING-2026-06-12.md — round-2 claim-by-claim scorecard
      (Sections 1.1-1.11) + the strategy paragraph (2) + the recommendation (3) + kill-switches (4).
KS-3  The three Perplexity deep-research reports (16 verification-market PT-BR / 17 moat-economics /
      18 market-direction), read IN FULL in the prior session; their primary citations are the KW set.
      On-disk path unknown (brief §0); the KW verifications stand on the PRIMARY sources, not the reports.
KS-4  ELAI-FEATURE-MAP.md (header stamp 234/543 at 2026-06-12 — STALE per KC-5; re-grep at every read).
      Source of truth for which prior-plan units K composes; K upserts its own PLANNED rows there (§6.6).

================================================================================
DESIGN-DOC CORRECTIONS (ACTIVATION-vs-BUILD reality, per template R-a/§15)
================================================================================
DC-K1  A-1 = ACTIVATE/INVOKE (shipped), not BUILD (KC-1).
DC-K2  K-3 = BUILD a renderer over a WIRED adapter (KC-2), not build the adapter.
DC-K3  K-2 live leg = BLOCKED-ON-J-28 (KC-3); deterministic fixture leg = BUILD-now.
DC-K4  A-4 = BUILD (PLANNED, KC-4); K's wedge artifact, time-boxed under KD-5.
DC-K5  K-4/K-5 = BUILD over present substrate (KG-5 qdelta runner, KG-8 allowlist inventory).

End of K-GROUNDING-MAP.
