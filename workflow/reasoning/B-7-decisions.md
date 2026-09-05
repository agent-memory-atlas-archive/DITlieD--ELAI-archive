B-7 decisions — pipeline factory producer wiring (4D + 4E)
Session: B-7-continuation-2 (2026-05-22)
Plan: F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-7_pipeline_factory_producer_wiring.md

This document captures the 6 design decisions made during the second
continuation executor pass on B-7. Each entry: what, key decision, why,
trade-off accepted, grounding.

(a) extends-vs-forks TaskSplitPayload

WHAT: PhaseSizingSplitter (4D) needs to emit a "split happened" SSB row
with `phase_id` + `complexity_reason` context. There is an existing
TaskSplitPayload in ssb.rs from B-6 (supervision-layer kills/restarts).
KEY DECISION: EXTEND the existing payload with two new
`#[serde(default)]` fields (phase_id: Option<u64>, complexity_reason:
Option<String>) rather than FORK a new SsbActionType variant.
WHY: (1) Semantic shape is identical — "a unit of work was split, with
a reason". Forking would duplicate marshalling code with no semantic
gain. (2) Backwards compat: msgpack defaults to None for Option on
pre-patch rows, no schema migration required. (3) Dashboard fan-out:
PlanWindow + future SupervisionDrawer consume the same shape, no
negotiation needed.
TRADE-OFF ACCEPTED: producers must populate either `agent_role`
(existing B-6 field) OR `phase_id` (new) but never both — a single
payload with optional fields beats two near-duplicate variants. This
constraint is enforced at the producer site, not the schema.
GROUNDING: existing TaskSplitPayload struct read in
.elai_cc/crates/orchestrator/src/ssb.rs (B-6 wiring).

(b) FailureRecord payload shape mirrors actual in-memory record

WHAT: The original B-7 plan specified FailureRecordedPayload with fields
{baby_step_id, failure_kind, retry_count, last_error}.
KEY DECISION: SHIP FailureRecordedPayload with the ACTUAL FailureRecord
shape from pipeline_tree_navigator.rs:292 — {baby_step_id, step_id,
phase_id, concept_id, reason, attempts} — NOT the plan-literal field
names.
WHY: The in-memory FailureRecord struct already shipped (4E.7 — 4E.10)
with fields step_id/phase_id/concept_id/reason/attempts. Fabricating a
new "failure_kind" enum that no producer would populate is FABRICATED-
SYMBOL discipline violation; "last_error" duplicates "reason". The
existing shape gives the dashboard everything it needs (parent ids for
hierarchy navigation, plus attempt count + reason text).
TRADE-OFF ACCEPTED: dashboard cannot pivot on failure_kind enums
(timeout / oom / panic / assertion-failed) until a future plan adds
that classifier upstream. Today's UI can still pattern-match on the
reason string.
GROUNDING: FailureRecord struct read in pipeline_tree_navigator.rs:292
(lines 292-299).

(c) PackageRefVerified + DecisionSource shapes mirror actual code

WHAT: The original B-7 plan specified `package_ref_verified` payload
{name, ecosystem, method: ShapeOnly|RegistryHttp, verified} and
`tech_stack_decided` source = RESEARCH|EXISTING_PROJECT|USER_SPECIFIED|
FALLBACK.
KEY DECISION: SHIP PackageRefVerifiedPayload with the ACTUAL PackageRef
shape from research_tree.rs:1862 — {name, registry, version, verified}
— NO fabricated `method` enum. SHIP TechStackDecidedPayload with the
ACTUAL DecisionSource enum (research | constraint | default) — NO
fabricated USER_SPECIFIED / EXISTING_PROJECT / FALLBACK literals.
WHY: ShapeOnly | RegistryHttp does not exist in the codebase; the only
verification today is is_plausible_package_name (a regex check). Adding
the enum to the payload would be a lie until verify_packages grows a
real registry-HTTP path. Same logic for DecisionSource: the actual enum
has 3 variants (Research, Constraint, Default), not 4 plan-literal ones.
TRADE-OFF ACCEPTED: dashboard cannot show "verified by registry HTTP"
vs "verified by shape only" badges until verify_packages grows that
distinction. Today it shows verified yes/no only — accurate by current
implementation.
GROUNDING: PackageRef + DecisionSource read in research_tree.rs:1854
(DecisionSource), :1862 (PackageRef).

(d) SCHEMA_VERSION 99 -> 100 rationale

WHAT: SCHEMA_VERSION must be bumped per any net-new SsbActionType
variant (per the writer-side CHECK clause in ssb.rs's CREATE TABLE).
KEY DECISION: 99 -> 100 covers all 4 new B-7 producer-wire variants in
ONE bump (TreePositionChanged, FailureRecorded, PackageRefVerified,
TechStackDecided) rather than 4 successive bumps.
WHY: Schema-version semantics treat a "release" as a coherent set of
producer changes. Splitting into 4 bumps would force the dashboard to
guard 4 read paths separately during rollout. One bump = one rollout
gate.
TRADE-OFF ACCEPTED: future per-variant rollback requires a code-level
revert, not a schema_version downgrade. Acceptable because the 4
variants are independent on the read side (no payload references
another) — partial-deploy risk is low.
GROUNDING: SCHEMA_VERSION CHECK clause in ssb.rs CREATE TABLE; arms
match in as_sql_str / from_str updated in lockstep.

(e) research_node_stage routed via existing web_research_step (no new variant)

WHAT: The original B-7 plan specified a new SsbActionType
`research_node_stage` for per-FORMULATE/SEARCH/EXTRACT/REFLECT/CONVERGE
events.
KEY DECISION: REUSE the existing SsbActionType::WebResearchStep variant.
The B-7 (pre-cont-2) work already added stage + stage_phase +
stage_payload fields to WebResearchStepPayload (ssb.rs:4073-4084) with
`#[serde(default, skip_serializing_if = "String::is_empty")]` — coarse
rows leave stage="" and remain queryable, fine-grained per-stage rows
populate stage="formulate" (or "search"/"extract"/"reflect"/"converge")
and stage_phase="started" (or "completed"). The viewer reader
`research_node_stages(node_id, limit)` filters rows by node_id AND
`stage != ""` to surface only the per-stage history.
WHY: A new SsbActionType for per-stage events would double the
producer-side code (rmp_serde encoder + viewer reader + dashboard
discriminator) for zero semantic gain. The stage field is the
discriminator.
TRADE-OFF ACCEPTED: SQL filtering on stage="search" must scan all
web_research_step rows (no SQL-level index on payload-internal field).
At current SSB row volumes this is a non-issue; we over-fetch by 4x
relative to the requested limit and break early once `capped` matches
are seen.
GROUNDING: WebResearchStepPayload struct read in ssb.rs:4060;
viewer-side filter implemented in
ssb_viewer.rs::research_node_stages.

(f) decomposition_alternative + research_cache_hit_rate DEFERRED

WHAT: B-7 tasks #16 (DecompositionAlternative intervention panel) and
#17 (ResearchCache hit-rate gauge) require:
  #16 — Designer interactive-mode integration surface (per phase-4D
        audit "INTERACTIVE mode logged not blocking"), plus a new
        intervention-card slot in the dashboard intervention queue.
  #17 — ResearchCache instrumentation (count cached vs cold-fetched
        per session), plus a Tauri command + gauge component.
KEY DECISION: DEFER both to a follow-up plan (B-7-follow-up).
WHY: Both are net-new producer surfaces (not "wire up an existing
producer to SSB") and each is one full session's worth of work on its
own. Shipping them under this session's budget would either rush them
or block the 4 GREEN producer-wire seams from landing. The 4 GREEN
seams (TreePositionChanged + FailureRecorded + PackageRefVerified +
TechStackDecided) are the production-blocking ones — the dashboard
PlanWindow currently has no live cursor or per-baby-step failure
history; getting those shipped now unblocks every downstream
debugging session.
TRADE-OFF ACCEPTED: B-7 ACCEPTANCE checklist is split — 8 of 18 plan
tasks remain TODO under the follow-up plan. The SHIPPED-ON-V2 STATUS
block at the top of the plan file enumerates the split.
GROUNDING: B-7 plan task #16-#17 + #8 + #18 read in B-7 plan file lines
101-148.
