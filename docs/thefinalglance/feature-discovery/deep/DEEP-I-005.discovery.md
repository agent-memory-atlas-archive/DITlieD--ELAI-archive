# DEEP-I-005 deep feature discovery

## Scope, method, and evidence classes

This ledger is a complete read of the ten original files assigned to manifest chunk `DEEP-I-005`. The only secondary source consulted was `09-I-SUMMARY_INDEX.discovery.md`, and it was used only to orient and audit the prior summary. No original source from another manifest chunk was read.

Anchors are inclusive, one-based source line numbers. Short names below map to the exact absolute paths in the processed-manifest table. Evidence classes are deliberately separate:

- `[E]` — explicit in an assigned original source.
- `[I]` — an inference made by this deep read; not a source commitment.
- `[A]` — genuinely ambiguous or underspecified in the assigned sources.
- `[C]` — conflict or material mismatch, including a summary statement that the assigned original does not support.

Each bold `DII005-*` identifier is a unique atomic finding definition. Later cross-references to an ID are not new findings.

## Exact processed manifest and integrity verification

Manifest section: `DEEP-SOURCE-MANIFEST.md:841-856`. Manifest header totals: 10 files, 1,169 logical lines, 95,848 bytes, target under 1,800 lines. `wc -l`, `wc -c`, and `sha256sum` were rerun against every exact path; every value matched.

| Alias | Exact absolute path | Manifest / actual lines | Manifest / actual bytes | Manifest / actual SHA-256 | Result |
|---|---|---:|---:|---|---|
| I6 | `docs/thefinalglance/I-plan/I-6_anchor_pane_slot.md` | 112 / 112 | 8,103 / 8,103 | `16e098e23c85b71ef887c3067f6c7a5ccdf31d366c81444b8a9495296c961e67` / same | PASS |
| I7 | `docs/thefinalglance/I-plan/I-7_cascading_escalation_offline_batch.md` | 123 / 123 | 8,922 / 8,922 | `0571a2d979e4559362d05d5a2988acc59b4720673bdddc4e838d03c4381f6d69` / same | PASS |
| I11 | `docs/thefinalglance/I-plan/I-11_pabu_belief_state_handoff.md` | 91 / 91 | 7,863 / 7,863 | `cf20fe5c2f23381769b192e8f8f16692ef494889ea812eab4f6037e46f8c165c` / same | PASS |
| I14 | `docs/thefinalglance/I-plan/I-14_deterministic_context_compression.md` | 102 / 102 | 9,064 / 9,064 | `1b1861d46cfc8790c793f4eb767c15ba5d4122be88f65c93eef6b72c66090f2c` / same | PASS |
| I22 | `docs/thefinalglance/I-plan/I-22_progress_shaped_rewards.md` | 117 / 117 | 11,279 / 11,279 | `6c94548b4d7912ca8df9f97d19e583cf17e8fec1f821f5c93a0f95811ba4c070` / same | PASS |
| I25 | `docs/thefinalglance/I-plan/I-25_ledger_trained_router.md` | 182 / 182 | 13,541 / 13,541 | `4d528329f9c628955e994ad3235bcd956e2c700a9170b0e70c48e31e14dbc8c6` / same | PASS |
| I31 | `docs/thefinalglance/I-plan/I-31_grpo_flywheel.md` | 147 / 147 | 14,227 / 14,227 | `fff87039bfe586e07bea2e8af938c75cebf02b5d54c643c6373165d9b149503d` / same | PASS |
| I39 | `docs/thefinalglance/I-plan/I-39_plan_state_from_repo.md` | 99 / 99 | 7,834 / 7,834 | `911f9ea47ed576d28ada35919a9628905682e29527ff8e3649de9b77f1006a34` / same | PASS |
| I44 | `docs/thefinalglance/I-plan/I-44_zero_retention_export.md` | 127 / 127 | 10,501 / 10,501 | `384eb267ae4495ebd1dd9e3a181b9fb9745cfc6b17346c5562ccf21d40c2256b` / same | PASS |
| I51 | `docs/thefinalglance/I-plan/I-51_model_capability_catalog.md` | 69 / 69 | 4,514 / 4,514 | `63e7cddae7af1c5f6f48f20bcebae3803956ae9b1f96f4235f6746e84587a449` / same | PASS |
| **Total** | **10 assigned originals** | **1,169 / 1,169** | **95,848 / 95,848** | **10/10 exact** | **PASS** |

## Per-file explicit ledgers

### I6 — `I-6_anchor_pane_slot.md`

- **DII005-I6-001** `[E][status]` I-6 is Track 13, lever `STEERING`, `STRONG`, `BUILD`, effort S, Wave 2. — I6:L1-L3
- **DII005-I6-002** `[E][problem]` Long contexts degrade effective attention to the task specification as context fills. — I6:L5-L9
- **DII005-I6-003** `[E][capability]` Add the tenth enum variant with the exact identifier `context_budget.rs::ContextSlot::AnchorPane`; it is an immutable privileged-last slot. — I6:L12-L18, I6:L25-L27, I6:L38-L40, I6:L59-L61
- **DII005-I6-004** `[E][layout/invariant]` Append `AnchorPane` after the final cache breakpoint, without displacing or rebalancing the existing nine slots. — I6:L12-L20, I6:L27-L30, I6:L73-L75
- **DII005-I6-005** `[E][data]` The anchor serializes the original `TaskSpec` bytes and keeps those bytes frozen. — I6:L13-L15, I6:L39-L40
- **DII005-I6-006** `[E][data]` The anchor serializes unresolved claims originating in I-17/I-20 verification passes. — I6:L14-L15, I6:L39-L40
- **DII005-I6-007** `[E][data]` The anchor serializes the active constraint list. — I6:L14-L16, I6:L39-L40
- **DII005-I6-008** `[E][generation]` The reply must open with a short acknowledgement tag reflecting the highest-priority active constraint. — I6:L15-L16
- **DII005-I6-009** `[E][enforcement]` The I-1 supervisor validates tag presence in the first reply-token window. — I6:L16, I6:L33-L34, I6:L41-L42, I6:L65
- **DII005-I6-010** `[E][failure/retry]` A missing acknowledgement tag causes a retry prompt with explicit clarification; it is logged and is not a hard error. — I6:L41-L42
- **DII005-I6-011** `[E][schema]` Anchor presence reuses the existing `ContextReduction` payload family; I-6 adds no SSB `action_type`. — I6:L21-L22, I6:L43, I6:L70-L72, I6:L81-L83, I6:L94-L95, I6:L102-L103
- **DII005-I6-012** `[E][integration]` Extend `context_budget.rs::assemble_context` with the anchor serialization branch, following H-31's lazy-ALTER/slot-add pattern. — I6:L31-L32, I6:L50-L53, I6:L59-L64
- **DII005-I6-013** `[E][determinism/cache]` `context_budget.rs::update_breakpoints` must retain its `bp.valid`/blake3 prefix-stability invariant; any cache bust is a test failure. — I6:L25-L30, I6:L63, I6:L96-L97
- **DII005-I6-014** `[E][reachability]` Production reachability is `task_runner.rs` context assembly → `assemble_context` → `ContextSlot::AnchorPane` serialization; test-only callers do not establish R7 reachability. — I6:L67-L69
- **DII005-I6-015** `[E][surface]` I-6 creates no dashboard surface; the anchor is prompt-internal and is measured through fixtures. — I6:L54-L56, I6:L77-L78
- **DII005-I6-016** `[E][scope]` The implementation is a targeted enum extension and assembly hook: no new crate, module, or context serialization layer. — I6:L18-L21, I6:L50-L53
- **DII005-I6-017** `[E][constraint]` A prompt instruction such as “remember the task” is not an acceptable substitute for the structural slot. — I6:L19-L20, I6:L74-L75
- **DII005-I6-018** `[E][deferred]` Adaptive constraint prioritization is Tier 2 and gated on at least three measured adherence runs showing the fixed list insufficient. — I6:L45-L47
- **DII005-I6-019** `[E][deferred]` The constraint-adherence cockpit tile is Tier 2 under the same GD-4 bound gate. — I6:L48
- **DII005-I6-020** `[E][artifact]` Ship `workflow/reasoning/I-6-decisions.md`. — I6:L44
- **DII005-I6-021** `[E][acceptance]` The long-context trigger is `cargo run --bin elai -- run --task fixtures/search/long_context_25k.toml`. — I6:L79-L80
- **DII005-I6-022** `[E][acceptance/evidence]` The journaled assembled request must show `AnchorPane` as the final block (`anchor_pane` is the stated byte-search marker), with `bp.valid` passing. — I6:L80-L83
- **DII005-I6-023** `[E][acceptance/evidence]` The response journal must show the acknowledgement tag in its first token window. — I6:L84-L85
- **DII005-I6-024** `[E][measurement]` Constraint adherence is measured—not asserted—against a `--no-anchor` control under the local-baseline-before-paper-threshold rule. — I6:L84-L87, I6:L99-L101
- **DII005-I6-025** `[E][negative test]` The `--no-anchor` broken arm must open without the tag and fail the tag-presence check. — I6:L85-L87
- **DII005-I6-026** `[E][dependency]` I-6 composes H-2 breakpoint discipline, H-31 slot-add precedent, and I-1 supervisor enforcement rather than reimplementing them. — I6:L29-L34, I6:L105-L110
- **DII005-I6-027** `[E][safety]` Production paths ban `.unwrap()`/`.expect()`, blake2/sha2, and fabricated measurements. — I6:L76

### I7 — `I-7_cascading_escalation_offline_batch.md`

- **DII005-I7-001** `[E][status/dependency]` I-7 is Track 13, `CHEAPER-SPEED`, `STRONG`, `BUILD`, effort M, Wave 4; it depends on I-11 and binds IC-1. — I7:L1-L3
- **DII005-I7-002** `[E][correction]` The former speculative Batch-API racing design is dead: live-loop batch jobs take 10–45 minutes with a 24-hour SLA and cancellation does not refund queued work. — I7:L5-L11, I7:L13-L15, I7:L103-L107
- **DII005-I7-003** `[E][workflow]` The local lane first fans the task out to K branches in I-2 isolated workspaces. — I7:L15-L16, I7:L72
- **DII005-I7-004** `[E][workflow]` `I-17 TriLayerStack::evaluate` runs deterministic checks on every local branch. — I7:L16, I7:L73
- **DII005-I7-005** `[E][state transition]` Escalation begins only when all local branches fail. — I7:L16-L17
- **DII005-I7-006** `[E][data]` The failure set is condensed into I-11 `BeliefState` fields `tried`, `failed-because`, and `constraints`, targeting no more than about 2K tokens. — I7:L17-L21, I7:L59-L60, I7:L84-L85
- **DII005-I7-007** `[E][routing]` The condensed payload escalates synchronously to the H-7 cloud architect. — I7:L17-L20, I7:L31-L32
- **DII005-I7-008** `[E][workflow]` The cloud architect response becomes the next task-runner candidate. — I7:L18-L19
- **DII005-I7-009** `[E][privacy/cost]` A local solution never leaves the machine and must incur zero cloud spend through the E-39 cost gate. — I7:L20-L22, I7:L37, I7:L71
- **DII005-I7-010** `[E][offline integration]` H-5 retains the Batch API only for offline Cosmos/research/bench lanes; I-7 does not reimplement its client or scheduler. — I7:L10-L11, I7:L19-L20, I7:L33-L34, I7:L83, I7:L115-L117
- **DII005-I7-011** `[E][security/architecture]` Live `escalation_waterfall.rs` must have no import path to the batch client, making batch routing structurally unreachable from interactive/search-loop execution. — I7:L45-L47, I7:L77-L80
- **DII005-I7-012** `[E][capability]` Create `crates/orchestrator/src/escalation_waterfall.rs::EscalationWaterfall::run` as the local-first dispatcher. — I7:L42-L43, I7:L56-L57, I7:L65-L67
- **DII005-I7-013** `[E][schema/events]` Add `EscalationCascade` to `ssb.rs` and `action_type_lock.rs`, baseline +1, with `LocalWin`, `Condensed`, `Escalated`, and `BatchRouted` kinds. — I7:L38, I7:L43-L44, I7:L65-L67, I7:L112-L113
- **DII005-I7-014** `[E][event constraint]` `BatchRouted` exists only for offline-path journal completeness and never on a live search loop. — I7:L78-L80
- **DII005-I7-015** `[E][integration]` Wire the waterfall at `cloud_mode.rs` / `escalation_policy.rs` and call I-11 `BeliefState::condense`. — I7:L48-L50, I7:L56-L60, I7:L68-L71
- **DII005-I7-016** `[E][reachability]` Production reachability is task-runner dispatch → `EscalationWaterfall::run`; test callers do not satisfy R7. — I7:L74-L76
- **DII005-I7-017** `[E][constraint]` Raw history is forbidden in cloud escalation; the condensed `BeliefState` is mandatory. — I7:L84-L85
- **DII005-I7-018** `[E][constraint]` Async/speculative local+cloud submission that relies on cancellation refunds is forbidden. — I7:L22-L25, I7:L81-L82
- **DII005-I7-019** `[E][constraint]` Heartbeat-primary routing is excluded from this feature. — I7:L22-L24
- **DII005-I7-020** `[E][deferred]` Multi-hop `local → cloud-small → cloud-large` escalation is Tier 2, gated on at least three runs showing single-hop insufficiency. — I7:L51-L53
- **DII005-I7-021** `[E][deferred/UI]` The cost-cockpit cascade-trace tile is deferred until the GD-4 bound is measured; `cross_lang_diff.py` becomes required when it ships. — I7:L54, I7:L61-L62, I7:L87-L90
- **DII005-I7-022** `[E][artifact]` Ship `workflow/reasoning/I-7-decisions.md`. — I7:L50
- **DII005-I7-023** `[E][acceptance]` Run paired fixtures `fixtures/search/local_solvable.toml` and `fixtures/search/local_fail.toml` with `--waterfall`. — I7:L91-L93
- **DII005-I7-024** `[E][acceptance/evidence]` The solvable fixture emits `EscalationCascade(LocalWin)` and journals zero cloud API calls. — I7:L94-L95
- **DII005-I7-025** `[E][acceptance/evidence]` The failing fixture emits `Condensed` followed by `Escalated`, and the journal proves a typed payload of at most 2K tokens. — I7:L95-L97
- **DII005-I7-026** `[E][negative test]` `rg 'batch_client\|MessageBatch' crates/orchestrator/src/escalation_waterfall.rs` must return zero. — I7:L98-L100
- **DII005-I7-027** `[E][negative test]` An injected live-waterfall call to the batch client must fail to compile. — I7:L100-L101
- **DII005-I7-028** `[E][status/evidence]` Existing `batches` tokens are confirmed only at file level; their wiring depth remains unverified and must be retraced before implementation. — I7:L27-L30
- **DII005-I7-029** `[E][safety]` Production paths ban `.unwrap()`/`.expect()`, blake2/sha2, and fabricated measurements. — I7:L86

### I11 — `I-11_pabu_belief_state_handoff.md`

- **DII005-I11-001** `[E][status]` I-11 is Track 14, `CHEAPER-ORCH`, `STRONG`, `BUILD`, effort M, Wave 2, bound by ID-3 and IC-8. — I11:L1-L3
- **DII005-I11-002** `[E][problem]` Existing cross-model/tier handoffs send raw history, increasing cloud payloads and hiding condensation cost. — I11:L4-L7
- **DII005-I11-003** `[E][schema]` Create exact symbol `crates/orchestrator/src/belief_state.rs::BeliefState`, typed and versioned. — I11:L13-L14, I11:L34, I11:L45, I11:L50-L53
- **DII005-I11-004** `[E][schema field]` `BeliefState` carries what was tried. — I11:L13
- **DII005-I11-005** `[E][schema field]` `BeliefState` carries why attempts failed. — I11:L13
- **DII005-I11-006** `[E][schema field]` `BeliefState` carries current AST state. — I11:L13
- **DII005-I11-007** `[E][schema field]` `BeliefState` carries active constraints. — I11:L13
- **DII005-I11-008** `[E][schema field]` `BeliefState` carries DKG-keyed entity links. — I11:L11-L14, I11:L51
- **DII005-I11-009** `[E][failure]` Decode on schema-version mismatch fails closed with typed `BeliefStateVersionMismatch`, never a silent parse. — I11:L13-L14, I11:L34, I11:L66, I11:L74-L75
- **DII005-I11-010** `[E][workflow]` Handoff sites are I-7 local-to-cloud escalation, H-7 architect-to-editor routing, and H-51 session resume. — I11:L15-L16
- **DII005-I11-011** `[E][privacy/context]` The escalation payload becomes prefix + `BeliefState`, never raw history; this removes raw history from escalation, not from session storage. — I11:L21-L22, I11:L37, I11:L64
- **DII005-I11-012** `[E][events]` Add `BeliefStateEvent` to `ssb.rs`/`action_type_lock.rs`, baseline +1, with `Condensed`, `Handoff`, and `HonestyDelta` kinds. — I11:L17-L18, I11:L35-L36, I11:L53, I11:L82
- **DII005-I11-013** `[E][observability]` Every condensation emits an ID-3 receipt carrying `schema_version` and `token_delta`. — I11:L17-L18, I11:L30, I11:L35, I11:L63-L65
- **DII005-I11-014** `[E][replay]` Journal evidence must reconstruct exactly what the escalated model received. — I11:L17-L18
- **DII005-I11-015** `[E][capability]` Create `crates/orchestrator/src/belief_state.rs::HonestyJob` for periodic full-history-versus-BeliefState reruns and delta ledgering. — I11:L36, I11:L45, I11:L52
- **DII005-I11-016** `[E][config/default]` The honesty interval is configurable and defaults to after every 10 escalations. — I11:L36
- **DII005-I11-017** `[E][determinism]` Honesty deltas must be recomputable under `ELAI_SEED`. — I11:L36
- **DII005-I11-018** `[E][integration]` Reuse `context_budget.rs::adaptive_reduce` as the reduction engine whose result the typed state represents. — I11:L26, I11:L46, I11:L56
- **DII005-I11-019** `[E][integration]` Share H-51's digest engine while keeping I-11 live-handoff scope distinct from H-51 sleep-time scope. — I11:L16, I11:L19, I11:L28, I11:L57, I11:L84-L87
- **DII005-I11-020** `[E][recovery]` The receiver uses `H-6::recover_stubs` for detail re-expansion; I-11 does not reimplement it. — I11:L20, I11:L58, I11:L87
- **DII005-I11-021** `[E][surface]` Add a required `BeliefState` inspector row in `SessionsDrawer`; `HonestyDelta` is a user-visible economics signal. — I11:L38, I11:L47, I11:L70-L71
- **DII005-I11-022** `[E][reachability]` Production encode reachability is `escalation_policy.rs::condense_and_escalate()` → `BeliefState::encode()`. — I11:L59-L60
- **DII005-I11-023** `[E][reachability]` Production decode reachability is `session_dag.rs::resume()` → `BeliefState::decode()`; test-only callers do not establish R7. — I11:L61-L62
- **DII005-I11-024** `[E][deferred]` Automated staleness detection for DKG links is deferred until live-graph update cadence exists. — I11:L40-L41
- **DII005-I11-025** `[E][deferred]` Schema migration tooling beyond fail-closed mismatch is deferred until a v2 schema exists. — I11:L42
- **DII005-I11-026** `[E][acceptance/evidence]` A local-fail run emits `Condensed` with `schema_version: 1`/token delta and `Handoff` on dispatch; the honesty job emits after at least 10 escalations or `--force-honesty-check`. — I11:L72-L75
- **DII005-I11-027** `[E][measurement]` The journal must show an approximately ≤2K-token escalation payload versus a ≥20K raw-history control. — I11:L75
- **DII005-I11-028** `[E][negative test]` A version-bumped blob must fail before any model call. — I11:L75
- **DII005-I11-029** `[E][negative test]` A raw-history bypass produces a payload over 10K tokens and fails the payload-size gate. — I11:L75
- **DII005-I11-030** `[E][dependency]` DKG must be reachable at decode time. — I11:L88
- **DII005-I11-031** `[E][safety]` Production decode bans `.unwrap()`/`.expect()` and uses blake3, not blake2/sha2. — I11:L68-L69
- **DII005-I11-032** `[E][status/evidence]` The typed module is confirmed absent, so this is a net-new build; routing/digest seams remain plan-asserted pending re-grep. — I11:L24-L30, I11:L77-L82

### I14 — `I-14_deterministic_context_compression.md`

- **DII005-I14-001** `[E][status]` I-14 is Track 14, `CHEAPER`, `STRONG`, `BUILD`, effort M, Wave 2, bound by ID-3 and IC-2/IC-3. — I14:L1-L3
- **DII005-I14-002** `[E][problem]` No middleware prevents full compiler panics, tracebacks, or AST dumps from entering context and destabilizing the cache prefix. — I14:L4-L8, I14:L27-L28
- **DII005-I14-003** `[E][capability]` Create `crates/orchestrator/src/context_transform.rs::ContextTransformer` with deterministic Tier-1 and gated Tier-2 pipeline semantics. — I14:L35-L41, I14:L48-L49, I14:L54-L59
- **DII005-I14-004** `[E][strategy]` `LspSliceTransform` uses headless rust-analyzer and compiler locations to extract the exact failing line, AST node, or declaration. — I14:L11-L14, I14:L37, I14:L57
- **DII005-I14-005** `[E][strategy]` `AstFoldTransform` uses tree-sitter to replace irrelevant function bodies with `/* elided */`. — I14:L14-L15, I14:L37, I14:L58
- **DII005-I14-006** `[E][semantic invariant]` AST folding preserves signatures and types so BPE semantic priors remain intact. — I14:L14-L15, I14:L37
- **DII005-I14-007** `[E][deferred strategy]` An LLMLingua-2-class encoder is gated Tier 2 for prose-only payloads. — I14:L11-L16, I14:L43-L45, I14:L71
- **DII005-I14-008** `[E][resource/failure]` Tier-2 neural compression requires a VRAM-headroom check and refuses closed when headroom is insufficient. — I14:L16, I14:L44, I14:L72, I14:L82
- **DII005-I14-009** `[E][dependency/security]` Adding LLMLingua-2 requires availability, license, current-version, and anti-slopsquat verification plus user approval. — I14:L44, I14:L91
- **DII005-I14-010** `[E][deferred strategy]` Numeric quantization survives only as a narrow Tier-2 option for tabular payloads. — I14:L17-L19
- **DII005-I14-011** `[E][constraint]` N-gram abbreviation/cipher substitution for code identifiers is unconditionally banned as “neural poison.” — I14:L17-L19, I14:L24-L25, I14:L68-L71
- **DII005-I14-012** `[E][cache invariant]` Transforms operate only on volatile-tail content and must never touch the blake3-frozen prefix. — I14:L20-L21, I14:L31, I14:L63, I14:L73
- **DII005-I14-013** `[E][events]` Add `ContextTransformReceipt` to `ssb.rs`/`action_type_lock.rs`, baseline +1, with `LspSlice`, `AstFold`, and `Tier2Compressed` kinds. — I14:L22-L23, I14:L38-L39, I14:L59, I14:L92
- **DII005-I14-014** `[E][receipt schema]` Every transform receipt carries before/after token counts and a blake3 reverse pointer to original bytes. — I14:L22-L23, I14:L38
- **DII005-I14-015** `[E][replay]` The reverse pointer must re-expand to byte-identical original content under a blake3 match. — I14:L22-L23, I14:L41, I14:L81-L82, I14:L88
- **DII005-I14-016** `[E][failure]` Any transform without a receipt is banned and must be caught by a post-transform receipt-presence check. — I14:L24-L25, I14:L33, I14:L70, I14:L81-L82
- **DII005-I14-017** `[E][integration]` Hook after tool-result finalization and before `context_budget.rs::adaptive_reduce` at the tool-result → context-assembly seam. — I14:L40, I14:L47-L49, I14:L65-L67
- **DII005-I14-018** `[E][dependency]` Share the E-52 headless rust-analyzer client; never reimplement the LSP wire protocol. — I14:L29, I14:L50, I14:L61, I14:L74
- **DII005-I14-019** `[E][dependency]` Share `knowledge::tree_sitter` grammar files; never duplicate them. — I14:L30, I14:L51, I14:L62
- **DII005-I14-020** `[E][composition]` H-6 masking drops turns while I-14 compresses payloads; the layers coexist without replacing one another. — I14:L32, I14:L64, I14:L99
- **DII005-I14-021** `[E][surface]` Per-transform receipt rows in the Transcript drawer are required and show token delta plus reversibility link. — I14:L52, I14:L77-L78
- **DII005-I14-022** `[E][fixture/acceptance]` The seeded fixture reduces a 10K-token compiler panic to a ≤500-token LSP slice and emits its receipt. — I14:L41, I14:L79-L82
- **DII005-I14-023** `[E][measurement]` Fixed-fixture pass rate must remain unchanged versus uncompressed context. — I14:L82
- **DII005-I14-024** `[E][negative test]` The unjournaled-transform broken arm must fail the receipt-presence check. — I14:L82
- **DII005-I14-025** `[E][deferred]` Python/TypeScript tree-sitter grammars beyond Rust are deferred pending grammar acquisition. — I14:L45
- **DII005-I14-026** `[E][safety]` Production transform/receipt paths ban `.unwrap()`/`.expect()` and reverse pointers must use blake3, not blake2/sha2. — I14:L75-L76
- **DII005-I14-027** `[E][done gate]` Full-form expansion is required before execution because this effort-M feature spans multiple crates. — I14:L93

### I22 — `I-22_progress_shaped_rewards.md`

- **DII005-I22-001** `[E][status/dependency]` I-22 is Track 15, `ANTI-GAMING`, `STRONG`, `BUILD`, effort M, Wave 4; it depends on I-4 and binds ID-4. — I22:L1-L3
- **DII005-I22-002** `[E][reward]` The step reward is the potential delta `phi(h_t) - phi(h_{t-1})`, not an absolute score. — I22:L4-L10, I22:L12-L17
- **DII005-I22-003** `[E][reward field]` `phi` includes test-pass count. — I22:L13-L15, I22:L46-L47
- **DII005-I22-004** `[E][reward field]` `phi` includes inverted compile-error count. — I22:L13-L15, I22:L46-L47
- **DII005-I22-005** `[E][reward field]` `phi` includes requirements-covered count from I-37 EARS when available, otherwise wiring-contract fields. — I22:L14-L15, I22:L46-L47
- **DII005-I22-006** `[E][anti-gaming]` Holding position earns zero, improvement earns positive reward, and regression earns negative reward. — I22:L15-L17
- **DII005-I22-007** `[E][measurement/default]` Reward weights are measured on ELAI fixtures and written by first deployment to `metrics/progress_reward_weights.toml`; no paper vector is a gate. — I22:L16-L20, I22:L50, I22:L71, I22:L84
- **DII005-I22-008** `[E][determinism]` Every `phi` component is a deterministic journal-enumerable observable, and the component breakdown is reproducible under `ELAI_SEED`. — I22:L9-L10, I22:L18-L20, I22:L82, I22:L94-L95
- **DII005-I22-009** `[E][constraint]` LLM opinion, self-report, plausible-looking output without observable state change, and other nondeterministic `phi` signals are structurally excluded. — I22:L18-L20, I22:L32-L33, I22:L82
- **DII005-I22-010** `[E][localization]` The first `I-17 TriLayerStack` `HardVeto` at step k is the first-error trigger. — I22:L21-L22, I22:L41, I22:L75
- **DII005-I22-011** `[E][recovery]` On that veto, truncate to k−1, preserve the last passing state as an H-49 fork node, and reinvest generation from that prefix. — I22:L21-L24, I22:L48, I22:L69, I22:L74-L75
- **DII005-I22-012** `[E][anti-spiral]` After D consecutive zero-delta steps, multiply the next reward by a decay factor less than one. — I22:L25-L27, I22:L49
- **DII005-I22-013** `[E][config/measurement]` D is measured on the ELAI fixture corpus and must not be an unbounded or hard-coded source constant. — I22:L25-L28, I22:L49-L50, I22:L85
- **DII005-I22-014** `[E][default algorithm]` Decay is linear-step until measured behavior supports another curve. — I22:L27-L28
- **DII005-I22-015** `[E][behavior]` Accumulated decay breaks isomorphic-state spirals, causing backtracking and sibling expansion in the stated fixture. — I22:L27-L28, I22:L93-L95
- **DII005-I22-016** `[E][composition]` This per-branch search mechanism is a sibling of, and does not replace or interact with, the main-session anti-spiral CLI skill. — I22:L29-L31, I22:L39, I22:L103, I22:L112-L113
- **DII005-I22-017** `[E][capability]` Create `crates/orchestrator/src/search/score.rs::DeltaProgressScorer` for potential, shaped reward, and decay. — I22:L46, I22:L56-L59, I22:L67-L68
- **DII005-I22-018** `[E][capability]` Create `crates/orchestrator/src/search/score.rs::FirstErrorTruncator` for H-49 fork-at-k−1 logic. — I22:L60-L61, I22:L69
- **DII005-I22-019** `[E][integration]` Wire the scorer into I-4 `search/ladder.rs` at the PUCT expansion/value-function seam. — I22:L38, I22:L57-L59, I22:L76, I22:L78-L80
- **DII005-I22-020** `[E][integration nuance]` I-22 shapes the input received by shipped D-2 `PpmClient` Layer 2; it does not reimplement or replace the PPM output. — I22:L35-L36, I22:L73
- **DII005-I22-021** `[E][deferred/stub lifecycle]` The I-37 EARS requirements component ships as a registered zero-contribution stub with `will_resolve_in=i-37` until I-37 lands. — I22:L52-L54, I22:L77
- **DII005-I22-022** `[E][deferred]` Exponential/adaptive decay schedules remain deferred until ELAI measurements justify them. — I22:L54
- **DII005-I22-023** `[E][events]` Add `ProgressRewardScored`, baseline +1, with `Delta`, `FirstErrorTruncated`, and `DecayApplied` discriminators. — I22:L51, I22:L62, I22:L70, I22:L106
- **DII005-I22-024** `[E][surface]` The I-4 branch board renders a per-branch `phi` sparkline with decay regions distinguished, consuming existing SSB subscriptions; `cross_lang_diff` must pass. — I22:L63-L64, I22:L87-L91
- **DII005-I22-025** `[E][observability]` Emit `Delta` rows per step with `test_pass`, `compile_err_inv`, and `req_covered` breakdown. — I22:L89-L90, I22:L93-L95
- **DII005-I22-026** `[E][observability]` Emit `DecayApplied` after D zero-delta steps and `FirstErrorTruncated` with the H-49 k−1 node identifier. — I22:L93-L95
- **DII005-I22-027** `[E][negative test]` An `--llm-phi` broken arm must fail the contract asserting fully deterministic `phi` components. — I22:L95
- **DII005-I22-028** `[E][safety]` Production paths ban `.unwrap()`/`.expect()`, fabricated `phi` measurements, and blake2/sha2. — I22:L86
- **DII005-I22-029** `[E][done gate]` Full-form expansion is required before execution. — I22:L115

### I25 — `I-25_ledger_trained_router.md`

- **DII005-I25-001** `[E][status]` I-25 is Track 16, `CHEAPER-SPEED`, `STRONG`, `EXTENSION`, effort M, Wave 4, bound by ID-5/ID-6. — I25:L1-L3
- **DII005-I25-002** `[E][learning]` Train routing on ELAI journal verifier outcomes keyed by task features, lane, and cost rather than human/proxy labels. — I25:L4-L8, I25:L10-L13
- **DII005-I25-003** `[E][feature]` The base feature family includes task class expressed through AST shape. — I25:L12-L14
- **DII005-I25-004** `[E][feature]` The base feature family includes DKG centrality. — I25:L12-L14
- **DII005-I25-005** `[E][feature]` The base feature family includes file count. — I25:L12-L14
- **DII005-I25-006** `[E][feature]` The base feature family includes historical cost shape per task class. — I25:L13-L14
- **DII005-I25-007** `[E][feature]` The base feature family includes recent pass rate per lane. — I25:L13-L14
- **DII005-I25-008** `[E][model]` The router is deliberately small; the revised Slice 2 selects a logistic classifier, while neural routing is deferred. — I25:L14-L15, I25:L46-L48, I25:L52-L54
- **DII005-I25-009** `[E][determinism]` Offline training uses `ELAI_SEED`-sealed journal exports. — I25:L15
- **DII005-I25-010** `[E][measurement]` Over-escalation is measured on ELAI's own mixed batch; RouteLLM paper numbers are forbidden as thresholds. — I25:L16-L18, I25:L87, I25:L104
- **DII005-I25-011** `[E][promotion]` A learned router that does not beat the H-7 heuristic baseline is not promoted; the promotion row must cite the ledger comparison. — I25:L16-L18, I25:L36, I25:L85, I25:L105
- **DII005-I25-012** `[E][routing]` Before ordinary routing, a historically expensive task-shape fingerprint match pre-routes to the deep lane. — I25:L18-L20
- **DII005-I25-013** `[E][events]` Reuse the live E-31 `RouteDecisionEvent`; do not add an action type or schema bump. — I25:L20-L21, I25:L27-L28, I25:L76, I25:L86, I25:L107, I25:L153-L154
- **DII005-I25-014** `[E][event schema]` Extend only `RouteDecisionEventPayload.details_json` with `feature_vec`, `predicted_lane`, and `reason`. — I25:L20-L21, I25:L43-L44, I25:L76, I25:L97
- **DII005-I25-015** `[E][flag/default]` The existing emission remains default-off behind `ELAI_ROUTE_DECISION_SSB_ENABLE`. — I25:L27-L28, I25:L80-L82
- **DII005-I25-016** `[E][seam correction]` The ratified primary live hook is exact function `task_runner.rs::run_cost_router_preclassify`, called on pre-cloud dispatch. — I25:L25-L31, I25:L60, I25:L71, I25:L74, I25:L80-L82
- **DII005-I25-017** `[E][seam constraint]` `crates/orchestrator/src/escalation_policy.rs` is E-32 handoff policy and must not be treated as the I-25 hook. — I25:L29-L31
- **DII005-I25-018** `[E][fallback]` `providers/src/runtime/escalation_policy.rs::RuntimeRoutingPolicy::evaluate` is a later fallback composition target and currently has no production callers; it cannot be Slice 1's sole hook. — I25:L31-L32, I25:L55, I25:L77, I25:L90
- **DII005-I25-019** `[E][gap/status]` `CostProfiler` is confirmed absent; `crates/orchestrator/src/routing/cost_profile.rs` is a genuine build gap alongside the existing C-46 routing modules. — I25:L33-L34
- **DII005-I25-020** `[E][capability/slice 1]` Create `crates/orchestrator/src/routing/cost_profile.rs::CostProfiler` as an approximately 80-LOC deterministic task-shape classifier with a `BTreeMap` cache. — I25:L39-L45, I25:L58-L60, I25:L68-L71
- **DII005-I25-021** `[E][routing order]` Consult `CostProfiler` in `run_cost_router_preclassify` before `route_for_difficulty_and_budget`; non-match/default-off preserves E-39. — I25:L42-L45, I25:L71, I25:L75, I25:L97, I25:L111
- **DII005-I25-022** `[E][event behavior]` An expensive match emits `details_json.reason=cost_profile_expensive_shape` plus predicted lane and feature vector. — I25:L43-L45, I25:L95-L98, I25:L140-L143
- **DII005-I25-023** `[E][test/slice 1]` Create deterministic in-process contract `crates/contracts/tests/i25_ledger_trained_router_contract.rs`. — I25:L44-L45, I25:L62, I25:L96, I25:L175-L177
- **DII005-I25-024** `[E][capability/slice 2]` Create `workflow/training/router/train_router.py::RouterTrainer`, approximately 150 LOC of Python stdlib. — I25:L46-L48, I25:L63, I25:L69-L70
- **DII005-I25-025** `[E][data/security]` The trainer opens `ssb.db` read-only via `sqlite3`. — I25:L46-L48
- **DII005-I25-026** `[E][training data]` The trainer joins `RouteDecisionEvent` to verifier rows and `UsageTurnEvent`. — I25:L47-L48
- **DII005-I25-027** `[E][artifact]` The trainer writes a TOML model artifact and `workflow/training/router/eval_report.toml`. — I25:L47-L48, I25:L98
- **DII005-I25-028** `[E][failure]` Insufficient labeled rows produce an explicit `REFUSED` report and no silent model. — I25:L48, I25:L144-L145
- **DII005-I25-029** `[E][capability/slice 3]` Load a router artifact from `elai_cc.toml` only when a trial-ledger row records a win. — I25:L49-L50
- **DII005-I25-030** `[E][fallback/recovery]` A blocked/losing artifact is never loaded; H-7/E-39 heuristics remain the fallback. — I25:L49-L50, I25:L85, I25:L98, I25:L146
- **DII005-I25-031** `[E][security/frozen boundary]` `trial_ledger.rs`, `deflation_gate.rs`, `holdout_budget.rs`, and `gate_self_validation.rs` are R8-frozen consume-only files; touching any is an abort. — I25:L35-L36, I25:L64, I25:L78-L79, I25:L89, I25:L158-L159
- **DII005-I25-032** `[E][ownership]` The Python trainer never writes ledger rows; `TrialLedger::register_trial` remains the Rust writer. — I25:L69-L70, I25:L153-L156
- **DII005-I25-033** `[E][deferred]` The neural router is RunPod/VRAM-gated Tier 2. — I25:L22-L23, I25:L52-L54
- **DII005-I25-034** `[E][deferred]` Continuous online retraining waits for I-4's high-volume trajectory stream. — I25:L54
- **DII005-I25-035** `[E][deferred/UI]` Route-explanation dashboard/Svelte work is deferred by `owner-ui-rework-hold`; this unit emits backend details only. — I25:L55-L56, I25:L92-L93, I25:L177
- **DII005-I25-036** `[E][integration]` Create/modify only the listed Slice files: `cost_profile.rs`, `task_runner.rs`, `routing/mod.rs`, the contract, and the Python trainer. — I25:L58-L64
- **DII005-I25-037** `[E][reachability]` Production reachability is `elai run <task>` → task-runner pre-cloud path → `run_cost_router_preclassify` → `CostProfiler`. — I25:L80-L82
- **DII005-I25-038** `[E][acceptance/evidence]` Contract/live fixtures require an expensive match event with the exact reason and a non-match leaving E-39 unchanged. — I25:L95-L98, I25:L140-L143
- **DII005-I25-039** `[E][artifact integrity]` `eval_report.toml` carries ELAI-measured over-escalation/mis-route rates, seed, and input blake3; the loser artifact path is absent when blocked. — I25:L98, I25:L144-L146
- **DII005-I25-040** `[E][difficulty fusion]` The J-11 amendment adds DeNovoSWE's convex feature `d_i = w_s·d_i^struct + w_g·ℓ_i^(g) + w_q·ℓ_i^(q)` with nonnegative weights summing to one. — I25:L118-L123
- **DII005-I25-041** `[E][difficulty feature]` The structural reference signal is executable Python lines, transformed by `log(1+x)` and min-max normalized against the 5th/95th percentiles. — I25:L124-L126
- **DII005-I25-042** `[E][difficulty feature]` Two independent LLM judges, g and q, each provide a five-level qualitative difficulty signal. — I25:L127-L128
- **DII005-I25-043** `[E][calibration]` Choose fusion weights by maximizing absolute Pearson correlation with empirical pass rate over a dense two-dimensional simplex grid. — I25:L129-L131
- **DII005-I25-044** `[E][honesty]` Winning DeNovoSWE weights are unpublished, so ELAI calibrates weights on its own rollouts. — I25:L129-L131
- **DII005-I25-045** `[E][cross-language adaptation]` Candidate ELAI structural substitutes are SCIP function count, cross-language bindings touched, and SSB action-type variants added per task. — I25:L132-L134
- **DII005-I25-046** `[E][authority]` Difficulty fusion is merely a router feature; ID-6 ledger gates remain authoritative. — I25:L134-L135
- **DII005-I25-047** `[E][measurement constraint]` The cited +1.2 percentage-point result is rationale only and never an ELAI target. — I25:L135-L136
- **DII005-I25-048** `[E][failure/recovery]` Missing event, wrong reason, or default-off E-39 regression directs the executor to re-read the live hook/emission sites and rerun the I-25 contract. — I25:L148-L151
- **DII005-I25-049** `[E][fork decision]` FORK-1 fixes payload evolution to `details_json` keys with no schema/action-type change. — I25:L153-L154
- **DII005-I25-050** `[E][fork decision]` FORK-2 fixes Slice 1 to the live `run_cost_router_preclassify` hook and leaves `RuntimeRoutingPolicy::evaluate` for later. — I25:L154-L155
- **DII005-I25-051** `[E][fork decision]` FORK-3 keeps the ledger writer in Rust `TrialLedger::register_trial`. — I25:L155-L156
- **DII005-I25-052** `[E][abort]` Stop rather than invent `SCHEMA_VERSION` if live SSB integers cannot be reread; inventing a new `RouteDecisionEvent` variant is also a stop condition. — I25:L158-L161
- **DII005-I25-053** `[E][done gates]` Off-agent verification includes `plan_gate_check.py`, the contract test, orchestrator all-target check, production-caller checks, Python byte-compilation, and `python_wire_check`. — I25:L163-L169
- **DII005-I25-054** `[E][red team]` Promotion without a ledger row, silent losing-artifact adoption, new action type, and a dark-only hook must all fail. — I25:L170-L173
- **DII005-I25-055** `[E][status/recon]` The source reports no open seam reconnaissance after rereading the live hook, routing inventory, live event, and trial writer. — I25:L179-L180
- **DII005-I25-056** `[E][composition]` I-25 composes E-39, H-7, E-31, C-46, E-20 privacy routing, and IS-16 without reimplementing them. — I25:L110-L116
- **DII005-I25-057** `[E][safety]` Production routing bans `.unwrap()`/`.expect()`, fabricated measurements, and paper thresholds. — I25:L84-L90

### I31 — `I-31_grpo_flywheel.md`

- **DII005-I31-001** `[E][status/dependency]` I-31 is Track 17, `SELF-IMPROVEMENT`, `STRONG`, `BUILD`, effort XL, Wave 5, explicitly LAST; it depends on I-30, H-43/H-44/H-28, and I-24 and binds ID-6/IC-7. — I31:L1-L3
- **DII005-I31-002** `[E][security premise]` A self-improving loop is invalid if it can influence its evaluator; the evaluator must be stronger than and unreachable by the generator. — I31:L4-L7
- **DII005-I31-003** `[E][kill switch]` The lane stays behind the existing `darwinian-evolve` kill-switched allowlist. — I31:L7-L8, I31:L64, I31:L132, I31:L144
- **DII005-I31-004** `[E][constraint]` DPO is inherited as E-55 `HARD-AVOID`; PPO is also excluded because its value-network overhead is unjustified here. — I31:L8-L9, I31:L44-L45, I31:L112, I31:L133
- **DII005-I31-005** `[E][done gate]` Full-form expansion is required before execution. — I31:L11
- **DII005-I31-006** `[E][algorithm]` GRPO generates K completions, scores them, normalizes reward within the group, and updates the policy. — I31:L13-L15
- **DII005-I31-007** `[E][service]` Create typed local supervisor-side reward service exact symbol `crates/training/grpo_harness.rs::RewardEndpoint`; Tier 1 chooses an Axum handler. — I31:L15-L17, I31:L40-L43, I31:L68-L69, I31:L90-L93
- **DII005-I31-008** `[E][security boundary]` The supervisor owns the endpoint address and reward code; the trained policy can receive only returned reward values. — I31:L15-L17, I31:L92, I31:L111
- **DII005-I31-009** `[E][reward]` Execution-grounded reward includes the I-17 L1 compile pass/fail. — I31:L17-L18
- **DII005-I31-010** `[E][reward]` Execution-grounded reward includes I-17 L1 test pass-count/total. — I31:L18
- **DII005-I31-011** `[E][reward]` Execution-grounded reward includes the I-20 CI-cleared mutation score. — I31:L18-L19
- **DII005-I31-012** `[E][reward]` Execution-grounded reward includes I-8's null-test halting/abort signal. — I31:L19
- **DII005-I31-013** `[E][reward]` XGrammar conformance contributes an additive format penalty. — I31:L19-L20
- **DII005-I31-014** `[E][constraint]` There is no neural reward model; only execution gates supply fitness. — I31:L20
- **DII005-I31-015** `[E][anti-collapse/config]` Apply `KL(policy || I-30 SFT init checkpoint)` with a configurable coefficient; skip the update when KL exceeds its threshold. — I31:L20-L22
- **DII005-I31-016** `[E][training]` Inject I-30 fluent-but-vetoed JSONL negative pairs as masked negative examples. — I31:L22-L24
- **DII005-I31-017** `[E][constraint]` Old `derived_alpha = blake3(config bytes)` fitness is retired and structurally forbidden. — I31:L24-L25, I31:L29, I31:L49-L52, I31:L109-L110
- **DII005-I31-018** `[E][anti-gaming]` Score AST-preserving I-24 canary pairs; divergence above threshold emits `ScorerNeutralized` and skips that batch's update. — I31:L25-L27
- **DII005-I31-019** `[E][blocking prerequisite]` H-43 execution-grounded fitness must be live with no byte-hash path. — I31:L28-L30, I31:L50-L52, I31:L95
- **DII005-I31-020** `[E][blocking prerequisite]` H-44 behavioral `score_candidate_for_traps` must be live, replacing the retired keyword-grep detector. — I31:L29-L30, I31:L53-L55, I31:L96
- **DII005-I31-021** `[E][blocking prerequisite]` I-24 canaries must already appear as `PrmCanaryEvent` rows in a live scoring run. — I31:L31, I31:L60, I31:L98
- **DII005-I31-022** `[E][blocking prerequisite/security]` H-28 reward-tampering gate plus evaluator/harness/thresholds must be in the R8 frozen set and pass `gate_freeze_check.py`. — I31:L32-L33, I31:L57-L58, I31:L70, I31:L97
- **DII005-I31-023** `[E][blocking prerequisite/promotion]` H-17 trial-N persistence and H-18 deflation must pass before production loads a checkpoint. — I31:L34-L35, I31:L99
- **DII005-I31-024** `[E][blocking prerequisite]` D-6 held-out evaluation uses data never exposed to training. — I31:L36
- **DII005-I31-025** `[E][blocking prerequisite]` D-7 adds a cross-domain promotion canary. — I31:L37
- **DII005-I31-026** `[E][operations]` Ship a deterministic small-fixture harness first, exercising the real reward endpoint, canary injection, and KL computation. — I31:L38-L40, I31:L66-L74
- **DII005-I31-027** `[E][dependency/security]` Any RL stack (TRL, veRL, or custom) requires package/repository/date/license anti-slopsquat verification and explicit user confirmation before import. — I31:L40-L42, I31:L76-L78, I31:L114
- **DII005-I31-028** `[E][dependency]` The reward endpoint itself adds no external dependency. — I31:L42-L43
- **DII005-I31-029** `[E][capability]` Create `crates/training/grpo_harness.rs::GrpoHarness` with reward handling, KL penalty, canary hook, masking, and SSB emission. — I31:L68-L69, I31:L83-L84, I31:L89-L93
- **DII005-I31-030** `[E][workspace lifecycle]` Create the `crates/training/` workspace crate only if absent; otherwise modify the existing crate after re-grep. — I31:L83-L84
- **DII005-I31-031** `[E][allowlist lifecycle]` Reuse the `darwinian-evolve` allowlist row if it covers GRPO; otherwise add a promotion row under R9. — I31:L85
- **DII005-I31-032** `[E][test]` Create `crates/contracts/tests/i31_grpo_flywheel_contract.rs` with a seeded H-28 reward hack, missing trial-N rejection, and byte-hash-absence check. — I31:L72-L74
- **DII005-I31-033** `[E][artifact]` Ship `workflow/reasoning/I-31-decisions.md` with the blocking-prerequisite checklist. — I31:L71-L74
- **DII005-I31-034** `[E][deferred]` The live multi-night GRPO run is a user-approved, dependency-approved RunPod Tier-2 checkbox. — I31:L38-L42, I31:L76-L79
- **DII005-I31-035** `[E][deferred]` Full checkpoint promotion waits for a training-produced checkpoint. — I31:L79
- **DII005-I31-036** `[E][deferred]` Online D-5/D-29 bandit integration waits for live-run volume. — I31:L80
- **DII005-I31-037** `[E][surface]` The flywheel panel shows checkpoint admits, H-28 blocks, and per-episode KL trend; it consumes `RewardServed`, `PromotionBlocked`, and `PromotionLedgered` rows and must pass `cross_lang_diff.py`. — I31:L87, I31:L117-L118
- **DII005-I31-038** `[E][reachability]` Tier-2 production reachability is `elai train grpo --init <sft-checkpoint>`; Tier-1 contract-only invocation does not itself satisfy production R7. — I31:L104-L107
- **DII005-I31-039** `[E][events]` Add `GrpoRewardComputed`, baseline +1, with `RewardServed`, `PromotionBlocked`, and `PromotionLedgered` kinds. — I31:L107, I31:L117-L118, I31:L120-L123, I31:L134
- **DII005-I31-040** `[E][traceability]` `RewardServed` rows cite execution-gate verdicts, including linked `TriLayerVerdict`/`MutationBudgetedRun` rows. — I31:L120-L123
- **DII005-I31-041** `[E][promotion evidence]` `PromotionLedgered` must cite trial-N, deflation, held-out, and D-7 canary row identifiers. — I31:L113, I31:L120-L123
- **DII005-I31-042** `[E][negative test]` Grep for `derived_alpha` or `blake3(config` in the harness must return zero. — I31:L109-L110, I31:L123
- **DII005-I31-043** `[E][negative test]` A seeded harness-edit/reward-hack fixture must yield `PromotionBlocked`, never `PromotionLedgered`. — I31:L72-L73, I31:L122-L123
- **DII005-I31-044** `[E][negative test]` A candidate lacking the complete promotion chain must not emit `PromotionLedgered`. — I31:L123
- **DII005-I31-045** `[E][security]` The evaluator endpoint and all gate code stay outside the training loop's write boundary. — I31:L46-L47, I31:L58, I31:L111
- **DII005-I31-046** `[E][composition]` Consume H-43..H-48, I-30, I-24, H-28, IS-16, D-5/D-29, and E-55 without reauthoring their pipelines or gates. — I31:L137-L145
- **DII005-I31-047** `[E][safety]` Production paths ban `.unwrap()`/`.expect()` and blake2/sha2. — I31:L115

### I39 — `I-39_plan_state_from_repo.md`

- **DII005-I39-001** `[E][status]` I-39 is Track 18, `ANTI-DRIFT`, `STRONG`, `BUILD`, effort S–M, Wave 1. — I39:L1-L3
- **DII005-I39-002** `[E][problem/data]` The five drifting manual status sources are `CLAUDE.md`, session handoffs, plan index files, grounding maps, and memory-bank. — I39:L5-L7
- **DII005-I39-003** `[E][capability]` Add CLI `elai plan status [--letter X] [--id I-N]`. — I39:L7-L9, I39:L35-L41
- **DII005-I39-004** `[E][derivation]` Source one is `wiring_contract_check.py --static`, mapping its verdict to `PLANNED`/`IN-PROGRESS`/`DONE`/`BLOCKED`. — I39:L14-L16, I39:L37, I39:L50, I39:L63
- **DII005-I39-005** `[E][derivation]` Source two is presence/absence of each plan's `observable_effect` SSB rows. — I39:L16-L17
- **DII005-I39-006** `[E][derivation]` Source three is plan-status fragments from the `s-plan-audit-2026-06-04` FINISH files. — I39:L17-L18, I39:L31, I39:L66
- **DII005-I39-007** `[E][traceability]` Every status line carries a `verified against <commit-hash/map-date>` anchor. — I39:L19
- **DII005-I39-008** `[E][lifecycle]` On first run, manual tallies become build artifacts; mechanically derived command output is the later source of truth. — I39:L19-L21
- **DII005-I39-009** `[E][determinism]` The same journal snapshot must produce byte-identical output. — I39:L20-L21, I39:L79-L81
- **DII005-I39-010** `[E][capability]` Create `crates/cli/src/plan_status_cmd.rs::derive_plan_state()`, `format_status_line()`, and `run_plan_status()`. — I39:L35-L38, I39:L47-L48, I39:L55-L59
- **DII005-I39-011** `[E][events]` Add flat beacon `PlanStateDerived`, baseline +1, carrying `letter`, `session_id`, `state`, and `evidence_anchor`. — I39:L39, I39:L56-L60, I39:L80, I39:L90
- **DII005-I39-012** `[E][reachability]` Production status reachability is CLI invocation → `run_plan_status()` → `derive_plan_state()` → `PlanStateDerived`. — I39:L67-L69
- **DII005-I39-013** `[E][migration guard]` Create `crates/contracts/tests/i39_plan_state_migration_contract.rs` and open a real F-13 archived old-schema `ssb.db` through the current E-44 decode path. — I39:L23-L26, I39:L40, I39:L51-L52, I39:L60, I39:L69-L70
- **DII005-I39-014** `[E][migration acceptance]` Decode N rows to N well-formed results with no silent skip. — I39:L23-L26, I39:L40, I39:L69-L70, I39:L79-L81
- **DII005-I39-015** `[E][failure]` Any corrupted/undecodable archive fails loudly and exits nonzero with a named error. — I39:L25-L26, I39:L40, I39:L72-L73, I39:L81
- **DII005-I39-016** `[E][test reachability exception]` Contract callers satisfy R7 only for the migration guard because the real archived DB—not a production entrypoint—is the guard's subject. — I39:L69-L70
- **DII005-I39-017** `[E][constraint]` A mock `ssb.db` is forbidden for the migration contract. — I39:L71-L72
- **DII005-I39-018** `[E][constraint]` Hard-coded/manual status fields are forbidden; state must be derived mechanically. — I39:L74
- **DII005-I39-019** `[E][integration]` Execute `wiring_contract_check.py` as a subprocess and parse exit code/stdout rather than reimplementing it. — I39:L49-L50, I39:L63, I39:L94
- **DII005-I39-020** `[E][integration]` Reuse SSB viewer read paths, the F-13 archive, E-44 migration decode, and audit fragments; do not copy/reencode or reimplement these substrates. — I39:L47-L52, I39:L61-L66, I39:L92-L97
- **DII005-I39-021** `[E][surface]` No dashboard is added; CLI output plus documentation are the developer-facing artifacts. — I39:L53, I39:L76-L77
- **DII005-I39-022** `[E][artifact]` Ship `workflow/reasoning/I-39-decisions.md`. — I39:L41
- **DII005-I39-023** `[E][deferred]` Automatic sync on each git push is deferred as infrastructure rather than correctness. — I39:L43-L45
- **DII005-I39-024** `[E][deferred]` Cross-plan dependency visualization waits for post-Wave-4 data. — I39:L45
- **DII005-I39-025** `[E][safety]` Production paths ban `.unwrap()`/`.expect()` and use blake3, not blake2/sha2. — I39:L75

### I44 — `I-44_zero_retention_export.md`

- **DII005-I44-001** `[E][status/dependency]` I-44 is Track 19, `AUDITABLE/TABLE-STAKES`, `STRONG`, `BUILD`, effort S, Wave 3; it depends on G-9 and composes G-2/F-13/I-40. — I44:L1-L3
- **DII005-I44-002** `[E][scope]` Self-host, zero retention, and SOC 2 are framed as table stakes; this feature implements only the post-verification retention leg. — I44:L4-L11
- **DII005-I44-003** `[E][policy/default]` Define `crates/orchestrator/src/retention.rs::RetentionPolicy` with `Full` as default and opt-in `ZeroAfterVerify`. — I44:L13-L15, I44:L44-L48, I44:L70-L73
- **DII005-I44-004** `[E][timing/state]` `ZeroAfterVerify` purges only after G-2 RRA and off-agent verdicts are committed/sealed. — I44:L14-L16, I44:L48, I44:L62
- **DII005-I44-005** `[E][purge class]` Purge raw prompt bytes. — I44:L16-L18
- **DII005-I44-006** `[E][purge class]` Purge raw tool stdin and stdout. — I44:L16-L18
- **DII005-I44-007** `[E][purge class]` Purge raw model input and output. — I44:L16-L18
- **DII005-I44-008** `[E][granularity/constraint]` Because SSB is an append-only WAL with no row update/delete, retention purges whole expired session DB files using F-13's archive-then-purge primitive. — I44:L16-L18, I44:L31-L39, I44:L79, I44:L83, I44:L93
- **DII005-I44-009** `[E][retained evidence]` Retain F-1 `source_blake3`, F-4 `stdout_hash`, and F-9 `dep_name_hash`. — I44:L19-L20
- **DII005-I44-010** `[E][retained evidence]` Retain the G-2 evidence manifest and blake3 root. — I44:L19-L21, I44:L36-L37
- **DII005-I44-011** `[E][retained evidence]` Retain the G-8 signed attestation/verdict. — I44:L20-L21, I44:L36-L37
- **DII005-I44-012** `[E][retained evidence/privacy]` Retain only G-9-redacted summaries, with G-9 redaction enforced before purge. — I44:L20-L21, I44:L26-L27, I44:L32-L33, I44:L47
- **DII005-I44-013** `[E][events]` Add SSB action type `RetentionPurged`, baseline +1, with `RawInputsPurged`, `EvidenceRetained`, and `PurgeVerified` arms. — I44:L22-L23, I44:L49-L50, I44:L74-L75, I44:L118
- **DII005-I44-014** `[E][receipt schema]` The purge receipt records what was purged, what was retained, and a blake3 over the retained set. — I44:L22-L23, I44:L49-L50, I44:L97-L98
- **DII005-I44-015** `[E][privacy invariant]` Post-purge grep of the exported bundle for raw-input markers must be empty. — I44:L24-L25, I44:L51-L52, I44:L100-L106
- **DII005-I44-016** `[E][verification invariant]` `elai replay --verify` must still pass for hash-checkable steps after purge; verification cannot depend on purged bytes. — I44:L24-L26, I44:L51-L52, I44:L103-L107
- **DII005-I44-017** `[E][constraint]` Never purge evidence hashes; zero retention removes raw source, not off-agent evidence. — I44:L27-L28, I44:L88-L90
- **DII005-I44-018** `[E][safety/default]` Purge is destructive and irreversible, so it cannot be default-on and default behavior must remain unchanged. — I44:L28-L29, I44:L92
- **DII005-I44-019** `[E][capability]` Create `crates/orchestrator/src/retention.rs::purge_after_verify` as the post-verdict driver. — I44:L45-L47, I44:L60-L64, I44:L71-L74
- **DII005-I44-020** `[E][CLI]` Add `elai run --zero-retention` and check policy at the seal point. — I44:L48, I44:L84-L86
- **DII005-I44-021** `[E][export integration]` I-40 wrapper export checks `RetentionPolicy` before emitting a purge-eligible bundle. — I44:L40-L41, I44:L82, I44:L86
- **DII005-I44-022** `[E][reachability]` Production reachability is the zero-retention CLI/run seal path and I-40 export path; test-only callers do not satisfy R7. — I44:L84-L87
- **DII005-I44-023** `[E][test]` Create `crates/contracts/tests/i44_zero_retention_verify_survives_contract.rs`. — I44:L51-L52, I44:L65, I44:L76
- **DII005-I44-024** `[E][artifact]` Create `workflow/reasoning/I-44-decisions.md`. — I44:L66
- **DII005-I44-025** `[E][surface]` Render receipts in the existing RRA viewer/session-retention drawer, exposing purge/retained classes and retained-root blake3. — I44:L67-L68, I44:L96-L98
- **DII005-I44-026** `[E][acceptance/evidence]` A planted-secret fixture must produce all three receipt arms and an export with neither raw markers nor the secret in retained summaries. — I44:L99-L106
- **DII005-I44-027** `[E][determinism]` Full and ZeroAfterVerify runs over the same fixture differ only in raw-byte presence; evidence and verdict bytes are identical. — I44:L106-L107, I44:L114-L115
- **DII005-I44-028** `[E][deferred/non-goal]` SOC 2 is process/audit work outside engineering scope. — I44:L11, I44:L54-L55, I44:L116-L117
- **DII005-I44-029** `[E][deferred]` Configurable retention TTL (N days rather than immediate post-verdict) is deferred. — I44:L56
- **DII005-I44-030** `[E][deferred]` Self-host packaging/distribution is deferred as a business decision; I-40 already provides the local wrapper. — I44:L57-L58
- **DII005-I44-031** `[E][composition]` Compose, never reimplement, G-9 redaction, F-13 session archive, G-2 RRA, and G-8 attestation. — I44:L63-L64, I44:L77-L83, I44:L94, I44:L120-L124
- **DII005-I44-032** `[E][safety]` Production paths ban `.unwrap()`/`.expect()` and use blake3, not blake2/sha2. — I44:L95

### I51 — `I-51_model_capability_catalog.md`

- **DII005-I51-001** `[E][status]` I-51 is Track 20, `SMARTER-ORCH`, `STRONG (SL-4)`, `EXTENSION/consolidation`, effort S–M, Wave 1. — I51:L1-L4
- **DII005-I51-002** `[E][problem]` Model metadata is scattered across provider pricing, two `vram_tier.rs` modules, pricing rollup, and adapter defaults, causing duplicated/hard-coded derivations. — I51:L6-L12, I51:L27-L31
- **DII005-I51-003** `[E][capability]` Create `crates/providers/src/model_catalog.rs` plus one repo-artifact `models.toml` keyed by model ID. — I51:L14-L18, I51:L33-L39, I51:L43-L44
- **DII005-I51-004** `[E][schema field]` Catalog entries carry `context_window`. — I51:L10-L12
- **DII005-I51-005** `[E][schema field]` Catalog entries carry input pricing. — I51:L10-L12
- **DII005-I51-006** `[E][schema field]` Catalog entries carry output pricing. — I51:L10-L12
- **DII005-I51-007** `[E][schema field]` Catalog entries carry cache pricing. — I51:L10-L12
- **DII005-I51-008** `[E][schema field]` Catalog entries carry thinking support. — I51:L10-L12
- **DII005-I51-009** `[E][schema field]` Catalog entries carry thinking-effort mapping. — I51:L10-L12
- **DII005-I51-010** `[E][schema field]` Catalog entries carry default `max_tokens`. — I51:L10-L12
- **DII005-I51-011** `[E][schema field]` Catalog entries carry VRAM tier. — I51:L10-L12
- **DII005-I51-012** `[E][freshness]` Every catalog entry has a dated staleness stamp so pricing can be reverified at edit time. — I51:L17-L21, I51:L60-L62
- **DII005-I51-013** `[E][security/config]` Gate-relevant `models.toml` fields are diffable repo data and R8-frozen. — I51:L17-L18
- **DII005-I51-014** `[E][migration]` Preserve `pricing.rs` public APIs but turn their data implementation into a catalog view. — I51:L18-L20, I51:L34-L35, I51:L39, I51:L45-L46
- **DII005-I51-015** `[E][migration]` Preserve both `vram_tier.rs` public APIs but re-source them as catalog views. — I51:L18-L20, I51:L34-L35, I51:L39, I51:L45-L46
- **DII005-I51-016** `[E][compatibility]` Existing call sites must continue compiling against preserved APIs. — I51:L19-L20, I51:L47-L48, I51:L50-L51
- **DII005-I51-017** `[E][constraint]` Wrap and preserve the measured 8/12/16/24/48GB VRAM-tier semantics; deletion requires explicit user confirmation. — I51:L22-L23, I51:L49
- **DII005-I51-018** `[E][constraint]` The catalog performs no live price fetch. — I51:L24-L25, I51:L49-L50
- **DII005-I51-019** `[E][non-goal]` I-51 makes no router changes and adds/removes no models. — I51:L24-L25
- **DII005-I51-020** `[E][deferred]` Migrating every per-adapter default is deferred to follow-on patch work after the catalog is proven. — I51:L33-L36, I51:L66
- **DII005-I51-021** `[E][integration]` `pricing_rollup` and existing cost/tier consumers read through preserved views; I-10 and I-25 are future clean-input consumers. — I51:L30-L31, I51:L45-L48, I51:L64-L66
- **DII005-I51-022** `[E][surface]` No UI is added; existing cost and tier surfaces remain unchanged. — I51:L38-L41
- **DII005-I51-023** `[E][events]` Add no SSB action type; existing cost/route rows carry catalog-derived values. — I51:L52-L55, I51:L67
- **DII005-I51-024** `[E][behavioral invariant]` Fixture cost and tier output must be byte-identical before and after consolidation. — I51:L52-L58
- **DII005-I51-025** `[E][test]` A drift contract requires the two VRAM-tier modules to agree byte-for-byte through the catalog and fails on seeded view divergence. — I51:L34-L35, I51:L43-L44, I51:L56-L58
- **DII005-I51-026** `[E][test]` Grep must find zero hard-coded price constants outside `models.toml` in migrated modules. — I51:L49-L50, I51:L57-L58
- **DII005-I51-027** `[E][dependency]` Before execution, read the gajae `api-registry.ts`, `models.json`, and `model-manager.ts` catalog shape under SL-10. — I51:L14-L16
- **DII005-I51-028** `[E][safety]` Production paths ban `unwrap`/`expect` and API breakage. — I51:L49-L51

## Inferred findings (not source commitments)

- **DII005-INF-001** `[I][ordering]` I-6 cannot enforce acknowledgement tags until I-1's supervisor gate exists, so same-wave implementation needs I-1 before the I-6 enforcement slice even though both are labeled Wave 2. — inferred from I6:L33-L34, I6:L41-L42
- **DII005-INF-002** `[I][routing cardinality]` With multi-hop deferred, the Tier-1 I-7 failure path appears intended to make one synchronous cloud-architect call, but the original never states a numeric “exactly one” acceptance assertion. — inferred from I7:L17-L20, I7:L51-L53, I7:L91-L97
- **DII005-INF-003** `[I][context ordering]` Because I-14 transforms tool-result volatile-tail content before context assembly and I-6 appends a frozen final anchor, the intended composed order appears to keep `AnchorPane` outside compression. No assigned source states that cross-feature order directly. — inferred from I14:L40, I14:L65-L73 and I6:L12-L18
- **DII005-INF-004** `[I][routing layers]` I-25 preclassification and I-7 post-local-failure escalation are distinct routing stages and can compose without being alternate implementations: I-25 acts pre-cloud, while I-7 acts after all local branches fail. — inferred from I25:L29-L31, I25:L80-L82 and I7:L15-L20
- **DII005-INF-005** `[I][promotion pattern]` I-25 router artifacts and I-31 checkpoints share the same architectural rule: evaluators/trainers do not own the frozen promotion writer, and only an IS-16 ledger win can activate an artifact. — inferred from I25:L31-L36, I25:L49-L50, I25:L155-L156 and I31:L32-L37, I31:L99-L101, I31:L141-L142
- **DII005-INF-006** `[I][future integration]` I-51 can eventually normalize pricing/VRAM inputs consumed by I-25, but I-25 does not currently declare `model_catalog` as a direct dependency; this is future composition, not present wiring. — inferred from I51:L64-L66 and I25:L73-L79
- **DII005-INF-007** `[I][journal lifecycle]` I-39's plan status can use new action rows from the other sessions only after their `observable_effect` events exist; it does not confer DONE merely because a plan file exists. — inferred from I39:L14-L21
- **DII005-INF-008** `[I][retention/replay]` Under I-44 `ZeroAfterVerify`, I-14 reverse pointers and I-11 exact-received-payload replay may become hash-verifiable but non-reexpandable once their raw backing bytes are purged. The sources do not define an exemption. — inferred from I14:L22-L23, I11:L17-L18, and I44:L16-L26
- **DII005-INF-009** `[I][behavior-neutral scope]` I-51's byte-identical claim applies to consolidation against a fixed catalog snapshot; a later reviewed pricing/staleness update would intentionally change cost output. — inferred from I51:L20-L21, I51:L52-L58

## Ambiguities and unresolved specifications

- **DII005-AMB-001** `[A][I6 protocol]` The acknowledgement tag's exact grammar and the size of the “first reply token window” are unspecified. — I6:L15-L16, I6:L41-L42, I6:L84-L87
- **DII005-AMB-002** `[A][I7 config]` K (local branch width) and the precise tokenizer/rounding semantics behind “≤~2K tokens” are not specified. — I7:L15-L18, I7:L84-L85
- **DII005-AMB-003** `[A][I11 scheduling]` The honesty job says “periodic” and defaults to ten escalations, but persistence, scheduler ownership, and behavior across restarts are unspecified. — I11:L35-L36, I11:L45-L46
- **DII005-AMB-004** `[A][I11 failure recovery]` A DKG must be reachable at decode time, but the typed failure/fallback when it is unavailable is not defined. — I11:L88
- **DII005-AMB-005** `[A][I14 storage]` The receipt points to original bytes by blake3, but storage location, retention duration, and authorization for re-expansion are unspecified. — I14:L22-L23, I14:L38, I14:L81-L82
- **DII005-AMB-006** `[A][I14 event typing]` Numeric-quantization Tier 2 survives, but the source does not say whether it uses generic `Tier2Compressed` or a distinct discriminator. — I14:L17-L23, I14:L38-L39
- **DII005-AMB-007** `[A][I22 parameters]` The source deliberately leaves weight values, D, decay factor, and the threshold that triggers sibling backtracking to measurement; their config schemas/ranges are not defined. — I22:L16-L17, I22:L25-L28, I22:L49-L50, I22:L93-L95
- **DII005-AMB-008** `[A][I22 fallback field]` The “wiring contract fallback” for requirements coverage is named but no exact field identifier/schema is given. — I22:L14-L15
- **DII005-AMB-009** `[A][I25 enable flag]` `ELAI_ROUTE_DECISION_SSB_ENABLE` explicitly gates event emission, while the wargame also mentions a default-off profiler config without naming whether it is the same or a separate switch. — I25:L27-L28, I25:L80-L82, I25:L140-L143
- **DII005-AMB-010** `[A][I25 fingerprint]` The exact task-shape fingerprint schema, expensive-match threshold, cache lifecycle, and model-artifact TOML schema are not specified. — I25:L18-L20, I25:L40-L48
- **DII005-AMB-011** `[A][I31 canary event]` `ScorerNeutralized` is required on canary divergence, but the source neither registers it as a new action type nor identifies an existing owning event family. — I31:L25-L27, I31:L66-L74, I31:L134
- **DII005-AMB-012** `[A][I31 endpoint protocol]` The mechanism section permits typed local gRPC/HTTP, while revised Tier 1 names Axum; the revised scope strongly implies HTTP, but the external contract is not stated. — I31:L15-L17, I31:L40-L43, I31:L68-L69
- **DII005-AMB-013** `[A][I31 allowlist]` Coverage by the existing `darwinian-evolve` row must be verified and may require a new R9 promotion row, so “existing row covers the lane” is not yet established. — I31:L64, I31:L85, I31:L144
- **DII005-AMB-014** `[A][I39 precedence]` When static-check, journal, and fragment evidence disagree, no precedence rule for the four `PlanState` values is specified. — I39:L14-L21, I39:L37
- **DII005-AMB-015** `[A][I39 fixture]` N and the exact repo-relative archived database path are left to implementation-time re-grep. — I39:L23-L26, I39:L31-L33, I39:L40, I39:L51-L52
- **DII005-AMB-016** `[A][I44 deletion boundary]` “Archive-then-purge” does not state whether the archive itself is immediately deleted, encrypted, moved out of the export, or retained under another policy. — I44:L16-L18, I44:L34-L35, I44:L79
- **DII005-AMB-017** `[A][I44 erasure claim]` Acceptance proves absence from the exported bundle and purges session DB files, but does not define deletion guarantees for backups, filesystem snapshots, caches, or remanence. — I44:L22-L26, I44:L100-L107
- **DII005-AMB-018** `[A][I51 artifact path]` The source names `models.toml` but does not give its exact repository path relative to `crates/providers/src/model_catalog.rs`. — I51:L17-L18, I51:L38-L39
- **DII005-AMB-019** `[A][I51 governance]` Gate-relevant catalog fields are R8-frozen, but the authorized update/reverification workflow is not specified. — I51:L17-L21
- **DII005-AMB-020** `[A][I51 test symbol]` A duplication-drift contract is required, but its exact test file and symbol are not named. — I51:L34-L35, I51:L43-L44

## Conflicts and material source/summary mismatches

- **DII005-CON-001** `[C][I6 identifier]` Summary `I-6.01` says `PrivilegedAnchor`; the original's exact and repeated identifier is `ContextSlot::AnchorPane`. Replace the summary name. — summary:L225; I6:L25-L27, I6:L38, I6:L59-L61
- **DII005-CON-002** `[C][I6 protocol]` Summary `I-6.06` loses “highest-priority,” supervisor validation, first-token-window, and retry-on-missing semantics. It must be narrowed/expanded to the original contract. — summary:L230-L231; I6:L15-L16, I6:L41-L42
- **DII005-CON-003** `[C][I7 UI status]` Summary `I-7.09` presents the cost-cockpit trace as an active feature, but the revised original explicitly defers it until the GD-4 bound is measured. — summary:L253; I7:L51-L54, I7:L87-L90
- **DII005-CON-004** `[C][I7 cardinality]` Summary `I-7.11` asserts a local-failure fixture escalates “exactly once”; the original specifies a single-hop Tier-1 sequence and defers multi-hop but never states that numeric acceptance assertion. Mark it inferred or restate as `Condensed` then `Escalated`. — summary:L255; I7:L17-L20, I7:L51-L53, I7:L95-L97
- **DII005-CON-005** `[C][I11 receipt wording]` Summary `I-11.10` says every handoff is an ID-3 receipt; the original says every condensation emits the receipt and separately emits `Handoff` when dispatched. — summary:L347; I11:L17-L18, I11:L35, I11:L73-L75
- **DII005-CON-006** `[C][I14 rollout]` Summary `I-14.06` reads as though Tier-2 token classification ships, but the revised original defers it behind dependency approval, prose-only, and VRAM gates. — summary:L423-L424; I14:L43-L45, I14:L71-L72, I14:L91
- **DII005-CON-007** `[C][I22 PPM semantics]` Summary `I-22.12` can be read as shaping the D-2 PPM score; the original explicitly says I-22 shapes the L2 input, not PPM output. — summary:L642; I22:L35-L36, I22:L73
- **DII005-CON-008** `[C][I25 live seam]` Summary `I-25.11` points to `escalation_policy/cloud_mode`; the ratified source explicitly moves Slice 1 to `task_runner.rs::run_cost_router_preclassify` and warns against the former/dark hook. — summary:L703; I25:L25-L32, I25:L74-L82
- **DII005-CON-009** `[C][I25 UI status]` Summary `I-25.12` says route explanations are shown in the cockpit; the source defers all dashboard/Svelte work under `owner-ui-rework-hold`. — summary:L704; I25:L52-L56, I25:L92-L93
- **DII005-CON-010** `[C][I25 event payload]` Summary `I-25.13` says `RouteDecisionEvent` contains predicted-versus-actual outcome; the source's live payload keys are only `feature_vec`, `predicted_lane`, and `reason`. Actual verifier outcome is joined offline by the trainer. — summary:L705-L706; I25:L20-L21, I25:L43-L48, I25:L76
- **DII005-CON-011** `[C][I25 run count]` Summary `I-25.14` requires at least three runs; the original requires an ELAI mixed-batch comparison but gives no three-run minimum. — summary:L707-L708; I25:L15-L18, I25:L100-L105
- **DII005-CON-012** `[C][I25 loss record]` Summary `I-25.15` asserts an “honest loss” record; the original specifies an eval report and no promotion/load for a loser, but no event or ledger row with that name. — summary:L709; I25:L47-L50, I25:L98, I25:L144-L146
- **DII005-CON-013** `[C][I25 blocked ledger]` Summary `I-25.16` says the losing-router block is ledgered. The source only requires promotion to depend on a ledger win and the loser path to remain absent; the Python trainer expressly cannot write ledger rows. — summary:L710; I25:L29-L32, I25:L49-L50, I25:L155-L156
- **DII005-CON-014** `[C][I31 allowlist certainty]` Summary `I-31.19` states the existing allowlist definitely covers GRPO; the source requires coverage verification and allows adding an R9 promotion row if it does not. — summary:L850-L851; I31:L64, I31:L85
- **DII005-CON-015** `[C][I39 event trigger]` Summary `I-39.15` attaches `PlanStateDerived` to successful archive replay; the original attaches it to the production `elai plan status` query and says migration replay reuses existing rows. — summary:L1074; I39:L39, I39:L59, I39:L67-L70
- **DII005-CON-016** `[C][I39 five-source ambiguity]` Summary `I-39.17` calls the five drifting sources unenumerated, but the assigned original enumerates all five explicitly. — summary:L1076-L1077; I39:L5-L7
- **DII005-CON-017** `[C][I44 granularity]` Summary `I-44.07` can imply each raw input is a separate file; the original's enforceable unit is the whole expired session DB because row deletion is forbidden. — summary:L1213-L1214; I44:L16-L18, I44:L38-L39, I44:L93
- **DII005-CON-018** `[C][I51 current consumers]` Summary `I-51.17` includes runtime defaults as though migrated now; the original defers per-adapter defaults and names I-10/I-25 as future consumers. — summary:L1401; I51:L24-L25, I51:L33-L36, I51:L64-L66

## Cross-file interaction ledger

| Interaction | Classification | Exact relationship and anchors |
|---|---|---|
| I7 → I11 | Explicit | I-7 cannot ship its condense-then-escalate path until I-11 supplies `BeliefState::condense`/typed encode; I-11 in turn names I-7 as its local-to-cloud consumer. See DII005-I7-006/015 and DII005-I11-010/022. |
| I6 ↔ I14 | Inferred ordering | I-14 transforms volatile tool-result tails before context assembly; I-6 appends a frozen last slot. The likely nonoverlap is recorded only as DII005-INF-003 because neither source directly specifies their composed order. |
| I11 ↔ I14 | Explicit shared doctrine | Both are ID-3 journaled middleware with reversible evidence, but one types handoffs and the other compresses tool-result payloads. See DII005-I11-012/014 and DII005-I14-013/016. |
| I7 ↔ I25 | Inferred stage separation | I-25 preclassifies on the task-runner pre-cloud path; I-7 escalates after all local branches fail. See DII005-INF-004; neither source makes the other a direct dependency. |
| I25 ↔ I31 | Explicit shared promotion substrate | Both consume the frozen IS-16 ledger chain and prevent their training-side code from authoring activation. See DII005-INF-005, grounded in DII005-I25-029/031/032 and DII005-I31-022/023/041/045. |
| I25 ← I51 | Explicit future composition | I-51 calls I-25 a future clean-input consumer while explicitly excluding router changes in I-51. See DII005-I51-019/021 and DII005-INF-006. |
| I22 ↔ I31 | Explicit common verifier dependency, no direct edge | I-22 consumes I-17 `HardVeto` for truncation; I-31 consumes I-17 compile/test verdicts for reward. Neither original says I-31 consumes I-22's shaped reward. See DII005-I22-010/020 and DII005-I31-009/010. |
| I39 ↔ “E-44” | Naming warning | I-39's migration dependency is **E-44 migration machinery**, not this chunk's I-44 retention feature. The identifiers must not be conflated. See DII005-I39-013/020 versus DII005-I44-001. |
| I44 ↔ I11/I14 | Inferred lifecycle tension | Zero retention purges raw model/tool bytes after verification, whereas I-11/I-14 promise exact re-expansion/reconstruction. No exemption is specified; see DII005-INF-008, DII005-AMB-005, and DII005-AMB-016. |
| I39 ↔ I44 | Ambiguous archive lifecycle | I-39 needs a real F-13 archived DB indefinitely enough for migration tests, while I-44 reuses archive-then-purge. Their archive-selection and retention rules are not reconciled in the assigned files. See DII005-I39-013 and DII005-AMB-016. |

## Exhaustive summary delta ledger

The orientation summary sections audited were `09-I-SUMMARY_INDEX.discovery.md:221-259,334-360,413-436,626-646,689-710,827-861,1053-1077,1202-1229,1380-1407`, plus its interaction/dependency ledgers at lines 1493-1524 and 1544-1570. The lists below are the complete nonduplicate change set: “add” means the original fact is absent or materially under-specified; “correct” points to a conflict definition above. Facts already represented accurately are intentionally not repeated.

### I6 summary delta

- **Correct:** DII005-CON-001 and DII005-CON-002.
- **Add:** DII005-I6-002, DII005-I6-009, DII005-I6-010, DII005-I6-012, DII005-I6-014, DII005-I6-016 through DII005-I6-020, DII005-I6-023, DII005-I6-025 through DII005-I6-027.

### I7 summary delta

- **Correct:** DII005-CON-003 and DII005-CON-004.
- **Add:** DII005-I7-002 through DII005-I7-005, DII005-I7-008, DII005-I7-010, DII005-I7-012 through DII005-I7-014, DII005-I7-016 through DII005-I7-020, DII005-I7-022, DII005-I7-023, and DII005-I7-026 through DII005-I7-029.

### I11 summary delta

- **Correct:** DII005-CON-005.
- **Add:** DII005-I11-002, DII005-I11-003 (exact symbol/path), DII005-I11-014 through DII005-I11-016, DII005-I11-018, DII005-I11-022 through DII005-I11-026, and DII005-I11-028 through DII005-I11-032.

### I14 summary delta

- **Correct:** DII005-CON-006.
- **Add:** DII005-I14-002, DII005-I14-003 (exact symbols/path), DII005-I14-009, DII005-I14-010, DII005-I14-012 through DII005-I14-014, DII005-I14-017, DII005-I14-020, and DII005-I14-024 through DII005-I14-027.

### I22 summary delta

- **Correct:** DII005-CON-007.
- **Add:** DII005-I22-006 through DII005-I22-008, DII005-I22-011 (exact k−1/reinvestment behavior), DII005-I22-013 through DII005-I22-018, DII005-I22-021 through DII005-I22-025, and DII005-I22-027 through DII005-I22-029.

### I25 summary delta

- **Correct:** DII005-CON-008 through DII005-CON-013.
- **Add:** DII005-I25-007 through DII005-I25-010, DII005-I25-013, DII005-I25-015, DII005-I25-019 through DII005-I25-055, and DII005-I25-057. This includes the full ratified three-slice design and J-11 difficulty-fusion amendment, neither of which the orientation summary captures.

### I31 summary delta

- **Correct:** DII005-CON-014.
- **Add:** DII005-I31-006, DII005-I31-007 (exact Axum/symbol choice), DII005-I31-014, DII005-I31-015, DII005-I31-018, DII005-I31-021 (live `PrmCanaryEvent` evidence), DII005-I31-026, DII005-I31-028 through DII005-I31-033, DII005-I31-035, DII005-I31-036, DII005-I31-038 through DII005-I31-044, DII005-I31-046, and DII005-I31-047.

### I39 summary delta

- **Correct:** DII005-CON-015 and DII005-CON-016.
- **Add:** DII005-I39-003 (the `--id` filter), DII005-I39-004 (exact state set), DII005-I39-010 through DII005-I39-020, and DII005-I39-022 through DII005-I39-025.

### I44 summary delta

- **Correct:** DII005-CON-017.
- **Add:** DII005-I44-002, DII005-I44-009 through DII005-I44-011, DII005-I44-014, DII005-I44-019, DII005-I44-021 through DII005-I44-026, and DII005-I44-028 through DII005-I44-032.

### I51 summary delta

- **Correct:** DII005-CON-018.
- **Add:** DII005-I51-002, DII005-I51-003 (exact module), DII005-I51-013, DII005-I51-017 (explicit-confirmation deletion guard), DII005-I51-019, DII005-I51-027, and DII005-I51-028.

## Coverage, omission, and duplicate audit

- Assigned-original files opened completely: **10/10**.
- Assigned-original logical lines read: **1,169/1,169**.
- Assigned-original bytes verified: **95,848/95,848**.
- Manifest SHA-256 matches: **10/10**.
- Other original chunks read: **0**.
- Orientation source: only `09-I-SUMMARY_INDEX.discovery.md`, and only for summary reconciliation.
- Explicit per-file atomic findings: **333** (`I6` 27; `I7` 29; `I11` 32; `I14` 27; `I22` 29; `I25` 57; `I31` 47; `I39` 25; `I44` 32; `I51` 28).
- Non-explicit atomic findings: **47** (9 inferred; 20 ambiguous; 18 conflicting).
- Total unique atomic finding definitions: **380**.
- Duplicate atomic IDs: **0**.
- Empty/unclassified atomic definitions: **0**.
- Requested domains audited per source: capabilities/actions/surfaces; workflows/states; schemas/data/lifecycle; flags/defaults/config; integrations/events; permissions/security/privacy/compliance; failures/fallback/retry/recovery; determinism/idempotency/concurrency; observability/evidence; tests/acceptance/done gates; constraints/non-goals; dependencies/status/deferred.
- Omission audit: every substantive normative sentence, schema field, named symbol/seam, event kind, state transition, flag/default, explicit exclusion, deferred item, observable, negative arm, and verification gate in the ten originals maps to at least one finding above. Repeated `Grounding`/`COMPOSES-WITH` restatements map to their first atomic definition rather than creating duplicates.
- Cross-file audit: producer/consumer, ordering, shared-ledger, archive/retention, event, and future-consumer relationships are covered in the interaction ledger; uncertain links are labeled inferred or ambiguous.
- Summary audit: all material additions and all identified corrections are listed in the exhaustive delta ledger; unchanged summary facts were not duplicated.

## Output digest convention

The final whole-file SHA-256 cannot be embedded in the file it hashes without changing that hash. The actual SHA-256 of this completed file is therefore reported in the delivery message after the final write and verification. This section is part of the hashed output.
