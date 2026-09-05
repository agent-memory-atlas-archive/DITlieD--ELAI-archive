D-10 mode-routing predicate — closure decisions
================================================

Session: s-25120 (completion-drive cluster, slot 3/4)
Plan: F:/.ELAI_workflow/docs/thefinalglance/D-plan/D-10_mode_routing_predicate.md
Prior recheck verdict: PARTIAL (2 partial + 8 missing + 2 not-wired + 0 drift)
Closure verdict: GREEN (6/6 GAPs closed; substrate preserved at SCHEMA 77, head 117)

This doc records the 8 binding decisions the closure pass made over
the plan-text, the recheck-report, and the substrate as it stood at
the start of the session. Every decision points back to its evidence
source so a future session can verify the reasoning is grounded.

------------------------------------------------------------------
DECISION-1 — NO SCHEMA bump (preserve head 117).
------------------------------------------------------------------

The plan text targets SCHEMA_VERSION 55->56 with two new SSB variants
(ModeSelected + ClaimDispatched). The substrate shipped the D-10
SSB plumbing as a single consolidated `ClaimRouted` variant at
SCHEMA 77 long before this closure pass; head SCHEMA is now 117.

Per the [SCHEMA-actual vs plan-stale] rule (septuple-confirmed across
D-2..D-8), the plan-blurb is stale and MUST NOT drive a bump. The
closure adds zero new SSB variants and therefore no schema bump is
warranted. The contract test pins the floor at >=77 (the D-10
substrate's actual ship version) so any future SSB-touching change
must reaffirm the existing ClaimRouted contract rather than re-bump.

Grounding:
  * .elai_cc/crates/orchestrator/src/ssb.rs:1298 (`SCHEMA_VERSION: i64 = 117`)
  * .elai_cc/crates/contracts/tests/d10_mode_routing_contract.rs:28
    (D10_SCHEMA_FLOOR = 77)
  * workflow/reasoning/D-3-decisions.md (DECISION-1, SCHEMA-actual
    vs plan-stale precedent).

------------------------------------------------------------------
DECISION-2 — Preserve shipped ClaimRouted variant; reject plan's
ModeSelected + ClaimDispatched 2-variant split.
------------------------------------------------------------------

The plan asks for two SSB variants (ModeSelected fired pre-dispatch,
ClaimDispatched fired post-dispatch). The substrate consolidated both
into a single ClaimRouted variant carrying
{claim_id, claim_type, mode, ts_ms, ctx_summary_blake3}. The single
variant is RICHER than the plan's split because it includes the
context summary blake3 hash that lets dashboards detect routing
context drift across emissions of the "same" claim.

Splitting now would shatter: commands_d10 (Tauri read surface),
commands_d27 (composite mode+refusal panel), ModeRoutingPanel.svelte,
d10_mode_routing_contract.rs (4 round-trip tests pinned on
ClaimRouted), action_type_lock.golden (333-variant TOTAL_VARIANTS),
and the D-21 ground-truth label set. Per D-9 DECISION-2 pattern
(preserve shipped SSB variant names when they functionally subsume
the plan-text), keep ClaimRouted and document the divergence here.

Grounding:
  * .elai_cc/crates/orchestrator/src/ssb.rs (ClaimRoutedPayload at
    line 11541; SsbActionType::ClaimRouted at line 3197).
  * .elai_cc/crates/orchestrator/src/agent_pool.rs:1990-2020
    (route_claim emits one ClaimRouted row per call).
  * workflow/reasoning/D-9-decisions.md (DECISION-2 preserve shipped
    SSB variant names precedent).

------------------------------------------------------------------
DECISION-3 — Mode design is fn+enum, NOT trait/struct.
------------------------------------------------------------------

The substrate exposes `pub fn select_grounding_mode(claim, ctx) ->
GroundingMode` as a free function over closed enums; no ModeRouter
trait, no router struct, no async surface. The plan-text §3.2
predicate is a pure function and matches this shape exactly. Routes
are deterministic, no I/O, no LLM, no clock — the entire surface
fits in 200 LOC and is testable without fixtures.

A trait+struct refactor would add no behaviour, would force every
caller to thread a router instance, and would invite the "agent
extends the trait" failure mode that D-9 L3 locks prevent. Per the
D-5/D-9 fn+enum precedent (free `compute_tau` + closed enum
RefusalDecision), keep fn+enum.

Grounding:
  * .elai_cc/crates/orchestrator/src/grounding_mode.rs:110 (free fn).
  * .elai_cc/crates/orchestrator/src/conformal_calibration.rs (D-16
    free fn `compute_tau` precedent).
  * workflow/reasoning/D-5-decisions.md (DECISION-1 free-fn pattern).

------------------------------------------------------------------
DECISION-4 — arxiv:2507.02778 (Self-Correction Bench) is SUPPORTING
evidence, NOT algorithmic ground. No paper-ingestor required.
------------------------------------------------------------------

The plan §10b cites arxiv:2507.02778 (64.5% blind-spot rate on
same-model critique) as rationale for "Mode E (Refuse) over Mode B
(cross-family) when no alt-family is registered". The arXiv ID has a
YYMM prefix of 2507 which is borderline post-cutoff; the citation is
unverified and the closure pass treats it as SUPPORTING evidence per
the D-7 NanoFlux pattern and D-8 reward-hacking pattern (cite as
context, do not let it drive code shape).

In code + decisions doc, the citation is tagged "(supporting
evidence; not implementation-specifying)" so future readers know the
choice is ELAI-policy + Cohen Boxed Myopic + Everitt current-RF
grounded, not paper-grounded. Paper-ingestor is NOT spawned because
no in-code algorithm depends on the paper.

Grounding:
  * docs/thefinalglance/D-plan/D-10_mode_routing_predicate.md:172
    (the cite in question).
  * workflow/reasoning/D-7-decisions.md (DECISION-2 NanoFlux policy:
    cite as motivation, do not paper-ground).
  * .claude/rules/anti-stale-training section + grounded-by-default
    rule.

------------------------------------------------------------------
DECISION-5 — cohen-boxed-myopic file path canonical name is
"cohen-boxed-myopic-interface.md" (plan referenced "aimag43-3" suffix
which is stale/incorrect).
------------------------------------------------------------------

Plan §"Reads" lists workflow/papers/cohen-boxed-myopic-aimag43-3-
interface.md. The file does not exist at that path — the ingested
canonical file from the D-9 paper-ingestor pass is
workflow/papers/cohen-boxed-myopic-interface.md (Cohen BoMAI =
arXiv:1905.12186, NOT 1908.04734=Everitt). The "aimag43-3" suffix
appears to be a venue+section misremembering from the plan author;
no such file ever shipped.

All citation strings in shipped code (grounding_mode.rs module doc,
this decisions doc, the integration test header) use the canonical
filename. Future plan authors should consult this entry before
re-introducing the long suffix.

Grounding:
  * workflow/papers/cohen-boxed-myopic-interface.md (the actually
    ingested file; D-9 paper-ingestor pass).
  * workflow/reasoning/D-9-decisions.md (PAPER_CITATION_ID_SWAP
    pitfall precedent).

------------------------------------------------------------------
DECISION-6 — GroundingMode + ClaimType variant names shipped differ
slightly from plan-text; SHIPPED NAMES PRESERVED for cross-cutting
stability.
------------------------------------------------------------------

Shipped GroundingMode 5-variant enum:
  Constructive | FalsifiabilityCrossFamily | ConformalRefusal
  | OperationalDeferred | ConservativeFallback

Plan-text 5-variant enum:
  Constructive | FalsifiabilityCrossFamily | OperationalFalsification
  | DiscoveryLabelOnly | Refuse

Divergence:
  * "Refuse" was SPLIT into ConformalRefusal (D-16 calibration ready,
    formal UniCR bound) and ConservativeFallback (no calibration,
    orchestrator-policy abstain). The split is a refinement, not a
    drift — same Mode E intent, two grounding regimes.
  * "OperationalFalsification" was renamed OperationalDeferred to
    reflect that the sandbox runner (D-18 ObservableEffect) is not
    yet plumbed; the mode reserves the slot rather than executing.
  * "DiscoveryLabelOnly" was DROPPED from the runtime enum because
    per real-problem §1 Mode C is a LABEL applied post-hoc to A/D
    results, not a runtime mode. The plan itself notes this.

Shipped ClaimType 7-variant enum:
  Factual | Procedural | Normative | Speculative | Hypothetical
  | Computational | MetaQuery

Plan-text 7-variant enum:
  Code | Math | Prose | Empirical | Definitional | Counterfactual
  | EthicalAesthetic

The shipped vocabulary is grounded in real-problem-solving §1
taxonomy + the orchestrator's actual emission surface (claims about
facts, procedures, normative judgements, etc.). The plan vocabulary
is domain-shape-keyed (programming-language style). Renaming would
shatter 8+ consumer files (adversary.rs, agent_pool.rs x3,
baby_step.rs, benchmarks/arc_agi2_runner.rs,
benchmarks/mode_routing_runner.rs, checker_registry.rs,
grounding_mode.rs) + D-21 ground-truth labels + on-disk SSB rows.

Per D-9 DECISION-2 (PLAN-VOCAB-vs-SHIPPED-NAMING preservation), keep
shipped names.

Grounding:
  * .elai_cc/crates/orchestrator/src/grounding_mode.rs:57-72.
  * .elai_cc/crates/orchestrator/src/claim.rs:35-43.
  * docs/specs/guide/real-problem-solving-grounded-2026-05-19.md §1.
  * workflow/reasoning/D-9-decisions.md (DECISION-2 precedent).

------------------------------------------------------------------
DECISION-7 — L3 lock activation for select_grounding_mode +
ClaimType IS D-10's responsibility (bootstrap registry becomes
non-empty starting with D-10).
------------------------------------------------------------------

D-9 DECISION-5 set the registry default to EMPTY so the D-9
orchestrate run itself could not trip the hook on its own commits.
D-10 is the legitimate first-mode use case to populate the registry:
the mode-routing predicate IS the function the agent must never
edit per Cohen Boxed Myopic + Everitt current-RF — locking it is
the entire point of D-10.

The closure activates two entries (uncommented):
  * crates/orchestrator/src/grounding_mode.rs (full file)
  * crates/orchestrator/src/claim.rs (full file)

The D-10 integration test (locked_keys_d10_integration.rs) writes a
self-contained registry with these two paths and asserts that a
synthetic edit fires LockedKeysViolationDetected. The live registry
at scripts/locked_keys_registry.toml carries the same entries so the
pre-commit hook gates real edits.

Grounding:
  * .elai_cc/scripts/locked_keys_registry.toml (entries activated
    by this closure).
  * workflow/reasoning/D-9-decisions.md (DECISION-5 BOOTSTRAP-EMPTY-
    REGISTRY precedent; D-10 is the documented exception).
  * arxiv:1908.04734 §3 (Everitt current-RF).

------------------------------------------------------------------
DECISION-8 — TS-side enum mirror is hand-authored (ts-rs not adopted).
------------------------------------------------------------------

GroundingMode + ClaimType + ClaimRoutedRow + RecentClaimRoutings TS
types are hand-authored in
crates/dashboard-v2/src/lib/api/types.ts with a comment block
warning future readers to regenerate from Rust source if ts-rs is
ever adopted. Per D-5 DECISION-7 + D-6 DECISION-8, ts-rs has been
considered + rejected across multiple D-N phases; introducing it
just for D-10 would create drift.

ModeRoutingPanel.svelte now imports GROUNDING_MODES +
GROUNDING_MODE_LABELS + ClaimRoutedRow + RecentClaimRoutings from
the shared module rather than inlining its own copy. commands_d10
+ commands_d27 still hold their own server-side seed list because
the seed is in Rust (BTreeMap construction in commands_d10.rs:87 +
commands_d27.rs:114); the TS side is the consumer.

Grounding:
  * .elai_cc/crates/dashboard-v2/src/lib/api/types.ts (the mirror).
  * .elai_cc/crates/dashboard-v2/src/lib/drawers/ModeRoutingPanel.svelte
    (the consumer).
  * workflow/reasoning/D-5-decisions.md (DECISION-7 ts-rs rejection).
  * workflow/reasoning/D-6-decisions.md (DECISION-8 ts-rs rejection
    re-affirmation).

------------------------------------------------------------------
Closure verdict
------------------------------------------------------------------

6/6 GAPs closed:
  * GAP-1 task_runner.rs claim-emit hook: dispatch_claim_routing
    wired into drive_task post-bai (mirrors D-3 / D-5 pattern).
  * GAP-2 L3 lock activation: select_grounding_mode + ClaimType
    entries uncommented + activated in locked_keys_registry.toml.
  * GAP-3 TS-side enum mirror: GroundingMode + ClaimType +
    ClaimRoutedRow + RecentClaimRoutings exported from api/types.ts;
    ModeRoutingPanel.svelte switched to imports.
  * GAP-4 Decisions doc: this file.
  * GAP-5 Contract tests: d10_mode_routing_contract.rs already
    pins SCHEMA floor >= 77, ClaimRouted round-trip (msgpack+JSON),
    GroundingMode closed-enum + serde stable, select_grounding_mode
    determinism (5 rows). No additional tests required.
  * GAP-6 locked_keys integration: new
    crates/orchestrator/tests/locked_keys_d10_integration.rs asserts
    a synthetic edit to grounding_mode.rs fires
    LockedKeysViolationDetected (D-9 auditor + D-10 lock entry).

ZERO new stubs. ZERO new dependencies. ZERO SCHEMA bumps. ZERO
unwrap() in production. blake3 only. async-fn-in-trait native (no
async-trait crate). Mirrors D-9 closure-pass discipline.
