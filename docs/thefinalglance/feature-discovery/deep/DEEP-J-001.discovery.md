# DEEP-J-001 exhaustive feature discovery

## Scope, evidence, and audit method

- Assigned manifest section: DEEP-SOURCE-MANIFEST.md lines 875-890, headed DEEP-J-001.
- Direct evidence: the ten files in the processed-manifest table below. Every physical line of every file was read.
- Orientation only: 10-J-SUMMARY_INDEX.discovery.md, specifically its cross-cutting invariants and J-28/J-40/J-43/J-66/J-76/J-90/J-99/J-105/J-112 entries. It was used only to identify expected summary coverage and compute deltas; it is not evidence for any source-ledger item.
- No other numbered J original, grounding map, reasoning source, code file, or adjacent deep-discovery chunk was opened.
- Evidence tags: [E] explicit in the named source; [I] bounded cross-source inference; [A] explicit ambiguity/choice/recon item; [C] conflict or stale assertion requiring reconciliation.
- Facet tags identify capabilities/actions/surfaces; workflows/states; schemas/data/lifecycle; flags/defaults/config; integrations/events; permissions/security/privacy/compliance; failures/fallback/retry/recovery; determinism/idempotency/concurrency; observability/evidence; tests/acceptance/done gates; constraints/non-goals; and dependencies/status/deferred.
- Atomic IDs are unique within this artifact. Repeated acceptance language is folded into the originating atom rather than counted again.

## Exact processed manifest and integrity verification

| Ordinal | Class | Lines declared/actual | Bytes declared/actual | SHA-256 declared/actual | Exact absolute path | Result |
|---:|---|---:|---:|---|---|---|
| 1 | auxiliary/grounding/fixture | 116 / 116 | 11547 / 11547 | 50e4be89e620d49b183aa1c4eb7328f9cebba96bcc7f4b11907f05a45458eddc / same | docs/thefinalglance/J-plan/_grounding/J-DOCTRINE-PROPAGATION-GROUNDING-MAP.md | MATCH |
| 2 | numbered/index original | 411 / 411 | 32213 / 32213 | 05dfe76a6cec744d7997a950c66e1a8de5989cddb66961054a6fd90507350f68 / same | docs/thefinalglance/J-plan/J-28_token_ledger_usage_unification.md | MATCH |
| 3 | numbered/index original | 52 / 52 | 8901 / 8901 | 48ec1348240d581bf9c8eec04435737db868414056d5eb06a8abced4f1e22d4a / same | docs/thefinalglance/J-plan/J-40_research_modes_extractive_compression.md | MATCH |
| 4 | numbered/index original | 64 / 64 | 13503 / 13503 | 7acf286a7333674f3fd8aef4c09263595df2b74c6e525d6f3f98389009f5073b / same | docs/thefinalglance/J-plan/J-43_doctrine_propagation_keystone.md | MATCH |
| 5 | numbered/index original | 74 / 74 | 10089 / 10089 | dea371a8d907eaf51862bbb4abe4a5793450c1f63f36c132fba1641495db04a3 / same | docs/thefinalglance/J-plan/J-66_ssb_temporal_monitor_engine.md | MATCH |
| 6 | numbered/index original | 71 / 71 | 11022 / 11022 | 5589a2ff109efac990569f6d21a4448ea1c2d199dc4650543a4c9ee12352e782 / same | docs/thefinalglance/J-plan/J-76_context_rot_calibration_riders.md | MATCH |
| 7 | numbered/index original | 85 / 85 | 11883 / 11883 | 65a33ecf1283cfea85f1cdc71c0d5d829eca3ee273b55d77b709f58a763ee925 / same | docs/thefinalglance/J-plan/J-90_crosslang_dynamic_blast_cone_composition_gate.md | MATCH |
| 8 | numbered/index original | 124 / 124 | 19571 / 19571 | b635caee013ece299a9104eaa7f909e198d62cd3db7810f269b422ecbf8df138 / same | docs/thefinalglance/J-plan/J-99_time_as_context_plane.md | MATCH |
| 9 | numbered/index original | 130 / 130 | 22538 / 22538 | 1ba7068df85ce878ba99579ef7b0b876621ee8d747d9a5b1f4b320ad2f3a40b1 / same | docs/thefinalglance/J-plan/J-105_memory_roi_attribution_curation.md | MATCH |
| 10 | numbered/index original | 153 / 153 | 16938 / 16938 | d89c578e2c4c32b291d47a40d1e63f015039036474db16b411499de02ae4b749 / same | docs/thefinalglance/J-plan/J-112_spoken_render_narration_dispatcher.md | MATCH |
| Total | 1 auxiliary + 9 numbered originals | 1280 / 1280 | 158205 / 158205 | all ten exact | manifest-prescribed set only | MATCH |

## Per-file exhaustive ledgers

### 1. J-DOCTRINE-PROPAGATION-GROUNDING-MAP.md — 116/116 lines

- DJ001-G-001 [E|evidence] The map was grounded 2026-06-16 by direct main-model reading with no agents; its method read the doctrine-propagation design/handoff, doctrine lens, four precedent rules, J-39 exemplar, both J indexes, and glob/grep-confirmed named seams. Detail files cite DG IDs; PLAN-asserted anchors must be re-grepped at edit and invented line numbers are forbidden. (L1-7)
- DJ001-G-002 [E|dependency/evidence] Four upstream baselines are named: determinism-doctrine-propagation-design sections 1-12 plus addenda b/c/d is authoritative; its handoff carries DP-1..DP-5 and V0/C1/C2/C3/V4/V5 corrections; context-controller carry-state and distill-reset reasoning are sibling/upstream sources. (L9-13)
- DJ001-G-003 [E|integration/doctrine] DG-1 establishes maintainability-review.md as the one-altitude-lower template: MR-1..MR-4 mechanical subset, qualitative catalog, unknown=FAIL, auditor-not-executor application, finding tags, and SELF-CHECK. (L18-21)
- DJ001-G-004 [E|tool/failure] DG-2 fixes the sibling checker CLI/behavior: --files/--base/--json, exit 1 for BLOCK, exit 2 for scan/config error, named thresholds, frozen manifest, stdlib-only; doctrine floor, reasoning-shape, and debt-rate tools mirror it. (L22-26)
- DJ001-G-005 [E|security/determinism] DG-3 makes gate_freeze_check.py plus gate-manifest.txt the immutability mechanism; a run modifying its own frozen gate blocks, and J-43 adds its rule/checker to the manifest. (L27-29)
- DJ001-G-006 [E|schema/config] DG-4 defines the debt posture from wire-dark-allowlist.toml: type/owner/expiry/promotion_plan_id plus an in-code marker; doctrine-debt-ledger and DOCTRINE-DEBT[plan-id] mirror it to conditionally downgrade BLOCK to WARN. (L30-32)
- DJ001-G-007 [E|workflow/permission] DG-5 makes .claude/agents/auditor.md the off-agent checker host; doctrine-floor, reasoning-shape, and substance checks join its existing production-caller/maintainability/traced-assert checks, and executors never self-pass. (L33-35)
- DJ001-G-008 [E|config/integration] DG-6 adds conditional-routing auto-load entries for determinism doctrine when shipping/auditing a feature or touching a gate, and for reasoning method when authoring an artifact. (L36-38)
- DJ001-G-009 [E|doctrine/dependency] DG-7 identifies ELAI-DETERMINISM-DOCTRINE.md as the canonical D1-D5 law, model/harness boundary, eight-question checklist, and named TD-3/D-13/VD-4/D-10/HD-3/F-4/R8/R9 instances; it is imported, not restated. (L39-41)
- DJ001-G-010 [A|evidence/dependency] DG-8 says the AUX method file and M1-M12/stance sketches were PLAN-asserted from another document but not opened; existence and numbering must be confirmed before load-bearing J-44/J-47 use. (L42-44)
- DJ001-G-011 [E|workflow/test] DG-9 records planning-workflow's nine-step protocol and its walking-skeleton PRE_GATE, acceptance/off-agent proof, grounding, and adversarial injections; J-44 generalizes the missing M2/stance/M7/M10 checks. (L45-47)
- DJ001-G-012 [E|workflow] DG-10 makes spec-eval's adversarial design review the host for a mandatory decision-ledger output dimension. (L48-49)
- DJ001-G-013 [E|workflow] DG-11 makes idea-intake's per-level auditor rubric consume the doctrine check as J-44 injection 2. (L50-51)
- DJ001-G-014 [E|workflow/integration] DG-12 adds a doctrine pointer and R10 seam-disposal check to the existing R-1..R-8 agent-dispatch preamble without replacing it. (L52-53)
- DJ001-G-015 [E|integration/security] DG-13 inventories connected gpt-bridge carriers (gpt, grok, perplexity, gemini, claude), shared apps_workflow pool, no repository access, serialized snippets, fragile rate limits, and a single advisory vote that can never ground a critical path. (L54-56)
- DJ001-G-016 [E|recovery/permission] DG-14 routes a cross-family finding through handoff mediation to an orchestrator/user decision entry, never a passive dashboard. (L57-59)
- DJ001-G-017 [E|integration] DG-15 names the oracle skill as a legitimate design-time host for a doctrine diversity-vote prompt. (L60-61)
- DJ001-G-018 [A|data/status] DG-16 identifies ELAI-FEATURE-MAP.md as the living status/composition/doctrine/rejection source and J-46 upsert target, but its existence was PLAN-asserted rather than globbed and must be confirmed at edit. (L62-65)
- DJ001-G-019 [E|measurement] DG-17 constrains SAFEDELTA to gate-ON-vs-OFF ablation rather than leaderboard rank; J-46 applies that D4 precedent to debt-rate. (L66-67)
- DJ001-G-020 [E|dependency/non-goal] DG-18 says J-39..J-42 already embody decomposition/research/grounding/reasoning doctrine; J-47 must preserve per-axis discipline without re-authoring them. (L68-70)
- DJ001-G-021 [E|test] DG-19 distinguishes EXP-DP1..DP4 per-gate falsifiers from J-47 EXP-DP5 at program altitude. (L71)
- DJ001-G-022 [E|concurrency/config] DG-20 reuses orchestrate/auto-orchestrate parallel machinery capped at three plus observe-only-then-promote for J-47's A/B/C harness; no new orchestrator is allowed. (L72-73)
- DJ001-G-023 [C|schema/status] The authoring baseline was computed, not literal: SCHEMA_VERSION = -309 + SSB_ACTION_TYPES.len(), yielding schema 268 and 577 variants; both values are known to age quickly and must be re-grepped at edit. (L75-80)
- DJ001-G-024 [E|status/test] The DP wave is harness-side; done is JD-1 evidence from a real session—checker findings, debt ledger, rate file, feature-map stamp, and A/B/C comparison—not mere file existence. (L81-83)
- DJ001-G-025 [E|event/default] Default J-43..J-47 schema impact is zero action types; optional DoctrineDebtEvent is one Pattern-2 variant with FloorFinding/DebtRegistered/DebtPromoted kinds only when runtime-queryable telemetry is desired, using live-baseline+1 rather than a constant. (L83-86)
- DJ001-G-026 [E|test/failure] JD-1 requires a real harness trigger and off-agent artifact; missing evidence is FAIL. (L89-94)
- DJ001-G-027 [E|data/determinism] JD-2 requires debt rate and ledger to be derived from recorded state rather than recollection. (L91-93)
- DJ001-G-028 [E|default/promotion] JD-3 holds every DP gate non-blocking until false-positive/friction evidence covers at least three sessions. (L92-94)
- DJ001-G-029 [E|security/change-control] JD-4 forbids editing a registered hook during the wave; new tools are new files and manifest-freeze enrollment is a user-reviewed change. (L93-94)
- DJ001-G-030 [E|constraints] Inherited W/GD/HD/TD/VD rules require run-trigger proof, composition without restatement, recomputable numbers, bounded false positives, and named residue; the wave adds enforcement but no new doctrine family. (L95-99)
- DJ001-G-031 [E|capability/status] J-43 is the Track-1 lens: determinism rule, DD-1..DD-5 floor, debt ledger, off-agent auditor integration, and freeze; net zero SSB by default. (L101-105)
- DJ001-G-032 [E|capability/status] J-44 is the Track-2 method: reasoning rule, artifact-shape checker, random substance panel, and four lifecycle injections; net zero SSB. (L106-108)
- DJ001-G-033 [E|capability/status] J-45 is calibrated cross-family advisory: 100-300-trace corpus, kappa greater than 0.6, escalation routing, and explicit single-family-degraded debt; net zero SSB. (L109-110)
- DJ001-G-034 [E|capability/status] J-46 is doctrine-debt measurement: doctrine_debt_rate.py, WIRED-only initial sweep, feature-map conformance stamp, and implausible-zero self-test; net zero SSB. (L111-112)
- DJ001-G-035 [E|capability/status] J-47 is the program falsifier: A/B/C experiment first, then measured-pain-only per-axis doctrine-family additions under a count cap; net zero SSB. (L113-114)

### 2. J-28_token_ledger_usage_unification.md — 411/411 lines

- DJ001-28-001 [E|status/capability] J-28 is PLANNED, Wave GSW-1, a build/extension data layer: fix local streaming capture, normalize lane semantics, emit one uniform turn row, catalog-price spend, and provide CLI dollar reporting; J-29 owns the dashboard window. (L1-18)
- DJ001-28-002 [E|failure/data] The diagnosed defects are: local MessageDelta-only capture loses MessageStart input tokens; local input means full prompt while cloud means uncached suffix; every model is priced as Sonnet; existing rows lack model/task/session join keys; current task_usd is latest-row rather than session total. (L10-17, L47-57)
- DJ001-28-003 [E|workflow/recovery] T1 accumulates MessageStart usage into latest_usage and merges MessageDelta output usage without clobbering either side. (L20-28)
- DJ001-28-004 [E|recon/test] Before T1/T2 coding, both the SSE usage path and runtime (tokens_in,tokens_out) read path must be mapped so the repair lands on the production-consumed assembly. (L26-28, L310-313)
- DJ001-28-005 [E|schema/data] T2 defines NormalizedUsage with raw TokenUsage, lane, semantics_tag, and normalized fields; equal numbers from local/cloud remain semantically distinct. (L29-35, L245-247)
- DJ001-28-006 [A|workflow/integration] T2 has two edit-time routes: sidecar emits spec-compliant suffix/cache fields and moves with runtime_client.rs, or the client normalizes by lane. Route B is default because it is fixture-only/Tier-1; Route A requires a resyncable remote deployment and cross-language diff. (L32-35, L224-225, L292-298)
- DJ001-28-007 [E|constraint/data] No aggregate may naively combine cloud and local input counts; aggregation must remain per-lane or explicitly dual-labeled, and the negative contract must make an unlabeled fold impossible or failing. (L34-35, L161-165, L248-251)
- DJ001-28-008 [E|event/integration] T3 emits exactly one UsageTurnEvent per turn at both existing seams: beside CloudSpend for cloud and BackendCostAccrued for local; both legacy rows remain. (L36-46)
- DJ001-28-009 [E|schema/privacy] UsageTurnEvent payload contains identity (String, default local; not multi-user infrastructure), lane, backend, model, task_id, session_id, four token counters, est_cost_micros, elapsed_ms, and semantics tag; secrets/key material are prohibited. (L39-43, L139-141, L166)
- DJ001-28-010 [E|event/lifecycle] UsageTurnEvent is a Pattern-2 single variant with TurnRecorded, EstimateApplied, and NormalizationApplied kinds; schema/golden discipline is exactly one added variant from the live baseline. (L42-46, L252-258)
- DJ001-28-011 [E|pricing/failure] T4 replaces the Sonnet-only fallback with pricing_for_model/the I-51 ModelCatalog plus family-alias normalization; unknown models produce explicit None/no-rate rather than an incorrect cost. (L47-52, L259-265)
- DJ001-28-012 [E|data/lifecycle] T5 usage_ledger folds by (identity, model, task_id, session_id, lane), accumulates across rows, and preserves Option semantics such that None+None=None rather than zero. (L53-57, L266-271)
- DJ001-28-013 [E|fallback/observability] T6 counts missing/partial provider usage with a per-lane tokenizer and blake3-keyed LRU, labels estimates as approximations, preserves their distinction in rollups, and emits EstimateApplied rather than silent zeros. (L58-61, L272-277)
- DJ001-28-014 [E|lifecycle/data] T7 treats compaction summary output as later input context and requires monotonic cumulative growth; it extends H-50 ContextPolicy or usage_ledger, exactly one owner, while leaving core TokenUsage unchanged. (L62-65, L278-283, L303-308)
- DJ001-28-015 [E|surface] T8 adds a catalog-backed dollar line and per-lane split to format_cost_report plus an elai usage verb grouped per model/task/session; lanes without rates show tokens-only rather than zero dollars. (L66-70, L284-290)
- DJ001-28-016 [E|deferred/dependency] Recorded riders route OTEL counters to E-29, compaction to H-50, XML tool fallback to E-35, and RunPod real-cost attribution to a future pod-lifecycle rider; they are not J-28 tasks. (L71-73)
- DJ001-28-017 [E|status/dependency] Existing reusable state is TokenUsage/UsageTracker, pricing APIs, CLI report, both event seams, and Goose Option/estimator/cache patterns; CumulativeTokenTracker remains cloud-only residue and pricing_rollup remains dark for J-29 to activate. (L75-96, L147-153)
- DJ001-28-018 [C|evidence/anchors] The 2026-07-09 red-team correction moves cloud_mode pricing anchors, pricing.rs APIs, sidecar usage anchors, and split SSB declarations into action_type.rs/payloads.rs; BackendCostAccruedPayload still lacks model/task/session. Several earlier line anchors remain accurate. Every anchor must still be re-grepped. (L97-107, L393-404)
- DJ001-28-019 [E|scope/status] Tier 1 ships T1-T8 with in-process fixtures, including both T2 candidates as code-testable and both event seams. (L109-112)
- DJ001-28-020 [E|deferred/permission] Tier 2 is the live remote-sidecar confirmation of nonzero local input plus lane tag; hardware must be flagged before boot and no remote boot is allowed merely to satisfy Tier 1. (L113-115, L323-325)
- DJ001-28-021 [E|integration/schema] SSB is module-split: action variant in ssb/action_type.rs, payload and SsbPayload arm in ssb/payloads.rs, registration/schema derivation/golden in ssb.rs; full six-part variant and nine-tuple golden lockstep apply. (L121-130)
- DJ001-28-022 [E|integration] Production edits span anthropic capture, optionally server.py/runtime_client as a cross-language pair, usage_ledger, both emit seams, cloud_mode pricing, and CLI; J-29 is the explicit frontend handoff. (L117-135)
- DJ001-28-023 [E|dependency/non-goal] UsageTracker, pricing_for_model/estimate_cost_usd_with_pricing, CloudSpend/BackendCostAccrued seams, and CumulativeTokenTracker are consumed, not reimplemented or churned. (L137-153)
- DJ001-28-024 [E|reachability] Required production roots are CloudGroundingClient::send_message, RuntimeOrchestrator::try_complete, and elai usage/live cost print; contracts-only callers never establish wiring. (L154-159)
- DJ001-28-025 [E|constraint] New hardcoded per-model rate tables, silent estimation, mid-stream SSB writes, naive cross-lane sums, secret-bearing payloads, production unwrap/expect, and blake2/sha2 for the cache are forbidden. (L160-167)
- DJ001-28-026 [E|flag/default] J-28 ships on with no new environment gate; copying Goose's default-off cost-display gate is explicitly rejected. (L168)
- DJ001-28-027 [E|surface] J-29 UsageWindow is the next-wave frontend pair; within J-28 the CLI report is the actual consumer. (L169-171)
- DJ001-28-028 [E|acceptance/evidence] The run-trigger requires one production-shaped cloud fixture, one local fixture, and elai usage; evidence is two lane-tagged rows, nonzero streaming-local input, Haiku catalog—not Sonnet—pricing, and a grouped separately labeled rollup. (L172-182)
- DJ001-28-029 [E|failure/test] Negative acceptance requires missing reported usage to become EstimateApplied, an unlabeled cloud+local sum to fail, and Haiku-at-Sonnet pricing to fail. (L179-182)
- DJ001-28-030 [E|test] Deterministic checks include wiring_contract_check with run trigger, UsageTurnEvent registration, MessageStart capture, at least one catalog lookup in cloud_mode, and a J-28-decisions artifact. The Haiku/unknown-model unit test is load-bearing; grep is corroboration only. (L184-195)
- DJ001-28-031 [C|schema/status] The 2026-06-12 schema 234/543 snapshot is explicitly stale; 2026-07-09 observed schema 271, and implementation must derive live-baseline+1 from the computed formula rather than preserve either snapshot. (L205-209, L299-302)
- DJ001-28-032 [E|dependency/security] No dependency is added; G-9 redaction, J-22 payload shape lock, W/GD/HD/R7 rules, I-51 catalog, H-9, H-50, C-14, sidecar, and J-29 compose with the unit. (L210-219)
- DJ001-28-033 [A|recon] Open implementation questions are which usage assembly read path owns normalization, sidecar-vs-client T2, and where cloud elapsed_ms obtains its send-seam wall clock. The ledger itself only writes SSB and has no Windows file-semantics issue. (L221-227)
- DJ001-28-034 [E|failure/recovery] T1 failure diagnosis distinguishes a missing MessageStart arm from overwrite instead of merge; recovery re-greps the arm, asserts input/output side preservation, and verifies the production tuple path. (L235-244)
- DJ001-28-035 [E|failure/recovery] T3 fails if either lane lacks emission or the golden diff is not exactly +1; recovery greps both seams, regenerates/diffs one variant, and recomputes schema from live state. (L252-258)
- DJ001-28-036 [E|failure/recovery] T4 failure is Haiku retaining Sonnet rates; recovery routes through catalog alias normalization and asserts Haiku correctness plus unknown=None. (L259-265)
- DJ001-28-037 [E|failure/recovery] T5 failure is last-row-only session total; recovery uses a multi-row hand-computed fixture and separately asserts None+None remains None. (L266-271)
- DJ001-28-038 [E|failure/recovery] T6 fails on TurnRecorded with all-zero counters; estimator activation plus a negative assertion forbids that dead-evidence state. (L272-277)
- DJ001-28-039 [E|failure/recovery] T7 fails on reset or double-count across compaction; a compaction fixture proves per-lane monotonicity and equality between the chosen T7 growth owner and T5 session sum. (L278-283, L350-361)
- DJ001-28-040 [E|failure/recovery] T8 fails on zero/wrong price or unreachable CLI; production-caller check and a fixture database must yield a nonempty, dual-lane grouped report. (L284-290)
- DJ001-28-041 [A|workflow] FORK-1 chooses sidecar Route A only if it is the single clean assembly point and deployment can be resynced; otherwise client Route B. (L292-298)
- DJ001-28-042 [A|schema] FORK-2 always recomputes schema and total variants when live state differs; the documented older numeric branch is historical, not a usable default. (L299-302)
- DJ001-28-043 [A|ownership] FORK-3 gives compaction growth to H-50 when it already threads deltas, otherwise usage_ledger; dual ownership is prohibited and the decision needs a file:line record. (L303-308)
- DJ001-28-044 [A|recon] Four explicit recon gates cover the two live usage reads, both emit seams, live schema/variant values, and cloud elapsed_ms timestamp source. (L310-321)
- DJ001-28-045 [E|abort/scope] Abort if Route A needs a remote boot for Tier 1, if golden regeneration changes more than one/unrelated variant, if an aggregate requires unlabeled cross-lane addition, or if proof would require real dogfood rows rather than fixtures. (L323-333)
- DJ001-28-046 [E|test/concurrency] The off-agent auditor is sole runner; cargo tests execute one at a time with -j 1 on the OOM-prone host, then production-caller and maintainability checks run. Cross-language diff is mandatory only for Route A. (L335-348)
- DJ001-28-047 [E|acceptance] The integration smoke must drive both real entrypoints on a hard multi-turn input crossing compaction, then assert two rows, separate rollups, nonzero local input, and T7/T5 equality on both lanes; a one-turn happy path is insufficient. (L350-361)
- DJ001-28-048 [E|evidence/correction] Non-author red-team changed three executable details: replaced a tautological pricing grep with an assertable catalog-call check plus negative test, documented the SSB module split, and added the single-owner compaction fork plus cross-boundary equality smoke. (L363-392)
- DJ001-28-049 [E|dependency/status] Completion closes audit gaps C-1/C-2/C-6/L-1/L-3/L-5 and supplies J-29; J-30 custom providers inherit the ledger because emission is at shared orchestrator seams. (L406-409)

### 3. J-40_research_modes_extractive_compression.md — 52/52 lines

- DJ001-40-001 [E|status/dependency] J-40 is a medium DD-2 build activating E-48 live search and extending A-RAG; it depends on J-38, E-48, and base-1E. (L1-4)
- DJ001-40-002 [E|workflow/determinism] The research cog boundary permits the model only to formulate queries and rank/select pages; the harness performs parallel fetch and extractive compression, and only grounded spans plus external step state enter reasoning context. (L3-4)
- DJ001-40-003 [E|capability/schema] QueryAwareExtractor implements ContentExtractor, scores sentence chunks by semantic overlap with the query vector rather than mere keyword containment, and retains the top spans verbatim. (L6-7)
- DJ001-40-004 [E|config/constraint] The extractor uses a per-source span budget sized so a 25-40K local window can hold roughly 100-150 sources; the cited greater-than-50:1 and roughly-200-token figures are external design rationale, not ELAI acceptance thresholds. (L4, L7, L33-37)
- DJ001-40-005 [E|integration] E-48's dark leg is activated by replacing StubSearchBackend with SearXngBackend behind WebResearchStepEmitter; /search plus SearXNG is the harness fetch layer and the model never receives raw-fetch control. (L8, L13-18)
- DJ001-40-006 [E|workflow/config] ResearchMode has FAST (one rank/fetch/compress/synthesis pass), STANDARD (breadth 2-3, depth 3), and DEEP (breadth 4-5, depth 5, iterative refine); all share the same cog invariant. (L9)
- DJ001-40-007 [E|data/lifecycle] Between iterations, hit lists and coverage maps serialize to SessionDag/journal; every research step starts with lean bounded context and recoverable SSB-backed external state. (L10, L18)
- DJ001-40-008 [E|non-goal] Generative re-summarization is forbidden on this path because it reduces faithfulness and disrupts the deterministic cache-prefix posture. (L11)
- DJ001-40-009 [E|non-goal] KV reuse is excluded as the fast-context lever because the local streaming path bypasses PrefixKVCache; extractive compression plus external state is the intended lever. (L11)
- DJ001-40-010 [E|deferred] Full Search-as-Code/retrieval-plan-as-program is deferred to a measured A/B and is not a default; deterministic assembly remains authoritative. (L11, L35)
- DJ001-40-011 [E|integration] QueryAwareExtractor replaces extract_keyword_snippet inside keyword_search/web_arag so raw fetched pages are compressed before synthesis; output is ranked, pre-cited spans that feed J-41 closed-set binding. (L15-16, L21-22)
- DJ001-40-012 [E|surface/reachability] Produced symbols are QueryAwareExtractor, ResearchMode, SearXngBackend, and compress_source_spans; production roots are the REPL research entrypoint, WebResearchStepEmitter, and CLI research/oracle --mode. (L24-28)
- DJ001-40-013 [E|constraint/failure] A placeholder span budget, dual-pathed surviving stub backend, or generative extractor violates the wiring contract. (L28)
- DJ001-40-014 [E|observability/event] Existing QueryTrace rows record per-source compression ratio and ResearchMode; default schema impact is zero. ResearchModeEvent is optional only if separately desired and must use a live-derived baseline. (L29-31)
- DJ001-40-015 [E|evidence/acceptance] A fetched span set must be strictly smaller than its raw page, logged with mode/ratio, and an end-to-end run must prove raw page bytes never occupy the reasoning-context slot. (L30)
- DJ001-40-016 [E|fallback] Live SearXNG requires the canonical remote sidecar, while compressor and modes remain offline-fixture-testable; an unavailable backend degrades through the existing 1I cascade. (L21-22, L46-49)
- DJ001-40-017 [A|performance] Sentence embeddings add latency and must be measured against budget; the existing embed/reranker stack is only a candidate, not a selected implementation. (L46-47)
- DJ001-40-018 [A|quality] Excess compression can discard a true supporting span, so tuning must optimize supporting-span recall as well as token count before J-41 grounding judges the result. (L48)
- DJ001-40-019 [E|dependency] The unit composes with J-38 retrieval, J-41 grounding, E-48 live search, base-1E A-RAG, base-4D FlashResearch, SessionDag, and the determinism doctrine; it restates none. (L39-44)
- DJ001-40-020 [E|status] Next is J-41's deterministic fingerprint/small-claim-model/closed-set citation stack over J-40 spans. (L51-52)
- DJ001-40-021 [E|surface] An existing research/oracle dashboard panel may optionally show selected mode and per-source compression ratio; if added, its cross-language types must pass cross_lang_diff. (L29)
- DJ001-40-022 [C|schema/status] The optional-event clause calls 268/577 the live SSB baseline at 2026-06-16; later evidence in this same chunk supersedes that snapshot, so only live derivation is executable. (L31)

### 4. J-43_doctrine_propagation_keystone.md — 64/64 lines

- DJ001-43-001 [E|status/dependency] J-43 is the DPW-1 Track-1 lens, a medium harness build depending on maintainability-review, gate freeze, and the auditor agent. (L1-4)
- DJ001-43-002 [E|doctrine/non-goal] The source rejects a self-applied passive lens and also rejects one comprehensive scorer: honest enforcement is a thin structural floor, calibrated cross-family advisory for semantics, and fail-closed debt, because a broad mechanical/LLM scorer would violate D3 or D5. (L3-4, L17)
- DJ001-43-003 [E|measurement/default] EXP-DP1 runs the checker over existing B..K plans and live code in observe-only mode; each DD class remains WARN until its own false-positive rate is bounded over at least three runs. (L6-7)
- DJ001-43-004 [E|evidence/constraint] External FP figures—17-53% generated code, roughly 10-20% tuned manual, below 5% tuned security—are provisional rationale; no zero-FP claim is allowed, and ELAI must measure the actual checker. Rust full-dataflow is the weak leg. (L7, L44-45)
- DJ001-43-005 [E|rule/config] determinism-doctrine.md follows the maintainability rule shape: short deterministic subset, qualitative D1-D5 catalog, imported eight-question checklist, unknown=FAIL, auditor-only application, and SELF-CHECK. (L8)
- DJ001-43-006 [E|integration] conditional-routing auto-loads that rule for any feature ship/audit or gate-edit task. (L8, L26)
- DJ001-43-007 [E|tool/failure] doctrine_floor_check.py is regex/AST, necessary-not-sufficient, mechanical-only, manifest-frozen, WARN-first, and exits 2 fail-closed on missing target or scan/config failure. (L9, L22, L36-37)
- DJ001-43-008 [E|security] DD-1 GATE-READS-OWN-OUTPUT detects a verdict computed from producer-emitted text and is a BLOCK candidate. (L10)
- DJ001-43-009 [E|failure] DD-2 SKIP-ON-MISSING-EVIDENCE detects missing/empty evidence yielding SKIP/UNVERIFIED/WARN rather than fail-close and is a BLOCK candidate. (L11)
- DJ001-43-010 [E|security/independence] DD-3 SAME-FAMILY-CRITIC detects producer and critic/verifier/adversary sharing a model family and is a BLOCK candidate. (L12)
- DJ001-43-011 [E|constraint] DD-4 LLM-VERDICT-ON-GATING-PATH detects pure LLM gate predicates without a mechanical leg but initially remains WARN because classification is semantic. (L13, L60)
- DJ001-43-012 [E|constraint] DD-5 CONSTRAINED-REASONING detects grammar/forced-JSON/constrained decoding on reasoning, coding, or planning roles rather than pure operation emission and initially remains WARN. (L14)
- DJ001-43-013 [E|schema/compliance] doctrine-debt-ledger rows carry decision_id, feature, sub_doctrine D1..D5, why_unmechanized, current_disposer, owner, expiry_date, and promotion_plan_id; a missing promotion plan is itself a finding. (L15)
- DJ001-43-014 [E|fallback/config] An in-code DOCTRINE-DEBT[plan-id] marker downgrades a DD BLOCK to REGISTERED-DEBT/WARN only when a matching real ledger row exists; the ledger cannot be an exception graveyard. (L15, L24)
- DJ001-43-015 [E|permission/evidence] The AUDITING-state auditor re-runs the floor off-agent and emits D-DETERMINISM-* findings; executor self-pass is forbidden. (L16, L25)
- DJ001-43-016 [E|security/change-control] determinism-doctrine.md and doctrine_floor_check.py join gate-manifest; a run modifying either blocks under gate_freeze_check.py. (L16, L23)
- DJ001-43-017 [E|integration/non-goal] J-43 adds one rule, tool, ledger, auditor invocation, manifest registration, and routing entry, wrapping but never editing/reimplementing wiring, production-caller, maintainability, tracing-assert, or F-4. (L29-30, L47-50)
- DJ001-43-018 [E|done-gate] JD-1 completion requires a real audit over a real/seeded changeset, off-agent findings, and frozen-gate proof—not created files. A seeded own-output gate yields DD1; a clean change yields none; missing input exits 2. (L29-30, L35-37)
- DJ001-43-019 [E|residue] The structural floor cannot detect disguised proxies or semantic boundary laundering; DP-3 advisory and DP-4 debt-rate remain required, and Rust recall is explicitly weakest. (L30, L57-61)
- DJ001-43-020 [E|event/default] Default SSB impact is zero. Optional DoctrineDebtEvent has FloorFinding/DebtRegistered/DebtPromoted kinds only for runtime-queryable telemetry, with live-baseline+1; harness artifacts otherwise supply evidence. (L38)
- DJ001-43-021 [A|config] Debt-ledger location is unresolved between .claude/ and beside wire-dark-allowlist; the source leans .claude/ because the checker/auditor and harness scope live there. (L57-58)
- DJ001-43-022 [A|failure/config] DD-3 needs a maintained model-id-to-family mapping; staleness can silently miss same-family dispatches. (L59)
- DJ001-43-023 [E|constraint] DD-4 may not be promoted to BLOCK solely from the floor, and a floor PASS never means doctrine conformance; an implausibly zero subsystem debt rate is itself suspicious. (L60-61)
- DJ001-43-024 [E|dependency/status] J-44 provides method shape, J-45 semantic cross-family review, J-46 debt aggregation, and J-47 the program A/B/C falsifier. (L51-55, L63-64)
- DJ001-43-025 [E|evidence/status] Thin floor + advisory + exception registry + measurement is grounded as a recognized governance pattern, but its composition in the ELAI harness remains an unproven bet delegated to DP-5 A/B/C. (L44)
- DJ001-43-026 [C|schema/status] The optional DoctrineDebtEvent clause snapshots schema 268/577; the same clause requires computing live-baseline+1, which is authoritative over the dated figures. (L38)

### 5. J-66_ssb_temporal_monitor_engine.md — 74/74 lines

- DJ001-66-001 [E|status/dependency] J-66 is a large NASA-GATES-2 build/extension filling I-35's unspecified compiler and depends on J-69 journal integrity. (L1-6)
- DJ001-66-002 [E|capability/determinism] New ssb_monitor compiles temporal policies over the append-only SSB with no LLM adjudication. (L8-9)
- DJ001-66-003 [E|algorithm/concurrency] Past-time LTL compiles into O(1)-work recurrences evaluated on every committed row in broadcast-after-commit order. (L9-10)
- DJ001-66-004 [E|security/policy] G1 always(write_tool -> diamond_past policy_allow) requires every write/edit tool call to have a preceding allow row. (L10-12)
- DJ001-66-005 [E|security/privacy] G2 always(egress -> not_since(scope_widen, declassify)) forbids egress following an undeclassified scope widen. (L12)
- DJ001-66-006 [E|non-goal] G3 test_passed -> prior stdout_hash already belongs to F-4 AntiFabricationGate and must not be rebuilt. (L13, L22)
- DJ001-66-007 [E|workflow/lifecycle] Future-time formulas rewrite into session obligations and resolve at MEMORIZING->COMPLETE; any undischarged obligation blocks completion rather than becoming inconclusive. (L14-16)
- DJ001-66-008 [E|concurrency/lifecycle] Oblig1 requires every agent_spawned to eventually receive agent_returned before completion. (L15)
- DJ001-66-009 [E|workflow/lifecycle] Oblig2 requires every finding_block to eventually receive finding_closed before completion. (L16)
- DJ001-66-010 [E|authoring-gate] feasibility::lint deterministically admits only safety properties counted by step/event, rejecting liveness or wall-clock predicates with a reason and routing them toward a watchdog. (L17-18)
- DJ001-66-011 [E|constraint] LLM trace scoring, SKIP/inconclusive end verdicts, wall-clock/liveness SSB monitors, and duplicate F-4 logic are excluded. (L18)
- DJ001-66-012 [E|status/integration] The monitor consumes SsbActionType total order, a PLAN-asserted broadcast seam, a PLAN-asserted FSM completion seam, planned I-35 registry if present, and J-69 chain scan; seams must be re-grepped at edit. (L20-24)
- DJ001-66-013 [E|integration] New module layout is ssb_monitor/{mod,ptltl,rewrite,registry,feasibility}.rs; online and session-end verdicts both feed the FSM gate without creating another journal writer. (L26-27)
- DJ001-66-014 [A|event/schema] Prefer reusing an I-35 event if it shipped; otherwise add one Pattern-2 SsbMonitorVerdict with PastTimeViolation, ObligationUndischarged, and FeasibilityRejected kinds, full variant/golden lockstep, and live-baseline+1. (L28)
- DJ001-66-015 [A|surface] Monitor rows should appear on an existing audit/monitor drawer, but a first runtime-only slice may instead carry a W-6-justified absence. (L29, L51)
- DJ001-66-016 [E|interfaces] Exact producers are ptltl::compile, rewrite::step, a G1/G2/Oblig1/Oblig2 registry, and feasibility::lint; consumers are SSB broadcast, MEMORIZING->COMPLETE, and J-69 chain verification. (L31-40)
- DJ001-66-017 [E|reachability] Only a real elai task where per-row and completion gates fire proves production reachability; tests/contracts alone do not. (L41-43)
- DJ001-66-018 [E|constraint/default] The engine has no LLM calls, no wall-clock/liveness registration, no inconclusive end state, no F-4 duplicate, and no new writer; blocking stays off until false positives are bounded over three real runs and an allowlist row exists. (L44-50)
- DJ001-66-019 [E|acceptance] A write lacking allow emits PastTimeViolation naming G1 at that row; an unreturned agent emits ObligationUndischarged at close; either prevents FSM advance. (L52-55)
- DJ001-66-020 [E|negative-test] A conforming run completes with no violation; wall-clock registration produces FeasibilityRejected; missing/tampered/J-69-failed journals make the engine refuse to run, never empty-pass. (L53-55)
- DJ001-66-021 [A|scope] Whether diamond_past crosses parent-session boundaries is unresolved per predicate; observe-only measurement must log would-block counts. (L66-67)
- DJ001-66-022 [E|failure/determinism] The feasibility checker itself must be deterministic and frozen; UNKNOWN classification is REFUSE, not accept. (L68)
- DJ001-66-023 [E|fallback] Properties not soundly monitorable online route to session-end rewriting or J-68 design-time checks rather than an online maybe verdict. (L69)
- DJ001-66-024 [E|dependency/lifecycle] The enforcement spine is J-69 integrity -> J-66 verdict -> J-67 typed action; inserted/suppressed J-67 actions return as SSB rows visible to monitors. H-54 obligation closure remains adjacent and separate. (L63-64, L71-72)
- DJ001-66-025 [C|schema/status] The file records 268/577 as live on 2026-06-22 for a possible +1 SsbMonitorVerdict, while also requiring a W-9 re-read; the later J-28 observation proves the snapshot cannot be reused. (L21, L28, L61)

### 6. J-76_context_rot_calibration_riders.md — 71/71 lines

- DJ001-76-001 [E|status/dependency] J-76 is a small/medium CR-3 extension authored as a new file rather than amending planned J-34/H-50; it packages two actionable riders and defers a third. (L1-5)
- DJ001-76-002 [E|measurement/constraint] Central doctrine is import method, not external numbers: all cap, placement, and trim decisions consume ELAI's measured per-(model,lane) J-34 curve, not frontier-model results or folklore. (L5, L8, L11)
- DJ001-76-003 [E|capability/config] The C6 retrieval controls cap injected chunk count per query. (L7-8)
- DJ001-76-004 [E|capability/config] Top-value chunks are placed at measured head/tail positions, never the middle, through the 3E slot ordering driven by J-34 positional_value_curve. (L8, L14, L26)
- DJ001-76-005 [E|capability] A focused pre-filtered prompt is preferred to a full-context dump, validating H-50 selective reload. (L8)
- DJ001-76-006 [E|lifecycle] The C7 local compaction recipe triggers near 85% of the sidecar window, preserves intermediate numerics, and rebuilds context as system + initial + summary + continue. (L9, L20)
- DJ001-76-007 [E|lane/determinism] The recipe is local-lane only; cloud retains deterministic masking and never generatively re-summarizes the cache prefix. (L9, L11, L45)
- DJ001-76-008 [E|deferred] C8 SproutRAG is recorded, not built: a function->class->module max-linkage hierarchy with multi-granularity summaries, beam retrieval of leaves/coarser spans, and ONNX-portable cosine fallback. (L10, L21)
- DJ001-76-009 [E|non-goal] Frontier thresholds, cloud-prefix LLM summarization, RLM REPL-as-context, and 32K-plus context as a depth-reasoning cure are excluded. (L11)
- DJ001-76-010 [E|status/dependency] Existing state is planned J-34 calibration, partial/uncommitted H-50 regime policy, wired 3E eight-slot/two-breakpoint allocator, and an existing Chroma/RULER rationale rider. (L13-17)
- DJ001-76-011 [E|scope] Tier 1 is deterministic cap/placement plus the local recipe; SproutRAG and the RLM architecture remain later experiments. (L19-21)
- DJ001-76-012 [E|integration] context_calibration.rs supplies curves, context_policy.rs owns local compaction, and context_budget.rs owns cache-safe static head/tail ordering; J-34/H-50/3E are extended, not re-authored. (L23-26)
- DJ001-76-013 [E|event/default] Prefer reusing ContextCalibrationEvent plus H-50's compaction/HandoffLifecycle path; expected schema impact is zero and is decided against live state at edit. (L27)
- DJ001-76-014 [A|integration/surface] If a sidecar field must be threaded, server.py forms a cross-language pair and cross_lang_diff applies; otherwise there is no pair. The surface is J-29 ContextStatePanel or a W-6-justified absence. (L28)
- DJ001-76-015 [E|reachability] Production proof needs both a real long local session crossing its compaction threshold and a real retrieval injection exceeding the cap; tests/contracts alone do not prove either. (L30-42)
- DJ001-76-016 [E|flags/constraints] No frontier number, cloud LLM resummary, duplicated J-34/H-50 policy, middle-position top chunk, or unregistered default-off flag is permitted. Any new ELAI_*_ENABLE requires an allowlist row and promotion plan. (L43-49)
- DJ001-76-017 [E|acceptance] Evidence is calibration/compaction rows plus a preserved-numerics rebuilt local context and capped head/tail prompt; cloud must remain unchanged, and a model lacking a curve uses a conservative fallback. (L50-53)
- DJ001-76-018 [E|evidence] Focused-300-vs-113K, 30%-plus middle loss, 5-7K onset, and the 85% RLM trigger are external design rationale only; the arXiv identifier must be reverified. (L55-60)
- DJ001-76-019 [A|fallback] If J-34 lacks a Qwen lane curve, placement may ship conservatively while the cap remains behind the curve. (L66-67)
- DJ001-76-020 [A|determinism/quality] The source says numeric preservation must be deterministic extraction rather than free-form local summarization and must be sidecar-verified; the exact summarizer/extractor mechanism is not specified. (L68)
- DJ001-76-021 [E|deferred/gate] SproutRAG is reconsidered only after C1-C5 and only after a measured multi-granularity gap in the flat index. (L69)
- DJ001-76-022 [C|config/measurement] The file repeatedly requires J-34's measured per-(model,lane) knee and rejects a 0.85 folklore default, yet also specifies/accepts an 85%-window local trigger. Implementation must decide whether 85% is merely an initial recipe/fixture or is overridden by calibrated_trim_fraction. (L5, L9, L14, L20, L51-53, L67-68)
- DJ001-76-023 [C|schema/status] The file snapshots post-J-66 SSB as approximately 269/578 on 2026-06-23 but requires edit-time re-grep; it is status evidence only because J-76 expects net-zero types. (L27, L61)

### 7. J-90_crosslang_dynamic_blast_cone_composition_gate.md — 85/85 lines

- DJ001-90-001 [E|status/dependency] J-90 is a large three-leg cone-composition build: L5 ships in Wave 1; L4/L7 mechanisms ship observe-only and enforcement waits for J-34. Dependencies are J-16, J-88 E2, J-56, and J-34. (L1-4)
- DJ001-90-002 [E|failure/capability] L5 closes J-16's blind spots where a static single-language cone misses Rust/TS Tauri, Rust/Python sidecar, or dynamic-dispatch dependents. (L6-8)
- DJ001-90-003 [E|integration] fuse_crosslang_cone combines all-language SCIP-enriched elai.map edges, cross_lang_diff seam pairs, and J-88 observed_calls into J-16 reverse dependencies. (L8, L31-32, L39-47)
- DJ001-90-004 [E|schema/integration] extract_typed_seams recognizes Tauri generate_handler! <-> TypeScript invoke() and sidecar route <-> Rust client boundaries already protected by golden locks. (L8, L40-41)
- DJ001-90-005 [E|failure/fallback] Opaque or unmappable seams always widen the cone or broaden the selected test suite and log what widened; neither an LLM nor runtime evidence may narrow it. (L8-9, L53-55, L76-77)
- DJ001-90-006 [E|non-goal] L5 does not build another cross-language graph and never lets an LLM infer missing edges. (L9)
- DJ001-90-007 [E|failure/capability] L4 addresses reachable-but-never-exercised branches that can pass a binary wiring smoke despite no representative user traffic. (L11-13)
- DJ001-90-008 [E|data/measurement] weighted_exercisedness scores each changed cone against a frozen corpus of journeys, incident repros, and boundary cases, and requires both a measured minimum weight and at least two scenario families. (L13-15, L42)
- DJ001-90-009 [E|test/fallback] select_safe_regression deterministically unions impacted, new, previously failing, and always-run canary tests; unsupported changes fall back to the full suite. (L13, L43)
- DJ001-90-010 [E|default/promotion] L4 score visibility ships observe-only; its minimum threshold is enabled only after J-34 measurement and cannot be a paper constant. (L14-15)
- DJ001-90-011 [E|failure/capability] L7 addresses truthful-but-vacuous tests that pass F-4's stdout-hash proof without meaningful assertions. (L17-19)
- DJ001-90-012 [E|test] assertion_validity_lint runs off-agent on every changed test and rejects print-only, no-exception-only, and self-referential oracles. (L19)
- DJ001-90-013 [E|test] changed_cone_mutation_floor composes C-28/I-20 over the J-16/L5 cone; passed=true additionally requires at least k killed mutants and no surviving critical mutant on changed lines. (L19)
- DJ001-90-014 [E|test] deterministic anti_vacuity heuristics are the third meaningfulness leg; no LLM judges test quality. (L19-21)
- DJ001-90-015 [E|lifecycle/config] Mutation is tiered: changed-line blocking in pull requests, changed-cone in merge queue, full nightly; k remains observe-only until J-34 calibration. (L20-21)
- DJ001-90-016 [E|constraint] Mutation is a lower bound, not an oracle; equivalent mutants remain residue and no paper-sourced floor is permitted. (L20-21, L79)
- DJ001-90-017 [E|status] Substrates are planned J-16/J-56, wired elai.map/cross_lang_diff/F-4/I-20, partial C-28, and same-wave J-88 runtime edges. (L23-29)
- DJ001-90-018 [E|event/integration] L5 reuses BlastRadiusEvent with CrossLangConeWidened/OpaqueSeamWidened; L4 reuses wiring evidence with optional UsageWeightExercised kind; L7 reuses TestIntegrityEvent/MutationBudgetedRun. Default schema impact is zero. (L31-34)
- DJ001-90-019 [E|flags/config] Flags are ELAI_CROSSLANG_BLAST_CONE_ENABLE, ELAI_USAGE_WEIGHTED_REACH_ENABLE, and ELAI_MUTATION_FLOOR_ENABLE; each is auditor-owned, frozen to the run, observe/default-off as stated, and requires a wire-dark allowlist row plus promotion plan. (L35)
- DJ001-90-020 [E|surface] Existing audit-timeline/blast-radius UI renders cone widening, weighted exercisedness, and mutation floor; cross-language UI changes require cross_lang_diff. (L36)
- DJ001-90-021 [E|reachability] Production proof occurs on AUDITING for a real fix touching a cross-language/dynamic dependent; tests/contracts alone never satisfy the contract. (L38-52)
- DJ001-90-022 [E|constraint/default] The blocking path forbids LLM gap filling/test judgment, cone shrinkage, one-family L4 passes, unmeasured thresholds, probabilistic selection, or BLOCK defaults before three-run FP bounds and allowlisting. (L53-60)
- DJ001-90-023 [E|acceptance] A Rust-to-Svelte seam must produce CrossLangConeWidened, include/run the Svelte regression, and block on pass-to-fail; a string-built seam produces logged OpaqueSeamWidened and broader tests. (L61-64)
- DJ001-90-024 [E|acceptance] A print-only test fails assertion lint; a critical changed-line mutant fails once enabled; one-family weighted evidence never passes; a mutation run lacking its corpus fails self-test rather than SKIP. (L62-64)
- DJ001-90-025 [E|measurement] L5 completion is a real dependent caught beyond the single-language cone; L4 completion is an ELAI score on J-56; L7 completion is a real vacuous-test rejection. No external benchmark is claimed. (L66-70)
- DJ001-90-026 [A|data/lifecycle] Frozen-corpus overfit is mitigated by a hidden holdout and incident-derived rotation on a separate cadence, but that cadence is not numerically specified. (L78)
- DJ001-90-027 [E|status] L4/L7 remain observable but non-blocking until measured; J-89 prevents at plan time and J-90 detects at audit time. (L73-80, L82-83)
- DJ001-90-028 [C|schema/status] The file snapshots SSB as approximately 268/577 on 2026-06-26 even though all three legs prefer event reuse; it is stale status evidence and cannot seed a later +Kind/type change. (L29, L71)

### 8. J-99_time_as_context_plane.md — 124/124 lines

- DJ001-99-001 [E|status/non-goal] J-99 is PLANNED Wave 1, net-zero SSB, and only the context plane: it supplies time facts for judgment while J-100 separately enforces time-as-control. (L1-7)
- DJ001-99-002 [E|capability] The complete context plane combines T-1 human-message time, T-2 within-turn agent/tool time, and T-3 cross-day resume time so elapsed context exists before the next action. (L9-10)
- DJ001-99-003 [E|risk/determinism] Highest risk is volatile time near the prompt prefix, which invalidates cloud cache bytes and makes RRA recompute a different clock; T-1 placement proof gates the file. (L12, L115-116)
- DJ001-99-004 [E|cache/config] TZ-1 adds a volatile TimeContext slot strictly after cache breakpoint 2 and immediately before OutputReserve, preserving prefix byte identity. (L13, L21-28)
- DJ001-99-005 [E|clock/data] TZ-2 sources human-facing time from real OsClock/system_now, never SeededClock, and records the rendered model-input bytes in the existing SSB action stream so replay uses exact captured bytes. SeededClock continues to govern gate/journal reproducibility. (L14)
- DJ001-99-006 [E|security/determinism] TZ-3 prohibits any rule, FSM transition, or promotion gate from branching on TimeContext; time is judgment context only. (L15)
- DJ001-99-007 [E|failure/recovery] IMP-10 routes a large elapsed delta into F-5/H-30/H-32 staleness re-verification, never into a model assumption that facts changed. (L16, L41)
- DJ001-99-008 [E|config/privacy] Timestamp rendering is minute-resolution local time when timezone is known, otherwise second-resolution explicitly labeled UTC/tz unknown; timezone is never fabricated. UTC-only remains a reversible privacy fallback. (L18-19, L118)
- DJ001-99-009 [E|status/dependency] Existing dependencies are wired ClockProvider/OsClock/SeededClock, wired ten-slot/two-breakpoint allocator, untimestamped conversation assembly, partial H-51 digest, and wired/partial staleness instruments. (L21-28)
- DJ001-99-010 [E|workflow/change-control] Before coding, the complete design, K-14 time rule, clock/allocator/conversation/handoff seams, and staleness instruments must be read/re-grepped; J-99 cites and obeys TZ-1..TZ-4 rather than re-authoring them. (L30-35)
- DJ001-99-011 [E|schema/workflow] T-1 adds ConversationMessage.created_at and renders absolute now, a day-rollover marker when dates differ, and time since the prior user message from two real wall-clock samples. (L37-38)
- DJ001-99-012 [E|constraint] T-1 forbids predicted next-message time, fabricated timezone, and relative statements not backed by recorded inputs. (L38, L61-66)
- DJ001-99-013 [E|workflow] T-2 refreshes the same volatile slot on every model re-entry within a tool/autonomous loop with elapsed-since-turn-start and elapsed-since-last-tool, including running-tool wording. (L39)
- DJ001-99-014 [E|workflow/lifecycle] T-3 extends H-51 resume with elapsed since last session computed from the last journaled row timestamp versus system_now, exposing a long gap before the model acts. (L40)
- DJ001-99-015 [E|evidence] J-99-decisions.md records timestamp granularity, the cache-placement smoke, and the H-51 header finding. (L42)
- DJ001-99-016 [E|integration] Produced surfaces are created_at, TimeContext, per-turn and agent-time blocks, and resume header; consumers are system_now/OsClock, 3E cache layout, conversation assembly, H-51, staleness tools, and K-14 rules. (L44-55)
- DJ001-99-017 [E|reachability] Production roots are a live prompt build, live multi-tool re-entry with monotonic advancement, and a live backdated resume; tests/contracts alone do not prove the legs. (L56-60)
- DJ001-99-018 [E|constraint] The block may not enter SystemPrompt/Rules/PhaseSpec or precede the final breakpoint; SeededClock may not source human time; no gate reads wall time; no unverifiable time phrasing or assume-changed route exists. (L61-66)
- DJ001-99-019 [E|surface] The block is visible in conversation/turn transcript; a dedicated dashboard tile is optional and requires either an edit-time decision or W-6 justification. (L67-68)
- DJ001-99-020 [E|event/default] No new action type is added: model-input bytes ride the existing action stream and T-3 reuses HandoffLifecycleEvent. (L69-72)
- DJ001-99-021 [E|acceptance] A two-turn day-boundary fixture proves correct day/delta strings after the breakpoint and an unchanged prefix hash; a placement-before-breakpoint negative arm must fail. (L69-72, L85-87, L99-100)
- DJ001-99-022 [E|acceptance] A three-tool fixture proves OsClock-sourced monotonic agent-time; a backdated resume proves the N-days header; a large delta proves re-verification rather than fabricated update. (L70-72, L88-91, L99-100)
- DJ001-99-023 [E|test] Deterministic checks cover wiring/production callers, production created_at/TimeContext/system_now sites, after-breakpoint placement, H-51 header, absence of SeededClock in conversation time, and a decisions artifact. (L74-83)
- DJ001-99-024 [E|done-gate] Acceptance includes cargo workspace green, off-agent orchestrator/contracts audit, net-zero schema, no new dependency, and live-trigger evidence rather than field existence. (L85-97)
- DJ001-99-025 [E|files] Planned edits are conversation.rs, context_budget.rs, handoff_digest.rs, a J-99 decisions record, and a dedicated approximately-120-line contract test. (L99-106)
- DJ001-99-026 [E|evidence/constraint] Temporal-blindness papers provide design rationale only; no external number becomes an ELAI claim or gate. (L108-113)
- DJ001-99-027 [A|privacy] If timezone privacy becomes a concern, the reversible local-if-known choice falls back to UTC-only. (L118)
- DJ001-99-028 [A|scope] If H-51 already exposes elapsed time, T-3 shrinks honestly to header rendering plus a contract test rather than padded implementation. (L119)
- DJ001-99-029 [E|dependency] J-99 informs model judgment; J-100 controls asynchronous progress/watchdogs; K-14 owns doctrine/package. (L121-122)
- DJ001-99-030 [E|flag/default] J-99 adds no feature flag; if an implementation introduces a kill switch, it must receive a wire-dark-allowlist row. (L94-96)
- DJ001-99-031 [C|schema/status] The file records an approximately-268/577 live baseline but adds no type; the dated number is informational and W-9 re-grep remains mandatory. (L28, L96, L110)

### 9. J-105_memory_roi_attribution_curation.md — 130/130 lines

- DJ001-105-001 [E|status/dependency] J-105 is PLANNED Wave 2 after J-21: a build extension adding per-entry causal ROI and a separately downgraded centrality rung; it does not copy sdl-mcp code or re-author J-21 defenses. (L1-7)
- DJ001-105-002 [E|capability/data] The residual G-7 gap is attribution at retrieved-entry grain: determine which specific memory caused success or failure, then persist proven-positive entries and demote/trim negative entries. (L9-13)
- DJ001-105-003 [E|evidence/constraint] External curated-memory gains, naive-accumulation degradation, poisoning, and 87.5% counterfactual results motivate the feature but are never ELAI thresholds or claims. (L11-13, L115-119)
- DJ001-105-004 [E|capability/default] G-3 code-graph centrality is not a default ranker: evidence is marginal/structural-only, so it is a default-off, observe-only tiebreaker for graph-shaped/architectural queries only. (L15)
- DJ001-105-005 [E|security/independence] The H-31 objectivity firewall is non-negotiable: auditor, critic, and adversary roles receive zero ROI-ranked or centrality-ranked memory. (L17)
- DJ001-105-006 [E|status/dependency] J-21 planned validation/quarantine/auth/provenance/decay is a prerequisite; E-5 memory substrate is partial; H-39/H-43 are partial; K-18 planned; C-45/C-47 credit is dark; H-31 partial; PageRank/B-29 and D-10 are wired. (L19-29)
- DJ001-105-007 [E|workflow/change-control] Mandatory reads cover the full J-105 reasoning section, research evidence as rationale, J-21 in full, live memory decay/retrieval code, and all signal/firewall producers; live seams are re-grepped at edit. (L31-36)
- DJ001-105-008 [E|algorithm/determinism] roi_attribution is off-agent and execution-grounded over a sealed completed session; for each retrieved entry it asks whether the gate-measured outcome would flip without that entry. (L38-39)
- DJ001-105-009 [E|data/integration] Attribution consumes H-43 execution fitness, C-45 per-entry credit, K-18 recurrence value, and H-39's correct=+1/abstain=0/confident-wrong=-t/(1-t) shape, reading the abstain bit from D-16 rather than re-deriving it. (L39)
- DJ001-105-010 [E|security/constraint] Neither an LLM judge nor model self-confidence nor mere retrieval occurrence can score memory ROI. (L39, L65-67)
- DJ001-105-011 [E|lifecycle/recovery] ROI drives J-21 activation decay: negative entries demote/trim faster and proven-positive entries persist; trim/demote reuses H-33 MemoryGovernanceEvent or J-21 MemoryValidationEvent without content in payloads. (L40)
- DJ001-105-012 [E|retrieval/config] retrieval.rs adds PageRank plus B-29 fan-in only as a tiebreaker, guarded by both the centrality flag and deterministic D-10 graph-query classification. (L41)
- DJ001-105-013 [E|test/security] A contract arm must prove an objectivity-role dispatch receives zero memory under either flag. (L42)
- DJ001-105-014 [A|event/schema] Per-entry journal evidence is optional: if selected, one Pattern-2 MemoryRoiAttributed carries only entry hash plus ROI score with full nine-tuple/golden lockstep and live-derived variant count; otherwise schema impact is zero through reused governance rows. (L43)
- DJ001-105-015 [E|evidence] J-105-decisions.md records the SSB choice, ROI-flip threshold decision, and centrality separation/observe-only result. (L44)
- DJ001-105-016 [E|interfaces] Producers are the ROI scorer, ROI-to-decay consumer, gated centrality rung, firewall enforcement, and optional event; consumers include J-21, H-43/H-39/K-18/C-45, memory storage/retrieval/decay, PageRank/B-29/D-10, H-31, and governance events. (L46-59)
- DJ001-105-017 [E|reachability] A live good/bad-memory session must attribute and curate entries; a live graph query demonstrates flag-on reranking and flag-off/non-graph inertness; a live auditor dispatch demonstrates the firewall. Tests/contracts alone are insufficient. (L60-64)
- DJ001-105-018 [E|privacy] No memory content may enter any SSB payload; optional attribution records only a hash and score. (L69-70)
- DJ001-105-019 [E|flags/default] ELAI_MEMORY_ROI_ATTRIBUTION_ENABLE and ELAI_MEMORY_CENTRALITY_RERANK_ENABLE both default off and each needs an experiment allowlist row with owner/expiry/smoke and promotion_plan_id J-105-promotion; centrality remains observe-only longest. (L68, L72, L113)
- DJ001-105-020 [E|promotion] No default-on promotion occurs until attribution accuracy and recovery from naive-accumulation degradation are bounded over at least three real runs. (L72)
- DJ001-105-021 [E|surface] Trim/demote provenance appears on the existing H-33 Memory Hub/governance surface; a dedicated ROI tile is optional with edit-time decision or W-6 justification. (L73-74)
- DJ001-105-022 [E|acceptance] Known-bad memory must receive negative ROI and be demoted/trimmed while known-good persists; measured ROI-driven curation must improve outcome against naive keep-all using ELAI's own accuracy. (L75-78, L90-92)
- DJ001-105-023 [E|acceptance] Centrality-off must show no regression; centrality-on is measured separately and fires only for graph queries; no lift is banked without evidence. The firewall always yields zero injection. (L76-78, L93-96)
- DJ001-105-024 [E|determinism/test] Contract tests require deterministic attribution across ELAI_SEED reruns, no model judge, no content-bearing payload, and action_type_lock only if the optional event is chosen. (L94-97, L104-105)
- DJ001-105-025 [E|done-gate] Done is live negative-ROI curation, not scorer existence; cargo workspace, elai-memory/orchestrator/contracts auditor, flags/allowlist, live schema re-grep, and production-caller checks must pass with no new dependency. (L80-102)
- DJ001-105-026 [E|files/config] Create roi_attribution.rs and decisions/contract artifacts; modify decay_scheduler.rs and retrieval.rs; touch SSB split/golden only for the optional event; add both allowlist rows. (L104-113)
- DJ001-105-027 [E|performance/fallback] Counterfactual cost is controlled by evaluating only retrieved entries, reusing C-45 credit, and running off-agent after session sealing rather than on the hot path. (L121-122)
- DJ001-105-028 [A|config/lifecycle] ROI flip threshold is tunable; current reversible decision demotes first and trims only after repeated negative evidence across N sessions, with N unspecified and trim observe-only until its rate is bounded. (L123)
- DJ001-105-029 [E|risk] Any firewall leak is P0; centrality overclaim is mitigated through a distinct measurement arm, graph-query restriction, and longest observe-only duration. (L124-125)
- DJ001-105-030 [E|dependency] J-105 composes J-21/K-18/H-43/H-39/C-45/PageRank/B-29/D-10, and pairs with J-104 so centrality reads a fresh graph. (L127-128)
- DJ001-105-031 [C|measurement/config] The ROI decision invokes Xiong's roughly-20%-noisy result as a trim target while ID-5 elsewhere forbids external numbers as gates; N and the actual flip/trim threshold therefore require ELAI measurement rather than literal adoption. (L117, L119, L123)
- DJ001-105-032 [C|schema/status] The optional event text snapshots SSB as approximately 268/577 but repeatedly requires live W-9 derivation; only the latter controls whether net-zero reuse or +1 is valid. (L29, L43, L101, L118)

### 10. J-112_spoken_render_narration_dispatcher.md — 153/153 lines

- DJ001-112-001 [E|status/dependency] J-112 is PLANNED, medium VOICE-1, build-on-shipped NotificationDispatcher, ships first, and is a prerequisite for J-111; it intentionally has no proof-spine classification. (L1-4, L19-25)
- DJ001-112-002 [E|capability/surface] The harness speaks completion, needs-input, step-grain progress, audit/cost/gate state, and on-demand cross-session status as an audio rendering of journal truth. (L6-12, L53-58)
- DJ001-112-003 [E|determinism] Event-path narration is a deterministic per-family template over typed SSB fields; no model authors those facts. (L7-13)
- DJ001-112-004 [E|model-boundary] The only model slot is on-demand digest phrasing over typed session digests, with receipt rows and a harness-computed footer appended outside model text. (L9-12)
- DJ001-112-005 [E|non-goal/integration] The unit may not create a second dispatcher; it adds a spoken target to H-36 NotificationDispatcher and migrates into N-7 render(verdict,target=spoken) if/when N-7 ships. (L13-17)
- DJ001-112-006 [E|identity/privacy] There is one supervisor voice plus one system-earcon set, never per-session voices. Personality is deterministic phrasing, not factual authority. (L14, L33-37)
- DJ001-112-007 [E|determinism] All template variation is seeded from row id through EntropySource; thread_rng/unseeded variance is prohibited. (L15, L44)
- DJ001-112-008 [E|status/dependency] Existing states are shipped NotificationDispatcher/agent_notification_emitted and entropy quarantine, planned N-7/E-3, and planned/partial H-51. (L39-44)
- DJ001-112-009 [E|change-control] Mandatory prereads are the voice grounding map, notify_dispatch.rs in full, H-36 and N-7, E-3 tasks 1/8, and hard rules for flags, production hooks, recomputability, deterministic model boundaries, unwrap, and dependency names. (L46-51)
- DJ001-112-010 [E|schema/capability] spoken_render.rs declares completion, needs-input, phase/step progress, audit verdict, cost ceiling, and gate-block render families as a pure render_spoken(row)->String table keyed by typed fields. (L53-54)
- DJ001-112-011 [E|config/determinism] Each family has two or three row-id-seeded variants and voice-anchor TOML phrasing constants, using an E-3-compatible fixture until the real anchor ships. (L54)
- DJ001-112-012 [E|failure/type-safety] A narratable family lacking a declared renderer is a compile-time exhaustiveness error rather than a silent skip. (L54)
- DJ001-112-013 [E|integration/event] The spoken target mirrors OS notify, calls render_spoken directly, and emits/reuses agent_notification_emitted with target=spoken; only a required new Kind may make schema impact +1. (L55)
- DJ001-112-014 [E|flag/default] ELAI_VOICE_ENABLE is shared with J-111, defaults off, and its complete R9 allowlist row is authored by J-112 as wave owner. (L55, L113, L122)
- DJ001-112-015 [E|queue/concurrency] Narration priority is needs-input above completion above progress; needs-input preempts queued progress. (L56, L65-67)
- DJ001-112-016 [E|rate/config] Progress is rate-limited and duplicate-coalesced at (session, step-id), never per tool call; silent/summary/verbose are deterministic row-class filters. (L56, L65-67)
- DJ001-112-017 [A|dependency/permission] Local TTS prefers Kokoro-82M with Piper fallback, but dependency identity must pass anti-slopsquatting and receive user confirmation at edit. (L57)
- DJ001-112-018 [E|fallback] If new TTS dependencies are declined, OS-native SAPI/say is the degraded leg and receives exactly the same rendered text. (L71-73)
- DJ001-112-019 [E|surface/integration] Local synthesis uses the localhost socket shared with J-111 and one voice/earcon set; the production status entry is elai voice --status until J-32 can own the chat/status lane. (L57-58)
- DJ001-112-020 [E|data/evidence] Every digest claim carries a row-id receipt; deterministic counts for sessions/events/pending decisions are computed from rows and spoken last, so omitted-body state contradicts the footer visibly/audibly. (L58, L68-70)
- DJ001-112-021 [E|fallback] If H-51 digest artifacts are absent, status deterministically folds the per-session journal tail; it may cost more tokens but preserves receipt rules and may not degrade to silence-as-success. (L74-75, L126)
- DJ001-112-022 [E|evidence] J-112-decisions.md records render-table-not-LLM, one voice/earcons, footer outside model text, N-7 migration, and the shared flag. (L59)
- DJ001-112-023 [E|acceptance/failure] MOVE 1 requires row -> render -> notification payload -> TTS input byte equality; divergence identifies an alternate authoring/lie channel and is repaired by direct renderer invocation. (L61-64)
- DJ001-112-024 [E|acceptance/failure] MOVE 2 requires needs-input preemption, exact verbosity suppression, and step-grain rate; progress flooding is recovered through session/step limiting plus duplicate coalescing. (L65-67)
- DJ001-112-025 [E|acceptance/failure] MOVE 3 requires receipt-bound model claims and a footer computed from SSB rows; deriving the footer from model output is forbidden self-grading. (L68-70)
- DJ001-112-026 [A|workflow] Edit-time forks choose direct N-7 target if available, real E-3 voice.toml if available, and confirmed local TTS versus OS-native fallback. (L71-73)
- DJ001-112-027 [A|recon] The live action-type list must settle narratable progress/audit/cost families, and live H-51 shape must settle digest artifact versus journal-tail input. (L74-75)
- DJ001-112-028 [E|abort/security] Abort on any utterance not originating in render_spoken or a receipt-bound digest, any second dispatcher after N-7, or any per-session voice differentiation. (L76)
- DJ001-112-029 [E|evidence/residue] Off-agent proof consists of rows, byte-equality assertions, checker exits, status/completion smoke, and the later J-111 end-to-end smoke. TTS mispronunciation remains honest residue: text is canonical and dashboard-visible. (L77-79)
- DJ001-112-030 [E|interfaces] Produced artifacts are spoken_render, spoken target, queue/levels, CLI status, and Python TTS; consumers are NotificationDispatcher triggers, SSB read side, E-3 shape, and H-51 or journal-tail fallback. (L81-89)
- DJ001-112-031 [E|reachability] Production paths are dispatch_stop/RequestHuman -> NotificationDispatcher -> spoken -> TTS and CLI elai voice --status -> digest; contracts-only callers do not prove wiring. (L90-93)
- DJ001-112-032 [E|constraint] No event-path model call, rowless utterance, model-derived footer, receiptless digest claim, post-N-7 second dispatcher, per-session voice, random variance, or production unwrap is allowed. (L94-97)
- DJ001-112-033 [E|surface] NotificationManager adds a minimal read of target=spoken rows; H-36 remains responsible for its own needs-input no-op gap. (L98-100)
- DJ001-112-034 [E|acceptance] With ELAI_VOICE_ENABLE=1, completion, RequestHuman, and status must yield backing rows, byte-equal payload/TTS text, receipts, and deterministic footer; rowless text, footer/body mismatch, or sub-step progress must fail. (L101-104)
- DJ001-112-035 [E|test] Checks require wiring run-trigger, render/target symbols, zero thread_rng/random calls, zero generate/complete/chat calls in spoken_render, production caller, allowlist row, and decisions artifact. (L106-115)
- DJ001-112-036 [E|test] Contract coverage includes rowless utterance unconstructibility, preemption, rate limit, exact row-class filters, seed stability, row-derived footer mismatch, and H-51-to-journal fallback. (L117-126)
- DJ001-112-037 [E|files/schema] Planned changes create voice/spoken_render.rs, Python TTS, contract and decisions files; modify NotificationDispatcher, CLI, and allowlist; SSB split/golden changes occur only if a new Kind is required. (L128-136)
- DJ001-112-038 [A|performance] Long TTS summaries should be sentence-chunked and streamed; synthesis must never block NotificationDispatcher. (L143-145)
- DJ001-112-039 [A|evolution] If later E-3 persona work adds model phrasing, it remains display-only and the byte-equality boundary moves to pre-normalizer text; that seam must be pre-recorded. (L146)
- DJ001-112-040 [E|dependency/status] J-112 delivers spoken alert/status alone, completes hands-free operation with J-111, and becomes a general render target with N-7. (L148-151)

## Explicit interaction and lifecycle map

- Doctrine propagation: the grounding map supplies DG-1..DG-20 and JD/W/GD constraints to J-43; J-43 then supplies the property-floor/ledger consumed by J-44/J-45/J-46/J-47. Relevant atoms: DJ001-G-003..DJ001-G-035 and DJ001-43-001..DJ001-43-024.
- Usage lifecycle: provider MessageStart/MessageDelta or estimator -> NormalizedUsage -> one UsageTurnEvent at the lane seam -> usage_ledger fold -> CLI report -> J-29 window. Compaction adds later input without resetting totals. Relevant atoms: DJ001-28-003..DJ001-28-015 and DJ001-28-027..DJ001-28-029.
- Research lifecycle: model query/rank -> harness parallel search -> QueryAwareExtractor verbatim spans -> external SessionDag state -> bounded synthesis -> J-41 closed-set grounding. Relevant atoms: DJ001-40-002..DJ001-40-016.
- Temporal enforcement lifecycle: J-69 chain verification -> committed-row ptLTL recurrences and completion-time obligation rewrite -> SsbMonitorVerdict/reused I-35 row -> FSM block or J-67 typed action. Relevant atoms: DJ001-66-003..DJ001-66-024.
- Context lifecycle: J-34 measured curves -> capped/head-tail retrieval through 3E -> H-50 local compaction at the selected trigger -> ContextCalibration/Handoff evidence; cloud stays mask-only. Relevant atoms: DJ001-76-002..DJ001-76-020.
- Blast/meaningfulness lifecycle: J-16 static cone + cross-language seams + J-88 runtime edges -> widened cone -> deterministic representative selector -> regression tests; J-56 weighted evidence and C-28/I-20 mutation/lints add observe-only L4/L7 until J-34 promotion. Relevant atoms: DJ001-90-002..DJ001-90-027.
- Time-context lifecycle: OsClock samples -> post-breakpoint TimeContext -> exact model-input bytes in existing SSB -> staleness re-verification on large gaps; multi-tool re-entry and H-51 resume use the same plane, while J-66/TZ-3 forbids gate use. Relevant atoms: DJ001-99-003..DJ001-99-022.
- Memory lifecycle: J-21-admitted retrieved entries -> sealed-session counterfactual ROI using H-43/H-39/C-45/K-18 -> J-21 decay/trim -> H-33/J-21 governance evidence; D-10 optionally admits a PageRank/fan-in tiebreaker and H-31 filters objectivity roles. Relevant atoms: DJ001-105-006..DJ001-105-025.
- Voice lifecycle: backing SSB row -> seeded render_spoken -> existing NotificationDispatcher target -> priority/rate queue -> local TTS and NotificationManager; status instead uses receipt-bound H-51/journal-tail digests plus an external deterministic footer. Relevant atoms: DJ001-112-002..DJ001-112-036.
- Explicit cross-chunk dependency within this chunk: J-99 invokes J-66/ND-1 to prohibit wall-clock gates (DJ001-99-006, DJ001-99-018); J-28 and J-76 both extend H-50 compaction but at different concerns (usage accounting versus context construction); J-112 ships before and is consumed by J-111.

## Bounded inferences (not source assertions)

- DJ001-I-001 [I|integration] Because J-28's T7 and J-76's local recipe observe the same compaction boundary through H-50, their implementation should expose one authoritative transition identity so token-growth proof and rebuilt-context proof cannot describe different compactions. This follows from DJ001-28-014/DJ001-28-043 and DJ001-76-006/DJ001-76-012; neither source explicitly mandates a shared identifier.
- DJ001-I-002 [I|governance] Once J-43 conditional routing ships, implementation work for J-66/J-90 and any later voice/memory gate changes should be scanned by the doctrine floor because those tasks ship or audit features/gates. This is the bounded consequence of DJ001-G-008 and DJ001-43-006, not an explicit per-plan edit list.
- DJ001-I-003 [I|integrity] J-112's “narrator cannot lie” claim is strongest only when the backing journal rows themselves have passed the integrity guarantees described elsewhere by J-69/J-66. J-112 requires backing rows and byte equality but does not explicitly declare J-69 as a dependency.

## Ambiguous, conditional, and unresolved ledger

- Grounding verification debt: DJ001-G-010 and DJ001-G-018 are PLAN-asserted seams requiring edit-time confirmation.
- J-28 edit-time choices/recon: DJ001-28-006, DJ001-28-033, DJ001-28-041, DJ001-28-042, DJ001-28-043, and DJ001-28-044 cover normalization locus, timestamp source, live schema, event seams, and the single compaction-accounting owner.
- J-40 measured unknowns: DJ001-40-017 and DJ001-40-018 leave embedding latency and supporting-span recall budgets open.
- J-43 policy unknowns: DJ001-43-021 and DJ001-43-022 leave ledger location and durable model-family lookup unsettled.
- J-66 implementation choices: DJ001-66-014, DJ001-66-015, and DJ001-66-021 cover I-35 event reuse, first-slice UI, and cross-session past scope.
- J-76 implementation choices: DJ001-76-014, DJ001-76-019, and DJ001-76-020 cover cross-language threading/UI, missing-curve fallback, and the unspecified deterministic numeric-preserving compactor.
- J-90 cadence: DJ001-90-026 requires holdout rotation but does not define frequency.
- J-99 reversible choices: DJ001-99-027 and DJ001-99-028 cover UTC-only privacy and a potentially trivial H-51 extension.
- J-105 choices: DJ001-105-014 and DJ001-105-028 cover optional SSB attribution and the unspecified repeated-negative N/ROI thresholds.
- J-112 permission/recon/evolution: DJ001-112-017, DJ001-112-026, DJ001-112-027, DJ001-112-038, and DJ001-112-039 cover TTS approval/fallback, live N-7/E-3/H-51/action-family state, streaming latency, and a future display normalizer boundary.

## Conflicts and stale assertions requiring reconciliation

- DJ001-G-023, DJ001-28-031, DJ001-40-022, DJ001-43-026, DJ001-66-025, DJ001-76-023, DJ001-90-028, DJ001-99-031, and DJ001-105-032: schema snapshots 234/543, 268/577, approximately 269/578, and later observed 271 are time-stamped plan evidence, not compatible constants. All event-bearing implementations must derive current values from live SSB state.
- DJ001-28-018: many June line anchors were superseded by July file movement and line drift. The semantic findings survived, but old anchors must not drive edits.
- DJ001-76-022: “measure the own curve / reject 0.85 folklore” conflicts with a literal 85% local trigger unless the latter is explicitly labeled an initial recipe and later replaced by calibration.
- DJ001-105-031: external roughly-20%-noisy evidence may motivate a fixture but cannot supply the ROI trim threshold under the same file's ID-5 rule.
- No other direct contradiction was found. Remaining alternatives are conditional choices enumerated above, not simultaneous requirements.

## Summary-layer additions and corrections

The comparison target is 10-J-SUMMARY_INDEX.discovery.md only. “Addition” means explicit source material absent from that orientation summary; “correction” means the summary wording would be unsafe if treated as the complete contract.

### Grounding map deltas

- ΔG-ADD-1: The orientation summary merely enumerated this grounding map and explicitly did not read it. This deep read adds all DG-1..DG-20 seam/status contracts, JD-1..JD-4, computed SSB policy, and the exact J-43..J-47 decomposition (DJ001-G-001..DJ001-G-035).
- ΔG-CORR-1: DG-8 and DG-16 were only PLAN-asserted in the grounding session, not verified files; downstream users must preserve the edit-time confirmation qualifier (DJ001-G-010, DJ001-G-018).

### J-28 deltas

- Δ28-ADD-1: Adds the exact NormalizedUsage shape, identity default, three event kinds, retention of CloudSpend/BackendCostAccrued, five-key lane rollup, blake3 estimator cache, and “ships on/no new env gate” rule (DJ001-28-005..DJ001-28-013, DJ001-28-026).
- Δ28-ADD-2: Adds SSB module-split destinations, exact production roots, secrets/unwrap/hash bans, Tier-2 remote/hardware boundary, three forks, four recon gates, four aborts, serial -j 1 auditor policy, and the compaction-crossing integration smoke (DJ001-28-019..DJ001-28-025, DJ001-28-041..DJ001-28-047).
- Δ28-CORR-1: The summary's high-level schema statement omits that historical baselines/anchors are stale: current implementation must use split SSB files and live-derived baseline rather than any documented number (DJ001-28-018, DJ001-28-031, DJ001-28-048).
- Δ28-CORR-2: “Account for compaction” means one—and only one—owner selected by FORK-3, plus equality between T7 growth and T5 per-lane session sum across a real compaction boundary (DJ001-28-014, DJ001-28-039, DJ001-28-043, DJ001-28-047).

### J-40 deltas

- Δ40-ADD-1: Adds semantic-overlap sentence scoring, exact FAST/STANDARD/DEEP breadth/depth semantics, SessionDag hit/coverage serialization, optional research panel/cross-language check, optional ResearchModeEvent policy, production roots, explicit offline cascade, and the dated schema qualifier (DJ001-40-003, DJ001-40-006..DJ001-40-007, DJ001-40-012..DJ001-40-016, DJ001-40-021..DJ001-40-022).
- Δ40-CORR-1: The summary's “keep about 200 verbatim tokens per source” must not be read as a fixed ELAI budget. Roughly 200 and greater-than-50:1 are external Perplexity rationale; ELAI selects a measured per-source budget that preserves supporting-span recall (DJ001-40-004, DJ001-40-018).

### J-43 deltas

- Δ43-ADD-1: Adds every DD-1..DD-5 exact detector/severity, the full debt-row schema/marker match, auto-load condition, manifest files, optional event kinds, and real seeded positive/negative/missing-target proof (DJ001-43-005..DJ001-43-020).
- Δ43-ADD-2: Adds the provisional external FP ranges, Rust weak leg, unresolved ledger location, model-family mapping risk, DD-4 no-promotion rule, implausible-zero debt warning, recognized-governance/unproven-harness split, and dated schema qualifier (DJ001-43-004, DJ001-43-021..DJ001-43-026).
- Δ43-CORR-1: No direct summary error; the source narrows “floor check” to necessary-not-sufficient structural tells and explicitly denies that PASS establishes doctrine conformance (DJ001-43-019, DJ001-43-023).

### J-66 deltas

- Δ66-ADD-1: Adds exact G1/G2/Oblig1/Oblig2 formulas, O(1) committed-row recurrence, completion transition, feasibility registration gate, three-run/allowlist promotion, parent-session scope risk, and monitorability routing (DJ001-66-003..DJ001-66-012, DJ001-66-018..DJ001-66-023).
- Δ66-CORR-1: The summary names the three verdict kinds as if emission were fixed; the source first prefers reuse of any I-35 action type and only otherwise adds SsbMonitorVerdict. UI is conditional and 268/577 is only a dated snapshot (DJ001-66-014, DJ001-66-015, DJ001-66-025).

### J-76 deltas

- Δ76-ADD-1: Adds the exact SproutRAG deferred shape, integration ownership, zero-event reuse, conditional cross-language pair, allowlist rule, external-number caveats, deterministic-extraction requirement for numerics, and the dated approximately-269/578 qualifier (DJ001-76-008, DJ001-76-012..DJ001-76-020, DJ001-76-023).
- Δ76-CORR-1: The summary's simple 85% trigger hides an internal calibration tension. The source simultaneously demands own-curve thresholds and rejects 0.85 folklore, so implementation must classify 85% as an initial recipe or replace it with calibrated_trim_fraction (DJ001-76-022).

### J-90 deltas

- Δ90-ADD-1: Adds exact typed seam extractors, deterministic regression-set union/full-suite fallback, assertion lint anti-patterns, PR/merge/nightly mutation tiers, three environment flags/allowlist requirements, reused event kinds, no-corpus self-test, holdout rotation, and the dated schema qualifier (DJ001-90-004, DJ001-90-009, DJ001-90-012..DJ001-90-020, DJ001-90-024, DJ001-90-026, DJ001-90-028).
- Δ90-CORR-1: No direct summary error; “opaque seams widen” additionally requires a logged broader-test fallback, never only a wider graph, and runtime edges are forbidden from shrinking (DJ001-90-005).

### J-99 deltas

- Δ99-ADD-1: Adds the exact post-breakpoint slot position, dual-clock rule, local/UTC granularity/privacy choice, full per-turn strings, mandatory prereads, transcript surface, no-new-event/no-new-flag policy, negative placement arm, production checks, H-51 shrink-if-existing fallback, and the dated schema qualifier (DJ001-99-004..DJ001-99-019, DJ001-99-020..DJ001-99-031).
- Δ99-CORR-1: “created_at from OsClock” is incomplete unless paired with exact-byte journaling and the continued use of SeededClock for gate/journal determinism; recomputing OsClock during replay is forbidden (DJ001-99-005).

### J-105 deltas

- Δ105-ADD-1: Adds the exact H-39 scoring shape/signals, off-agent sealed-session boundary, decay/event payload rules, two named flags plus promotion row, production reachability, deterministic ELAI_SEED test, file/config changes, demote-first/trim-after-N decision, and the dated schema qualifier (DJ001-105-008..DJ001-105-020, DJ001-105-024..DJ001-105-028, DJ001-105-032).
- Δ105-CORR-1: The summary correctly says paper numbers are not imported, but the source later calls the roughly-20%-noisy result a trim target. ID-5 controls: it can motivate a fixture, not set the threshold (DJ001-105-031).
- Δ105-CORR-2: MemoryRoiAttributed is optional, not a guaranteed new row; reused H-33/J-21 rows are the preferred net-zero path (DJ001-105-014).

### J-112 deltas

- Δ112-ADD-1: Adds exact narratable families, compile-time render exhaustiveness, row-seeded variants, one voice/earcons, shared default-off flag, queue order/coalescing, user-confirmed TTS plus OS-native fallback, N-7/E-3/H-51 forks, nonblocking sentence streaming, and future pre-normalizer boundary (DJ001-112-006..DJ001-112-022, DJ001-112-026..DJ001-112-039).
- Δ112-ADD-2: Adds exact byte-equality failure diagnosis, footer self-grading prohibition, rowless-unconstructible and missing-digest fallback tests, production roots, and the residual distinction between correct text and TTS mispronunciation (DJ001-112-023..DJ001-112-036).
- Δ112-CORR-1: The summary's event list should not imply a new SSB type: target=spoken normally reuses agent_notification_emitted and schema changes only if a new Kind proves necessary (DJ001-112-013, DJ001-112-037).

## Facet coverage audit

| Source | Capabilities/actions/surfaces | Workflow/state + data/lifecycle | Flags/config + integrations/events | Security/privacy/compliance + failure/recovery | Determinism/concurrency + observability/evidence | Tests/done + constraints/non-goals + dependency/status |
|---|---|---|---|---|---|---|
| Grounding map | DG seam/tool/rule inventory | JD lifecycle and five DP packages | conditional routing, ledger, optional event | off-agent authority, freeze, fail-closed debt | computed schema, recorded-state metrics, cap-3 harness | real-session JD-1; verify PLAN assertions; no new doctrine |
| J-28 | normalized usage, CLI report | split SSE merge, rollup, estimator, compaction | no flag; UsageTurnEvent + retained rows | no secrets/silent zeros; explicit forks/aborts/recovery | lane semantics, live-derived schema, serial auditor | dual-lane multi-turn smoke; no naive sum/rates/remote Tier-1 |
| J-40 | extractor, modes, live search | parallel fetch, external step state | mode config, QueryTrace/optional event | raw bytes excluded; remote cascade | verbatim extraction, compression evidence | context-slot absence proof; no generation/KV/Search-as-Code default |
| J-43 | rule, floor, debt registry | observe -> measured promote | routing, manifest, optional debt event | auditor ownership, frozen self-edit, unknown fail | mechanical DD classes and rate evidence | seeded positive/negative/missing proof; comprehensive scorer excluded |
| J-66 | ptLTL/rewrite monitors | per-row recurrences, completion obligations | predicate registry, optional/reused verdict | allow/declass policies, tamper refuse | O(1), no LLM/wall clock, total order | violated/clean/feasibility arms; F-4 rebuild excluded |
| J-76 | cap, placement, local compactor | measured curve -> local rebuild | curve config, reused events, optional UI/pair | conservative fallback; cloud unchanged | cache-safe static order; numeric extraction | long-session/retrieval proof; external thresholds/SproutRAG excluded |
| J-90 | widened cone, score, meaningfulness | audit-time selection and tiered mutation | three flags, reused events/UI | widen-only, full-suite fallback | deterministic selection/lint/mutation | seam/vacuity/corpus tests; unmeasured blocking excluded |
| J-99 | TimeContext, message/resume surface | turn/reentry/resume time | granularity/privacy choice; reused events | no gate time; stale fact reverify | dual clocks, exact-byte replay, prefix stability | day/tool/resume negatives; no predicted/fabricated time |
| J-105 | ROI scorer/curator/centrality surface | sealed outcome -> decay/trim | two flags, reused/optional event | objectivity firewall, content-free payload | off-agent deterministic score, ELAI_SEED proof | good/bad/noisy/firewall arms; no LLM judge/default centrality |
| J-112 | spoken render/target/status/TTS | row -> queue -> synthesis/digest | shared flag, verbosity/voice, reused/optional Kind | receipts/footer, dependency approval, aborts | seeded variants, byte equality, nonblocking queue | completion/input/status negatives; no second dispatcher/model event narration |

All twelve requested discovery facets were checked against every file. A blank facet was not silently inferred: conditional or absent behavior is recorded as [A], a constraint/non-goal, or an explicit no-new-event/no-flag result.

## Quantitative closeout and zero-omission/duplicate audit

| Ledger | Source lines read | Atomic declarations | [E] | [A] | [C] | [I] |
|---|---:|---:|---:|---:|---:|---:|
| J-DOCTRINE-PROPAGATION-GROUNDING-MAP | 116/116 | 35 | 32 | 2 | 1 | 0 |
| J-28 | 411/411 | 49 | 41 | 6 | 2 | 0 |
| J-40 | 52/52 | 22 | 19 | 2 | 1 | 0 |
| J-43 | 64/64 | 26 | 23 | 2 | 1 | 0 |
| J-66 | 74/74 | 25 | 21 | 3 | 1 | 0 |
| J-76 | 71/71 | 23 | 18 | 3 | 2 | 0 |
| J-90 | 85/85 | 28 | 26 | 1 | 1 | 0 |
| J-99 | 124/124 | 31 | 28 | 2 | 1 | 0 |
| J-105 | 130/130 | 32 | 28 | 2 | 2 | 0 |
| J-112 | 153/153 | 40 | 35 | 5 | 0 | 0 |
| Direct-source total | 1280/1280 | 311 | 271 | 28 | 12 | 0 |
| Bounded cross-source inference | n/a | 3 | 0 | 0 | 0 | 3 |
| Grand atomic total | 1280 source lines | 314 | 271 | 28 | 12 | 3 |

- File coverage: 10/10 prescribed files; 1/1 auxiliary and 9/9 numbered originals; 1,280/1,280 lines; 158,205/158,205 bytes; ten of ten SHA-256 values match.
- Structural token coverage: DG-1..DG-20, JD-1..JD-4, J-28 T1..T8/FORK-1..3/RECON-1..4/ABORT-1..4, J-43 DD-1..DD-5, J-66 G1/G2/Oblig1/Oblig2, J-76 C6/C7/C8, J-90 L5/L4/L7, J-99 T-1/T-2/T-3/TZ-1..TZ-3/IMP-10, J-105 G-7/G-3, and J-112 Tasks 1..6/Moves 1..3 were each dispositioned exactly once in a primary source atom.
- Evidence-separation audit: 271 explicit atoms, 28 ambiguous/conditional atoms, 12 conflict/stale atoms, and 3 bounded inference atoms. Conflict IDs are DJ001-G-023, DJ001-28-018, DJ001-28-031, DJ001-40-022, DJ001-43-026, DJ001-66-025, DJ001-76-022, DJ001-76-023, DJ001-90-028, DJ001-99-031, DJ001-105-031, and DJ001-105-032.
- Unique-ID audit: every DJ001-* atomic ID occurs once as a primary ledger declaration; later interaction/delta sections cite those IDs but do not declare or recount them.
- Semantic duplicate audit: repeated WIRING_CONTRACT, ACCEPTANCE, WARGAME, and “Next” restatements were merged into their originating behavior atom. No repeated source assertion received a second atomic ID.
- Summary-delta audit: 25 uniquely labeled additions/corrections cover all ten files; the orientation summary contributes zero direct-evidence atoms.
- Omission audit result: zero undispositioned manifest files, zero unread source lines, zero missing named task/gate/fork/abort families, zero duplicate primary IDs.

## Output integrity

The canonical output SHA-256 below hashes this file from its first byte through the newline immediately before the checksum line; the checksum line is excluded to avoid a self-referential digest.
- Canonical output SHA-256: ca6d1270a50c817702f851bd693e9ef06238dd06739809b68a33a0c2b7d57186
