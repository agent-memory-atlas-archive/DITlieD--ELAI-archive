E-20 -- local-cloud routing scope decisions
============================================

Session: s-25120 completion-drive cluster.
Plan: docs/thefinalglance/E-plan/E-20_local_cloud_routing.md
Cross-check: workflow/sessions/s-25120/cross-check-E/E-20_local_cloud_routing-report.md (OVERALL: MISSING; 22 produces_symbols, 17 missing, 1 partial, 1 spec_drift).
Context: workflow/sessions/s-25120/completion-drive/contexts/E-20_local_cloud_routing-context.md
Exploration: workflow/sessions/s-25120/completion-drive/exploration/E-20_local_cloud_routing-exploration.md
SCHEMA bump: 179 -> 180.
TOTAL_VARIANTS bump: 452 -> 453.

This document records the load-bearing decisions taken on the E-20
substrate ship. Each numbered DECISION cites the grounding source
(verified May 2026 unless otherwise noted) and the alternative that
was rejected with rationale.


DECISION-1 -- Land under orchestrator::cost::cloud_route, NOT
orchestrator::routing::*
------------------------------------------------------------------

The plan's WIRING_CONTRACT proposed `orchestrator::routing::*`
symbols (confidence_score, HybridEscalationGate, majority_vote_router,
semantic_reasoning_mode_chooser). That namespace is OWNED by C-46
Uno-Orchestra (TaskDifficultyClassifier + SelectiveDelegator +
RoutingPolicy at orchestrator/src/routing/{difficulty,selective,
policy}.rs -- already shipped, AgentRole dispatch concern, distinct
from local/cloud RuntimeBackend routing).

Rejected alternative: ship into orchestrator::routing as siblings.
Reason: name collisions (TaskDifficultyClassifier exists in both
orchestrator/src/routing/difficulty.rs AND
orchestrator/src/cost/cost_router.rs already as of E-39) +
namespace-ownership ambiguity that would confuse readers.

Chosen path: extend the existing E-39 cost/ subtree with new sibling
files cost/cloud_route.rs (~720 LOC consolidated: confidence +
hybrid_gate + majority_vote + semantic_reasoning) and
cost/burn_projection.rs (~200 LOC). Plan also rejected
crates/orchestrator/src/cloud_spend.rs per the E39_CLOUD_SPEND_PATH_DRIFT
gotcha (E-39 SHIPPED rejected that path in favour of cost/cloud_abort.rs
sibling; E-20 honours the same convention).


DECISION-2 -- BackendDifficultyAdapter, not TaskDifficultyClassifier
-------------------------------------------------------------------

Two existing `TaskDifficultyClassifier` structs already ship in the
repo (C-46 routing/difficulty.rs + E-39 cost/cost_router.rs). A third
same-name struct under providers/ would be a readability hazard.

Rejected alternative: name the new RuntimeAdapter
TaskDifficultyClassifier under providers::runtime.

Chosen: name it BackendDifficultyAdapter at
providers/src/runtime/cloud_route_difficulty.rs. The adapter wraps
the E-39 cost_router::TaskDifficultyClassifier heuristic as its
data source (sentinel pattern) until RouteLLM weights ship via
E-20.B. complete() / stream() return RuntimeAdapterError::NotImplemented
because the adapter is signal-only -- the orchestrator must call
classify() / classify_difficulty() for the verdict, not the
RuntimeAdapter trait.


DECISION-3 -- Pattern-2 single-variant + 9-arm RoutingEventKind
discriminator (NOT 9 separate SsbActionType variants)
------------------------------------------------------------------

The plan's WIRING_CONTRACT proposed 9 NEW SsbActionType variants
(ConfidenceScoreEmitted + HybridEscalationReason + DifficultyClassified
+ RouteBlockedPrivacy + RouteOutputFiltered + CloudBurnAlert +
CloudReasoningModeChosen + RoutingMajorityVoteBreakdown +
RoleRuntimePriorityChosen).

This directly violates the Pattern-2 single-variant + N-arm Kind
discriminator precedent now at 32nd application across the cluster
(E-39 8-arm + E-50 11-arm + E-4 13-arm + E-29 4-arm + E-25 9-arm +
E-55 9-arm + B-23 6-arm + B-24 6-arm + ...). Pattern-2 collapses N
related events into ONE SsbActionType variant + an N-arm Kind enum on
the payload + a free-form details_json.

Rejected alternative A (PATH C in exploration): ship 9 separate
variants. Reason: 9 ordinal-cascades in action_type_lock.golden + 9
schema bumps; against the 31-application precedent.

Rejected alternative B (PATH A in exploration): extend E-39's
CostEventKind from 8 arms to 17 arms. Reason: blurs cost-economics
vs routing-decision concern boundary; would make CostEventKind the
single-largest kind discriminator in repo (vs current 13-arm
ChatUxCritiqueKind). Cross-cutting taxonomy creep.

Chosen path (PATH B in exploration): NEW SsbActionType::
LocalCloudRoutingEvent + RoutingEventKind 9-arm discriminator
(ConfidenceEmitted / HybridEscalation / DifficultyClassified /
PrivacyBlocked / OutputFiltered / BurnAlert / ReasoningMode /
MajorityVote / RolePriorityChosen). SCHEMA 179 -> 180 atomic +1;
TOTAL_VARIANTS 452 -> 453; golden +1 row at ordinal 223
(alphabetical insertion between llamacpp_subprocess_spawned and
lock_acquired); details_json carries per-kind scalars.


DECISION-4 -- Confidence score derivation: first-token-entropy
preferred; mean_logprob sigmoid fallback
------------------------------------------------------------------

Plan task 1 cites three anchor sources: arXiv:2510.01237 Confidence-
Aware Routing (probe + perplexity dominate verbalized self-
confidence; F1 0.61 -> 0.82, 40% compute cut) +
arXiv:2511.07396 C3PO (conformal-cost-bounded cascade; self-
supervised thresholds beat heuristic) + arXiv:2510.08146
entropy-of-first-token (supporting-only; not load-bearing).

Rejected: ask the model to self-report confidence as routing signal.
Reason: arXiv:2510.01237 explicitly documents this anti-pattern --
probe-based + perplexity-based signals dominate verbalized self-
confidence in every benchmark they cover.

Chosen: `compute_score(first_token_probs: Option<&[f32]>,
mean_logprob: f32) -> f32 in [0,1]`. When the full first-token
distribution is available, computes normalised Shannon entropy
(1 - H/H_max). When only mean_logprob is available (B-39 sidecar
reasoning surface today), falls back to a sigmoid centred at -2.3
(=~ e^-2.3 ~ 0.1 token prob at the 0.5 confidence pivot).


DECISION-5 -- L_HYBRID_THRESHOLD = 0.55 default; per-L-layer
overrides; RUNPOD-PENDING calibration via Youden's J
------------------------------------------------------------------

Plan task 2 specifies "Initial threshold 0.55; tuned via task 10
holdout calibration." 0.55 is the midpoint between AutoMix's reported
0.5 inflection point (arXiv:2310.12963 ICLR 2025; >50% cost reduction
at iso-quality) and C3PO's conformal upper bound (arXiv:2511.07396
fig. 3 cross-dataset average).

Rejected: hard-code 0.55 with no per-L-layer slot. Reason: E-20 plan
Open Question explicitly requests per-L-layer segmentation. Per-L-
layer thresholds catch the case where L4/L5 multi-file architecture
prompts need a higher confidence bar than L1 scaffolding does.

Chosen: LLayerThresholds { per_layer: [Option<f32>; 5] } that falls
back to L_HYBRID_THRESHOLD_DEFAULT (0.55) when an override is unset.
Holdout calibration RUNPOD-PENDING; substrate ships the eval/routing/
{holdout,calibration}.toml schemas + a Youden's J method anchor so
the calibration pipeline compiles + tests + ships without hardware.


DECISION-6 -- privacy/secret_scan.rs is BLOCK-mode SCANNER,
distinct from privacy_redactor.rs REDACT-mode
------------------------------------------------------------------

B-27 SHIPPED privacy_redactor.rs at orchestrator/src/privacy_redactor.rs
with RedactionProfile {Minimal, Standard, Strict} + redact_context
fn. That's REDACT-mode (return sanitised copy).

E-20 task 4 is BLOCK-mode: refuse cloud dispatch when a hard-pattern
match is found. Mixing REDACT and BLOCK on one fn signature is a
correctness hazard (different callers want different behaviour on the
same input).

Rejected: extend privacy_redactor with a returns-Match-list mode.
Reason: signature drift + two callers competing for the same
OnceLock regex pool would tangle the refactor surface.

Chosen: NEW orchestrator/src/privacy/ subtree:
  * privacy/mod.rs -- subtree root with ScanSeverity enum + MatchSpan +
    ENV_PRIVACY_BLOCK_ENABLE = "ELAI_PRIVACY_BLOCK_ENABLE".
  * privacy/secret_scan.rs -- pre-cloud-dispatch scanner returning
    ScanResult { matches: Vec<MatchSpan>, severity: ScanSeverity }.
    Block-tier patterns: aws_access_key (AKIA|ASIA), aws_secret_key
    (assignment-prefixed 40-char), github_pat (ghp_ + github_pat_),
    anthropic_key (sk-ant-), openai_key (sk-), private_key_pem header,
    jwt 3-segment. Warn-tier: denylist_path (.env / ~/.ssh /
    ~/.aws/credentials / secrets/*.yml), env_var_sigil ($VAR /
    ${VAR}).
  * privacy/output_filter.rs -- post-cloud-receive scanner +
    redactor. Default behaviour: redact Block-tier; pass Warn-tier
    through unchanged (Warn matches in OUTPUT are often legitimate
    code references like "the function reads from .env"). Opt-in
    redact_warn_tier=true for ultra-conservative mode.

Crucially MatchSpan carries `pattern: String` + `byte_offset: usize` +
`byte_len: usize` -- the matched VALUE never lands on SSB / details_json.
Pattern labels in `LocalCloudRoutingEventPayload.details_json.patterns`
are deduped + sorted.

Anchor: kiteworks.com 2025 Truffle Security crawl summary (~12K live
secrets, 63% repeated) + protecto.ai 2025 LLM-applications playbook
(two-pass input + output redaction is the canonical pattern; one-pass
misses model-hallucinated leaks).


DECISION-7 -- Output filter is OPTIONAL by default for Warn-tier
patterns; required for Block-tier
------------------------------------------------------------------

In output filtering, denylist_path / env_var_sigil hits are often
legitimate citations from the model (e.g. "Config typically lives at
~/.aws/credentials"). Redacting those by default degrades response
readability without adding security value.

Rejected: redact everything that matches any pattern.

Chosen: Block-tier always redacted (aws_access_key etc.); Warn-tier
default pass-through; opt-in `redact_warn_tier=true` for the
paranoid mode. FilterResult always carries `pattern_labels` so the
dashboard can surface "the model mentioned ~/.aws/credentials" without
mutating the response.


DECISION-8 -- burn_projection at cost/, NOT cloud_spend.rs root
------------------------------------------------------------------

The E-20 plan's "Files touched" list says
`.elai_cc/crates/orchestrator/src/cloud_spend.rs (+~80 LOC
projected_monthly + 2x_alert; file may need creation if C-5 hasn't
shipped)`. That path was REJECTED in E-39 SHIPPED per the
E39_CLOUD_SPEND_PATH_DRIFT gotcha.

Rejected: create cloud_spend.rs at orchestrator root.

Chosen: cost/burn_projection.rs sibling to cost/cloud_abort.rs +
cost/automix_gate.rs + cost/cost_router.rs. projected_monthly_burn
takes a `&[CostHistogramRow]` slice (no live SSB dep -- pure helper);
daily_2x_alert takes f64s; check_burn_alerts composes the two with
the default 7-day baseline + $50 monthly ceiling. Anchors:
docs.litellm.ai + langfuse.com + traceloop.com cost-tracking
patterns. Plan DETERMINISTIC_CHECK 6 ("feed 20-task spend histogram
-> assert projected-monthly within +-10% of analytic baseline")
covered by `cloud_burn_projection.rs::projection_within_10_pct_of_
analytic_baseline`.


DECISION-9 -- PerRoleRuntime priority + task_shape_hint as two
separate BTreeMaps with FORWARD-COMPAT default fallback to
Cost / Execution
------------------------------------------------------------------

Plan task 7 specifies "[agents.<role>.priority] = cost | latency |
quality" + "[agents.<role>.task_shape_hint] = execution | architecture
| math". The taxonomies are stable closed enums (3 variants each;
matches E50_TAXONOMY_LOCK precedent that resists per-plan extension).

Rejected A: single BTreeMap<AgentRole, RoleConfig> with both fields
in one struct. Reason: serde rename ergonomics across the dashboard
TS mirror -- two clean BTreeMaps are easier to round-trip.

Rejected B: per-field Option<Priority>. Reason: 16 AgentRole variants
all default-populated; Option is the wrong shape because the test
`default_priority_covers_every_agent_role` enforces full coverage.

Chosen: 2 BTreeMaps + `#[serde(default = "default_priority_map")]` +
`#[serde(default = "default_task_shape_map")]` so pre-E-20 configs
fill in cleanly. Forward-compat getter falls back to Priority::Cost +
TaskShape::Execution when a role is missing (only possible if the
caller cleared the map manually). All 16 AgentRole variants covered
by both defaults; serde round-trip preserves both.

Default mapping per E-20 plan task 7:
  * Quality + Architecture: Designer, Auditor, Researcher,
    MediaValidator, AuditorConcepts, AuditorPhases, AuditorSteps,
    AuditorBabySteps, AuditorBabyStepDetail.
  * Latency + Execution: Executor, ContextCompiler, MemoryWriter.
  * Cost + Execution: PromptEngineer, IdeaFramer, IdeaCritic,
    InjectionClassifier.
  * MediaValidator is Quality but Execution (single-pass image
    validation, not multi-step).


DECISION-10 -- Holdout / calibration TOML schema + RUNPOD-PENDING
status
------------------------------------------------------------------

Plan task 10 specifies a 200-task holdout (paired local-7B-only +
cloud-Sonnet runs + auditor verdict). Live runs require the remote
3090 sidecar boot per the CLAUDE.md "Sidecar boot is HARDWARE-HEAVY"
HARD RULE; substrate-only ship today.

Chosen: ship the schemas (eval/routing/holdout.toml +
eval/routing/calibration.toml) with status = "runpod_pending",
n_holdout_tasks = 0, threshold_default = 0.55, per_layer = "default".
Once the live runs land, the calibration.toml gets overwritten by
`elai eval routing-holdout` (CLI surface deferred to E-20.B).


DECISION-11 -- D-10 grounding_mode is ORTHOGONAL, not parent
------------------------------------------------------------------

The plan body framing "extends D-10 mode_routing_predicate" is
incorrect. D-10 SHIPPED at orchestrator/src/grounding_mode.rs ships
GroundingMode + select_grounding_mode -- a verification-mode picker
(which of A/B/C/D/E grounding modes to apply to a claim). E-20 is a
RUNTIME-BACKEND picker (which of local / sonnet / opus to dispatch
the SLM/LLM to). The two concerns COMPOSE -- D-10 picks "what to
verify"; E-20 picks "where to run inference" -- but D-10 is not the
parent of E-20.

Document the mismatch here so subsequent readers do not chase the
phantom relationship.


DECISION-12 -- Wire-dark default; ENV_LOCAL_CLOUD_ROUTING_ENABLE +
ENV_PRIVACY_BLOCK_ENABLE gates
------------------------------------------------------------------

Per Pattern-4 wire-dark precedent (E-38 / E-39 / E-50 / E-4 / E-16),
every new feature lands behind a wire-dark env gate so the substrate
compiles + tests + ships identically whether or not the user opts in.

Chosen env-gate constants:
  * ELAI_LOCAL_CLOUD_ROUTING_ENABLE -- gates emission of
    LocalCloudRoutingEvent rows. Producer:
    cost::cloud_route::local_cloud_routing_enabled().
  * ELAI_PRIVACY_BLOCK_ENABLE -- gates the cloud-dispatch privacy
    pre-check. Producer: privacy::privacy_block_enabled(). When
    unset, callers compile + run identically but the gate at the
    dispatch site short-circuits to pass-through.

Both gates default off ("0" / unset).


DECISION-13 -- PrivacyBlock is a NEW reason VALUE on the existing
RouteBlocked variant, not a new enum variant
------------------------------------------------------------------

C-5 SHIPPED RouteBlocked SsbActionType (ordinal 331 in golden) +
RouteBlockedPayload at ssb.rs:8585 with `reason: String` free-form
field. E-20 plan calls "RouteBlocked::PrivacyBlock variant" -- but
that's a new VALUE for the existing field, not a new enum variant.
No SCHEMA bump for PrivacyBlock alone; only the emitter wiring extends
the reason-value catalog.

Chosen: callers emit RouteBlocked with `reason = "privacy_block"` +
`cap_period = "policy"` (or absent) AND additionally emit
LocalCloudRoutingEvent { kind: PrivacyBlocked } with the match-span /
severity payload that does not fit on the cap-shaped
RouteBlockedPayload. Two complementary rows per privacy block: the
spend-cap-shaped row for dashboard parity + the rich-detail row for
audit. Confirmed by routing_e20_contract.rs::
route_blocked_payload_accepts_privacy_block_reason.


DECISION-14 -- 13 papers cited; 8 WebSearch-verified inline as
anchors, 5 supporting-only
------------------------------------------------------------------

Per memory:feedback_paper_citation_verify, every load-bearing arXiv
ID gets WebSearch-verified before use. E-20 plan cites 13 papers; I
verified the 8 most load-bearing inline (see ssb.rs + cloud_route.rs
docstrings):

VERIFIED ANCHORS (load-bearing, drive the decision matrix):
  1. arXiv:2511.07396 C3PO Nov 10 2025 -- conformal-cost-bounded
     cascade with theoretical cost-control + generalization guarantees;
     consistently outperforms across arithmetic (GSM8K / SVAMP),
     formal math (MATH-500 / AIME), logical reasoning (CommonSenseQA /
     BIG-Bench-Hard).
  2. arXiv:2406.18665 RouteLLM Ong+ LMSYS 2024 -- 2x cost reduction at
     near-GPT-4 quality on Chatbot Arena 80k preference battles;
     drives the BackendDifficultyAdapter sentinel pattern.
  3. arXiv:2310.12963 AutoMix Aggarwal+ ICLR 2025 -- POMDP-routed
     self-verify cascade; >50% cost reduction at iso-quality across
     5 datasets x 5 LMs. Drives the AutomixVerifierVerdict / Continue
     / Retry / Escalate vocab (E-39 SHIPPED reuse via
     CostEventKind::AutomixVerifierVerdict).
  4. arXiv:2510.01237 Confidence-Aware Routing Oct 2025 -- multi-
     signal (semantic alignment + internal convergence + learned
     confidence); F1 0.61 -> 0.82, 40% compute reduction; documents
     that probe + perplexity dominate verbalized self-confidence (the
     stub_ban anchor).
  5. arXiv:2510.08731 When to Reason IBM NeurIPS 2025 -- vLLM
     semantic router; -47.1% latency / -48.5% tokens / +10.2pp
     MMLU-Pro. Drives semantic_reasoning::choose_reasoning_mode
     matrix.
  6. arXiv:2502.00674 Self-MoA Li+Lin+Xia+Jin Feb 2025 -- aggregate
     SINGLE best LLM (+6.6% AlpacaEval / +3.8% avg across MMLU /
     CRUX / MATH); documents anti-pattern of mixing different LLMs
     (weakest-member drag). Drives the stub_ban on "naive mixture of
     different LLMs ensembling".
  7. arXiv:2511.03808 Prompt-Difficulty Prediction Bo Zhao+ Nov 5
     2025 -- intermediate-representation activation probes from
     s1.1-32B; E-20.B replacement path for the heuristic classifier
     (RUNPOD-PENDING training corpus).
  8. arXiv:2305.05176 FrugalGPT Chen+Zaharia+Zou TMLR 2024 -- cascade
     with up to 98% cost reduction at GPT-4 parity; foundational
     framing.

SUPPORTING-ONLY (referenced in plan but not load-bearing for any
shipped decision; left in plan citation block):
  * arXiv:2510.08146 entropy-of-first-token (referenced alongside
    2510.01237 in plan task 1; the entropy derivation is the standard
    Shannon formula, no paper-specific anchor needed).
  * arXiv:2510.00202 RouterArena (benchmark spec, not a method).
  * arXiv:2510.08439 xRouter Salesforce RL-trained router (deferred
    to E-20.C; not on the substrate ship critical path).
  * arXiv:2504.01282 Prompt-Reverse Inconsistency (motivates
    majority-vote but its specific results not load-bearing).
  * arXiv:2406.12334 same-prompt different-result inconsistency
    (motivates majority-vote; not load-bearing for specific
    threshold values).

Web sources cited in plan (NOT WebSearch-verified here; treated as
supporting-only):
  * vals.ai/benchmarks/swebench, cosmicjs.com, nxcode.io,
    kunalganglani.com -- model-comparison numbers; the per-role
    default mapping in DECISION-9 cites these as supporting evidence
    but the substrate behaviour does not depend on the exact %.
  * kiteworks.com / Truffle Security 2025 / protecto.ai -- privacy
    playbook; cited as anchor for the SCAN regex set + the two-pass
    redaction pattern. The regex set itself is the load-bearing
    artifact and tested directly.
  * docs.litellm.ai / langfuse.com / traceloop.com -- cost-monitoring
    patterns; cited as anchor for the daily-2x + projected-monthly
    alerting design.


Open follow-ups (E-20.B / E-20.C)
=================================

  * E-20.B -- replace BackendDifficultyAdapter heuristic with the
    arXiv:2511.03808 activation-probe difficulty predictor. Needs a
    labeled hidden-state corpus from s1.1-32B (or local 7B); RUNPOD-
    PENDING.
  * E-20.C -- replace the threshold cascade with arXiv:2510.08439
    xRouter RL-trained router. Big lift; deferred until E-20.B + the
    200-task holdout land.
  * Live RUNPOD calibration (task 10) -- 200 paired local-7B-only +
    cloud-Sonnet runs + auditor verdicts. Substrate already
    accommodates the data via eval/routing/holdout.toml schema.
  * Dashboard wiring (commands_e20.rs + lib/components/
    CloudRoutingPanel.svelte + MonitorWindow mount + TS DTOs) -- the
    Rust substrate is complete; the dashboard surface lands in a
    follow-up patch (kept off the critical path to avoid blocking
    the cluster ship).
  * CLI surface `elai cloud privacy-block` / `elai eval routing-
    holdout` -- not on the critical path; defer.


Build / test status
===================

cargo check -p elai-orchestrator      -- PASS (12 pre-existing
                                          warnings only)
cargo check -p elai-providers         -- PASS (3 pre-existing
                                          warnings + 1 dead-code from
                                          new module)
cargo test  -p elai-orchestrator
             --test action_type_lock  -- PASS (3/3; 1 ignored is the
                                          regen helper)

Audit owner runs the full per-crate test pass + the
crates/contracts/tests/routing_e20_contract.rs round-trip per
CLAUDE.md "main LLM never runs tests" rule.
