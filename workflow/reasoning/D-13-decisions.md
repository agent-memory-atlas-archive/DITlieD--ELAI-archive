D-13 decisions — adversary role + cross-family verifier closure

Session: s-25120 completion-drive cluster D-13.
Substrate: shipped 2026-05-20 (changelog 17094-17137). Closure session:
2026-05-24 post-C-47 (SCHEMA 122).

Status going in: 13 IMPLEMENTED + 9 MISSING + 0 PARTIAL + 0 NOT_WIRED +
2 SPEC_DRIFT per the recheck report. Status going out: closure files
authored, production dispatch wired, paper-grounded decision record
written. SCHEMA preserved at 122 (no bump); TOTAL_VARIANTS preserved at
343 (no new SSB variants added).

Spec ref: docs/thefinalglance/D-plan/D-13_adversary_role_cross_family.md
+ docs/specs/guide/real-problem-solving-grounded-2026-05-19.md §4.3
(lines 240-251) — Mode B Adversary role.
Plan grounding files: workflow/papers/
  self-correction-bench-2507.02778-interface.md (64.5% blind-spot rate);
  constitution-collapse-2504.04918-interface.md (cross-family GPT-4o
    critic on 8B small-model collapse);
  constitutional-small-2503.17365-interface.md (4-outcome taxonomy);
  latent-verifiers-2504.16760-interface.md (D-2 reuse — heterogeneous
    verifier failure on same-family transfer);
  rstar-math-2501.04519-interface.md (D-2 reuse — heterogeneous
    verifier strategy).

---

DECISION-1 — NO SCHEMA bump
The D-13 plan blurb says SCHEMA_VERSION 58->59. Current head is 122
(post-C-47 / Case-Based Calibration of Tool-Use). The D-13 SSB row pair
(AdversaryGenerated + ConsensusReevaluatedWithCounter) landed at an
earlier shipped SCHEMA in the 2026-05-20 substrate. Closure adds NO
new SSB variants, so the SCHEMA stays at 122. This is the ninth+
confirmation of the [SCHEMA-actual vs plan-stale] rule across
D-2..D-13. Recorded for the rule's running counter.

DECISION-2 — Plan vocabulary vs shipped naming preserved
Plan task 8 names two SSB variants: AdversaryVerdict + CrossFamilyUnavailable.
Shipped substrate has AdversaryGenerated + ConsensusReevaluatedWithCounter
(both already-wired through commands_d13.rs + AdversaryPanel.svelte + 4
adversarial_*.rs tests). Renaming would cascade through the dashboard +
tests + the action_type_lock golden + would inflate TOTAL_VARIANTS via
duplicate-then-deprecate. Per the CONSOLIDATED-SSB-VARIANT precedent
(D-9 DECISION-2 / D-10 DECISION-2 / D-11 DECISION-1 / D-12 DECISION-2 —
now quintuple confirmation), closure PRESERVES the shipped names and
documents the divergence here. Same decision applies to the plan-named
ConsensusRole-vs-AgentRole and VerifierKind-vs-SignalKind drifts —
shipped names win.

DECISION-3 — CrossFamilyUnavailable variant NOT added
Verification: grep "unavailable", "no_adversary", "skip.*mode_b" across
adversary.rs + agent_pool.rs + grounding_mode.rs returned the existing
HeterogeneousVerifierMissing tracing::warn! at agent_pool.rs:1366
(emitted from run_adversarial_pass when the verifier pool spans <2
families). That warning + the ConsensusReevaluatedWithCounter row pair
already cover the "no cross-family adversary available" signal via
verdict-inversion semantics: when the dispatch hook refuses a same-
family pair, no row pair is emitted and the consensus stays at the
original verdict (== "no counter-evidence available"). Adding a 3rd
SSB variant for this gap would bump TOTAL_VARIANTS 343->344 + .golden
regen + duplicate semantic coverage already served by the shipped
substrate. Closure does NOT add CrossFamilyUnavailable.

DECISION-4 — SourceTierCitation enum added (genuine additive type)
This is the ONE genuinely missing-without-substitute type per the D-13
plan. EvidenceRef (claim.rs) carries kind + content + ref but does
NOT discriminate by source tier (T1 peer-reviewed vs T2 web vs T3
unverified training memory). Tier discrimination matters because the
adversary prompt's grounding contract requires T1/T2 evidence and
explicitly forbids T3 — citing training memory as if it were verified
evidence is the failure mode that C42_PAPER_DESIGN_VS_GROUND_TRUTH
catches. Per arxiv:2503.17365 (T1) 4-outcome taxonomy, small-LM
critics flagged as "amplification" almost always failed to
distinguish T3 training memory from T1 cited evidence. The enum
makes that discrimination first-class.

Closure ships SourceTierCitation in adversary.rs as an internal
prompt-construction type. It is NOT carried in SSB rows so no
SCHEMA bump is required. 4 variants: T1Arxiv { id, title } / T1Nature
{ doi, title } / T2Web { url, title } / T3Training. Helper methods:
tier_label() + is_load_bearing() (T1+T2 = true; T3 = false). 3 unit
tests landed in adversary.rs tests module + 1 contract test in
crates/contracts/tests/adversary_contract.rs locks 4-variant
exhaustiveness.

DECISION-5 — Production dispatch wired via ELAI_ADVERSARY_ENABLE env
gate
Before closure: GroundingMode::FalsifiabilityCrossFamily existed as a
variant but had ZERO production dispatch to run_adversarial_pass
(grep "GroundingMode::FalsifiabilityCrossFamily =>" returned only
the mode_routing_runner.rs:165 count test and the
grounding_mode.rs:81 label). The mode was identified at the routing
layer but unused at the dispatch layer.

After closure: task_runner::dispatch_adversarial_pass is the new
post-task hook called once per completed task by drive_task. The hook
follows the D-10 / D-11 / D-12 dispatch-hook precedent exactly:

  - Read resolved.adversary_runtime (Option<Arc<AdversaryRuntimeState>>).
    None = no-op + debug trace; closure default is None so the wire is
    "in place but dark" until the construction site lands in the
    sidecar/CLI boot path.
  - Check AdversaryRuntimeConfig::is_enabled() (reads
    ELAI_ADVERSARY_ENABLE). Unset = no-op + debug trace.
  - Resolve worker family via ModelFamily::from_model_id on
    resolved.backend_config.model.
  - Run runtime_config.pair_is_allowed(worker_label,
    adversary_label) guard. Refused = no-op + debug trace.
  - Invoke run_adversarial_pass with the configured adversary +
    config + synthesised verifier-family pool +
    TaskRunnerConsensusEmitter (the SSB seam already in use by
    dispatch_verifiers per D-3). The producer emits exactly 2 SSB
    rows per call (AdversaryGenerated + ConsensusReevaluatedWithCounter).

The hook does NOT propagate adversary errors upward (the
adversarial-pass is a quality signal, not a gate on task completion).
Errors are traced at warn level; the SSB row pair is the durable
signal the auditor reads.

DECISION-6 — ModelFamily::from_provider_kind + from_model_id bridges
added
Before closure: cross-family check relied on caller-asserted family
(MockAdversary::with_counter(ModelFamily::Anthropic, ...) with no
runtime check that the underlying adapter actually emits from
Anthropic). This was GAP-6 in the exploration report (MEDIUM
severity).

After closure: adversary.rs grows two pure-fn bridges next to
ModelFamily::as_str:
  - from_provider_kind(ProviderKind) -> ModelFamily — total mapping
    for the 4 production providers (Anthropic, OpenAi, Xai
    [-> Other("xai")], Ollama [-> Local]; Vllm under #[cfg(feature
    = "vllm")] also maps to Local).
  - from_model_id(&str) -> ModelFamily — substring classifier
    mirroring the D-2 PpmAdapter _PPM_MODEL_HINTS approach.
    Case-insensitive; first match wins; unknown id defaults to
    Local (conservative — same-family pair trips
    SameFamilyPairing observably rather than silently dispatching).

11 unit tests landed in adversary.rs covering all 4 ProviderKind
arms + 7 model_id substring cases (claude / gpt / grok / deepseek /
llama / mistral / mixtral / gemma / qwen / unknown / empty). 2
contract tests in adversary_contract.rs lock the bridge surface.

DECISION-7 — adversary.md agent prompt grounds in 3 D-13 papers
Author at .claude/agents/adversary.md. The agent is registered as a
production agent in the canonical agent list (adversary.md is a new
entry alongside auditor / context-compiler / memory-writer / etc).
Runs on sonnet (cross-family from local Qwen2.5-Coder-7B). System
message includes 6 hard constraints:
  H-1 Cross-family check (refuses claude-family worker)
  H-2 Strongest plausible critic (no hedging)
  H-3 Structured output (single JSON object, no markdown wrapper)
  H-4 SourceTierCitation enforcement (no T3 for load-bearing claims)
  H-5 4-outcome self-classification per arxiv:2503.17365 (T1)
  H-6 No self-citation loop (refuses if claim cites sonnet itself)

Output schema mirrors CounterEvidenceSchema + adds
source_tier_citations: Vec<SourceTierCitation> + self_assessment field.
Three worked examples: robust claim / falsified claim / same-family
refusal.

DECISION-8 — Paper finding labels per C43_THIRD_BAD_CITE_PATTERN
discipline
Per CLAUDE.md Grounded-by-default rule, every cited paper has its
finding-strength labeled:

  - Self-Correction Bench (arxiv:2507.02778, T1): the 64.5%
    blind-spot rate is DIRECT EXPERIMENTAL EVIDENCE from the paper's
    measured data. Label = "negative-evidence grounding". This is the
    strongest paper finding type — measured rate from the cited
    benchmark.

  - Constitution or Collapse (arxiv:2504.04918, T1): the 8B
    small-model collapse outcome IS direct experimental evidence in
    the paper's experimental section. The "cross-family GPT-4o
    critic breaks the loop" finding is direct evidence too. The
    extrapolation from "GPT-4o critic works against 8B Llama-2" to
    "sonnet critic works against 7B Qwen2.5-Coder" is design
    extrapolation, NOT directly proved. Label = "design extrapolation
    grounded in experimental evidence on adjacent model pair".

  - Constitutional AI in Small LLMs (arxiv:2503.17365, T1): the
    4-outcome taxonomy (detect_refuse / detect_warn / non_detection
    / amplification) is a paper-defined classification, NOT a
    measured rate. Label = "design extrapolation" — the taxonomy
    is the paper's contribution; the assumption that classifying
    self-assessment against this taxonomy reduces amplification rate
    is ELAI design extrapolation.

  - Lightweight Latent Verifiers (arxiv:2504.16760, T1) and
    rStar-Math (arxiv:2501.04519, T1) are D-2 reuses; their
    finding labels are documented in workflow/reasoning/D-2-decisions.md.

NO new paper-ingestor runs in this closure — all 5 papers were
already ingested in the substrate landing wave (Glob verified
present at workflow/papers/). No arXiv id is cited that has not
been ingested (paper-ingestor remains the SOLE PRODUCER of
arXiv: tokens per C43_THIRD_BAD_CITE_PATTERN).

---

DECISION-9 — VerificationResult.adversary field NOT added
Plan task 7 ("Extend VerificationResult struct: + adversary:
Option<CounterEvidence>") would require adding a VerifierKind::Adversary
5th variant + cascade to the aggregator + ALL PRMBench doc-comments + 4
verifier-pair tests + tests/adversary_integration.rs structure. The
shipped substrate handles the adversary signal at a DIFFERENT layer:
ConsensusRole::Adversary tag at the run_adversarial_pass call site +
the 2-SSB-row pair (AdversaryGenerated + ConsensusReevaluatedWithCounter)
emitted at the call site. The two-layer separation is cleaner than
inlining adversary onto VerificationResult (per CLAUDE.md "shipped
wins" + the D-12 DECISION-4 code-placement precedent). Closure
preserves the divergence.

DECISION-10 — SignalKind enum NOT created
Plan task 7 refers to a "SignalKind enum" but grep across the
codebase shows SignalKind exists only as a doc-comment string in
verification_result.rs:11 — there is no Rust enum by that name. The
actual closed enum is VerifierKind (Ppm / Intrinsic / Deterministic /
CrossContext — 4 arms, locked by 4 verifier-pair tests in adversary.rs
at lines 438-471). Creating a new SignalKind enum just to satisfy the
plan token-match would be cosmetic + would invite drift between
VerifierKind (the real enum) and SignalKind (the new shadow enum).
Closure does NOT create SignalKind; the doc-comment vs enum naming
gap is documented here.

DECISION-11 — EvidencePoint type NOT added
Plan task 3 specifies "EvidencePoint { kind: EvidenceKind, content:
BoundedString<2048>, citation_ref: Option<SourceTierCitation> }". The
shipped EvidenceRef from claim.rs:97 already carries this shape:
(kind: EvidenceKind, content via from_content, ref string). Adding
EvidencePoint would duplicate the existing type with a slightly
different name. SourceTierCitation (the genuinely-missing new type)
DOES land per DECISION-4; the rest of EvidencePoint is already
shipped under the name EvidenceRef.

DECISION-12 — CounterVerdict enum NOT added
Plan task 3 specifies "CounterVerdict { ClaimRobust, ClaimWeak,
ClaimFalse }". The shipped CounterEvidenceSchema uses
severity: f64 scalar [0.0, 1.0] which covers the trichotomy via
threshold (the AdversaryConfig.severity_threshold default 0.5
partitions: < 0.5 = robust; >= 0.5 = invalidating). Adding the enum
would be plan-token-match cosmetic — the scalar already encodes
finer gradation (a CounterEvidence at severity 0.6 is meaningfully
weaker than one at 0.95) which the enum collapses. Closure preserves
the scalar.

---

CLOSURE FILES TOUCHED

Modified:
  - .elai_cc/crates/orchestrator/src/adversary.rs
      + ModelFamily::from_provider_kind + ModelFamily::from_model_id
      + SourceTierCitation enum + tier_label + is_load_bearing
      + 14 new unit tests (11 bridge + 3 SourceTierCitation)
  - .elai_cc/crates/orchestrator/src/runtime_config.rs
      + ELAI_ADVERSARY_ENABLE_ENV const
      + AdversaryRuntimeConfig (enable / min_family_distance /
        max_disagreement_score_delta / preferred_cross_family_pairs)
      + AdversaryRuntimeState (adversary Arc + adversary_config +
        runtime_config)
      + 8 new unit tests (env-var token + defaults + toml round-trip
        + pair_is_allowed partitioning + truthy/falsy)
  - .elai_cc/crates/orchestrator/src/task_runner.rs
      + ResolvedConfig.adversary_runtime field (None default)
      + dispatch_adversarial_pass async hook + drive_task call site
  - .elai_cc/elai_cc.toml
      + [adversary] section (L3-locked, commented defaults)

Authored:
  - .claude/agents/adversary.md (production agent definition + 6 hard
    constraints + output schema + 3 worked examples)
  - .elai_cc/crates/orchestrator/tests/adversary_integration.rs
    (4 scenarios A/B/C/D + SourceTierCitation bonus)
  - .elai_cc/crates/contracts/tests/adversary_contract.rs (10 contract
    tests covering SCHEMA floor + ModelFamily serde + ConsensusRole
    serde + SourceTierCitation 4-variant exhaustive + from_provider_kind
    bridge + from_model_id classifier + AdversaryGenerated round-trip
    + ConsensusReevaluatedWithCounter round-trip +
    heterogeneous_verifier_check + CounterEvidenceSchema round-trip)
  - workflow/reasoning/D-13-decisions.md (this file)

Preserved (out of closure scope):
  - ConsensusRole + ModelFamily + AdversaryConfig + CounterEvidenceSchema
    + AdversaryError + Adversary trait + DynAdversary + AdversaryErased
    + MockAdversary + heterogeneous_verifier_check (shipped 2026-05-20)
  - AdversaryGenerated + ConsensusReevaluatedWithCounter SSB variants
    + payloads + dispatch arms (shipped 2026-05-20)
  - run_adversarial_pass + AdversarialRunSummary +
    gate_with_adversarial_check + recent_adversarial_runs +
    adversary_config_snapshot (shipped 2026-05-20)
  - commands_d13.rs (get_adversarial_runs_recent +
    get_adversary_config) + AdversaryPanel.svelte (shipped 2026-05-20)
  - 4 sibling adversarial_*.rs integration tests (pass_smoke +
    research + ipc + state_space — shipped 2026-05-20; this closure
    adds the 5th, adversary_integration.rs)

---

PRE-COMMIT VERIFICATION CHECKLIST

  - cargo check --workspace --lib  -> expected clean (no new compile
    errors; new types are additive)
  - cargo check --tests -p elai_orchestrator  -> expected clean
  - cargo check --tests -p elai_cc_contracts --features mcp  -> expected
    clean
  - rg "SourceTierCitation" .elai_cc/crates/orchestrator/src/adversary.rs
    -> non-empty
  - rg "from_provider_kind"
    .elai_cc/crates/orchestrator/src/adversary.rs  -> non-empty
  - rg "ELAI_ADVERSARY_ENABLE"
    .elai_cc/crates/orchestrator/src/runtime_config.rs  -> non-empty
  - rg "dispatch_adversarial_pass|run_adversarial_pass"
    .elai_cc/crates/orchestrator/src/task_runner.rs  -> non-empty
  - test -f F:/.ELAI_workflow/.claude/agents/adversary.md  -> exists
  - rg "^\[adversary\]" .elai_cc/elai_cc.toml  -> finds the commented
    section header

SCHEMA invariants:
  - SCHEMA_VERSION still 122 (preserved)
  - TOTAL_VARIANTS still 343 (preserved)
  - action_type_lock.golden NOT touched (preserved)
  - No new SSB variants added
  - No new ConsensusRole / ModelFamily / VerifierKind / CheckerKind
    variants
