# DEEP-J-002 exhaustive feature discovery

## Scope, evidence, and audit method

- Assigned manifest section: `DEEP-SOURCE-MANIFEST.md` lines 892-907, headed `DEEP-J-002`.
- Direct evidence: the ten files in the processed-manifest table below. Every physical line of every assigned file was read; the auxiliary grounding map is treated as first-class evidence.
- Orientation only: `10-J-SUMMARY_INDEX.discovery.md`, specifically its J-22/J-27/J-36/J-44/J-52/J-93/J-94/J-106/J-109 entries and cross-cutting interaction ledgers. It was used only to measure summary additions/corrections, never as evidence for a source-ledger finding.
- No other numbered J original, grounding map, reasoning source, code file, or adjacent deep-discovery chunk was opened. A neighboring discovery artifact was consulted only for output-layout convention; it is not source evidence.
- Evidence tags: `[E]` explicit in the named source; `[I]` bounded cross-source inference; `[A]` explicit ambiguity/choice/reconciliation item; `[C]` conflict, stale assertion, or material summary mismatch.
- Facet tags cover capabilities/actions/surfaces; workflows/states; schemas/data/lifecycle; flags/defaults/config; integrations/events; permissions/security/privacy/compliance; failures/fallback/retry/recovery; determinism/idempotency/concurrency; observability/evidence; tests/acceptance/done gates; constraints/non-goals; and dependencies/status/deferred.
- Atomic IDs are unique within this artifact. Repeated acceptance, grounding, `COMPOSES-WITH`, and file-list language is folded into the first defining atom rather than double-counted.

## Exact processed manifest and integrity verification

| Ordinal | Class | Lines declared/actual | Bytes declared/actual | SHA-256 declared/actual | Exact absolute path | Result |
|---:|---|---:|---:|---|---|---|
| 1 | auxiliary/grounding/fixture | 167 / 167 | 15553 / 15553 | 93be94ae51e434ad135f4350b34d5150b8eadd4232764607fda238fa031a629e / same | docs/thefinalglance/J-plan/_grounding/J-GRC-V3-GROUNDING-MAP.md | MATCH |
| 2 | numbered/index original | 68 / 68 | 9572 / 9572 | 6e81b60bdb1d0538d530ed16ea0a753fca1b17fdd56aa2020c3acdd6159e1dae / same | docs/thefinalglance/J-plan/J-22_journal_payload_schema_lock.md | MATCH |
| 3 | numbered/index original | 128 / 128 | 8968 / 8968 | 4234c551973f8bd86560ed6f39ec2869ac9916da6e05d49c7c4dddf8912b257c / same | docs/thefinalglance/J-plan/J-27_harness_protected_blocks_webcache_dispatch.md | MATCH |
| 4 | numbered/index original | 266 / 266 | 20212 / 20212 | 15743fc53d887d83602458878467f32199b0284095129d5559bfe4ca69a55717 / same | docs/thefinalglance/J-plan/J-36_freeze_head_local_prefix_kv_reuse.md | MATCH |
| 5 | numbered/index original | 59 / 59 | 13429 / 13429 | 2c04f6012404e6c05bec7bd4685d3d3530d22b837dc7cc60b664cc3d632c20b8 / same | docs/thefinalglance/J-plan/J-44_reasoning_method_artifact_shape.md | MATCH |
| 6 | numbered/index original | 136 / 136 | 25682 / 25682 | 4e5abfb374146890e421759970c98afb23580301aeb792972c219e1cc38f75be / same | docs/thefinalglance/J-plan/J-52_returntime_perEdit_enforcement_hooks.md | MATCH |
| 7 | numbered/index original | 91 / 91 | 13486 / 13486 | 02d7cfafeb2bfc6c3dc5cf9fe780ab524692b319824dac1c3e4b953c9ac259cc / same | docs/thefinalglance/J-plan/J-93_lacuna_tier2_composition_fills.md | MATCH |
| 8 | numbered/index original | 73 / 73 | 10880 / 10880 | 2f8dac276d70931c1f3b6a1d985db1c9588578a6a9314beddded75b219153c50 / same | docs/thefinalglance/J-plan/J-94_lacuna_tier3_honesty_thin_measure_first.md | MATCH |
| 9 | numbered/index original | 122 / 122 | 18369 / 18369 | ecc9f455e5d8e1780cfdd7149fcd85b9537584bb9b301e8fb8af96319a878196 / same | docs/thefinalglance/J-plan/J-106_output_metacommentary_enforcement.md | MATCH |
| 10 | numbered/index original | 114 / 114 | 16401 / 16401 | 2fe0a4e11d65d8e6c36ebf4a14e82234c6ac5cde5238d9889b1c2aaac8055ffc / same | docs/thefinalglance/J-plan/J-109_learned_artifact_admission_hardening.md | MATCH |
| Total | 1 auxiliary + 9 numbered originals | 1224 / 1224 | 152552 / 152552 | all ten exact | manifest-prescribed set only | MATCH |

## Per-file exhaustive ledgers

### 1. J-GRC-V3-GROUNDING-MAP.md — 167/167 lines

- DJ002-G-001 `[E|evidence/status]` The map was authored 2026-06-26 in-band before J-83..J-86 from a four-round reasoning run; K-15/K-16 are the same run's separate K-native half, while this map grounds only the J technical half. (L1-7)
- DJ002-G-002 `[E|architecture/non-goal]` GRC-v3 is explicitly a unification/activation of seven scattered ELAI pieces plus one genuinely new deterministic overthinking guard, not a new reasoning mechanism. (L9-13)
- DJ002-G-003 `[E|security/determinism]` One typed `BeliefState` is the uniform carrier, domain-routed validators provide the differentiated checks, and only externally confirmed receipts—not model self-confidence or majority self-judgment—may advance a part. (L12-18)
- DJ002-G-004 `[E|status/integration]` GS-1 corrects I-11 from PLANNED to PARTIAL: `belief_state.rs`, `BeliefState::new`, `BeliefState::emit_condensed_receipt`, `BeliefStateEventKind::Condensed`, `escalation_waterfall.rs:239`, and `i11_belief_state_contract.rs` exist; J-83 adds carrier unification beyond the live condense seam. (L23-26)
- DJ002-G-005 `[E|status/dependency]` GS-2 corrects H-50 to WIRED through `context_policy.rs` and `live_cli.rs:50`; J-83 consumes its local unload/cloud cache-prefix behavior and must not re-author it. (L27-30)
- DJ002-G-006 `[E|status/dependency]` GS-3 corrects I-5 to PARTIAL: `pristine_path_filter.rs` and `i5_pristine_context_contract.rs::pristine_rebuild_zero_traceback_bytes` exist and support J-83 commit/freeze and K-12 preservation. (L31-33)
- DJ002-G-007 `[E|data/security]` GS-4 says H-39 is WIRED but incomplete: `abstention_score.rs::parse_eval_log_outcome` reads correct/wrong/abstain from an eval-log receipt and emits `AbstentionScored` (then SSB 218); each `BeliefState` confidence/abstention bit must be read from this external receipt and never re-derived. (L34-37)
- DJ002-G-008 `[E|workflow/integration]` GS-5 says I-7 is WIRED with `EscalationCascadeKind::{LocalWin,Escalated,BatchRouted}` and a fail-condense-escalate contract; J-83 threads its carrier through the production hook at `escalation_waterfall.rs:239`. (L38-40)
- DJ002-G-009 `[E|status/workflow]` GS-6 says I-4's evidence-ranked search ladder is built but dark behind `ELAI_SEARCH_LADDER_ENABLE`; `search/ladder.rs::run_best_of_n`, `structural_retry.rs`, `SearchNodeEvent`, and branch-prune/nonpromotion tests exist, while PUCT and the receipt-absent vote fallback remain J-86 activation work. (L41-44)
- DJ002-G-010 `[E|status/dependency]` GS-7 says H-42 is PARTIAL via `h42_serial_depth_budget_contract.rs` and `ReasoningBudgetScaled` (then SSB 220); J-84 wires its two-axis classifier into H-42 and I-26. (L45-46)
- DJ002-G-011 `[E|determinism/config]` GS-8 says D-10's locked `select_grounding_mode` A-E router is WIRED and model-independent; it supplies J-84's TYPE axis, while dark `ELAI_DYNAMIC_BUDGET_ENABLE` supplies I-26's effort controller. (L47-49)
- DJ002-G-012 `[E|security/status]` GS-9 places dark D-2 PPM beneath Tier-1 receipts and guards it with dark I-24 isomorphic canaries and I-22 progress-shaped rewards; J-86 activates the scorer only in that subordinate, anti-gaming role. (L50-52)
- DJ002-G-013 `[E|measurement/dependency]` GS-10 distinguishes J-34 `context-fill` calibration from K-15 `reasoning-budget`; K-15 composes dark I-27 convergence instrumentation, and J-86 reuses H-49 SessionDag/backtracking rather than inventing tree search. (L53-58)
- DJ002-G-014 `[E|schema/lifecycle]` GA-1 specifies the absent J-83 carrier: typed entries hold claim, grounding receipt, H-39 abstention/confidence bit, and DKG provenance; a per-step receipt gate, I-7 escalation/H-50 unload carriage, and commit-and-freeze complete the PARTIAL `belief_state.rs` seam. (L63-66)
- DJ002-G-015 `[E|routing/measurement]` GA-2 specifies J-84's absent two-axis effort classifier: TYPE comes from D-10; COST/difficulty proxies combine H-39 confidence, C-45/C-47 case hit, J-38 coverage, and C-29 `IntentPair` count, then route zero/depth/width/abstain over H-42/I-26. (L67-69)
- DJ002-G-016 `[E|capability/determinism]` GA-3 defines the only genuinely new component, J-85's deterministic overthinking guard: a per-call ceiling plus answer oscillation, hesitation markers, and declining-confidence signals; `flip_ratio>1.0` stops, while `anti_spiral_extended.rs` remains the distinct repetition detector. (L70-74)
- DJ002-G-017 `[E|deferred/non-goal]` GA-4 keeps I-14 deterministic context compression PLANNED and bans ciphers; only the preserve-receipts-losslessly/fold-prose-lossily rule is a rider, not re-authored J work. (L75-78)
- DJ002-G-018 `[E|test/status]` GA-5 says `elai bench reasoning-budget` does not exist and belongs to K-15 as GRC-v3's proof gate, using I-27's convergence-bench shape rather than J-34's context-fill axis. (L79-80)
- DJ002-G-019 `[E|grounding/constraint]` Nine named prompting/search/scaling papers plus 2026 corroborations ground design rationale only; older prompting techniques are partly stale, independent results converge on receipts over self-evaluation, and no external threshold may become an ELAI acceptance constant. (L83-96)
- DJ002-G-020 `[E|status/acceptance]` Every cited arXiv ID must be reverified at detail authoring because prior B-40/B-42 citations were wrong; the overthinking paper itself forbids transferring its 6K/7K knee to Qwen2.5-Coder-7B. (L88-96)
- DJ002-G-021 `[E|interfaces]` J-83's symbol table names `BeliefState::new`, `emit_condensed_receipt`, the escalation hook, `abstention_score.rs`, `context_policy.rs`, `select_grounding_mode`, reasoning `GrcState`, and preferably reused `BeliefStateEvent(+Kind)`. (L101-104)
- DJ002-G-022 `[E|interfaces]` J-84 consumes `select_grounding_mode`, confidence/case/retrieval/intent proxies, H-42/I-26, I-27, and H-9; it should reuse `ReasoningBudgetScaled(+Kind)` or add `ReasoningEffortRouted`. (L105-108)
- DJ002-G-023 `[E|interfaces]` J-85 sits beside `anti_spiral_extended.rs` in the GRC Verify step, consumes I-26's ceiling, is measured by K-15, and either adds Pattern-2 `OverthinkingGuardFired` or reuses `ReasoningBudgetScaled`. (L109-111)
- DJ002-G-024 `[E|interfaces]` J-86 consumes `run_best_of_n`/`SearchNodeEvent`, D-2 port 8101, `CanaryInjector`, `ProgressRewardScored`, SessionDag/H-49 backtrack, and I-7, preferring reuse of existing search/reward/canary events for net-zero schema impact. (L112-114)
- DJ002-G-025 `[E|schema/determinism]` `SCHEMA_VERSION = -309 + SSB_ACTION_TYPES.len()` and `TOTAL_VARIANTS = ALL_VARIANTS.len() = SSB_ACTION_TYPES.len()`, hence variants equal schema+309; the authoring snapshot 268/577 is derived and must be re-read, not hard-coded. (L117-124)
- DJ002-G-026 `[E|schema/concurrency]` Schema additions are sequential, one session at a time, with nine-tuple golden lockstep; the prefer-reuse J wave targets net 0..+1 per session, avoiding the prior partial-add 91-error failure. (L124-126)
- DJ002-G-027 `[C|status/correction]` DC-1 explicitly reverses the reasoning document's I-11 label: J-83 is a lower-risk completion/extension over a built condense seam, not a from-scratch build. (L131-132)
- DJ002-G-028 `[C|status/correction]` DC-2 explicitly reverses the H-50 label to WIRED but preserves a real local limitation: streaming bypasses PrefixKVCache, so J-83's local lever is unload/reload, not KV reuse. (L133-135)
- DJ002-G-029 `[E|workflow/security]` DC-3 forbids self-consistency/MCTS-first selection: the ladder is best-of-N then tournament then PUCT, receipts select, and statistical voting is legal only at J-86's receipt-absent rung and never for factual recall. (L136-139)
- DJ002-G-030 `[E|data/determinism]` DC-4 defines compression as deterministic and reversible: preserve receipts/observations losslessly, fold rederivable reasoning prose lossily, and on cloud use deterministic trailing-observation masking behind the cache breakpoint—never LLM resummarization. (L140-143)
- DJ002-G-031 `[E|routing/constraint]` DC-5 reconciles depth-not-width with test-time compute: context window is never the lever; easy/medium tasks use serial BeliefState revision, hard tasks use parallel best-of-N/BSP width. (L144-147)
- DJ002-G-032 `[E|integration/acceptance]` R-1 names composition seams—not component feasibility—as the risk; each J session must ship producer, consumer, and integration smoke together, and J-83 specifically requires carrier, I-7 consumer, and a multi-turn handoff smoke. (L150-155)
- DJ002-G-033 `[E|measurement/default]` R-2 makes the imperfect difficulty proxy observe-only until K-15 calibration, never assumed correct. (L156-158)
- DJ002-G-034 `[E|failure/measurement]` R-3 treats oscillation as the lead and flip ratio as lagging, accepts some false early stops, and requires thresholds tuned on ELAI data rather than the paper's reported precision or constants. (L159-161)
- DJ002-G-035 `[E|security/constraint]` R-4 makes the local Qwen2.5-Coder-7B lane the hardest case: the engine must impose steps and external receipts because small-model self-evaluation is noise even though the model needs structured reasoning. (L162-165)

### 2. J-22_journal_payload_schema_lock.md — 68/68 lines

- DJ002-22-001 `[E|status/scope]` J-22 is a small Wave-2 PLANNED BUILD with no dependency and the one genuinely new P0 journal item; the existing single-writer SQLite-WAL, append-only blake3-chain foundation is harden-not-replace. (L1-5)
- DJ002-22-002 `[E|gap]` The current golden lock protects action-type names but not payload field shapes, so a rename, addition, or type change can escape while preserving the action name. (L5, L13-16)
- DJ002-22-003 `[E|schema/security]` The payload lock adds `#[serde(deny_unknown_fields)]` to every SSB payload struct and extends snapshots to the payload field set; field rename/add/type-change fails while source-field-order changes do not. (L7-8, L39-40, L46-48)
- DJ002-22-004 `[E|integration]` The Rust field-shape lock complements E-45's ts-rs cross-language snapshot and extends the existing `action_type_lock` family instead of creating a parallel schema system. (L8, L15-16, L21-22)
- DJ002-22-005 `[E|determinism/test]` A writer-isolation contract asserts that only the single SSB writer may commit a row; any future second commit path fails loudly. (L9, L14, L23, L31, L41)
- DJ002-22-006 `[E|deferred/dependency]` A batched RFC-6962 Merkle checkpoint—roughly every 1,000 events, Ed25519-signed in a `MerkleCheckpoint` row—belongs to I-18, supplying O(log n) inclusion/consistency proofs instead of per-row O(n) replay; J-22 records but does not build it. (L10, L17, L51)
- DJ002-22-007 `[E|non-goal]` Protobuf/prost/buf migration, Google Trillian, a separate journal-writer process, per-event Merkle, and re-authoring I-18/G-2/G-8 are explicitly excluded. (L11, L42-43)
- DJ002-22-008 `[E|dependency/status]` Existing consumed substrate is SSB single-writer WAL+broadcast-after-commit, the nine-tuple golden lock, E-45 ts-rs drift checking, planned I-18 chain/AI-BOM, wired G-2 RRA, partial G-8 attestation, and I-1's separate process-level supervisor. (L13-18, L57-58)
- DJ002-22-009 `[C|schema/status]` The source's 2026-06-12 `SCHEMA_VERSION 231` / `TOTAL_VARIANTS 542` values are historical snapshots and must not be treated as current constants. (L19, L55)
- DJ002-22-010 `[E|integration/files]` Runtime delta is only derive attributes plus a test under orchestrator and a writer contract under `crates/contracts/tests/`; there is no new action type and all then-542 payloads are guarded. (L21-24)
- DJ002-22-011 `[E|surface/non-goal]` No frontend is required because this is a test-layer integrity gate; the existing wiring-audit is the surface. (L25, L44)
- DJ002-22-012 `[E|interfaces]` Produced symbols are the deny-unknown attribute, payload field-shape snapshot, and writer-isolation contract; consumed symbols are the existing golden-row mechanism and single commit path, neither re-authored. (L27-34)
- DJ002-22-013 `[E|reachability]` Correct reachability is the auditor's normal cargo/action-type-lock run, not a runtime UI path; the guard is explicitly a Layer-2 static leg protecting runtime writes. (L35-37)
- DJ002-22-014 `[E|constraint]` The gate forbids name-only coverage, any payload without `deny_unknown_fields`, a second writer, a hidden Protobuf migration, and scope creep into chain/RRA/attestation work. (L38-43)
- DJ002-22-015 `[E|acceptance]` A deliberate field rename with unchanged action type must fail the new snapshot where today's name lock passes, while the current single-writer contract passes. (L45-48)
- DJ002-22-016 `[E|negative-test]` A payload lacking the serde attribute and a second commit path must fail; a field-order-only change must not false-fail because the snapshot is set-based. (L46-48)
- DJ002-22-017 `[E|grounding/compliance]` RFC 6962/9162, the experimental PunkGo sole-commit-point prototype, serde behavior, and in-toto/SLSA/Rekor/TSA riders are rationale; the latter remain owned by G-8/I-18. (L50-55)
- DJ002-22-018 `[A|dependency]` Snapshot implementation is an edit-time choice between a new `insta` dev dependency and extending the dependency-free existing golden-row mechanism; the source prefers the latter and requires anti-slopsquat review if `insta` is chosen. (L60-61)
- DJ002-22-019 `[A|schema/risk]` Blanket `deny_unknown_fields` must first audit `#[serde(flatten)]` and untagged-enum interactions, which can be incompatible. (L62)
- DJ002-22-020 `[E|deferred/risk]` Merely naming the Merkle rider does not close the O(n) proof gap; I-18 must actually adopt it and track it against orphaning. (L63, L65-66)

### 3. J-27_harness_protected_blocks_webcache_dispatch.md — 128/128 lines

- DJ002-27-001 `[E|status/scope]` J-27 is PLANNED Wave RSW-1: two new local deterministic hooks plus one dispatch-amendment bundle, built from a medium-reliability source read without importing an external skill. (L1-14)
- DJ002-27-002 `[E|security/workflow]` T1 `PreToolUse(Read)` replaces opt-in language-comment regions between `elai-protect-start`/`elai-protect-end` with content-hashed placeholders before model visibility. (L16-20)
- DJ002-27-003 `[E|recovery/security]` T1 `PostToolUse(Edit|Write)` restores original bytes only when placeholders survive intact and blocks the write on any mangled or partially edited placeholder. (L20-23)
- DJ002-27-004 `[E|config/default]` Protected blocks use stdlib `hashlib.blake2b`, new wrapper files, user-reviewed settings registration, and opt-in annotations; unannotated files remain byte-behavior-identical. (L23-26)
- DJ002-27-005 `[E|integration/non-goal]` The new hook files use the existing rag/hooks seam and may not edit registered J-3/J-4 or other existing hooks. (L24-26, L52-55, L67-69)
- DJ002-27-006 `[E|network/cache]` T2 `PreToolUse(WebFetch)` sends conditional HEAD using cached ETag/Last-Modified; HTTP 304 is fresh revalidation and serves cached content, while HTTP 200/changed proceeds to fetch and `PostToolUse` records body plus validators. (L27-31)
- DJ002-27-007 `[E|fallback/security]` A response without ETag or Last-Modified is never cached and fails open to a live fetch; there is no TTL-based staleness window. (L29-32, L92-94)
- DJ002-27-008 `[E|data/privacy]` Web cache keys are `sha256(url)` beneath the shared session cache; the original prompt is retained as metadata and surfaced on hits so the model, not the cache, assesses relevance. (L32-33)
- DJ002-27-009 `[E|workflow/security]` T3 claim-stripped auditor dispatch sends only artifact+contract, never executor claim/rationale, through orchestrate, auto-orchestrate, b-plan-runner, and the dispatch preamble. (L34-38)
- DJ002-27-010 `[E|workflow]` T3 extends R-5 with a `NOTICED BUT NOT TOUCHING` disclosure for out-of-scope observations that must not be acted on. (L38-39)
- DJ002-27-011 `[E|permission]` Prompt-transform and grill gain mandatory `Out of scope:` plus an explicit rule that “sounds good,” “whatever you think,” and silence are not approval. (L39-41)
- DJ002-27-012 `[E|orchestration]` Orchestrate/auto-orchestrate add four pre-dispatch questions: independence, distinct finding kinds, merge-context fit, and whether waiting justifies parallelism. (L41-43)
- DJ002-27-013 `[E|non-goal]` Severity labels, browser trust axioms, define-questions-first observability, source-driven development, and context-trust taxonomy are rejected as already covered. (L44-47)
- DJ002-27-014 `[E|evidence/status]` Authoring found no protected-block or WebFetch-cache hook under `.claude/` or rag/hooks; implementation must re-confirm with glob before build. (L49-51)
- DJ002-27-015 `[E|dependency]` The hooks inherit JD-1..JD-4 log/session evidence, derived-not-recalled behavior, observe-first posture, and frozen-hook change control; J-3/J-4 are parallel-safe shape precedents. (L52-55)
- DJ002-27-016 `[E|scope/status]` Revised Tier 1 ships both tools/wrappers and all T3 rule lines with stdlib-only Python; nothing is deferred to Tier 2, but both hook registrations require user review. (L61-64)
- DJ002-27-017 `[E|files/interfaces]` New artifacts are `protect_blocks.py`, read/edit-write wrappers, `webfetch_cache.py`, and WebFetch wrappers; named rule/skill files receive amendment lines only. (L66-75)
- DJ002-27-018 `[E|surface/non-goal]` No dashboard/frontend is required because this is a harness leg. (L75, L121)
- DJ002-27-019 `[E|observability/reachability]` Production proof requires a real annotated Read, validator-bearing WebFetch, and orchestrator dispatch, each leaving a `hook-executions.log` row; file existence is wire-dark. (L77-90)
- DJ002-27-020 `[E|constraint]` The hook paths ban TTL caches, validator-less storage, any placeholder remaining on disk, partial restore, LLM decisions, and existing-hook edits. (L91-94)
- DJ002-27-021 `[E|acceptance]` Live smoke reads an annotated scratch block and edits around it without changing protected disk bytes; fetching an ETag URL twice must log a 304 and serve the cached payload. (L95-103)
- DJ002-27-022 `[E|negative-test]` A mangled placeholder must block, validator stripping must prevent storage, and no-annotation/no-cache sessions must remain byte-identical to current behavior. (L98-103)
- DJ002-27-023 `[E|test]` Deterministic self-tests require swap/restore byte identity, nonzero exit on mangle, refusal without validators, cache serving on 304, the R-5 amendment, and a durable `J-27-decisions.md`. (L105-112)
- DJ002-27-024 `[E|event/schema]` J-27 adds no SSB action type; harness logs are its evidence surface. (L114-121)
- DJ002-27-025 `[E|compliance]` Any future skill import still must pass `/skill-audit` and `workflow/skill-imports.log`; this unit imports no skill and does not bypass policy. (L12-14, L84-86, L123-126)
- DJ002-27-026 `[E|dependency]` The unit composes with J-3/J-4, dispatch-preamble, orchestrate/auto-orchestrate/b-plan-runner, prompt-transform, grill, skill-import policy, and I-21's zero-shared-context idea without restating them. (L123-126)

### 4. J-36_freeze_head_local_prefix_kv_reuse.md — 266/266 lines

- DJ002-36-001 `[E|status/scope]` J-36 is PLANNED post-CFW activation of shipped `PrefixKVCache`; Tier 1 adds a streaming frozen-head lease, model/quant guard, and journaled prefill artifact, while TurboQuant coexistence and a dashboard savings tile are Tier-2 deferred. (L1-14)
- DJ002-36-002 `[E|runtime/constraint]` The resident local NF4 sidecar loads weights once, so repeat-call cost is prefill+decode rather than weight reload; the unit targets repeated stable-head prefill. (L16-23)
- DJ002-36-003 `[E|failure/status]` Defect one is the explicit `server.py:2480` non-streaming-only lease: interactive local streaming never leases the prefix cache. (L23-26)
- DJ002-36-004 `[E|failure/status]` Defect two is the `input_token_count <= PREFILL_CHUNK_SIZE` whole-input gate; long heads plus tails fall to `_chunked_prefill` and receive no prefix reuse. (L27-29)
- DJ002-36-005 `[E|failure/status]` Defect three is `init_prefix_kv_cache` disabling the cache under TurboQuant because `DynamicCache` is incompatible with `_KVQuantCache`. (L30-31)
- DJ002-36-006 `[E|architecture/workflow]` The central invariant is “freeze the head, minimize the tail”: system prompt, rules, and I-6 anchor pane stay fixed/hot while H-50 may minimize only the task tail. (L33-40)
- DJ002-36-007 `[E|non-goal]` This is a deterministic throughput/rot-avoidance lever on a fixed model, not a model-size or intelligence change. (L42-48)
- DJ002-36-008 `[E|security/fallback]` Correctness is model-agnostic through a conservative eligibility guard, not universal cache assumptions: TurboQuant, unknown architectures, sliding-window, or nonstandard-attention models cold-prefill rather than risk incorrect KV reuse. (L48-56)
- DJ002-36-009 `[E|schema/data]` FH-1 defines the frozen head as system+rules+I-6 anchor at the existing `marker_boundary`; token-identical prefixes use the I-15 blake3/xxhash fast path, and H-50 may never trim, reorder, or unload the segment. (L58-65)
- DJ002-36-010 `[E|workflow/integration]` FH-2 extends the proven cache lease into streaming generation; a hit starts prefill at the head boundary and reuses the existing crop-to-`_prefix_len - 1` RoPE/mask invariant. (L67-71)
- DJ002-36-011 `[E|workflow/performance]` FH-3 keys the lease on the head boundary rather than whole input, allowing a long total prompt to reuse the head while the tail proceeds through `_chunked_prefill`. (L73-76)
- DJ002-36-012 `[E|config/determinism]` FH-4 defines `prefix_kv_eligible(model_profile, quant) -> bool`, consuming `ModelCapabilityProfile` and the quant flag; eligibility requires TurboQuant off and a known full-attention standard causal architecture. (L78-83)
- DJ002-36-013 `[E|observability/fallback]` Every ineligible decision cold-prefills and logs its reason; J-9 may later own a catalog column, but J-36's safe default ships independently. (L82-85)
- DJ002-36-014 `[E|measurement/config]` FH-5 sizes tail budget from J-34's per-model fill/positional degradation curves when available, otherwise a conservative default; hard-coded context-window fractions are forbidden. (L87-89)
- DJ002-36-015 `[E|interfaces/status]` Verified sidecar substrate is `PrefixKVCache` at `server.py:1819`, its LRU `lease()`/admit-store methods, and `init_prefix_kv_cache` with `turboquant_bits` and max-entry disabling. (L91-96)
- DJ002-36-016 `[E|observability]` The current lease region already carries `marker_boundary`, `cache_read_tokens`, `cache_creation_tokens`, and `prefix_cache_hit`; these counters are the prefill evidence rather than a new measurement proxy. (L97-99)
- DJ002-36-017 `[E|correctness/dependency]` `_chunked_prefill` and crop-to-`_prefix_len - 1` are shipped correctness machinery that must be reused, not re-derived. (L100-101)
- DJ002-36-018 `[A|integration/evidence]` The cited Rust streaming caller `runtime_client.rs:547` was not found at that path during authoring; implementation must re-grep the real caller, though the sidecar streaming bypass is independently confirmed. (L102-104, L256-258)
- DJ002-36-019 `[C|status/dependency]` The file labels H-50, I-6, I-15, I-11, J-9, J-34, and H-42 as PLANNED at its authoring point, whereas the assigned grounding map explicitly corrects H-50 to WIRED. (L105-107; grounding map L27-30)
- DJ002-36-020 `[C|schema/status]` The documented 268/577 SSB baseline is a historical derived snapshot; both schema and action lock must be re-grepped at edit. (L108-110)
- DJ002-36-021 `[E|dependency/process]` Mandatory pre-code reads cover the local-context thesis, the two sidecar cache/prefill regions, `ModelCapabilityProfile`, H-50, J-9, and J-34. (L112-120)
- DJ002-36-022 `[E|task/files]` Tier-1 task 1 marks and blake3-keys the existing marker-boundary head; task 2 adds the streaming crop-and-feed lease; task 3 removes the whole-input restriction in favor of boundary-keyed tail chunking. (L122-129)
- DJ002-36-023 `[E|task/fallback]` Tier-1 task 4 adds the Rust/sidecar eligibility predicate and makes TQ/unknown/sliding-window fall back safely with a logged reason. (L130-132)
- DJ002-36-024 `[E|event/schema]` Tier-1 task 5 adds Pattern-2 `LocalPrefixCacheEvent` with `HeadCacheHit`, `HeadCacheMiss`, `TailReprefill`, and `CacheIneligible`; payload carries head/tail prefill token counts and eligibility reason. (L133-135)
- DJ002-36-025 `[E|evidence/config]` `J-36-decisions.md` records the H-50 invariant, eligibility predicate/default, TQ defer, and J-9/J-34 rider seams. (L136-137)
- DJ002-36-026 `[E|interfaces]` Produced runtime symbols are the streaming lease, boundary key, `prefix_kv_eligible`, and `LocalPrefixCacheEvent`; the dashboard tile is explicitly deferred. (L139-145)
- DJ002-36-027 `[E|dependency/non-goal]` Consumed symbols are existing lease/admit/init/crop/marker machinery, `ModelCapabilityProfile`, H-50 local arm, I-15 keying, J-34 curves, and J-9 catalog; none may be reimplemented. (L146-151)
- DJ002-36-028 `[E|reachability]` Production reachability is the live local streaming request path plus a per-step journal event, not a non-streaming fixture. (L152-154)
- DJ002-36-029 `[E|constraint/security]` The contract bans a second marker system, unconditional reuse, head mutation by H-50, production unwrap/expect, non-blake3 hashing, and new dependencies. (L155-161)
- DJ002-36-030 `[E|surface/deferred]` Tier 1 exposes journal rows only; the prefill-savings tile and its cross-language pass remain Tier 2. (L162-163, L236)
- DJ002-36-031 `[E|acceptance/observability]` Two eligible consecutive streaming turns with identical system+rules+anchor must produce turn-2 `HeadCacheHit`, nearly zero head-prefill tokens, tail tokens equal to the delta, and measurably smaller head prefill than cold turn 1. (L164-171)
- DJ002-36-032 `[E|negative-test/fallback]` TurboQuant, unknown, and sliding-window fixtures must emit `CacheIneligible` with full cold prefill; any hit on them fails. A mutated head must emit `HeadCacheMiss`, and a stable head must not unexpectedly mutate. (L171-175)
- DJ002-36-033 `[E|test]` Deterministic checks assert production trigger reachability, removal/extension of the non-streaming-only seam, marker reuse, guard wiring, TQ respect, no production unwrap/expect, and a decisions artifact. (L177-186)
- DJ002-36-034 `[E|acceptance]` Acceptance separately requires long-input head leasing, the H-50 tail-only invariant, consume-only cache/crop code, full six-part SSB lockstep, green scoped Rust/sidecar checks, no new dependency, and recorded Tier-2 checkboxes. (L188-201)
- DJ002-36-035 `[E|flag/default]` Eligible streaming reuse is default-on; conservative eligibility is the gate, not a dark enable flag. Any added flag requires a wire-dark allowlist row. (L203-210)
- DJ002-36-036 `[E|determinism/evidence]` Prefill counts and eligibility must recompute byte-identically from the journal under `ELAI_SEED`; live journal evidence, not build-green, is the done gate. (L203-215)
- DJ002-36-037 `[E|test]` The contract suite covers shared-head hits, TurboQuant exclusion, unknown/sliding-window exclusion, long total inputs, head mutation miss, and partial-prefix crop correctness without RoPE/mask errors. (L217-225)
- DJ002-36-038 `[E|test/observability]` A sidecar fixture must prove turn-2 head prefill is far below turn 1 using token counts, avoiding latency-only evidence. (L226)
- DJ002-36-039 `[E|files]` Expected edits span sidecar `server.py`, orchestrator context/eligibility, SSB/action lock, H-50 local policy, a new J-36 contract, and the decisions file. (L228-236)
- DJ002-36-040 `[E|constraint/grounding]` Architecture eligibility must be confirmed per live model; full-attention fixed-position soundness is design reasoning, while sliding/nonstandard soundness is deliberately not asserted. (L238-246)
- DJ002-36-041 `[E|risk/fallback]` Gemma/Qwen3/MoE eligibility remains untested, so eligibility is opt-in per architecture and unknown remains correct-but-slower. (L248-251)
- DJ002-36-042 `[E|deferred]` A cache compatible with `_KVQuantCache` is genuinely harder and deferred; under TurboQuant J-36 is an explicit cold-prefill no-op, not a regression. (L252-253)
- DJ002-36-043 `[E|dependency/deferred]` Folding eligibility into J-9's `model_catalog.toml` is a rider, not a J-36 blocker; H-42 serial depth becomes the next bottleneck after head-prefill removal. (L254-255, L260-264)

### 5. J-44_reasoning_method_artifact_shape.md — 59/59 lines

- DJ002-44-001 `[E|status/architecture]` J-44 is a medium-large DPW-2 Track-2 METHOD build depending on J-43 and existing planning/idea/spec/dispatch seams; J-43 supplies the WHAT/lens, while J-44 mechanizes the HOW/artifact shape. (L1-4)
- DJ002-44-002 `[E|constraint]` Reasoning quality itself cannot be regex-gated: deterministic checks cover required structure and tags only, semantic substance stays advisory, and missing structure eventually fails closed. (L4)
- DJ002-44-003 `[E|security/measurement]` A random non-author substance audit is first-class because shape-only mandates invite boilerplate and partial compliance is treated as null, not partial benefit. (L4, L10)
- DJ002-44-004 `[E|measurement/default]` EXP-DP2 first runs observe-only over existing reasoning and plan files, measuring shape false positives and boilerplate rate; no blocking default precedes bounded FP and observed audit catch rate. (L6-7)
- DJ002-44-005 `[E|rule/schema]` `.claude/rules/reasoning-method.md` generalizes the nine-step planning protocol to all reasoning/design/plan artifacts. (L8)
- DJ002-44-006 `[E|rule/evidence]` M1 requires every external/load-bearing grounding claim to carry a source or explicit epistemic tag. (L8)
- DJ002-44-007 `[E|rule/evidence]` M2 requires each load-bearing claim to be classified `VERIFIED`, high-confidence, or training-unsearched. (L8)
- DJ002-44-008 `[E|rule/decision]` STANCE requires an explicit classification of what the model decides versus what the harness could decide instead. (L8)
- DJ002-44-009 `[E|rule/integration]` M4/M6 require reinvention/extension/net-new classification plus a named file:line integration seam. (L8)
- DJ002-44-010 `[E|rule/security]` M7 requires an `ATTACK` section containing the strongest case against the design. (L8)
- DJ002-44-011 `[E|rule/test/default]` M9 requires the cheapest falsifier as literal step 1 and forbids default-on behavior until false-positive/over-block rates are bounded across at least three runs. (L8)
- DJ002-44-012 `[E|rule/status]` M10 requires a `RESIDUE` section naming unverified, provisional, and deferred work. (L8)
- DJ002-44-013 `[E|rule/evidence]` M12 requires durable reasoning-chain, lens, and handoff artifacts. (L8)
- DJ002-44-014 `[E|tool/failure]` `reasoning_artifact_shape_check.py` is heading/regex-aware, necessary-not-sufficient, exits 2 on missing/empty targets, begins WARN-only, and eventually fails missing `ATTACK` or untagged load-bearing claims. (L9)
- DJ002-44-015 `[E|audit/workflow]` The recurring panel randomly samples reasoning artifacts and DP-4 rows, deep-checks whether attacks/falsifiers/classifications are substantive, uses a held-out subset, and tracks remediation velocity rather than section counts. (L10)
- DJ002-44-016 `[E|permission/security]` The substance measurer must not be the artifact author; all-or-nothing applies within each sampled subset so half-enforcement cannot claim half benefit. (L10)
- DJ002-44-017 `[E|workflow/integration]` Design-time injection adds a mandatory spec-eval decision ledger that enumerates decisions, splits model-class from harness-class, and records model-owned harness decisions as design weaknesses. (L11-12)
- DJ002-44-018 `[E|workflow/security]` Plan-time injection adds doctrine pre-gates to planning-workflow steps 4 and 7 and idea-intake's auditor; every deliverable declares decisions, model/harness owner, an off-agent disposer, and debt for legitimate unmechanized decisions. (L13)
- DJ002-44-019 `[E|failure]` A producer-only or disposer-less harness-class decision is refused during decomposition. (L13)
- DJ002-44-020 `[E|workflow/integration]` Dispatch-time injection adds the doctrine/method pointer and extends R10 with seam disposal so harness-class decisions cannot silently fall back to the model at integration seams. (L14)
- DJ002-44-021 `[E|non-goal]` Automated LLM quality scoring on a gate, constrained decoding of reasoning, rewriting the existing workflow, and treating section presence as quality are forbidden. (L15)
- DJ002-44-022 `[A|evidence/dependency]` `ELAI-DETERMINISM-DOCTRINE-AUX.md` and its M-numbering were PLAN-asserted but not confirmed in this source session; implementation must verify them before load-bearing use. (L17-20, L37-38, L56)
- DJ002-44-023 `[E|dependency]` Existing hosts are planning-workflow, spec-eval, idea-intake, dispatch preamble, and `.claude/agents/auditor.md`; each is extended rather than rewritten. (L20-27)
- DJ002-44-024 `[E|acceptance/evidence]` JD-1 completion requires a real shape-checked artifact, a real non-author substance verdict, and a remediation-velocity row; mere rule/tool existence is insufficient. (L26-27)
- DJ002-44-025 `[E|constraint/residue]` The checker can accept empty-but-shaped theater; only the panel plus DP-4 debt-rate address it, no controlled artifact-shape RCT is claimed, and recurring review has real human/cross-family cost. (L27, L40-41)
- DJ002-44-026 `[E|interfaces]` Produced artifacts include the rule, checker, audit-panel protocol, decision-ledger dimension, planning/idea pre-gates, and dispatch pointer/R10 check; they consume existing workflow and auditor seams. (L29-32)
- DJ002-44-027 `[E|reachability]` Production reachability is auditor execution on a real artifact, a real plan pre-gate, and a real spec-eval decision-ledger pass. (L32)
- DJ002-44-028 `[E|acceptance]` Missing `ATTACK` produces shape failure; a model-owned sizing/admission/verdict without disposer is refused; boilerplate attack produces panel `SUBSTANCE-FAIL` plus remediation. (L33-34)
- DJ002-44-029 `[E|negative-test]` A deliberately shape-complete but empty artifact must pass the mechanical checker and be caught only by the panel, demonstrating that shape and quality are distinct. (L34)
- DJ002-44-030 `[E|event/schema]` J-44 adds no SSB action type; shape output, substance verdict, and remediation rows live in the harness audit log and feed DP-4. (L35)
- DJ002-44-031 `[E|grounding/constraint]` Process/medical checklist evidence is provisional domain transfer, so EXP-DP2 measures local boilerplate rather than importing reported outcome gains. (L37-41)
- DJ002-44-032 `[E|dependency]` J-43 supplies feature-property floor, J-45 cross-family advisory, and J-46 debt-rate; planning/spec/idea/dispatch/oracle/deep-research/grill/GRC surfaces consume the method. (L43-50)
- DJ002-44-033 `[A|data/lifecycle]` Decision-ledger storage is unresolved between inline plan blocks and a sibling file; the source leans inline for visibility and drift resistance. (L52-53)
- DJ002-44-034 `[A|config/measurement]` Panel sampling rate and remediation-velocity SLO are unspecified tuning knobs that must be observed, while under-sampling reopens the theater gap. (L54)
- DJ002-44-035 `[E|risk]` Random hold-out and DP-4's implausibly-zero-debt self-test are the only structural defenses against authors minimally filling every section; the risk is named, not solved. (L55)

### 6. J-52_returntime_perEdit_enforcement_hooks.md — 136/136 lines

- DJ002-52-001 `[E|status/scope]` J-52 is a PLANNED medium-large LazyCodex-reintake extension shipping all three in-process legs in Tier 1 over the wired hook dispatcher, subagent executor, diagnostic probe, and context Rules slot. (L1-5)
- DJ002-52-002 `[E|evidence/correction]` The source distinguishes an older v0.2.2 summary-only intake that found two shallow ideas from a v4.11.1 real-source re-intake that exposed the three deeper enforcement mechanisms. (L5-6, L115-117)
- DJ002-52-003 `[E|gap/integration]` Existing hooks cover `PreToolUse`, `PostToolUse`, `Stop`, and `PreCompact` but no `SubagentStop`; J-52 turns prompt disciplines into structural, fail-closed, off-agent tripwires. (L6, L18-20)
- DJ002-52-004 `[E|workflow/schema]` Leg 1 parses a subagent's last line as exactly `EVIDENCE_RECORDED: <path>` before accepting DONE. (L8-10)
- DJ002-52-005 `[E|security/filesystem]` The receipt must resolve to a real nonempty file within the evidence directory; path checks canonicalize, reject symlinks, and guard traversal before and after realpath. (L10, L16)
- DJ002-52-006 `[E|retry/lifecycle]` Missing evidence increments a persisted atomic attempt counter, blocks with a nag, retries up to `MAX_EVIDENCE_NAGS=3`, then escalates to auditor/handoff rather than crashing. (L10, L16)
- DJ002-52-007 `[E|integration]` The port inserts `subagent_stop` at the real `SubagentToolExecutor` return/`### Final response` seam, one audit cycle earlier than prompt-only R-8/D-030 enforcement. (L10, L20-21)
- DJ002-52-008 `[E|security/constraint]` The receipt proves artifact existence only; it never substitutes for auditor correctness, and stdout/self-claim alone cannot pass. (L10, L16, L61-65)
- DJ002-52-009 `[E|workflow/concurrency]` Leg 2 runs edit-like `PostToolUse` diagnostics through a long-lived reused daemon with `MAX_CONCURRENT_DIAGNOSTICS=4` and block-feeds real findings inline. (L12)
- DJ002-52-010 `[E|failure/recovery]` Daemon-unreachable is transient and must not be recorded, while unsupported-extension is permanent and is recorded to suppress futile repeats. (L12, L66)
- DJ002-52-011 `[E|context/config]` Diagnostic output is bounded to 8,000 normally and 1,200 under context pressure. (L12)
- DJ002-52-012 `[E|integration/non-goal]` Leg 2 extends E-52's existing language-to-tool map and the shipped ruff PostToolUse block-feed, launching rust-analyzer/tsc/svelte-check once per session rather than building a second toolchain. (L12, L22-23)
- DJ002-52-013 `[E|workflow/data]` Leg 3 fingerprints each edited path, resolves only matching local rules, injects them into `context_budget.rs`'s Rules slot, deduplicates against current contents, and priority-orders `.claude/rules` first. (L14)
- DJ002-52-014 `[E|integration]` `SubagentToolExecutor.active_rules` is the worker target; the injector makes `conditional-routing.md` mechanical instead of relying on model memory. (L14, L21, L24, L26)
- DJ002-52-015 `[E|constraint/default]` All three gates are default-on and expose only disable kill switches; each disabled state must be logged, and no new `*_ENABLE` flag is allowed. (L16, L69-70)
- DJ002-52-016 `[E|dependency/status]` Wired ground truth includes `HookInvocation::for_task_level`, `dispatch_post_tool_use`, precompact token state, `SubagentToolExecutor`, ruff cache/timeout, eight `SlotWeights`, and VibeGate as a distinct precedent—not a routing destination. (L18-26)
- DJ002-52-017 `[C|status/dependency]` E-52 is PARTIAL and its existing `ELAI_LSP_DIAGNOSTIC_PROBE_ENABLE` is described as unallowlisted with real subprocess work deferred; J-52 must resolve that substrate state before claiming daemon reuse. (L22)
- DJ002-52-018 `[E|scope/status]` Tier 1 ships return gate+nag+escalation, per-edit per-session daemon, and static path-rule injection entirely in-process without network. (L28-31)
- DJ002-52-019 `[E|deferred]` Cross-session daemon pooling, receipt-content schema validation, and learned per-path relevance scoring are Tier-2; Tier 1 checks only real/nonempty existence, reuses static fingerprints, and leaves content verdict to auditor. (L30-31, L129-132)
- DJ002-52-020 `[E|files/interfaces]` New modules are `subagent_evidence_gate.rs`, `lsp_daemon_client.rs`, and `path_rule_inject.rs`; existing tools/lib, hook dispatcher, diagnostic probe, and context budget receive narrow call-site changes. (L33-36)
- DJ002-52-021 `[E|event/schema]` Preferred telemetry adds Pattern-2 `SubagentEvidenceGated::{EvidencePresent,EvidenceMissingBlocked,NagExhaustedEscalated}`, `EditDiagnosticGated::{DiagnosticBlocked,DaemonTransient,ExtensionUnsupported}`, and `PathRuleInjected::{RuleInjected,AlreadyResident,NoMatch}`. (L37)
- DJ002-52-022 `[A|event/schema]` Action-lock review may collapse the three action types into one `HookEnforcementEvent` with 6-7 kinds, but all three decision classes must remain journal-distinct. (L37, L100, L126)
- DJ002-52-023 `[E|surface]` `HookEnforcementPanel` in the Monitor/Hook family counts evidence blocks, diagnostics, and rule injections per session and requires Tauri, ts-rs, and cross-language pairing. (L38, L72-73)
- DJ002-52-024 `[E|interfaces]` `SubagentEvidenceGate`, the new dispatcher axis, `LspDaemonClient`, and `PathRuleInjector` are exact producers; their exact consumers are the executor return, PostToolUse/ruff shape, E-52 map, Rules slot, and instruction rules. (L40-54)
- DJ002-52-025 `[E|reachability]` Production proof must drive a real subagent return and real edit-like PostToolUse call; tests alone never satisfy, and all decision rows plus the UI tile are off-agent artifacts. (L56-59)
- DJ002-52-026 `[E|constraint]` A worker DONE cannot pass on stdout, a receipt cannot count as verdict, symlink/traversal cannot escape, retries cannot hard-kill, daemon failures cannot poison permanent state, and rules cannot double-inject. (L61-70)
- DJ002-52-027 `[E|security/code]` New production code forbids unwrap/expect and blake2/sha2; receipt/rule fingerprints use blake3, with no silent fallback. (L69-70)
- DJ002-52-028 `[E|acceptance/observability]` A live missing-receipt return must emit `EvidenceMissingBlocked`, eventually `NagExhaustedEscalated`, and redispatch with the directive until real evidence or escalation. (L75-78)
- DJ002-52-029 `[E|acceptance/observability]` An edit introducing a real diagnostic must emit `DiagnosticBlocked` and feed it before the next tool call; a matching path must emit `RuleInjected` and place the rule in the next-turn Rules slot. (L75-78)
- DJ002-52-030 `[E|negative-test]` Acceptance explicitly fails stdout-only completion, symlink/outside receipts, recording transient daemon outages as unsupported, duplicate rule injection, and silent disabled gates. (L78)
- DJ002-52-031 `[E|test]` Deterministic checks require the dispatcher axis, path-hardening tokens, bounded-nag constant, all event names, zero forbidden hashes/unwraps and enable flags, plus `J-52-decisions.md`. (L80-90)
- DJ002-52-032 `[E|acceptance]` Leg 2 must use the reused E-52 map and daemon, preserve transient classification, and bound output under pressure; leg 3 must dedup and priority-source; all gates must log kill-switch state. (L92-100)
- DJ002-52-033 `[E|done-gate/default]` W-1..W-10 require live rows, default-on disables, unconditional production seams, artifact—not-proxy—measurement, UI, re-grepped anchors/schema, frozen preexisting checkers, and an independent real dispatch+edit smoke. (L102-103)
- DJ002-52-034 `[E|measurement/determinism]` Default-on gates require false-positive block-rate measurement, and their rows must be recomputable under `ELAI_SEED`. (L103)
- DJ002-52-035 `[E|test]` Leg-1 contract cases cover valid artifact, missing artifact/redispatch, exhausted escalation without crash, symlink, traversal, and proof that the auditor still runs. (L105-106)
- DJ002-52-036 `[E|test]` Leg-2 contract cases cover diagnostic block, transient daemon outage not recorded, unsupported extension recorded once, and pressure-bounded output. (L107)
- DJ002-52-037 `[E|test]` Leg-3 contract cases cover matched injection, `AlreadyResident`, `NoMatch`, and priority ordering. (L108)
- DJ002-52-038 `[E|files]` The implementation creates three modules, three contract tests, and one decisions file, while modifying the exact runtime/SSB/UI call sites; no new dependency is allowed. (L110-113, L98-100)
- DJ002-52-039 `[E|grounding/licensing]` The three designs are derived from MIT LazyCodex/OmO v4.11.1 source: executor-verify, LSP, and dynamic-rule components; old F1 stop-hook continuation remains deferred. (L115-121)
- DJ002-52-040 `[A|integration]` If the PLAN-asserted `tools/lib.rs` return anchor moved, implementation must re-grep `AgentResult`/`### Final response`; material relocation is a handoff fork, not a silent choice. (L123-124)
- DJ002-52-041 `[A|recovery]` When the per-session daemon crashes, the source requires graceful degradation but leaves the exact fallback to the existing E-52 batch `CoreValidationTier` for execution-time decision. (L125)
- DJ002-52-042 `[E|risk/security]` A worker can game existence by writing arbitrary bytes; mitigation is deliberately layered—return tripwire first, authoritative off-agent auditor afterward—not stronger receipt semantics in Tier 1. (L127)
- DJ002-52-043 `[E|deferred]` Post-compact rule reinjection is a rider on H-50/H-51 and an aggregate end-of-plan validator is a rider on auditor; neither is a J-52 leg. (L129-134)

### 7. J-93_lacuna_tier2_composition_fills.md — 91/91 lines

- DJ002-93-001 `[E|status/scope]` J-93 is a Wave-2 medium-large container for seven Shape-A composition gaps whose halves already exist or are planned; every leg ships producer+consumer+smoke and begins observe-only. (L1-4)
- DJ002-93-002 `[E|dependency]` J-93 depends on J-87 journal completeness, specifically for trustworthy outcome labels in leg F and the taint/completeness relationship later reiterated for leg C. (L3, L31-34, L88-89)
- DJ002-93-003 `[E|capability/gap]` Leg A corrects E-6 MAST's post-hoc limitation by supervising the live broadcast-after-commit SSB stream while processes are still alive. (L6-9)
- DJ002-93-004 `[E|determinism]` Leg A detects FM-2.6 plan-vs-tool divergence, FM-1.3 semantic action-type sequence repetition, and `StuckLoop` after three identical calls through deterministic pattern matching, never an LLM watcher. (L7-9)
- DJ002-93-005 `[E|workflow/fallback]` Leg A redirects/restarts mid-run and reuses MAST kinds; conservative thresholds warn before halting to avoid monitor-driven human overload. (L8-9)
- DJ002-93-006 `[E|capability/gap]` Leg B fills H-49/H-35's inability to reverse external effects by registering a compensation for each side-effecting Tier-C tool when invertible. (L11-14)
- DJ002-93-007 `[E|recovery/security]` Example compensations are git-push revert plus force-with-lease and file-create deletion; `/rewind` executes them and journals the reversal rather than rewinding only conversation/file state. (L12-14)
- DJ002-93-008 `[E|permission/failure]` Noninvertible effects remain explicitly marked for human handling and are never auto-reverted; email send and unrecoverable delete exemplify the boundary. (L13-14)
- DJ002-93-009 `[E|security/gap]` Leg C fixes persistent-memory taint loss by storing G-1 `Provenance` with facts through E-5 write/retrieve and H-51 cross-session handoff so capability gates see retrieved taint. (L16-19)
- DJ002-93-010 `[E|fallback/lifecycle]` Preexisting unlabeled facts are revalidated/relabelled on next retrieval through J-21/H-32; until then they are UNTRUSTED, never trusted by default. (L19)
- DJ002-93-011 `[E|event]` Leg C reuses `FactValidationFailed` or `MemoryGovernanceEvent(+Kind)` instead of inventing a separate memory event. (L18-19)
- DJ002-93-012 `[E|security/gap]` Leg D routes each GRC grounding tag through deterministic non-LLM D-10 `SOURCE_CHECK` against the actual source span at J-83's per-step receipt gate. (L21-24)
- DJ002-93-013 `[E|failure/fallback]` Source/file/test/API modes can produce step receipts; reasoning-only steps without an external receipt must abstain or escalate and cannot silently pass. (L23-24)
- DJ002-93-014 `[E|event]` Leg D reuses the grounding-verification event with an added kind. (L24)
- DJ002-93-015 `[E|security/gap]` Leg E diffs the exact dropped constraint against I-45's sealed typed objective when `GoalDriftCanary` fires, separating preference drift from capability drift. (L26-29)
- DJ002-93-016 `[E|workflow/permission]` Dropping a declared SAFETY/PRIVACY/SECURITY constraint escalates to a human with no auto-retry; capability drift instead receives persona switching. (L27-29)
- DJ002-93-017 `[E|fallback/schema]` Constraint class must be declared by L1/J-89; an undeclared dropped constraint cannot be diffed and therefore fail-safely escalates. (L29)
- DJ002-93-018 `[E|capability/gap]` Leg F deterministically derives rule-engine labels from complete SSB outcomes rather than an in-boundary LLM judge. (L31-34)
- DJ002-93-019 `[E|data/lifecycle]` A block is a false positive only when the same change later passes every deterministic wiring/cant-lie/mutation/blast gate and ships without downstream regression. (L32-34)
- DJ002-93-020 `[E|failure/dependency]` An incomplete journal makes the outcome UNLABELED, never assumed clean; J-87 completeness is load-bearing. (L33-34)
- DJ002-93-021 `[E|capability/gap]` Leg G adds stdlib `.claude/tools/feature_map_derive.py` to derive status from I-39, schema/variant counts from grep, and composition edges from `elai.map`/codegraph contracts. (L36-39)
- DJ002-93-022 `[E|data/constraint]` Only design-intent “why” remains hand-authored; disagreement between mechanical status and narrative is flagged rather than silently overwriting the narrative. (L37-39)
- DJ002-93-023 `[E|event/surface]` Leg G adds no SSB row and uses harness log/tool output; K-22 is the only cross-track tie. (L38-39)
- DJ002-93-024 `[E|status/dependency]` Ground-truth status spans WIRED E-6/G-1/GRC/D-10/rules, WIRED/PARTIAL H-49, PARTIAL memory/handoff/canary, and PLANNED J-21/J-83/I-45/I-39; each seam must be re-grepped. (L41-42)
- DJ002-93-025 `[C|schema/status]` The approximate 268/577 authoring baseline is historical and must not be used as a current schema literal. (L42, L80)
- DJ002-93-026 `[A|event/schema]` All legs prefer existing action types with new kinds, but the exact per-leg review may still yield net 0..+2 across the file. (L44-45)
- DJ002-93-027 `[E|default/config]` Every gate leg is auditor-owned and observe-only first; any new flag needs a wire-dark allowlist row and promotion plan. (L44-46)
- DJ002-93-028 `[E|surface]` Legs A/B/E surface in audit/canvas, leg G through tool output, and absent frontends require W-6 justification. (L46, L71)
- DJ002-93-029 `[E|interfaces]` Exact producers are `mast_inloop_supervisor`, `compensation_registry`/`rewind_external`, `persist_provenance`/retrieved-taint gate, `per_step_source_check`, `dropped_constraint_diff`, `derive_outcome_labels`, and `feature_map_derive.py`. (L48-56)
- DJ002-93-030 `[E|integration]` Exact consumers pair each producer to E-6+SSB, H-49/H-35+Tier-C, G-1+memory+H-51, GRC+D-10+J-83, GoalDrift+I-45+J-89, J-87+rules-3F, or I-39+map/grep. (L57-58)
- DJ002-93-031 `[E|reachability]` Each leg must execute at its real production seam during a live run; a contracts-only caller never establishes wiring. (L59-61)
- DJ002-93-032 `[E|constraint]` No leg may put an LLM on its decision path; individual hard bans also prohibit noninvertible auto-revert, default trust for unlabeled memory, receiptless step pass, safety-drift retry, incomplete-record label, and narrative overwrite. (L62-70)
- DJ002-93-033 `[E|default/measurement]` No gate blocks until false positives are bounded on at least three runs and an allowlist row exists. (L70)
- DJ002-93-034 `[E|acceptance]` Live examples require mid-run divergence detection before completion, actual reversal and journaling of an invertible git push, human surfacing of a noninvertible effect, and privacy-drift halt without retry. (L72-75)
- DJ002-93-035 `[E|negative-test]` Unparseable leg-A stream fails instead of skipping, incomplete leg-F outcomes return UNLABELED, and leg-G disagreement flags without overwriting. (L75)
- DJ002-93-036 `[E|grounding/constraint]` External reports/Atomix/Bedrock motivate the designs only; the ELAI observable effects, not external claims, are the done gates. (L77-80)
- DJ002-93-037 `[E|risk/concurrency]` Seven gates risk overblocking, so a per-changeset traversal budget caps firings and sequence prioritizes A and C before the rest. (L85-86)
- DJ002-93-038 `[E|dependency/status]` C and F close through J-87, D lands at J-83, and G is K-22's cross-track tie; the file explicitly restates none of its many upstream halves. (L82-89)

### 8. J-94_lacuna_tier3_honesty_thin_measure_first.md — 73/73 lines

- DJ002-94-001 `[E|status/scope]` J-94 separates small observe-only Tier-3 honesty fixes from uncertain THIN items that must be measured before build; it depends on J-87 for shadow anomalies and J-90 for resilience strictness. (L1-4)
- DJ002-94-002 `[E|measurement/routing]` T3-A replays sampled D-6 held-out claims through every grounding method offline and tightens D-10 when cheap and expensive methods disagree too often. (L6-7)
- DJ002-94-003 `[E|constraint/evidence]` T3-A declares sample coverage and may not claim global sufficiency; it reuses the existing grounding router and adds no SSB type. (L7)
- DJ002-94-004 `[E|capability/routing]` T3-B deterministically classifies query modality from query AST plus C-29 `IntentPair`, selecting channel subsets before fusion—exact-symbol can be grep-only; conceptual logic can use dense+graph. (L9)
- DJ002-94-005 `[E|non-goal]` T3-B answers WHICH retrieval channel, while J-73 separately answers WHEN to retrieve. (L9)
- DJ002-94-006 `[E|fallback]` Low-confidence or unparseable queries widen to the full channel set and never narrow speculatively; `QueryTrace` records behavior with no new SSB type. (L9, L46-47, L53-56)
- DJ002-94-007 `[E|security/observability]` T3-C propagates `QueryTrace/SearchMode` completeness into run metadata; `DEGRADED(BM25_ONLY)` stamps `context-incomplete` and raises audit strictness. (L11)
- DJ002-94-008 `[E|constraint/event]` The degradation marker reuses `knowledge_degradation(+Kind)` and labels rather than blocks; the auditor remains decision authority. (L11, L48)
- DJ002-94-009 `[E|compliance/determinism]` T3-D grants local `ReproPack` tier T1 only when attention is batch-invariant; otherwise it honestly downgrades to T0, reusing `ReproPackEvent(+Kind)`. (L13)
- DJ002-94-010 `[E|non-goal]` T3-D does not fix Ampere/FA2 attention nondeterminism; it prevents overclaiming token-faithful reproduction. (L13, L68)
- DJ002-94-011 `[E|surface]` T3-E is a read-only SSB projection comparing I-45's sealed plan skeleton with executed actions and flags skipped, substituted, and drifted steps, coloring safety-class drift red. (L15)
- DJ002-94-012 `[E|workflow/non-goal]` T3-E surfaces drift to a human but does not gate it; J-93 leg E owns safety-drift action. (L15)
- DJ002-94-013 `[E|deferred/measurement]` THIN-1 proposes H-49 edit-grain superstep barriers between tool phases but defers them until mid-agent failure rate demonstrates value. (L17-20)
- DJ002-94-014 `[E|deferred/constraint]` THIN-2 proposes phase-boundary model switching only after THIN-1 and only if measured routing benefit exceeds HD-2 cache-breakpoint/cache-bust cost. (L20)
- DJ002-94-015 `[E|status/non-goal]` THIN-3 largely reframes existing deterministic checkers as the local stronger verifier; only moving mutable=0 immune-tier membership into frozen `.claude/gate-manifest.txt` is buildable now. (L21-22)
- DJ002-94-016 `[E|deferred/dependency]` THIN-4 renders shadow effects lacking journal rows as canvas anomalies only after J-87 creates the shadow stream. (L22)
- DJ002-94-017 `[E|status/dependency]` The source enumerates D-6/D-10/J-73/C-29/QueryTrace/J-13/J-14/J-91/I-45/GoalDrift/I-39/H-49/HD-2/H-43/J-68/J-87 as the existing substrate, with mixed dark/wired/partial/planned state. (L24-25)
- DJ002-94-018 `[C|schema/status]` The approximate 268/577 baseline is a historical authoring snapshot and must be re-grepped. (L26, L61)
- DJ002-94-019 `[E|event/schema]` All Tier-3 legs reuse existing kinds or are read-side views, for net-zero new action types; observe-only flags require allowlist only where a flag exists. (L28-29)
- DJ002-94-020 `[E|surface/integration]` T3-E requires a Svelte drawer, Tauri command, and dashboard smoke; THIN-4 later adds J-60 canvas anomaly nodes. (L29-31)
- DJ002-94-021 `[E|scope/status]` The only THIN implementation in this wave is frozen-manifest hardening; THIN-1/2/4 stay recorded with explicit measurement triggers. (L30)
- DJ002-94-022 `[E|interfaces]` Producers are `calibrate_grounding_router`, `classify_query_modality`, `stamp_context_incomplete`, `attention_determinism_tier`, `plan_vs_execution_drift`, and the frozen immune-tier row. (L33-42)
- DJ002-94-023 `[E|reachability]` Each Tier-3 behavior must fire from a real live seam, and THIN-3 from the real freeze checker; tests alone never satisfy. (L43-45)
- DJ002-94-024 `[E|constraint]` The contract bans speculative narrowing, trust-marker blocking, T1 claims on nondeterministic attention, unmeasured THIN builds, and mutable-L1 immune-tier membership. (L46-51)
- DJ002-94-025 `[E|acceptance]` Forced BM25 degradation must label the run and raise audit strictness; a skipped sealed step must render red; moving immune membership out of the manifest must fail the freeze checker. (L53-56)
- DJ002-94-026 `[E|negative-test]` Unparseable retrieval widens, nondeterministic-attention T1 is refused, and mutable immune-tier relocation fails rather than passes. (L56)
- DJ002-94-027 `[E|grounding/constraint]` Reasoning sections 5/6 are design rationale only; live observable effects are the acceptance evidence. (L58-61)
- DJ002-94-028 `[E|risk/non-goal]` Gate proliferation has diminishing trust return and rising false-positive cost; building THIN-1/2/4 without their measured cases would violate the file's core discipline. (L66-67)
- DJ002-94-029 `[E|dependency/status]` T3-E consumes J-93 leg E, THIN-4 waits for J-87, and the feature explicitly closes the track by shipping cheap fixes while preserving honest residue. (L63-71)

### 9. J-106_output_metacommentary_enforcement.md — 122/122 lines

- DJ002-106-001 `[E|status/scope]` J-106 is a small-medium Wave-1 PLANNED runtime validator, default-off/observe-only, covering output-side behavior only for op-emission roles. (L1-5)
- DJ002-106-002 `[E|gap]` I-32 strips narrative persona on input, but the product lacks a runtime banned-phrase validator, work-log grammar, and action-density metric, so ceremony can still appear on output. (L3, L9-10)
- DJ002-106-003 `[E|grounding/constraint]` This is house doctrine and a code-grep gap, not a paper claim; reported output-formatting signatures are targets to measure on ELAI's corpus, not externally justified constants. (L5-7)
- DJ002-106-004 `[E|security/non-goal]` HD-3 is load-bearing: the validator and grammar apply only to executor/status op-emission roles; reasoning and synthesis roles are provably untouched and retain free prose. (L12)
- DJ002-106-005 `[E|workflow]` Output-side validation complements input persona stripping by deterministically rejecting ceremony and retrying, rather than merely hoping I-32's prompt shaping controls the result. (L14)
- DJ002-106-006 `[E|dependency/status]` Reused substrate is planned I-32 role classification, planned I-47 dropped-row scream path, partial E-35 structured outputs, planned I-46 frozen fixup table, and partial E-49 metrics. (L16-21)
- DJ002-106-007 `[A|event/schema]` J-106 prefers an existing violation/emission row for net zero but may add one Pattern-2 `OutputCommentaryRejected`; the choice is deferred to the decisions artifact and live schema review. (L22, L36-38)
- DJ002-106-008 `[C|schema/status]` The approximately 268/577 baseline is a historical authoring snapshot; only a chosen new event triggers a live-derived increment. (L22)
- DJ002-106-009 `[E|process/dependency]` Mandatory reads include the full source reasoning section, output-formatting doctrine, I-32 role code, I-47 scream path, E-35/E-49 seam, op-emission output call site, and I-46 fixup pattern. (L24-29)
- DJ002-106-010 `[E|capability/determinism]` Task 1 creates frozen `output_commentary_gate.rs` with a mutable=0 banned-phrase table of ceremony/affirmation/hedging openers and deterministic reject plus bounded retry. (L31-32)
- DJ002-106-011 `[E|security/change-control]` The phrase table is gate-manifest-eligible, seeded only from explicit doctrine bans, and expands only through manifest review. (L32, L105, L115)
- DJ002-106-012 `[E|schema/workflow]` Task 2 defines READ/FOUND/DECISION/ACTION/RESULT-shaped work-log output for op-emission and uses reject-and-retry-to-shape, not constrained token decoding or forced JSON. (L33)
- DJ002-106-013 `[E|test/security]` Task 3 reuses I-32 classification for both checks and requires the same ceremony-laden prose from a reasoning role to pass byte-unchanged. (L34)
- DJ002-106-014 `[E|measurement/non-goal]` Task 4 records tool-actions per prose block and words per message on E-49's evaluation corpus; action density is an eval signal, never a runtime gate. (L35)
- DJ002-106-015 `[E|failure/observability]` Task 5 routes every dropped/violating output through I-47's loud scream path rather than silently swallowing it. (L36)
- DJ002-106-016 `[E|event/schema]` If selected, `OutputCommentaryRejected` requires one Pattern-2 action type and the complete nine-tuple/golden lockstep; otherwise existing rows are reused. (L37)
- DJ002-106-017 `[E|evidence/config]` `J-106-decisions.md` records event choice, phrase-table seed, and the reasoning-corpus zero-FP observation. (L38)
- DJ002-106-018 `[E|interfaces]` Exact producers are frozen phrases, bounded retry, work-log shape, HD-3 role gate, action-density metrics, scream path, and optional event; consumers are I-32, I-47, E-35/E-49, I-46, and live op-output. (L40-53)
- DJ002-106-019 `[E|reachability]` A live op-emission output must reject/retry, the same live reasoning prose must pass untouched, and an eval run must record metrics; tests alone never establish either path. (L54-58)
- DJ002-106-020 `[E|constraint]` Any constraint on reasoning, hard constrained decoding even for op-emission, LLM ceremony judging, silent discard, or early default-on behavior is forbidden. (L59-64)
- DJ002-106-021 `[E|flag/default]` `ELAI_OUTPUT_METACOMMENTARY_GATE_ENABLE` is default-off with a wire-dark allowlist row and `promotion_plan_id=J-106-promotion`; promotion waits for zero false positives on the reasoning corpus. (L64, L88, L92-94)
- DJ002-106-022 `[A|surface]` Runtime frontend may be absent if W-6-justified, while action density uses E-49's metrics surface; an extra dashboard tile is an edit-time choice. (L65-66)
- DJ002-106-023 `[E|acceptance]` Ceremony from op-emission must be rejected and retried into compliant work-log shape, while identical reasoning output remains untouched and the evaluation corpus receives action-density values. (L67-70)
- DJ002-106-024 `[E|negative-test]` Any reasoning-role constraint is a blocking failure; optional reject rows, if used, must be journal-recomputable. (L68-70)
- DJ002-106-025 `[E|test]` Deterministic checks require a real production caller, explicit role scoping, phrases/grammar/retry, action-density wiring, and the decisions file. (L72-80)
- DJ002-106-026 `[E|acceptance/default]` The phrase table must be frozen/deterministic, retries bounded, violations loud, no LLM judge introduced, no new dependency added, and action-lock green if the optional event lands. (L82-89)
- DJ002-106-027 `[E|done-gate]` Completion is an end-to-end live reject+retry plus live untouched reasoning output, not table existence; anchors and schema are re-grepped and HD-3/TD-3/GD-4 remain direct gate clauses. (L91-94)
- DJ002-106-028 `[E|test]` The contract covers op-role reject/retry, identical reasoning pass-through, explicit failure on reasoning constraint, deterministic reruns under `ELAI_SEED`, bounded retry, scream path, and metric recording. (L96-97)
- DJ002-106-029 `[E|files/config]` Implementation creates the gate and decisions file; modifies `report_fns.rs` and eval seams; optionally updates split SSB lockstep; adds an experiment allowlist row and freezes the phrase table in `gate-manifest.txt`. (L99-105)
- DJ002-106-030 `[E|risk]` A single reasoning-role false positive blocks default-on; mitigation is I-32 classification, observe-only mode, and a zero-FP corpus requirement. (L113-115)
- DJ002-106-031 `[A|config/risk]` Phrase-list breadth trades missed ceremony against legitimate-output overblocking; the explicit doctrine list is the initial frozen seed. (L115)
- DJ002-106-032 `[E|failure/recovery]` Work-log rigidity is mitigated with bounded reject/retry followed by pass-through-with-a-scream, not infinite retry or forced decoding. (L117)
- DJ002-106-033 `[E|dependency/status]` The feature closes I-32's output half and composes I-47/E-35 without re-authoring them; result-first/action-density doctrine becomes measured product behavior. (L107-120)

### 10. J-109_learned_artifact_admission_hardening.md — 114/114 lines

- DJ002-109-001 `[E|status/scope]` J-109 is a PLANNED S-M Wave STEAL-JUL04 extension: one admission pipeline fronts memory, playbook, case-reuse, and skill learning seams using mechanisms reimplemented—not copied—from AgenticSTS. (L1-10)
- DJ002-109-002 `[E|gap/security]` Existing admission checks dedup, poisoning, and evidence but not duplication of immutable instruction surfaces or whether a candidate changes the originating failure. (L5, L12-13)
- DJ002-109-003 `[E|architecture/security]` One `AdmissionPipeline` adds a `StaticSpanIndex`, trigger-aware dedup band, and prewrite counterfactual replay, bound by K-19 execution-grounded fitness, H-31 auditor isolation, and TD-3 advisory-only judges. (L5)
- DJ002-109-004 `[E|default/status]` The pipeline is default-off and observe-only first; estimated effort is one to two sessions, with leg A as index/decision and leg B reusing journal prompts and lane runtime. (L5-7)
- DJ002-109-005 `[E|dependency]` Required seams are E-5 Mem0 ADD/UPDATE, I-13 injection scan, I-12 curator, J-96/J-97 cases, J-102 immutable manifest, H-43 fitness, dual ONNX+circuit breakers, and SessionDag/SSB prompt records. (L8)
- DJ002-109-006 `[E|concurrency/dependency]` J-109 is parallel-safe with J-110; J-110's late-abstention flags later enter as I-12 stopping-rule candidates through this pipeline. (L9, L111-112)
- DJ002-109-007 `[E|risk/data]` The two motivating failures are rule rediscovery that displaces novel memory and inert lessons that do not flip the mistake they claim to fix. (L12-13)
- DJ002-109-008 `[E|security/evidence]` ELAI replay is stronger than the reference where a runnable F-4 outcome exists: byte-exact journal prompts feed execution-grounded gates; LLM judgment may annotate only nonexecutable residue. (L13)
- DJ002-109-009 `[E|workflow/idempotency]` AD-1 routes exact/near-exact matches to E-5 UPDATE rather than creating a duplicate row. (L15-16)
- DJ002-109-010 `[E|security/data]` AD-2 rejects `cosine >= T_immutable` against immutable spans from J-102's ACTIVE rules/operator prompts/CLAUDE-class config; the starting threshold is 0.70. (L17)
- DJ002-109-011 `[E|determinism/lifecycle]` `StaticSpanIndex` fingerprints J-102 manifest hashes with blake3 and rebuilds before verdict if stale. (L17, L37-38)
- DJ002-109-012 `[E|workflow/data]` AD-3 rejects existing-store similarity at `T_dup=0.85`; in the 0.70-to-0.85 band, trigger Jaccard at `T_ctx=0.60` chooses MERGE for same context or COEXIST for distinct context. (L18)
- DJ002-109-013 `[E|config/security]` All three thresholds are named constants frozen with the gate rather than inline literals, and are merely starting values requiring ELAI recalibration. (L18, L42, L59)
- DJ002-109-014 `[E|workflow/data]` AD-4 fetches byte-exact originating prompt and outcome from SSB, injects the candidate, and performs `N=3` redecisions on the same originating lane. (L19)
- DJ002-109-015 `[E|acceptance/security]` Off-agent aggregation admits only with zero harmful outcomes and at least `ceil(2N/3)` flips. (L19)
- DJ002-109-016 `[E|evidence]` A flip is the executable F-4/gate outcome change when runnable, otherwise a deterministic decision diff; an LLM judge is advisory annotation only and never the verdict. (L19)
- DJ002-109-017 `[E|lifecycle/observability]` Candidates without an originating failure, such as user `/remember`, receive explicit `SKIPPED-BY-TYPE` with an enumerated covered set rather than a silent pass. (L19)
- DJ002-109-018 `[E|fallback/security]` Embedder failure degrades AD-2/AD-3 to labeled lexical token-set overlap and never blind-accepts. (L20)
- DJ002-109-019 `[E|status/dependency]` Existing E-5 ADD/UPDATE/DELETE/NOOP+MINJA, I-13 scan, I-12 k-outcome promotion, J-96/J-97 cases, H-43 `FitnessEvaluated`, J-102 manifest, C-48, and J-21 are composed rather than duplicated. (L22-29)
- DJ002-109-020 `[E|workflow/ordering]` The wider seam order is injection scan, trust tier, then J-109 AD-1..AD-4; admission precedes I-12's promotion k-counter. (L25, L29)
- DJ002-109-021 `[E|permission/privacy]` H-31 firewall remains absolute: no artifact admitted here may enter auditor or critic context. (L5, L35, L82)
- DJ002-109-022 `[E|process/dependency]` Mandatory reads cover AgenticSTS mechanisms, live Mem0/I-12/J-96/J-97 seams, J-102's manifest, self-improvement SF-1..SF-4, and H-31/TD-3/K-19. (L31-35)
- DJ002-109-023 `[E|task/integration]` Task 1 chunks and embeds J-102 surface through existing Granite dual-ONNX/circuit-breaker infrastructure, with blake3 freshness and lexical fallback. (L37-38)
- DJ002-109-024 `[E|schema/lifecycle]` Task 2 defines verdicts `Update`, `RejectImmutableSpan`, `RejectDuplicate`, `Merge`, `Coexist`, `ReplayRequired`, and `SkippedByType`, with every decision journaled. (L39)
- DJ002-109-025 `[A|event/schema]` Telemetry prefers `MemoryGovernanceEvent`/`FitnessEvaluated(+Kind AdmissionVerdict)` for net 0..+1, with exact action-type impact decided against the live lock. (L39)
- DJ002-109-026 `[E|task/concurrency]` Task 3 performs N=3 same-lane redecisions through `PerRoleRuntime`, uses J-96 provenance as the SSB pointer, and aggregates off-agent. (L40)
- DJ002-109-027 `[E|flag/default]` Task 4 wires E-5 ADD, I-12 intake, and J-97 admission behind `ELAI_ADMISSION_HARDENING_ENABLE`; observe-only rows land while writes remain unblocked until false positives are bounded over at least three real runs. (L41)
- DJ002-109-028 `[E|config]` The enable flag requires an experiment allowlist row with owner, expiry, `promotion_plan_id=J-109`, and smoke. (L41)
- DJ002-109-029 `[E|evidence/config]` `J-109-decisions.md` records threshold provenance/recalibration, ordered legs, advisory-judge constraint, and covered SKIPPED types. (L42)
- DJ002-109-030 `[E|interfaces]` `admission_pipeline.rs` produces `StaticSpanIndex`, `AdmissionPipeline::check`, and `CounterfactualReplay`; consumers are Mem0, I-12, J-97, J-102, ONNX breaker, SSB/J-96, H-43, and `PerRoleRuntime`. (L44-51)
- DJ002-109-031 `[E|reachability]` Production evidence is a real E-5 write with the enable flag and observe-only rows; tests alone never satisfy. (L52-54)
- DJ002-109-032 `[E|constraint]` The pipeline bans load-bearing LLM judgment, blind acceptance, verdict-less writes, separate immutable enumeration, inline thresholds, and production unwrap. (L55-59)
- DJ002-109-033 `[A|surface]` Memory governance should render immutable/duplicate/merge/replay counts by reusing H-33; exact UI reuse is left to edit-time W-4 review. (L60-61)
- DJ002-109-034 `[E|acceptance/observability]` A live immutable-span candidate must produce AD-2, and an originating-failure candidate must produce N=3 replay receipts; rows carry candidate hash, leg, thresholds, and receipts for recomputation. (L62-65)
- DJ002-109-035 `[E|negative-test]` A write with no verdict row and a claimed replay flip without redecision receipts must fail as fabricated admission fitness. (L65)
- DJ002-109-036 `[E|test]` Deterministic checks require named constants, zero judge-as-verdict patterns, explicit skip/fallback/breaker paths, a real production caller, and the decisions artifact. (L67-75)
- DJ002-109-037 `[E|acceptance]` The ordered pipeline must front all named seams, reject immutable duplication from a fresh index, admit only zero-harmful supermajority flips, label degradation, preserve H-31 isolation, and produce live auditor evidence. (L77-83)
- DJ002-109-038 `[E|licensing/compliance]` Mechanisms are reimplemented from Apache-2.0 source summaries; AGPL code is untouched and no source code is copied. (L85-89)
- DJ002-109-039 `[E|test]` Contract cases cover immutable duplicate, store duplicate, same-trigger merge, distinct-trigger coexist, useful versus inert replay, one-harm rejection, breaker lexical fallback, missing row rejection, and stale-index rebuild. (L91-92)
- DJ002-109-040 `[E|files]` Implementation creates one approximately 330-LOC orchestrator module, modifies three admission call sites, conditionally changes SSB lockstep, adds one allowlist row, one contract, and one decisions file. (L94-99)
- DJ002-109-041 `[A|measurement/risk]` Thresholds 0.70/0.85/0.60 came from game-strategy text and another embedder; they must be recalibrated from ELAI observe-only data under `ELAI_SEED`. (L101-107)
- DJ002-109-042 `[E|concurrency/cost]` AD-4's N=3 toll is bounded by a per-session budget and queue while I-12's k-counter continues independently. (L108)
- DJ002-109-043 `[E|risk/recovery]` AD-2 may reject a genuinely sharper restatement; the merge band and measured false-positive phase mitigate this, and hard blocking is prohibited before evidence. (L109)
- DJ002-109-044 `[E|dependency/status]` Completion supplies missing admission-time redundancy/fitness beneath promotion-time filters; J-110 closes the feedback loop by sending late-abstention candidates through the same pipe. (L111-112)

## Inferred findings (not asserted as direct source facts)

- DJ002-INF-001 `[I|schema/integration]` J-22's all-payload field-shape lock would automatically govern any new `LocalPrefixCacheEvent`, J-52 enforcement event(s), optional `OutputCommentaryRejected`, or J-109 admission kind once those payloads enter the SSB lock; none of those four plans explicitly names J-22 as its schema gate. (J-22:L8-L16; J-36:L133-L135; J-52:L37; J-106:L37; J-109:L39)
- DJ002-INF-002 `[I|workflow/ordering]` On a protected-file edit, the safe composed order appears to be J-27 placeholder restoration/block first, followed by J-52 diagnostics and path-rule injection; the sources share PostToolUse but do not state hook order, so this is an inferred requirement rather than a contract. (J-27:L17-L26; J-52:L12-L14,L35-L36)
- DJ002-INF-003 `[I|cache/integration]` J-52's post-edit Rules-slot injection can legitimately change the rules portion of J-36's frozen-head hash between turns; the safe consequence is a visible head miss and fresh cache entry, not reuse of stale rules. Neither file defines this interaction. (J-36:L60-L65,L171-L175; J-52:L14,L45-L46)
- DJ002-INF-004 `[I|workflow/integration]` Once J-44's reasoning-method rule participates in conditional routing, J-52's static path-rule injector is a plausible mechanical carrier for it; J-44 itself specifies routing/preamble injections but never names J-52. (J-44:L8-L14,L29-L32; J-52:L14,L53-L54)
- DJ002-INF-005 `[I|security/data]` J-93 leg C and J-109 are complementary at distinct lifecycle points: persistent provenance protects retrieved facts, while admission hardening filters new learned artifacts before storage. No assigned source directly orders the two gates. (J-93:L16-L19; J-109:L15-L29)
- DJ002-INF-006 `[I|evidence/integration]` J-109's replay/verdict journal depends operationally on J-22's sole commit point and stable payload schema for durable audit, although J-109 names SSB/H-43 rather than J-22. (J-22:L8-L9; J-109:L19,L39-L40,L62-L65)
- DJ002-INF-007 `[I|security/non-goal]` J-94 THIN-3's frozen gate-manifest and J-109 AD-2's J-102 immutable instruction manifest are different enumerations serving different threats; treating one as the other's source would violate J-109's no-re-enumeration rule. (J-94:L21,L30,L40-L42; J-109:L17,L28,L49,L58)
- DJ002-INF-008 `[I|workflow/dependency]` J-93 leg D is a downstream realization of the grounding map's receipt-selected `BeliefState`: per-step GRC tags obtain the external validator receipt the carrier requires. (grounding:L12-L18,L63-L66,L101-L104; J-93:L21-L24)
- DJ002-INF-009 `[I|surface/workflow]` J-94's read-only drift drawer and J-93's preference-drift gate share the same I-45 seal but deliberately split visibility from action; UI red status should therefore not itself trigger retry/escalation. (J-93:L26-L29; J-94:L15,L42,L54-L56)
- DJ002-INF-010 `[I|determinism/concurrency]` J-36 cache state, J-52 nag counters/daemon state, and J-109 replay queues all introduce session-scoped mutable state; their sources separately demand deterministic evidence but do not define restart serialization, so cross-restart equivalence cannot be assumed. (J-36:L93-L101,L214-L215; J-52:L10-L12,L30-L31; J-109:L40,L108)

## Ambiguities and unresolved decisions

The following already-defined `[A]` atoms are the source-declared choice/reconciliation set and are not redefined here: `DJ002-22-018..019`, `DJ002-36-018`, `DJ002-44-022`, `DJ002-44-033..034`, `DJ002-52-022`, `DJ002-52-040..041`, `DJ002-93-026`, `DJ002-106-007`, `DJ002-106-022`, `DJ002-106-031`, `DJ002-109-025`, `DJ002-109-033`, and `DJ002-109-041`.

- DJ002-AMB-001 `[A|schema]` J-22 promises type-change detection but alternates between “field shape” and “field set”; the canonical snapshot representation needed to encode Rust types is unspecified. (J-22:L5,L8,L30,L39,L48)
- DJ002-AMB-002 `[A|schema]` J-27 says annotation comment syntax is “per language” but defines no grammar, nesting/overlap policy, duplicate-hash handling, or binary/non-UTF-8 behavior. (J-27:L17-L20,L67-L69)
- DJ002-AMB-003 `[A|privacy/lifecycle]` J-27 stores body, validators, URL hash, and original prompt metadata but gives no eviction, retention, access-control, or prompt-redaction rule for the shared session cache. (J-27:L27-L33,L70-L71)
- DJ002-AMB-004 `[A|config]` J-36 does not enumerate which concrete architectures initially pass `prefix_kv_eligible` or define the exact `ModelCapabilityProfile`/catalog field representation. (J-36:L78-L85,L130-L132,L248-L255)
- DJ002-AMB-005 `[A|workflow]` J-44 leaves sample count N, sampling cadence, held-out construction, and the remediation-velocity promotion SLO undefined. (J-44:L10,L27,L54)
- DJ002-AMB-006 `[A|schema]` J-52 imports a reference format whose last line must exactly be `EVIDENCE_RECORDED: <path>`, but the ELAI port only says the return “carries an evidence receipt”; whether the exact serialization is normative is not stated. (J-52:L10,L35,L43-L44)
- DJ002-AMB-007 `[A|workflow/recovery]` J-93 leg A mandates redirect/restart but does not specify which classification produces redirect versus restart or how supervisor state survives that transition. (J-93:L7-L9)
- DJ002-AMB-008 `[A|recovery/idempotency]` J-93 leg B does not define compensation idempotency, partial-compensation recovery, or concurrency when later effects depend on the effect being reversed. (J-93:L11-L14,L50-L51)
- DJ002-AMB-009 `[A|measurement/config]` J-94 leaves D-6 calibration sample size, acceptable cheap/expensive disagreement, modality low-confidence threshold, and each `context-incomplete` audit-strictness increment unspecified. (J-94:L7-L11,L35-L39)
- DJ002-AMB-010 `[A|retry/failure]` J-106 requires bounded retry but names no retry count; pass-through-with-a-scream appears only as a risk mitigation, so the final runtime verdict after exhaustion is not fully contracted. (J-106:L32-L33,L42-L43,L83-L85,L117)
- DJ002-AMB-011 `[A|workflow]` J-109 does not define the similarity boundary between AD-1 “near-exact -> UPDATE” and AD-3 `>=T_dup -> REJECT`, so some candidates could satisfy both descriptions unless AD-1 receives a stricter exactness predicate. (J-109:L16-L18)
- DJ002-AMB-012 `[A|concurrency/config]` J-109 names a per-session AD-4 budget and queue but no budget, queue order, cancellation, restart persistence, or behavior when the budget is exhausted. (J-109:L108)

## Conflicts and material mismatches

The source-declared/stale `[C]` atoms already defined in the per-file ledgers are `DJ002-G-027..028`, `DJ002-22-009`, `DJ002-36-019..020`, `DJ002-52-017`, `DJ002-93-025`, `DJ002-94-018`, and `DJ002-106-008`.

- DJ002-CON-001 `[C|hash/config]` J-36 FH-1 says the key may compose I-15 “blake3/xxhash,” while its task and stub ban require blake3 only. The implementation must resolve this to the stricter blake3 path; the orientation summary already states blake3. (J-36:L60-L65,L124-L125,L161; summary:L408)
- DJ002-CON-002 `[C|scope/integration]` J-109's objective includes skill L-12 as a fronted write seam, but its task, consumer, observable, acceptance, and file-change lists wire only E-5, I-12, and J-97. Skill admission is currently declared scope without an implementation slice. (J-109:L3-L5,L8,L41,L48,L63,L78,L96)
- DJ002-CON-003 `[C|summary/rollout]` Summary J-44 says missing ATTACK/untagged claims “fail” without qualification; the original checker begins observe-only WARN and fails only after promotion, while the rule-level contract is fail-closed. (summary:L493; J-44:L7,L9,L33)
- DJ002-CON-004 `[C|summary/failure]` Summary J-52 broadens “transient diagnostics are not recorded”; the original explicitly guarantees non-recording for daemon-unreachable and permanence for unsupported extension, not an open-ended diagnostic taxonomy. (summary:L579; J-52:L12,L66,L94,L107)
- DJ002-CON-005 `[C|summary/scope]` Summary J-109 lists memory, playbook, and case-reuse seams, omitting the original objective's skill L-12 seam; because the original implementation list also omits skill, the summary hides rather than resolves `DJ002-CON-002`. (summary:L1154; J-109:L3-L5,L41,L48)
- DJ002-CON-006 `[C|summary/idempotency]` Summary J-109 narrows AD-1 to exact match; the original explicitly says exact/near-exact routes to UPDATE. (summary:L1155; J-109:L16)
- DJ002-CON-007 `[C|summary/lifecycle]` Summary J-109 describes AD-3 only as merge/coexist splitting, omitting the original `>=T_dup` `RejectDuplicate` terminal branch. (summary:L1157; J-109:L18,L39)
- DJ002-CON-008 `[C|status]` J-52's new per-edit daemon is default-on behind a disable, while its consumed E-52 probe is described as dark behind an unallowlisted enable with real subprocess execution deferred; the plan does not state whether J-52 supersedes, removes, or bypasses that old enable. (J-52:L16,L22,L30,L69,L96)

## Cross-file interaction ledger

| Interaction | Classification | Exact relationship and anchors |
|---|---|---|
| Grounding map → J-36 | Explicit scope distinction | The map says J-83's carrier consumes H-50 unload and does not itself solve streaming KV reuse; J-36 is the separate later unit that activates streaming prefix caching. This is compatible only if the two scopes remain distinct. Grounding:L27-L30,L133-L135; J-36:L67-L89. |
| Grounding map → J-93 D | Explicit | J-83's carrier admits externally receipted entries; J-93 D wires deterministic `SOURCE_CHECK` at exactly that per-step receipt seam. Grounding:L63-L66,L101-L104; J-93:L21-L24. |
| J-22 → proposed SSB changes | Inferred | All payloads proposed by J-36/J-52/J-106/J-109 would fall under the global payload-shape and sole-writer tests; see `DJ002-INF-001`. |
| J-27 → J-52 | Inferred ordering | Protected bytes must be restored or blocked before per-edit LSP/path-rule handling can safely inspect the final edit, but hook order is not contracted; see `DJ002-INF-002`. |
| J-36 ↔ J-52 | Inferred cache invalidation | A path-triggered Rules-slot change changes the frozen-head content and should cause a miss/rekey, never stale KV reuse; see `DJ002-INF-003`. |
| J-44 → J-52 | Inferred carrier | J-44 adds a reasoning-method conditional rule; J-52 can mechanically inject path-keyed rules, but neither file declares that direct wiring; see `DJ002-INF-004`. |
| J-93 E → J-94 E | Explicit producer/view split | J-93 deterministically escalates dropped safety-class constraints; J-94 renders the same sealed-plan drift read-only and explicitly says the J-93 gate acts. J-93:L26-L29; J-94:L15,L42,L54-L56,L70-L71. |
| J-93 C/F → J-87 | Explicit dependency | Persistent-taint completeness and outcome labels require a complete effect journal; incomplete outcomes remain unlabeled. J-93:L3,L16-L19,L31-L34,L88-L89. |
| J-94 THIN-4 → J-87 | Explicit deferred ordering | Canvas shadow anomaly nodes build only after J-87 provides shadow-effect evidence. J-94:L3,L22,L30-L31,L70-L71. |
| J-109 → J-110 | Explicit feedback loop | Late-abstention signals become stopping-rule candidates via I-12 and then traverse J-109 admission; J-109 and J-110 remain parallel-safe. J-109:L9,L111-L112. |
| J-93 C ↔ J-109 | Inferred lifecycle layering | J-109 filters writes while J-93 C preserves taint across storage/retrieval; see `DJ002-INF-005`. |
| J-94 THIN-3 ↔ J-109 AD-2 | Inferred non-equivalence | Frozen gate membership and the J-102 immutable instruction manifest are separate canonical sources; see `DJ002-INF-007`. |
| J-27/J-44/J-52 evidence | Explicit/inferred | J-27 uses harness logs and no SSB, J-44 uses audit/remediation logs and no SSB, while J-52 proposes journaled hook events and a panel. They share off-agent enforcement doctrine but not a unified event schema. J-27:L87-L103,L121; J-44:L26-L35; J-52:L37-L38,L56-L59. |

## Exhaustive summary delta ledger

The orientation sections audited were `10-J-SUMMARY_INDEX.discovery.md:255-263,306-314,406-415,490-498,572-582,876-915,1021-1041,1119-1127,1152-1162,1225-1240`. “Add” means absent or materially under-specified in that summary. “Correct/qualify” refers to a conflict/ambiguity atom above. Facts represented accurately at equivalent grain are intentionally not repeated.

### Grounding-map delta against summarized J-83..J-86

- **Add:** `DJ002-G-001`, `DJ002-G-004..013`, `DJ002-G-017..020`, `DJ002-G-021..026`, `DJ002-G-030`, and `DJ002-G-032..035`: exact shipped/dark/partial substrate, symbols, event choices, schema derivation, external-evidence limits, compression rider, and seam/measurement residue.
- **No summary correction:** the summary accurately carries the map's central carrier, two-axis router, overthinking guard, and receipt-selected ladder. `DJ002-G-027..028` are corrections to the older reasoning document that the summary already incorporated.

### J-22 summary delta

- **Add:** `DJ002-22-001`, `DJ002-22-006`, `DJ002-22-008..014`, and `DJ002-22-017..020`: harden-not-replace posture, Merkle rider ownership, exact substrates/paths/reachability, test-layer frontend rationale, historical schema warning, grounding, dependency choice, serde edge case, and orphan-rider risk.
- **Qualify:** `DJ002-AMB-001`; the summary's “field names/types” is the intended contract, but the source's proposed snapshot representation says only “field set” and must encode types explicitly to deliver it. (summary:L259)

### J-27 summary delta

- **Add:** `DJ002-27-001`, `DJ002-27-005`, `DJ002-27-008`, `DJ002-27-013..020`, and `DJ002-27-023..026`: source/status, hook-freeze constraint, exact hashes/metadata, rejected ideas, ground-truth and tiers, files/reachability, no-SSB evidence, deterministic checks, and composition/policy details.
- **Add ambiguity:** `DJ002-AMB-002..003` for annotation grammar and cache privacy/lifecycle left unspecified.

### J-36 summary delta

- **Add:** `DJ002-36-001..008`, `DJ002-36-015..023`, `DJ002-36-025..030`, `DJ002-36-033..034`, and `DJ002-36-036..043`: the three concrete bypasses, resident-cog premise, exact code/counter/crop seams, tasks/files/reachability, checks, deterministic evidence, architecture residue, and downstream bottleneck.
- **Correct/qualify:** `DJ002-CON-001` resolves the source's xxhash/blake3 tension to the summary's stricter blake3 statement; `DJ002-36-019` corrects H-50's stale status; `DJ002-AMB-004` records the unenumerated eligibility catalog.

### J-44 summary delta

- **Add:** `DJ002-44-001..004`, `DJ002-44-005..009`, `DJ002-44-011..019`, `DJ002-44-022..027`, and `DJ002-44-030..035`: exact M1/M2/STANCE/M4/M6/M9/M10/M12 rules, observe-first experiment, disposer refusal, hosts, done evidence, no-SSB behavior, domain-transfer limit, storage/sample choices, and anti-theater residue.
- **Correct:** `DJ002-CON-003`; mechanical failure is rollout-dependent rather than immediately blocking during observe-only.

### J-52 summary delta

- **Add:** `DJ002-52-001..003`, `DJ002-52-004..007`, `DJ002-52-009..019`, `DJ002-52-020..027`, and `DJ002-52-028..043`: exact receipt grammar provenance, MAX=3/persistence/escalation, daemon concurrency/output bounds, current substrate state, tier riders, exact event kinds/files/flags/reachability, tests, grounding, relocation/fallback risks, and deferred riders.
- **Correct/qualify:** `DJ002-CON-004` narrows “transient” to the specified daemon-unreachable case; `DJ002-CON-008` records the unresolved old-E-52-enable versus new-default-on rollout.

### J-93 summary delta

- **Add:** `DJ002-93-001..002`, `DJ002-93-004..005`, `DJ002-93-007..011`, `DJ002-93-013..014`, `DJ002-93-016..020`, and `DJ002-93-021..038`: exact failure signatures/threshold, compensation safeguards, unlabeled-memory lifecycle, event reuse, drift fallback, outcome-label definition, feature-map narrative guard, producers/consumers/reachability, acceptance, and gate-budget sequencing.
- **Add ambiguity:** `DJ002-AMB-007..008` for redirect/restart policy and compensation idempotency/concurrency.

### J-94 summary delta

- **Add:** `DJ002-94-001..003`, `DJ002-94-004..012`, and `DJ002-94-013..029`: exact calibration and routing inputs, event reuse/nonblocking semantics, view/gate separation, each THIN measurement trigger, exact producers/reachability/negative arms, UI requirements, status, and honest-downgrade residue.
- **Add ambiguity:** `DJ002-AMB-009` for all four unnamed calibration/threshold/strictness values.

### J-106 summary delta

- **Add:** `DJ002-106-001..003`, `DJ002-106-005..020`, and `DJ002-106-022..033`: code-gap/doctrine provenance, input/output distinction, exact reused substrate, reject-not-force-decode behavior, optional event, decisions/config/files, real reachability/tests, UI choice, and exhausted-retry risk.
- **Add ambiguity:** `DJ002-AMB-010`; bounded retry has no numeric bound or fully normative exhaustion result.

### J-109 summary delta

- **Add:** `DJ002-109-001..008`, `DJ002-109-010..013`, `DJ002-109-017`, `DJ002-109-019..033`, and `DJ002-109-034..044`: declared skill scope, exact thresholds and manifest fingerprint, skipped-by-type, guard ordering, H-31 firewall, verdict enum/event choice, flags/allowlist, interfaces/UI, tests/licensing, queue/cost, and over-rejection residue.
- **Correct:** `DJ002-CON-005..007`: restore the unresolved skill seam, AD-1 near-exact update, and AD-3 terminal duplicate rejection omitted by the summary.
- **Add ambiguity:** `DJ002-AMB-011..012` for AD-1/AD-3 overlap and replay-budget/queue semantics.

## Coverage, omission, and duplicate audit

- Assigned files opened completely: **10/10** (one auxiliary/grounding source and nine numbered originals).
- Assigned logical lines read: **1,224/1,224**; bytes verified: **152,552/152,552**.
- Manifest line-count matches: **10/10**; byte-count matches: **10/10**; SHA-256 matches: **10/10**.
- Other original chunks read: **0**. Orientation used only from `10-J-SUMMARY_INDEX.discovery.md`; no source-ledger item is grounded in the orientation summary.
- Per-file atomic definitions: **346** (`G` 35; `J-22` 20; `J-27` 26; `J-36` 43; `J-44` 35; `J-52` 43; `J-93` 38; `J-94` 29; `J-106` 33; `J-109` 44).
- Classification totals: **321 explicit `[E]`**, **10 inferred `[I]`**, **28 ambiguous `[A]`** (16 source-ledger choices + 12 cross-audit ambiguities), and **17 conflicting/stale `[C]`** (9 source-ledger corrections + 8 cross/summary conflicts): **376 total unique atomic definitions**.
- Unique atomic definition IDs: **376/376**; duplicate definition IDs: **0**; empty/unclassified definitions: **0**.
- Requested facet audit: every source was checked for capabilities/actions/surfaces; workflows/states; schemas/data/lifecycle; flags/defaults/config; integrations/events; permissions/security/privacy/compliance; failures/fallback/retry/recovery; determinism/idempotency/concurrency; observability/evidence; tests/acceptance/done gates; constraints/non-goals; dependencies/status/deferred.
- Omission audit: every substantive normative sentence, named FH/GS/GA/DC/R/AD/T3/THIN/LEG/task identifier, symbol/seam, state/kind/verdict, threshold, flag/default, explicit exclusion, deferred rider, observable effect, negative arm, acceptance clause, and open risk in all ten sources maps to at least one atomic definition. Headings, end markers, citations repeated from a defining section, `COMPOSES-WITH` restatements, and repeated acceptance wording were folded into the first defining atom to avoid duplicates.
- Interaction audit: schema locking, hook ordering, rules/head invalidation, doctrine injection, receipt-carrier composition, drift gate/view separation, taint/admission layering, journal-completeness dependencies, and deferred feedback loops are covered above; links not explicitly declared are labeled inferred or ambiguous.
- Summary audit: every material source fact absent or under-specified in the orientation entries is referenced in the delta ledger, and every identified material mismatch is corrected/qualified. Accurately summarized facts are not duplicated as deltas.

## Output digest convention

The completed file cannot contain its own whole-file SHA-256 without changing that digest. The actual SHA-256 is therefore computed after the final write and reported to the parent/delivery message. This convention line is itself included in the hashed output.

