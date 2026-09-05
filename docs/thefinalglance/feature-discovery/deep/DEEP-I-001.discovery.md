# DEEP-I-001 exhaustive feature discovery

## Scope, integrity, and method

- Processed manifest: docs/thefinalglance/feature-discovery/DEEP-SOURCE-MANIFEST.md:L776-L792, heading ### DEEP-I-001. The following chunk begins at L793.
- Declared chunk: family I, 10 files, 1,169 logical lines, 99,202 bytes, comprising nine numbered/index originals and one auxiliary/grounding source; target status under 1,800.
- Mechanical verification: fresh sha256sum and wc -l -c checks matched every row and both aggregates exactly. Missing paths: 0; unexpected paths: 0; duplicate paths: 0; SHA mismatches: 0; line mismatches: 0; byte mismatches: 0.
- Read boundary: all ten listed sources were read completely, first through last logical line. No other original deep-source chunk was opened. The auxiliary I-GROUNDING-MAP.md is first-class evidence. 09-I-SUMMARY_INDEX.discovery.md was used only for orientation/delta comparison; no atom is sourced solely from it.
- Anchor notation refers to the manifest-verified images: IG:Lx-Ly for I-GROUNDING-MAP.md and I-8:Lx-Ly, I-13:Lx-Ly, and so on for numbered plans.
- Repeated premise, scope, wiring, grounding, and acceptance statements are collapsed into one atom with all load-bearing anchors. Interaction and summary-delta records point to atoms and are not duplicate feature atoms.
- Classification: E = explicit source fact/requirement; I = bounded inference, never promoted to a requirement; A = unresolved ambiguity/choice; C = conflict/correction that cannot be silently flattened. Domains: STATUS, CAP, SURFACE, FLOW, STATE, DATA, CFG, INT, EVENT, SEC, PRIV, COMP, FAIL, RECOVERY, DET, IDEM, CONC, OBS, TEST, LIMIT, DEP, DEFER.

### Exact processed manifest and verification

| Manifest class | Lines | Bytes | SHA-256 | Exact absolute path | Verification |
|---|---:|---:|---|---|---|
| auxiliary/grounding/fixture | 207 | 19,331 | ce1f436fb27ffce089a287819303292ebdebaface6ade9a19c1eb3a84c7fc4bc | docs/thefinalglance/I-plan/_grounding/I-GROUNDING-MAP.md | exact |
| numbered/index original | 132 | 9,754 | ccab90fbbb09287db8a48034d0456cec1db3772a4ead607ed1a5dddb84662039 | docs/thefinalglance/I-plan/I-8_aletheia_voluntary_abort.md | exact |
| numbered/index original | 96 | 8,373 | ea8f8892b102b2d63a660258f6cdf7c34be9bd487bc7febe9eaebfb9e4123f7f | docs/thefinalglance/I-plan/I-13_memory_guard_dual_serialization.md | exact |
| numbered/index original | 91 | 7,905 | 37ef02f48c9516f096a6be9d9c7fe02be2ea6430e709ec8f1bc908446119bcee | docs/thefinalglance/I-plan/I-15_semantic_cache_blake3_fastpath.md | exact |
| numbered/index original | 107 | 9,384 | 2860acc45deb41191eaaea364e1660426133a1300e406bab0dae9ce794866a9d | docs/thefinalglance/I-plan/I-18_upip_chain_sigstore_aibom.md | exact |
| numbered/index original | 109 | 10,168 | db9dd2b95a380588951ce8770f808906ae5ebb35e2858a7c215a211d7dc94ebf | docs/thefinalglance/I-plan/I-23_unified_conformal_calibration.md | exact |
| numbered/index original | 126 | 11,150 | 1002977a47734e94adf737f305eb47547dfb08e22e7b1efa6de5e3dccb02a7b6 | docs/thefinalglance/I-plan/I-28_dag_credit_assignment.md | exact |
| numbered/index original | 115 | 9,143 | 1da497b94fd36a98295b97884af93011f71c69ac3e9a25b0ee0bb2480e6b698e | docs/thefinalglance/I-plan/I-36_cross_model_prompt_compilation.md | exact |
| numbered/index original | 118 | 9,493 | 32a8ee7339c40fbad52c46beb8bcf56a0dc8aa4a8b8882cd160e5b8b4dbd4dd8 | docs/thefinalglance/I-plan/I-37_ears_requirement_objects.md | exact |
| numbered/index original | 68 | 4,501 | ee8dd28f1b2e37953a369a71300f2314be868da7a05938a6928d29ec38711f95 | docs/thefinalglance/I-plan/I-48_prompts_as_artifacts.md | exact |
| **Aggregate** | **1,169** | **99,202** | — | **10 exact paths** | **exact** |

## Per-file atomic ledgers

### I-GROUNDING-MAP — first-class grounding, corrections, and seams

Source: docs/thefinalglance/I-plan/_grounding/I-GROUNDING-MAP.md (207/207 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| IG-E001 | E/STATUS | The map was grounded 2026-06-10 from live read-only source greps, the 13-file prefable-research corpus, B–H summaries, the wire-remediation handoff, and the finish index. | IG:L1-L6 |
| IG-E002 | E/TEST | elai.map was intentionally not regenerated because it was stale versus concurrent H edits; every load-bearing anchor was source-grep confirmed and executors must re-grep at EXECUTING under W-7. | IG:L3-L7 |
| IG-E003 | E/DATA | The dated live constants were SCHEMA_VERSION=221 at ssb.rs:2981 and SCHEMA_VERSION_PUBLIC at :2986. | IG:L9-L12 |
| IG-E004 | E/DATA | The dated action registry held TOTAL_VARIANTS=532 at action_type_lock.rs:601 and ALL_VARIANTS at :610. | IG:L12-L13 |
| IG-E005 | E/CFG | Prior values 191/464, 207/484, 220/497, 224/501, 234/511, 236/513, 240/517, 209/520, and 522 are explicitly stale; every I-N must express a baseline-relative bump from a freshly reread live pair. | IG:L14-L16 |
| IG-E006 | E/FAIL | The pre-existing ssb_schema_v97_payloads::schema_version_is_97 test still asserts 97; its fix belongs to a patch-N, not an I-N. | IG:L17-L18 |
| IG-E007 | E/STATE | The SSB schema is golden-locked and append-only. | IG:L21-L23 |
| IG-E008 | E/STATE | darwin_admit_loop.rs currently derives fitness as blake3(config bytes), a placeholder that H-43 replaces; I-31 consumes execution-grounded fitness and never the byte hash. | IG:L24-L25 |
| IG-E009 | E/STATE | trap_tasks.rs currently uses keyword-grep trap scoring; H-44 replaces it and I-24/I-31 must compose H-44. | IG:L26-L26 |
| IG-E010 | E/STATE | config_archive.rs already exposes insert_variant, select_parent, and select_parent_with for the Darwin archive consumed by I-28/I-31. | IG:L27-L27 |
| IG-E011 | E/STATE | Anthropic CacheControl and ephemeral markers exist with a four-marker maximum, but the RuntimeAdapter path still supplied cache_control=None at grounding time; I-10 is only the TTL/economics/guard layer if H-1 lands first. | IG:L28-L30 |
| IG-E012 | E/STATE | context_budget.rs already provides blake3 prefix stability, adaptive_reduce, and nine ContextSlot values including Memory; I-6 adds its anchor slot last and I-5/I-14 must preserve breakpoint stability. | IG:L31-L33 |
| IG-E013 | E/STATE | Banded canary deltas and conformal calibration/bootstrap consumers already exist as the D-16/D-30 substrate for I-23. | IG:L34-L36 |
| IG-E014 | E/STATE | RelationLibrary, MutationRunner, and a blake3-keyed 30-day EquivalentMutantCache already exist; I-20 extends both testing systems and I-24 extends the relation library. | IG:L37-L39 |
| IG-E015 | E/RECOVERY | bsp/engine.rs recover_from_checkpoint and a test exist, correcting the stale “stub” framing; production reachability remained unverified and must be retraced. | IG:L40-L41 |
| IG-E016 | E/STATE | build_rra/verify_rra and attestation.rs already exist; I-18 extends them with row chaining, Rekor push, and AI-BOM rather than rebuilding them. | IG:L42-L43 |
| IG-E017 | E/SEC | Taint/provenance substrate exists in capability_flow.rs and the dedicated safety crate; G-1 ingest liveness is the composition seam for I-13/I-35/I-40. | IG:L44-L46 |
| IG-E018 | E/STATE | DecodingMode::Grammar already carries ebnf_path and crane_adaptive, but crane_adaptive only affects eligibility because the CRANE runtime is absent; I-34 builds that runtime leg. | IG:L47-L49 |
| IG-E019 | E/STATE | TemplateCrystallizer and CompiledPipelineStore already ship; they provide the zero-token curator seam for I-12 and the storage precedent for I-36. | IG:L50-L51 |
| IG-E020 | E/INT | production_agent_rpc.rs build_tool_catalog exposes the three-arm convert/dispatch tool-array pattern; I-10 must preserve its prefix stability and I-35 gates it. | IG:L52-L53 |
| IG-E021 | E/STATE | IntentPair, IntentPairSet, and IntentPairsGate already ship as the substrate generalized by I-37. | IG:L54-L55 |
| IG-E022 | E/STATE | The named Track-3/4 ledger, budget, deflation, self-validation, drift, leakage, null, and manifest gate files exist; ID-6 binds weight/prompt promotions to them. | IG:L56-L58 |
| IG-E023 | E/STATE | The workspace already contains safety, memory, eval-corpus, ssb-otel-export, web_search, web_validator, agent, ipc-types, scaffold, dashboard-v2, and parked/backup dashboard crates. | IG:L59-L60 |
| IG-E024 | E/STATUS | streaming_validator, batch-message, memory retrieval, and conformal-calibration tokens exist at file level, but their wiring depth was not verified and must be retraced under R7. | IG:L61-L64 |
| IG-E025 | E/OBS | Wiring and freeze tooling exists: wiring_contract_check.py, gate_freeze_check.py, wire-dark allowlist, failure_taxonomy.py, and impact_radius.py. | IG:L65-L66 |
| IG-E026 | E/STATUS | The dated static D/E/F/G tally was 34 wired, 21 dark, 50 partial, and 3 empty-contract, with no clean wiring flips left; remaining work was feature engineering, GPU, or new dependencies. | IG:L66-L69 |
| IG-E027 | E/STATUS | No MCTS module exists; I-4 is a genuine ladder build rather than a day-one tree. | IG:L71-L75 |
| IG-E028 | E/STATUS | No typed BeliefState exists; I-11 is a build. | IG:L76-L76 |
| IG-E029 | E/STATUS | No AnchorPane/privileged-last slot exists among the nine ContextSlots; I-6 is a build. | IG:L77-L77 |
| IG-E030 | E/STATUS | No sprint_contract module exists; I-21 builds one on H-37/H-38 seams. | IG:L78-L78 |
| IG-E031 | E/STATUS | No per-row chain_prev_hash exists; I-18 builds that field. | IG:L79-L79 |
| IG-E032 | E/STATUS | No compression middleware exists; I-14 is a deterministic-first build. | IG:L80-L80 |
| IG-E033 | E/STATUS | No EARS requirement object exists; I-37 is a build. | IG:L81-L81 |
| IG-E034 | E/STATUS | No per-backend compiled-prompt artifact store exists; I-36 builds it using CompiledPipelineStore as precedent. | IG:L82-L82 |
| IG-E035 | E/STATUS | No QDELTA quality harness exists though G-6 SAFEDELTA exists; I-38 builds the sibling quality harness. | IG:L83-L84 |
| IG-E036 | E/STATUS | No elai plan status repo-derived command exists; I-39 is a build. | IG:L85-L85 |
| IG-E037 | E/STATUS | No CEFL externalization, I6Q rationale-quality, or commit-reveal entropy primitives exist; I-33 builds them. | IG:L86-L86 |
| IG-E038 | E/STATUS | FSM masks, argv allowlists, and tool-set commitments are per-call gates; no temporal ordering-over-history rule exists, so I-35 builds it. | IG:L87-L88 |
| IG-E039 | E/STATUS | No cache-economics planner, TTL selector, or cache-hit hard guard exists; I-10 builds them on H-1/H-2. | IG:L89-L90 |
| IG-E040 | E/STATUS | No error-amplification alpha_t harness exists; I-9 builds it from drift and mutation components. | IG:L91-L91 |
| IG-E041 | E/CORRECTION | I-7 interactive Batch API racing is architecturally rejected because batch runs take 10–45 minutes, have a 24-hour SLA, and cancellation does not refund queued work; corrected behavior is local-first, condense-then-escalate, with batch confined to offline lanes. | IG:L93-L100 |
| IG-E042 | E/CORRECTION | I-14 is deterministic LSP slicing and tree-sitter folding first, replacing irrelevant bodies with an elision marker while preserving signatures; LLMLingua-2 is a gated prose-only Tier 2 because its XLM-R encoder consumes VRAM. | IG:L101-L104 |
| IG-E043 | E/LIMIT | CompactPrompt/n-gram code-symbol abbreviation is killed as neural poison; only narrow numeric quantization for tabular data survives as a Tier-2 option. | IG:L105-L106 |
| IG-E044 | E/CORRECTION | Cache micro-heartbeats are last-resort only; I-10 primarily uses explicit one-hour frozen-prefix and five-minute volatile-tail TTLs plus wide-local/burst-cloud scheduling. | IG:L107-L110 |
| IG-E045 | E/CORRECTION | H-4 tool deferral and I-10 prefix invariance are both valid levers; catalog choice must be based on journal-measured cache-hit-times-token cost, with either inline-frozen small catalogs or byte-stable deferral structure for large catalogs. | IG:L111-L114 |
| IG-E046 | E/CORRECTION | I-4 is a value-yielding ladder: v1 best-of-N with deterministic verifier ranking, v2 tournament, v3 PUCT with virtual loss; the XL tree does not block Tracks 14/15. | IG:L115-L117 |
| IG-E047 | E/CORRECTION | Track 17 is resequenced: verifier-ranked rejection-sampling I-30 precedes I-31 GRPO, and I-31 is blocked by canaries, tamper gates, ledger promotion, KL penalty, and an evaluator outside the write boundary. | IG:L118-L122 |
| IG-E048 | E/DEP | The canonical local model remains Qwen2.5-Coder; any MoE retarget is a RunPod-gated evaluation, never an assumed swap. | IG:L120-L122 |
| IG-E049 | E/CORRECTION | I-11's differentiator over commodity BeliefState patterns is typed, DKG-linked, journaled, and honestly measured against periodic full-history reruns. | IG:L123-L124 |
| IG-E050 | E/CORRECTION | I-2 is cross-platform: overlayfs on Linux, pooled warm git worktrees on Windows/macOS, and an optional cloud-sandbox adapter behind one trait. | IG:L125-L127 |
| IG-E051 | E/CORRECTION | I-13 must compose H-31/H-32, E-37, and memory MINJA; net-new scope is write-time deterministic scanning, dual serialization, and TraceQuery, with regex/Aho-Corasick before any approved YARA dependency. | IG:L128-L131 |
| IG-E052 | E/CORRECTION | I-18 is an extension of shipped RRA/attestation: per-row previous-hash chain, five-layer stack hash, optional Rekor, and AI-BOM for the named AISVS expectation. | IG:L132-L134 |
| IG-E053 | E/CORRECTION | I-19 does not rebuild TDD; it adds a symbol-level pre-tool source-edit gate with a coverage-delta refactor exemption and must bound false positives before default-on. | IG:L135-L137 |
| IG-E054 | E/CORRECTION | I-23 unifies shipped conformal calibration across refusal, claim scores, and I-4 priors; it introduces no Brier implementation or new math. | IG:L138-L140 |
| IG-E055 | E/CORRECTION | I-20's real gaps are diff-cone mutant selection, time-boxed sampled scoring with a confidence interval, and surviving-mutant regeneration directives because an equivalent-mutant cache already ships. | IG:L141-L143 |
| IG-E056 | E/CORRECTION | The old checkpoint-recovery “stub” claim is stale; remaining work is production reachability and resume-from-journal UX owned by H-35/H-51. | IG:L144-L145 |
| IG-E057 | E/STATUS | Trust-spine wiring is substantially executed; I sessions must not restate it, must put trust-reinforcing work before search/flywheel, and must bind I-31 to its gates. | IG:L146-L150 |
| IG-E058 | E/SEC | Narrative personas are removed from factual/code roles but retained for alignment-dependent hostile-critic/safety roles; I-32 mechanizes that split. The carried evidence is -3.6 to -5.3 percentage points on MMLU and -0.65 MT-Bench coding, not independently reverified here. | IG:L151-L153 |
| IG-E059 | E/SEC | I-34 must use CRANE-style free-thinking-between-commitments, constrain final output only in reflection loops, post-parse revalidate, and pin grammars as control-plane artifacts. Carried risks include 10–15-point reasoning loss, 38 XGrammar silent failures, and CDA attack success of 96.2% / 75.8–99.5%; none was independently reverified here. | IG:L154-L159 |
| IG-E060 | E/EVIDENCE | The 13 research files are themselves web-grounded but this session did not independently verify their identifiers or vendor numbers; every cited arXiv ID must be web-confirmed before heading a gate or metric. | IG:L161-L172 |
| IG-E061 | E/CFG | Dated cache economics: Anthropic's default TTL changed from one hour to five minutes on 2026-03-06, one-hour writes cost 2x and break even around three reads, while five-minute writes cost 1.25x and break even at one read. | IG:L173-L176 |
| IG-E062 | E/CFG | TPM is input plus max_tokens; max_tokens must be clamped to schema size, and 60 RPM is enforced sub-second at roughly one request per second. | IG:L174-L176 |
| IG-E063 | E/CFG | Opus 4.7+ rejects manual thinking budget_tokens and uses effort; batch lanes are 10–45 minutes with a 24-hour SLA. | IG:L176-L177 |
| IG-E064 | E/DET | temperature=0 is explicitly non-deterministic, with cited variance figures carried only as dated research landmines. | IG:L177-L178 |
| IG-E065 | E/EVIDENCE | SWE-bench Verified is treated as retired/contaminated, K-Prize as contamination-free but low-performing, and OPRO-style self-optimization as unsuitable below roughly 100B; these figures are carried, not independently verified. | IG:L177-L178 |
| IG-E066 | E/INT | Track 12 anchors I-1 to task-runner/hook/safety seams, I-2 to BSP/sandbox/worktrees, and I-3 to SSB pragmas/checkpoint writes. | IG:L180-L184 |
| IG-E067 | E/INT | Track 13 anchors I-4 to SessionDag/verifiers/catalog, I-5/I-6 to ContextBudget, I-7 to batches/routing, I-8 to calibration/refusal/abstention, and I-9 to mutation/drift comparison. | IG:L185-L187 |
| IG-E068 | E/INT | Track 14 anchors I-10 to cache substrates, I-11 to BeliefState/context/routing, I-12 to compiled pipelines/rules/GEPA, I-13 to safety/memory, I-14 to LSP/tree-sitter, I-15 to Oracle cache/A-RAG, and I-16 to the Python sidecar. | IG:L188-L191 |
| IG-E069 | E/INT | Track 15 anchors I-17 to verifier/gate/mutation seams, I-18 to RRA/SSB, I-19 to pre-tool hooks, I-20 to mutation/impact radius, I-21 to interactive gate/adversary, I-22 to PPM/undo-tree, I-23 to conformal calibration, and I-24 to relations/trap detection. | IG:L192-L195 |
| IG-E070 | E/INT | Track 16 anchors I-25 to escalation/cloud routing, I-26 to serial depth/cost, I-27 to metrics/benchmarks, I-28 to archive/Phase 4H, and I-29 to MiniCritic/research with media/CRDT absent. | IG:L196-L198 |
| IG-E071 | E/INT | Track 17 anchors I-30 to RRA/ledger/SFT and I-31 to corrected fitness/trap scoring, ledger, held-out evaluation, and SFT. | IG:L199-L200 |
| IG-E072 | E/INT | Track 18 anchors I-32 to agent configs, I-33 to rules/refusal, I-34 to decoding/grammar substrates, I-35 to temporal/per-call gates, I-36 to compiled-store/routing/optimizer, I-37 to intent/contract/cockpit/anchor, I-38 to quality/corpus probes, I-39 to repo/SSB/wiring/archive, and I-40 to RRA/provenance/adapters/bridge. | IG:L201-L205 |
| IG-E073 | E/EVIDENCE | The unverified citation list carried exact IDs for CaMeL 2503.18813, FIDES 2505.23643, MAST 2503.13657, scaling-agents 2512.08296, sycophancy-propagation 2604.02668, Compiled-AI 2604.05150, AgentSpec 2503.18666, Agent-C 2512.23738, Pro2Guard 2508.00500, CRANE 2502.09061, structure-snowballing 2604.06066, Let-Me-Speak-Freely 2408.02442, XGrammar 2411.15100, JSONSchemaBench 2501.10868, CFGzip 2605.04066, SELF-DISCOVER 2402.03620, and METR 2507.09089. | IG:L164-L172 |
| IG-E074 | E/EVIDENCE | The same unverified list carries LLMLingua-2, PABU/ACON at 26.9% steps, ACE +10.6%, DSPy/MIPROv2 +13%, StateFlow +13–28% and 3–5x, and persona-evaluation deltas. | IG:L167-L172 |
| IG-I001 | I/STATUS | Because the map repeatedly labels live values as dated and mandates re-grep, none of its implementation-status or numeric-baseline statements can safely be treated as current after 2026-06-10. | IG:L2-L7, L9-L18, L61-L69 |
| IG-A001 | A/EVIDENCE | The map lists external papers and vendor figures but explicitly leaves their identity, revision, and current accuracy unverified. | IG:L161-L178 |
| IG-A002 | A/STATUS | File-level token presence for streaming, batches, memory retrieval, and calibration does not resolve production reachability. | IG:L61-L64 |

Category coverage: capabilities/surfaces ✓ through substrate and build-status inventory; workflows/states ✓; schemas/data/lifecycle ✓; flags/defaults/config ✓; integrations/events ✓; security/privacy/compliance ✓ through provenance, dependency, and AISVS constraints (no standalone privacy policy); failures/fallback/recovery ✓; determinism/concurrency ✓ through hashing, recovery, and single-path corrections; observability/evidence ✓; tests/done gates ✓ through re-grep and wiring rules; constraints/non-goals ✓; dependencies/status/deferred ✓. Retry and idempotency APIs are not specified by this map.

### I-8 — aletheia voluntary abort

Source: docs/thefinalglance/I-plan/I-8_aletheia_voluntary_abort.md (132/132 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| I8-E001 | E/STATUS | I-8 is a strong-grounded Track-13 BUILD, effort M, Wave 1, CANT-LIE lever, bound to RD-1. | I-8:L1-L3 |
| I8-E002 | E/CAP | It adds acknowledge_infeasible / abort_trajectory to the production tool schema because the shipped abstention scorer lacks a structural invocation mechanism. | I-8:L5-L15 |
| I8-E003 | E/FLOW | The tool is registered through the existing three-arm catalog pattern and runs through normal production tool conversion/dispatch. | I-8:L13-L15, L28-L29, L58-L63, L68-L83 |
| I8-E004 | E/EVENT | An abort calls the existing H-39 scorer and emits AbstentionScored with abort_reason Voluntary, DeadEndHash, or HaltingBound; I-8 never reimplements reward/credit logic. | I-8:L13-L16, L49-L51, L72-L77, L85-L87 |
| I8-E005 | E/RECOVERY | Voluntary abort rolls the trajectory back to the last good H-49 trunk node and journals the restored node/sequence identity. | I-8:L16-L16, L37-L39, L62-L63, L100-L106 |
| I8-E006 | E/DATA | An aborted AST or approach shape is fingerprinted as a blake3 hash. | I-8:L16-L18, L44-L46 |
| I8-E007 | E/DATA | DeadEndCache is a bounded LRU in crates/orchestrator/src/dead_end_cache.rs with check_and_insert; v1 persistence is session-local. | I-8:L17-L18, L45-L46, L54-L55, L59-L59, L68-L70 |
| I8-E008 | E/FLOW | Every branch expansion checks the dead-end cache before a model call; an isomorphic match is a final prune costing zero model calls/tokens. | I-8:L18-L19, L24-L25, L45-L46, L88-L89 |
| I8-E009 | E/FAIL | A deterministic check running past the empirical 15-second default triggers automatic abort and a HaltingBound row through the I-17 runner seam. | I-8:L19-L21, L47-L48, L61-L63, L77-L83 |
| I8-E010 | E/CFG | The 15-second bound must be a named configurable key whose default is sourced from I-17's specification; it cannot be an unnamed hard-coded constant. | I-8:L19-L21, L90-L91 |
| I8-E011 | E/EVENT | I-8 registers TrajectoryAborted with kinds Voluntary, DeadEndHash, and HaltingBound in ssb.rs/action_type_lock.rs as a live-baseline-relative +1. | I-8:L40-L40, L49-L49, L68-L71, L119-L120 |
| I8-E012 | E/DEP | D-16/D-17 supply refusal/SLA substrate, H-39 supplies scoring, H-49 supplies rollback, IS-14 supplies tool registration, and I-17 supplies the watchdog seam. | I-8:L27-L39, L58-L63, L109-L130 |
| I8-E013 | E/STATE | Wave 1 ships the tool/cache before H-49 and I-17 are necessarily live; rollback and watchdog integrations become fully exercised when those later seams land. | I-8:L35-L39, L129-L130 |
| I8-E014 | E/DEFER | Durable cross-session dead-end persistence is deferred until at least three sessions show repeated isomorphic retries. | I-8:L53-L55 |
| I8-E015 | E/DEFER | The ReasoningDrawer abort/dead-end strip is deferred/wire-dark behind a GD-4 bound and is W-6-justified for v1; when shipped it must pass cross_lang_diff.py. | I-8:L53-L56, L64-L65, L94-L96 |
| I8-E016 | E/LIMIT | A learned infeasibility classifier is excluded because no ground-truth corpus exists; the structural tool is first. | I-8:L21-L22 |
| I8-E017 | E/LIMIT | A whole-task hard timeout is excluded because context budget already governs tasks; I-8 is trajectory-local. | I-8:L22-L23 |
| I8-E018 | E/LIMIT | Resurrecting or overriding an aborted/isomorphic path in the same session is explicitly declined. | I-8:L23-L25, L88-L89 |
| I8-E019 | E/DET | Dead-end hashing must use blake3, not blake2/sha2; production paths prohibit unwrap/expect and fabricated measurements. | I-8:L92-L93 |
| I8-E020 | E/DATA | The required decision artifact is workflow/reasoning/I-8-decisions.md. | I-8:L52-L52 |
| I8-E021 | E/TEST | The live acceptance trigger runs impossible_task and spin_task with a 15-second halting bound. | I-8:L97-L100 |
| I8-E022 | E/OBS | impossible_task must emit TrajectoryAborted(Voluntary), AbstentionScored(Voluntary), and a journal-verifiable trunk rollback. | I-8:L100-L102, L105-L106 |
| I8-E023 | E/OBS | spin_task must emit TrajectoryAborted(HaltingBound) within budget. | I-8:L99-L104 |
| I8-E024 | E/TEST | An isomorphic second attempt must emit DeadEndHash with journal token/model-call count exactly zero. | I-8:L102-L104 |
| I8-E025 | E/TEST | The broken no-cache arm must exceed three isomorphic iterations and fail the spiral detector. | I-8:L105-L107 |
| I8-E026 | E/INT | The production chains are model tool call -> catalog dispatch -> abort handler -> cache plus score, and check-runner timeout -> abort handler -> HaltingBound; test-only callers do not satisfy reachability. | I-8:L78-L84 |
| I8-E027 | E/LIMIT | D-16/D-17 refusal logic, H-39 scoring, H-49 rollback logic, and the E-35 catalog are composed and must not be duplicated. | I-8:L85-L87, L122-L130 |
| I8-A001 | A/DATA | The source alternates between AST shape, approach fingerprint, and approach hash without defining the canonicalization algorithm whose equality decides isomorphism. | I-8:L8-L9, L16-L18, L45-L46 |
| I8-A002 | A/CAP | acknowledge_infeasible and abort_trajectory are written as a slash pair, but the source does not resolve whether they are one catalog name, aliases, or tool-plus-handler names. | I-8:L7-L8, L13-L14, L44-L44 |
| I8-A003 | A/CFG | The requirement says the bound must have a named key but does not name that configuration key. | I-8:L90-L91 |
| I8-C001 | C/SURFACE | The orientation summary presents the ReasoningDrawer strip as an unqualified feature; the numbered source explicitly defers it behind a GD-4 bound and justifies no v1 frontend. | I-8:L53-L56, L64-L65, L94-L96; orientation:L275-L275 |

Category coverage: capability/action/surface ✓; workflow/state/lifecycle ✓; schema/data ✓; flag/default/config ✓; integrations/events ✓; security/compliance ✓ through hard rules; failure/fallback/recovery ✓; determinism/idempotency ✓ (final same-session prune; no explicit cross-process idempotency); concurrency ✓ only through pre-call branch pruning, with no locking policy; observability/evidence ✓; tests/done gates ✓; constraints/non-goals ✓; dependencies/status/deferred ✓. No privacy or retry policy beyond prohibited reattempt is specified.

### I-13 — memory guard and dual serialization

Source: docs/thefinalglance/I-plan/I-13_memory_guard_dual_serialization.md (96/96 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| I13-E001 | E/STATUS | I-13 is a strong-grounded Track-14 BUILD, effort M, Wave 2, SECURITY-EFFICIENCY lever, bound to IC-10. | I-13:L1-L3 |
| I13-E002 | E/CAP | It ships three orthogonal mechanisms together: deterministic memory-write guard, dual tool-result serialization, and TraceQuery expansion. | I-13:L4-L8 |
| I13-E003 | E/SEC | Every Recall, Archival, and playbook memory write is scanned at write time for injection markers, invisible-Unicode/multibyte tricks, and secret patterns. | I-13:L11-L19, L30-L36 |
| I13-E004 | E/SEC | Tier 1 uses regex plus Aho-Corasick first. | I-13:L13-L15, L31-L32 |
| I13-E005 | E/DEP | A YARA engine is a new dependency requiring package anti-slopsquat verification and explicit user approval; it may be added only after measurement shows the first-line scanner insufficient. | I-13:L14-L15, L28-L28, L38-L40, L66-L68 |
| I13-E006 | E/INT | The scanner composes H-31/H-32 memory objectivity, E-37 PromptArmor, MINJA, G-9 redaction precedent, H-6 masking, and safety Provenance; it must not re-filter their scopes. | I-13:L10-L20, L22-L28, L55-L60, L81-L94 |
| I13-E007 | E/FLOW | MemoryWriteScanner is inserted at mem0_dispatcher.rs/store.rs write choke points and registers as a write-side provenance hook. | I-13:L31-L33, L42-L46, L49-L64 |
| I13-E008 | E/FAIL | A detected poison write is refused, never silently swallowed, and produces no store entry. | I-13:L66-L71, L76-L79 |
| I13-E009 | E/EVENT | MemoryGuardEvent(PoisonBlocked) records the named pattern class. | I-13:L31-L35, L49-L54, L76-L79 |
| I13-E010 | E/DATA | ToolResult gains to_ui_dict for the full dashboard payload and to_llm_content for condensed model context. | I-13:L16-L18, L32-L34, L42-L45, L49-L54 |
| I13-E011 | E/FLOW | Every production tool result entering context must pass through to_llm_content; adapters are updated while the full UI form remains intact. | I-13:L32-L34, L43-L45, L61-L70 |
| I13-E012 | E/OBS | DualSerialized records the token delta between dashboard and context representations in the journal. | I-13:L32-L35, L47-L47, L76-L79 |
| I13-E013 | E/CAP | TraceQuery is a production tool that expands an exact on-disk line range on demand when condensed content is insufficient. | I-13:L16-L18, L33-L35, L42-L45 |
| I13-E014 | E/INT | TraceQuery is implemented in crates/tools/src/trace_query.rs and registered through build_tool_catalog's three-arm pattern. | I-13:L34-L35, L45-L45, L49-L64 |
| I13-E015 | E/EVENT | TraceQuery expansion emits MemoryGuardEvent(TraceQueryExpanded) with the requested byte/range evidence. | I-13:L34-L35, L54-L54, L76-L79 |
| I13-E016 | E/EVENT | MemoryGuardEvent has kinds PoisonBlocked, DualSerialized, and TraceQueryExpanded and is a live-baseline-relative +1 SSB addition. | I-13:L35-L35, L49-L54, L86-L86 |
| I13-E017 | E/CFG | The write guard cannot become default-on until a clean corpus across at least three real runs establishes a GD-4 false-positive bound. | I-13:L19-L19, L35-L36, L66-L71, L81-L86 |
| I13-E018 | E/DEFER | Automated secret-pattern updates from a versioned threat feed are deferred for governance. | I-13:L38-L40 |
| I13-E019 | E/SURFACE | MemoryFactsDrawer must show guard pattern class and serialization token delta; it is a required user-visible security/efficiency surface. | I-13:L47-L47, L74-L75 |
| I13-E020 | E/FLOW | Required production reachability is memory write -> scan, ToolResult assembly -> to_llm_content, and model tool call -> TraceQuery.expand; test-only callers do not qualify. | I-13:L61-L65 |
| I13-E021 | E/TEST | A seeded invisible-Unicode fixture must emit PoisonBlocked and leave no memory-store entry. | I-13:L76-L79 |
| I13-E022 | E/TEST | A 500-file listing must remain complete in the dashboard while the model receives a condensed form and the journal records the token delta. | I-13:L76-L79 |
| I13-E023 | E/TEST | A TraceQuery call must emit TraceQueryExpanded for the exact requested range. | I-13:L76-L79 |
| I13-E024 | E/TEST | A --no-guard negative arm accepts the malicious write and must fail the fixture check. | I-13:L79-L79 |
| I13-E025 | E/DET | Production scan/serialization paths prohibit unwrap/expect; hash use is blake3-only. | I-13:L66-L73 |
| I13-A001 | A/CFG | The source requires a quantified false-positive bound but provides neither threshold nor exact clean corpus beyond “at least three real runs.” | I-13:L19-L19, L35-L36, L71-L71 |
| I13-A002 | A/DATA | TraceQuery is described in line ranges but the event example uses byte_range; encoding, authorization, and path-boundary semantics are not specified. | I-13:L17-L18, L34-L35, L78-L78 |
| I13-A003 | A/SEC | “Secret patterns” and “injection markers” are required classes, but the source supplies no concrete pattern catalog/versioning scheme for Tier 1. | I-13:L13-L15, L31-L32 |

Category coverage: capabilities/actions/surfaces ✓; workflows/states/lifecycle ✓; schema/data ✓; flags/defaults/config ✓; integrations/events ✓; security/privacy/compliance ✓ (secret protection; no standalone retention policy); failures/fallback/recovery ✓ through refusal/TraceQuery expansion; determinism ✓; idempotency/concurrency/retry are not specified; observability/evidence ✓; tests/done gates ✓; constraints/non-goals ✓; dependencies/status/deferred ✓.

### I-15 — semantic cache and blake3 fast path

Source: docs/thefinalglance/I-plan/I-15_semantic_cache_blake3_fastpath.md (91/91 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| I15-E001 | E/STATUS | I-15 is a strong-grounded Track-14 EXTENSION, effort S, Wave 1, CHEAPER-SPEED lever. | I-15:L1-L3 |
| I15-E002 | E/CAP | It extends the existing C-30/H-8 OracleAnswerCache to cache intermediate DKG chunk summaries and research extracts, not only final answers. | I-15:L4-L13, L24-L29 |
| I15-E003 | E/FLOW | Semantically similar follow-up queries may reuse intermediate summaries and skip re-summarization. | I-15:L11-L15 |
| I15-E004 | E/PERF | An O(1) lookup keyed by blake3(canonical_query_bytes) must execute before vector cosine work or any ONNX embedding call. | I-15:L7-L8, L14-L15, L31-L36, L49-L60 |
| I15-E005 | E/FALLBACK | An exact-key miss proceeds to the normal semantic-vector/reranker path. | I-15:L14-L15, L46-L47 |
| I15-E006 | E/SEC | Durable cache confinement is limited to idempotent grounding content; code-generation output, context-dependent tool results, and session-specific reasoning chains must never enter it. | I-15:L16-L18, L62-L65 |
| I15-E007 | E/DEP | The existing H-8 store, TTL logic, and confinement test are inherited and extended, never reimplemented or replaced. | I-15:L10-L11, L24-L29, L43-L46, L54-L57, L77-L89 |
| I15-E008 | E/CFG | Intermediate summaries default to H-8's volatile five-minute tier, but I-10 CacheEconomicsPlanner may override per call class; I-15 must read the plan rather than hard-code TTL. | I-15:L31-L35, L54-L57, L84-L88 |
| I15-E009 | E/EVENT | Preferred observability is reuse of OracleCacheHit/Miss with a semantic_cache discriminator and sources blake3_exact or semantic_vector. | I-15:L19-L22, L31-L36, L49-L53 |
| I15-E010 | E/EVENT | Only if the existing payload cannot carry that discriminator may SemanticCacheEvent be added; the reuse-versus-new-event decision is deferred to edit time and must be recorded. | I-15:L19-L22, L49-L53, L77-L82 |
| I15-E011 | E/INT | The production insertion point is knowledge/src/arag.rs DKG/research chunk summarization; the exact-hit path bypasses the Phase-1G reranker/ONNX call. | I-15:L24-L27, L31-L37, L43-L47, L58-L61 |
| I15-E012 | E/SURFACE | Research answers show a required cache-source badge with blake3_exact, semantic_vector, or miss. | I-15:L37-L37, L43-L47, L70-L71 |
| I15-E013 | E/DEFER | Cross-session prewarming is deferred until H-51 cross-session topology is live. | I-15:L39-L40 |
| I15-E014 | E/DEFER | Similarity-threshold auto-tuning is deferred until ELAI measures its own research-corpus precision/recall baseline. | I-15:L40-L41 |
| I15-E015 | E/LIMIT | The research dossier's 50–60% token-reduction figure remains design-only; I-15 measures its own two-run input-token delta and cannot advertise a hard-coded similarity target. | I-15:L39-L41, L62-L65, L72-L75, L77-L82 |
| I15-E016 | E/TEST | Running the same research query twice must make the second run an exact blake3 hit with trace-verified ONNX call count zero. | I-15:L72-L75 |
| I15-E017 | E/OBS | The first/second runs journal cache source, latency/call count, and measured RAG input-token delta. | I-15:L72-L75 |
| I15-E018 | E/TEST | A code-generation query must create no durable entry; the inherited H-8 confinement test and a broken caching arm must enforce this. | I-15:L63-L65, L73-L75 |
| I15-E019 | E/DET | Hashing uses blake3, not blake2/sha2, and production lookup/insert paths prohibit unwrap/expect. | I-15:L67-L69 |
| I15-E020 | E/INT | Exact production chains are A-RAG summarize -> exact lookup and DKG summarize -> intermediate insert; tests alone do not satisfy R7. | I-15:L58-L61 |
| I15-A001 | A/EVENT | The final SSB schema is intentionally unresolved between extending OracleCacheHit/Miss and adding SemanticCacheEvent. | I-15:L19-L22, L49-L53, L82-L82 |
| I15-A002 | A/DATA | canonical_query_bytes is the exact-key input, but canonicalization rules are not defined in the source. | I-15:L14-L15 |
| I15-A003 | A/CFG | The semantic-vector similarity threshold and its eventual tuning method remain deliberately unspecified pending local measurement. | I-15:L40-L41, L65-L65 |
| I15-C001 | C/EVENT | The orientation summary says I-15 reuses OracleCacheHit/Miss; the numbered source makes reuse preferred but conditionally permits a new SemanticCacheEvent if payload compatibility fails. | I-15:L19-L22, L49-L53, L82-L82; orientation:L449-L449 |

Category coverage: capabilities/actions/surfaces ✓; workflows/states ✓; schema/data/lifecycle ✓; flags/defaults/config ✓; integrations/events ✓; security/privacy ✓ through cache confinement; failures/fallback ✓; determinism/idempotency ✓ through exact-key reuse; concurrency/retry/recovery are not specified; observability/evidence ✓; tests/done gates ✓; constraints/non-goals ✓; dependencies/status/deferred ✓.

### I-18 — UPIP chain, Sigstore, and AI-BOM

Source: docs/thefinalglance/I-plan/I-18_upip_chain_sigstore_aibom.md (107/107 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| I18-E001 | E/STATUS | I-18 is a strong-grounded Track-15 EXTENSION, effort M, Wave 3, AUDITABLE lever, bound to IC-11; full-form expansion is required before execution. | I-18:L1-L3, L105-L105 |
| I18-E002 | E/STATE | G-2 RRA build/verify and G-8 attestation already ship; the absent capability is sequence-wide row tamper evidence. | I-18:L4-L9, L28-L34 |
| I18-E003 | E/DATA | Every SSB insert adds chain_prev_hash equal to blake3(prior_row_bytes). | I-18:L11-L14, L36-L39, L49-L55 |
| I18-E004 | E/CONC | The SSB single writer owns the rolling hash, and no async/batch path may reorder inserts or bypass previous-hash order. | I-18:L12-L14, L24-L26, L50-L51, L74-L79 |
| I18-E005 | E/SCHEMA | chain_prev_hash extends the append-only/golden-locked row; the executor must coordinate all six lockstep bump sites. | I-18:L12-L14, L29-L31, L49-L51, L89-L93 |
| I18-E006 | E/DATA | RRA StackHash has five blake3 layers: STATE=SSB snapshot, DEPS=Cargo.lock, PROCESS=execution environment, RESULT=output artifact, VERIFY=verifier-stack outcome. | I-18:L15-L18, L36-L40, L59-L64 |
| I18-E007 | E/FAIL | A Cargo.lock mutation during a run invalidates DEPS immediately and hard-aborts with RowChainSealed(ChainBroken). | I-18:L17-L18, L39-L40, L84-L87 |
| I18-E008 | E/ARTIFACT | build_rra seals the five-layer hash into the existing RRA and the verifier gains a chain-verification leg. | I-18:L15-L18, L39-L39, L49-L54, L59-L68 |
| I18-E009 | E/ARTIFACT | Every sealed attestation gains an adjacent AI-BOM/CycloneDX-ML component/file expressing the cited OWASP AISVS AC.8 expectation. | I-18:L22-L23, L36-L43, L52-L55, L89-L96 |
| I18-E010 | E/TEST | The AI-BOM must validate against the published CycloneDX schema whose URL/version is web-confirmed at execution. | I-18:L22-L23, L84-L87, L93-L95 |
| I18-E011 | E/CFG | Offline blake3-Merkle verification is the complete default; external Rekor transparency is opt-in. | I-18:L19-L21, L44-L47 |
| I18-E012 | E/DEP | Rekor uses cosign keyless via a binary or cosign-rs, but exact dependency identity/version/repository/license must pass anti-slopsquat checks and user approval before landing. | I-18:L19-L21, L34-L34, L44-L47, L89-L95 |
| I18-E013 | E/DEFER | The --rekor interface/checkbox may be scaffolded now, but the actual push is a separately gated Tier-2 capability. | I-18:L44-L47 |
| I18-E014 | E/FAIL | If --rekor is passed and the push fails, the run halts with a clear error; silent SKIP is forbidden. | I-18:L74-L79 |
| I18-E015 | E/CLI | elai verify-attestation --chain walks the chain and reports the first broken link and exact sequence number. | I-18:L41-L43, L49-L55, L70-L73 |
| I18-E016 | E/EVENT | RowChainSealed kinds are ChainAdvanced, RekorPushed, AibomEmitted, and ChainBroken, registered as a live-baseline-relative +1 action type. | I-18:L41-L43, L54-L55, L59-L64, L97-L97 |
| I18-E017 | E/IDEM | Once ChainAdvanced seals a sequence, the row bytes are immutable; editing a sealed row is the primary banned design. | I-18:L24-L26, L74-L76 |
| I18-E018 | E/SEC | Chain, stack, and AI-BOM integrity fields are blake3-only; sha256/blake2 are prohibited. | I-18:L24-L26, L74-L77 |
| I18-E019 | E/INT | Changes target ssb.rs single-writer insert, rra.rs build/verify/CLI chain leg, attestation.rs AI-BOM emission, and the CLI verify subcommand; G-2/G-8/F-4 are composed, not redefined. | I-18:L49-L55, L59-L73, L99-L103 |
| I18-E020 | E/SURFACE | The existing RRA viewer gains a chain badge, not a new drawer; it subscribes to RowChainSealed and shows CHAIN OK or BROKEN plus break seq. | I-18:L56-L57, L80-L83 |
| I18-E021 | E/TEST | cross_lang_diff.py must pass for the RRA-viewer row/badge extension. | I-18:L56-L57, L80-L83 |
| I18-E022 | E/TEST | A normal emit-RRA run must produce ChainAdvanced rows for every insert, AibomEmitted on seal, and verify-attestation --chain exit 0. | I-18:L84-L87 |
| I18-E023 | E/TEST | Editing database row N must make chain verification fail at N and name the exact break. | I-18:L84-L87 |
| I18-E024 | E/TEST | The opt-in Rekor arm must yield an inclusion proof verifiable without rerunning ELAI. | I-18:L84-L87 |
| I18-E025 | E/LIMIT | Existing RRA format/build logic, attestation write path, SSB writer, and F-4 stdout_hash are composition boundaries and may not be forked/reimplemented. | I-18:L65-L69, L99-L103 |
| I18-E026 | E/DET | Production paths prohibit unwrap/expect and fabricated chain-verification results. | I-18:L78-L79 |
| I18-A001 | A/DEP | The source leaves the Rekor implementation choice open between a cosign binary and cosign-rs. | I-18:L19-L21 |
| I18-A002 | A/SCHEMA | It calls chain_prev_hash “an extend, not a schema change” while also requiring golden-lock/schema coordination; the exact storage migration semantics are not resolved. | I-18:L12-L14, L29-L31, L49-L55 |
| I18-A003 | A/COMP | The CycloneDX-ML version/schema URL and exact Rekor endpoint are deliberately deferred to execution-time web verification. | I-18:L22-L23, L34-L34, L89-L95 |
| I18-C001 | C/DEFER | The orientation summary presents Rekor push and chain/Rekor CLI options together; the numbered source explicitly makes the push Tier 2 and only scaffolds --rekor now. | I-18:L44-L47; orientation:L529-L535 |
| I18-C002 | C/DEP | The orientation adds coordinated sealed-record layout with I-45, but this numbered source names G-2, G-8, SSB, and F-4 composition and contains no I-45 dependency. The summary-only coordination claim is not imported. | I-18:L65-L69, L99-L103; orientation:L542-L543 |

Category coverage: capability/action/surface ✓; workflows/states/lifecycle ✓; schema/data ✓; flags/default/config ✓; integrations/events ✓; permissions/security/compliance ✓ through approval, anti-slopsquat, OWASP/CycloneDX; failure/fallback/recovery ✓; determinism/idempotency/concurrency ✓; observability/evidence ✓; tests/acceptance ✓; constraints/non-goals ✓; dependencies/status/deferred ✓. No automatic retry policy exists; Rekor failure halts.

### I-23 — unified conformal calibration

Source: docs/thefinalglance/I-plan/I-23_unified_conformal_calibration.md (109/109 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| I23-E001 | E/STATUS | I-23 is a strong-grounded Track-15 EXTENSION/UNIFICATION, effort S–M, Wave 1, SMARTER-ORCH lever, bound to IC-13. | I-23:L1-L3 |
| I23-E002 | E/STATE | conformal_calibration.rs and calibration_bootstrap.rs with warm_start_tau already ship; the task is structural unification, not Brier or new mathematics. | I-23:L4-L9, L29-L41, L94-L101 |
| I23-E003 | E/CAP | A thin CalibrationFacade exposes typed get_tau(CalibrationRequest)->TauResult, update_tau(ObservedOutcome), and warm_start(). | I-23:L11-L15, L43-L48, L64-L68 |
| I23-E004 | E/INT | After migration the facade is the only import path; no consumer may import conformal_calibration.rs directly. | I-23:L12-L15, L22-L24, L64-L84 |
| I23-E005 | E/TEST | A structural contract greps direct conformal-calibration imports and requires a zero result after migration. | I-23:L22-L24, L43-L48, L79-L84 |
| I23-E006 | E/INT | D-16 refusal migrates only its import/call site; refusal logic and all existing observable effects remain unchanged. | I-23:L16-L18, L35-L38, L43-L47, L54-L62, L89-L92 |
| I23-E007 | E/DEP | G-7 claim_score is a planned consumer, but its migration is deferred to and owned by G-7; I-23 does not block on it. | I-23:L18-L18, L38-L40, L50-L52, L69-L74, L94-L100 |
| I23-E008 | E/INT | I-4 PUCT consumes facade uncertainty; when TauResult confidence-interval width exceeds a locally measured threshold it inserts a divergent exploration branch rather than repeating the current hypothesis. | I-23:L19-L28, L40-L40 |
| I23-E009 | E/CFG | The uncertainty threshold is measured on ELAI fixtures, persisted in metrics/calibration_fork_threshold.toml, and cannot be a paper constant. | I-23:L19-L21, L25-L28, L31-L33, L43-L48, L64-L68, L79-L84 |
| I23-E010 | E/STATE | The facade can ship in Wave 1; I-4 fork logic is scaffolded as a conditional and becomes live only when Wave-4 PUCT exists. | I-23:L40-L40, L54-L59 |
| I23-E011 | E/EVENT | I-23 adds no SSB type and reuses RefusalDecision and CalibrationUpdate, carrying a shared calibration_id/tau identity. | I-23:L43-L49, L54-L60, L79-L84, L89-L101 |
| I23-E012 | E/OBS | A single run triggering refusal and a search fork must join both reused rows on the same calibration_id, proving one tau source. | I-23:L89-L92 |
| I23-E013 | E/TEST | A split-calibrations negative arm with independent refusal/search tau values must fail i23_unified_calibration_contract.rs. | I-23:L89-L92 |
| I23-E014 | E/TEST | The D-16 suite is rerun verbatim as a regression guard. | I-23:L92-L92 |
| I23-E015 | E/SURFACE | No new dashboard component is added; existing D-16 calibration/refusal tiles remain and I-4's existing branch board surfaces forks. | I-23:L60-L62, L85-L88 |
| I23-E016 | E/DEFER | Multi-model tau divergence detection for cloud/local model families is deferred; a single tau ships first. | I-23:L50-L53 |
| I23-E017 | E/LIMIT | The facade may not inline I-4 fork logic, absorb G-7 logic, change refusal logic, or reimplement H-39 scoring/calibration math. | I-23:L103-L107 |
| I23-E018 | E/DET | Production paths prohibit unwrap/expect and fabricated tau/CI values. | I-23:L79-L84 |
| I23-E019 | E/FLOW | Production reachability is the existing D-16 refusal path via the facade plus the I-4 PUCT expansion caller; contract-only callers do not satisfy R7. | I-23:L75-L78 |
| I23-A001 | A/CFG | The confidence-interval width metric, fixture corpus, and threshold-selection procedure are not specified beyond local measurement and a TOML artifact. | I-23:L19-L21, L25-L28, L48-L48 |
| I23-A002 | A/INT | The exact I-4 file/seam is PLAN-asserted as ladder/PUCT and must be re-grepped once I-4 exists. | I-23:L54-L59 |
| I23-C001 | C/STATUS | The orientation says I-23 “uses” the facade for G-7; the numbered source says G-7 is planned and explicitly defers that call-site migration to G-7. | I-23:L18-L18, L38-L40, L50-L52, L74-L74; orientation:L654-L654 |

Category coverage: capabilities/actions/surfaces ✓; workflows/states ✓; schema/data/lifecycle ✓; flags/default/config ✓; integrations/events ✓; security/compliance ✓ through frozen import/measurement controls; failure/fallback ✓ through split-arm and unchanged refusal behavior; determinism ✓; idempotency/concurrency/retry/recovery are not specified; observability/evidence ✓; tests/done gates ✓; constraints/non-goals ✓; dependencies/status/deferred ✓; no privacy feature is specified.

### I-28 — DAG credit assignment

Source: docs/thefinalglance/I-plan/I-28_dag_credit_assignment.md (126/126 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| I28-E001 | E/STATUS | I-28 is a strong-grounded Track-16 BUILD, effort M–L, Wave 5, SELF-MOD lever, blocked on I-4; its refreshed verdict is ratified BUILD-WITH-SEAM and requires full-form expansion. | I-28:L1-L6, L17-L18, L69-L74 |
| I28-E002 | E/STATE | I-4 SearchLadderOutcome supplies a winner and ranked siblings but v1 lacks per-node exclusion and per-branch chunk provenance. | I-28:L4-L4, L8-L11, L23-L25 |
| I28-E003 | E/STATE | The live task_runner drive_task calls dispatch_search_ladder before maybe_run_escalation_waterfall, but currently discards SearchLadderOutcome. | I-28:L8-L11 |
| I28-E004 | E/FLOW | dispatch_search_ladder must become an Option<SearchLadderOutcome> producer and pass the value directly to dispatch_dag_credit; credit must not reconstruct it by rereading SSB. | I-28:L10-L11, L27-L30 |
| I28-E005 | E/FLOW | dispatch_dag_credit is inserted immediately after dispatch_search_ladder and before maybe_run_escalation_waterfall. | I-28:L19-L21, L27-L29, L47-L50 |
| I28-E006 | E/CAP | Tier-1 Slice 1 adds DagCreditAssigner and deterministic branch-grain credit. | I-28:L17-L21, L34-L38 |
| I28-E007 | E/DET | Slice-1 credit is winner score minus sibling score, clamped to the contract range; no LLM-authored credit score is permitted. | I-28:L52-L54, L110-L111 |
| I28-E008 | E/EVENT | Slice 1 emits DagCreditAssigned rows with Kind=CreditWalk, one per ranked sibling. | I-28:L19-L21, L34-L38, L64-L67 |
| I28-E009 | E/DATA | Each CreditWalk row carries winner score, sibling score, ranked position, and deterministic credit delta. | I-28:L64-L66 |
| I28-E010 | E/CFG | The production path is default-off behind ELAI_DAG_CREDIT_ENABLE and also requires the search-ladder flag. | I-28:L19-L21, L32-L32, L47-L50 |
| I28-E011 | E/OBS | With ELAI_SEARCH_LADDER_ENABLE=1 and ELAI_DAG_CREDIT_ENABLE=1 credit rows appear; with the credit flag unset there must be zero rows. | I-28:L47-L50, L76-L84, L110-L117 |
| I28-E012 | E/DEP | New public Rust symbols require a real production caller or a wire-dark allowlist row; Slice 1 adds one with promotion_plan_id=I-28-promotion. | I-28:L15-L15, L19-L21, L96-L101, L103-L108 |
| I28-E013 | E/TEST | Slice 1 adds crates/contracts/tests/i28_dag_credit_contract.rs covering CreditWalk, schema floor, sql_str round-trip, flag-on, and flag-unset paths. | I-28:L19-L21, L76-L84, L103-L108, L117-L117 |
| I28-E014 | E/SCHEMA | DagCreditAssigned is appended after ScopeCeilingAdvisory using the existing flat action-type plus payload-kind pattern. | I-28:L27-L31 |
| I28-E015 | E/SCHEMA | At refresh time the registry had 606 entries and SCHEMA_VERSION_DDL_EPOCH=-309, deriving schema 297; implementation must recount and derive rather than hard-code a future value. | I-28:L30-L32, L69-L73 |
| I28-E016 | E/STATE | I-17 verdicts and frozen VerdictRow are read-only; a HardVeto branch cannot be assigned or invented an L2 score, and VETO_SCORE/ID-4 may not change. | I-28:L12-L12, L40-L44, L52-L59, L84-L86, L96-L101 |
| I28-E017 | E/CAP | Tier-1 Slice 2, only after FORK-1, extends SearchLadderOutcome with pruned verdict data and extracts DPO pairs. | I-28:L19-L22, L34-L38, L64-L67 |
| I28-E018 | E/EVENT | Slice 2 emits DpoPairExtracted with verifiable provenance from a sealed winner and selected pruned sibling; an unsealed RRA yields zero pairs. | I-28:L21-L22, L52-L56, L64-L67, L76-L86, L110-L113 |
| I28-E019 | E/DEP | A DPO JSONL writer must ship with I-30's first reader or remain explicitly wire-dark with promotion_plan_id=I-30. | I-28:L14-L15, L21-L22, L40-L45, L90-L94 |
| I28-E020 | E/DEFER | Slice 3 chunk demotion is prohibited until each branch has chunk_id or equivalent provenance and the owning layer is decided. | I-28:L23-L25, L52-L57, L64-L67, L82-L88, L94-L101 |
| I28-E021 | E/DEFER | Node-grain credit waits for v3 PUCT plus provenance capture. | I-28:L23-L25, L90-L93 |
| I28-E022 | E/LIMIT | Shapley/SHARP attribution remains excluded and requires a separate RL-infrastructure decision. | I-28:L23-L25, L52-L57, L96-L99 |
| I28-E023 | E/SURFACE | No dashboard work ships in this unit; SSB rows are the proof, and a credit-heat view is deferred until trajectory volume justifies it. | I-28:L61-L63 |
| I28-E024 | E/DEP | IS-4 archive and H-46 clade scoring are downstream context only and must not be reimplemented. | I-28:L13-L13, L40-L45 |
| I28-E025 | E/FAIL | If both flags are enabled and no CreditWalk appears, dispatcher wiring is fixed and the test repeated; rows with the credit flag unset abort as an environment-gate leak. | I-28:L84-L84 |
| I28-E026 | E/FAIL | If a DPO extractor accepts unsealed RRA or an L2-scored hard-veto branch, extraction stops while frozen verdict contracts are preserved. | I-28:L85-L86 |
| I28-E027 | E/FAIL | Missing branch chunk provenance keeps Slice 3 deferred; implementation may not synthesize a chunk identifier. | I-28:L87-L88, L110-L115 |
| I28-E028 | E/DET | Production paths prohibit unwrap/expect, fabricated measurements, stale SSB baselines, and synthetic/neural attribution. | I-28:L52-L59 |
| I28-E029 | E/TEST | Off-agent verification runs plan_gate_check, the I-28 contracts, production_caller_check, wiring_contract_check, action_type_lock, schema derivation, allowlist promotion ID, and default-off confirmation. | I-28:L103-L108 |
| I28-E030 | E/SEC | Red-team arms cover fabricated credit, unsealed RRA, veto score, environment-gate leakage, and provenance-free demotion. | I-28:L110-L115 |
| I28-E031 | E/TEST | Integration smoke repeats the seeded ladder with both flags on and then credit off, checking rows, schema floor, SQL round-trip, and production task-runner reachability. | I-28:L117-L117 |
| I28-E032 | E/STATUS | Recon is closed, but the executor must re-grep task runner, ladder, verifier, registry, and allowlist immediately before implementation. | I-28:L119-L119 |
| I28-E033 | E/LIMIT | This source is a plan-only refresh unit: it must not edit product Rust, SSB, task runner, frozen gates, or evaluator files, and the worker cannot self-certify DONE. | I-28:L121-L124 |
| I28-A001 | A/DATA | FORK-1 leaves the DPO-negative key unresolved among preserved progress_score, progress_score plus gate id, or first veto by generation order; L2 score is disallowed in all branches. | I-28:L90-L92 |
| I28-A002 | A/ARTIFACT | FORK-3 leaves JSONL writer timing unresolved between paired I-30 reader and wire-dark emission. | I-28:L93-L93 |
| I28-A003 | A/DEP | FORK-4 leaves chunk-provenance ownership unresolved between pristine-context construction and retrieval. | I-28:L94-L94 |
| I28-A004 | A/SCHEMA | The source names DagCreditAssigned as the append target and DpoPairExtracted as a row but does not say whether the latter is a distinct action type or payload kind under an existing type. | I-28:L30-L30, L34-L38 |
| I28-C001 | C/CAP | The orientation summary claims backward DAG/node credit to chunks, subplans, and templates. The refreshed source explicitly limits Slice 1 to branch-grain sibling deltas and defers node-grain/chunk provenance. | I-28:L4-L4, L17-L25, L52-L57, L90-L94; orientation:L757-L760 |
| I28-C002 | C/DATA | The summary chooses the highest-Layer-2-scoring vetoed sibling; the refreshed source freezes HardVeto semantics, forbids any L2 score for a vetoed branch, and leaves a non-L2 FORK-1 choice open. | I-28:L12-L12, L52-L57, L90-L92, L96-L98; orientation:L763-L763 |
| I28-C003 | C/FLOW | The summary says credit runs at PhaseComplete; the refreshed production seam is drive_task immediately after search-ladder dispatch and before escalation. | I-28:L27-L30, L47-L50; orientation:L766-L766 |
| I28-C004 | C/DEFER | The summary specifies demotion after three abort-preceding appearances and a changed next ranking; the refreshed source emits no demotion row until per-branch chunk provenance and ownership exist. | I-28:L23-L25, L64-L67, L82-L88, L94-L101; orientation:L764-L772 |
| I28-C005 | C/SURFACE | The summary allows an optional plan-view heat map; the refreshed execution unit explicitly does no dashboard work and defers heat until trajectory volume justifies it. | I-28:L61-L63; orientation:L767-L767 |
| I28-C006 | C/DATA | I-GROUNDING-MAP's dated 221/532 baseline is superseded within this source by the refreshed 606-entry/derived-297 registry observation; both demand reread at implementation. | I-28:L30-L32, L69-L73; IG:L9-L18 |

Category coverage: capabilities/actions/surfaces ✓; workflows/states/lifecycle ✓; schema/data ✓; flags/defaults/config ✓; integrations/events ✓; permissions/security/compliance ✓ through frozen verdicts, sealed provenance, and allowlist controls; failures/fallback/recovery ✓; determinism/idempotency/concurrency ✓ (deterministic score/default-off sequencing; no parallel-write design); observability/evidence ✓; tests/acceptance/done gates ✓; constraints/non-goals ✓; dependencies/status/deferred ✓; no privacy or retry API is specified.

### I-36 — cross-model prompt compilation

Source: docs/thefinalglance/I-plan/I-36_cross_model_prompt_compilation.md (115/115 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| I36-E001 | E/STATUS | I-36 is a strong-grounded Track-18 BUILD, effort M, Wave 4, CHEAPER-PORTABLE lever, bound to ID-3 and ID-6. | I-36:L1-L3 |
| I36-E002 | E/CAP | It creates a versioned compiled-prompt artifact store keyed by role and backend so prompts can be promoted, rolled back, and replayed as configs. | I-36:L5-L12 |
| I36-E003 | E/DATA | CompiledPromptVariant is a plain-string artifact, allowing existing call sites to substitute text without refactor. | I-36:L12-L17, L43-L46, L67-L72 |
| I36-E004 | E/FLOW | Offline compilation searches instructions and exemplars for each role/backend against the C-28 fixture suite. | I-36:L14-L16 |
| I36-E005 | E/DEP | Only a cloud-lane MIPROv2-class optimizer may compile variants; local-lane self-optimization is prohibited because the cited OPRO pattern fails below roughly 100B. | I-36:L14-L17, L34-L41, L84-L88 |
| I36-E006 | E/INT | Routing applies I-32 role_contracts first, then looks up a promoted variant by B-37 PerRoleRuntime role/backend key. | I-36:L43-L46, L58-L64, L73-L82, L108-L113 |
| I36-E007 | E/INT | CompiledPipelineStore/TemplateCrystallizer are storage precedents, E-8 GEPA is the optimizer lane, and I-12/IS-16 supply the promotion gates; none is reimplemented. | I-36:L34-L41, L58-L64, L73-L78, L96-L113 |
| I36-E008 | E/OBS | Transfer-gap measurement compares source prompt versus compiled prompt on the same target backend and records the fixture pass-rate delta. | I-36:L19-L23, L43-L48, L91-L94 |
| I36-E009 | E/DET | TransferGapMeasured is recomputable from the same ELAI_SEED. | I-36:L19-L23, L91-L94 |
| I36-E010 | E/RECOVERY | A compiled variant below its fixture floor automatically rolls back and emits Rolledback. | I-36:L19-L23, L43-L49, L79-L82, L91-L94 |
| I36-E011 | E/SEC | Compiler constraints remove explicit step-by-step/CoT scaffolds when extended thinking is enabled. | I-36:L25-L28, L43-L50, L84-L86 |
| I36-E012 | E/CFG | Compiled prompts respect each backend's temperature defaults and do not treat temperature zero as determinism. | I-36:L25-L28 |
| I36-E013 | E/COMP | A compiled variant promotes only through H-17 persisted-N, H-18 deflation, D-6 held-out, and D-7 canary gates, with the evaluator outside the optimizer write boundary. | I-36:L30-L32, L43-L52, L84-L88 |
| I36-E014 | E/EVENT | PromptCompiled has kinds Compiled, TransferGapMeasured, and Rolledback and is registered as a live-baseline-relative +1 event. | I-36:L43-L49, L67-L72, L91-L106 |
| I36-E015 | E/OBS | Compiled rows name variant hash and backend; transfer and rollback receipts establish the exact served/promoted artifact. | I-36:L67-L72, L91-L94, L96-L106 |
| I36-E016 | E/SURFACE | SettingsWindow shows a per-role compiled-variant Svelte badge via a Tauri read and must pass cross_lang_diff.py. | I-36:L43-L50, L58-L65, L89-L94 |
| I36-E017 | E/DEFER | Automated per-backend optimizer runs are Tier 2 because they need cloud budget and sit behind the ELAI_PROMPT_COMPILE_DISABLE kill switch. | I-36:L54-L56 |
| I36-E018 | E/DEFER | Cross-family transfer-gap curve analysis across local/cloud model families waits for post-Wave-4 data. | I-36:L54-L57 |
| I36-E019 | E/DATA | Tier 1 produces prompt_artifacts.rs with CompiledPromptVariant, CompiledPromptStore, promote_variant, and rollback_on_regression, plus workflow/reasoning/I-36-decisions.md. | I-36:L43-L52, L58-L60, L67-L72 |
| I36-E020 | E/FLOW | Production chains are prompt routing -> lookup -> Compiled, fixture runner -> TransferGapMeasured -> ledger, and regression detection -> rollback -> Rolledback; test-only callers do not satisfy R7. | I-36:L79-L83 |
| I36-E021 | E/TEST | A two-backend/seeded run with a promoted variant must emit all applicable PromptCompiled receipts under ELAI_SEED. | I-36:L91-L94 |
| I36-E022 | E/TEST | A cloud extended-thinking request must contain zero explicit CoT/step-by-step scaffold, verifiable from the compiled string. | I-36:L84-L86, L91-L94 |
| I36-E023 | E/DET | Production paths prohibit unwrap/expect and require blake3 rather than blake2/sha2. | I-36:L84-L88 |
| I36-E024 | E/LIMIT | Variant promotion without ID-6 gates is P0, and optimization logic, backend adapters, crystallizer pipeline, and ledger implementation remain composition boundaries. | I-36:L84-L88, L108-L113 |
| I36-E025 | E/EVIDENCE | SELF-DISCOVER, MIPROv2/DSPy, and Compiled-AI citations must be web-confirmed before headline use. | I-36:L96-L106 |
| I36-E026 | E/STATE | B-37 routes across 11 backends, which is the stated portability surface motivating per-backend compilation. | I-36:L5-L9 |
| I36-A001 | A/DEFER | Tier 1 requires compiled-store substitution and transfer measurement while automated compilation is Tier 2; the source does not specify how initial Tier-1 compiled variants are produced or admitted. | I-36:L43-L56 |
| I36-A002 | A/CFG | ELAI_PROMPT_COMPILE_DISABLE is named as a Tier-2 kill switch, but its default value and relationship to manually served Tier-1 variants are not stated. | I-36:L54-L56 |
| I36-A003 | A/TEST | The fixture-floor metric and rollback threshold are required but not numerically or procedurally defined. | I-36:L19-L23, L47-L48, L93-L94 |
| I36-I001 | I/IDEM | Versioned role/backend keys plus exact variant hashes make serving a promoted variant replay-identifiable, but byte-idempotent store updates are not explicitly specified. | I-36:L5-L9, L67-L72, L91-L94 |
| I36-C001 | C/STATUS | The orientation describes offline production of variants as a present workflow; the numbered source defers automated per-backend optimizer runs to Tier 2, leaving Tier-1 store/harness operation partially manual or unspecified. | I-36:L43-L56; orientation:L979-L980 |

Category coverage: capabilities/actions/surfaces ✓; workflows/states/lifecycle ✓; schema/data ✓; flags/default/config ✓; integrations/events ✓; permissions/security/compliance ✓ through promotion boundary and cloud-only optimizer; failures/fallback/recovery ✓ through rollback; determinism/replay ✓; explicit concurrency/retry/idempotent mutation are absent; observability/evidence ✓; tests/done gates ✓; constraints/non-goals ✓; dependencies/status/deferred ✓; no privacy control is specified.

### I-37 — EARS requirement objects

Source: docs/thefinalglance/I-plan/I-37_ears_requirement_objects.md (118/118 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| I37-E001 | E/STATUS | I-37 is a strong-grounded Track-18 BUILD, effort M–L, Wave 3, VERIFIED-INTENT lever, requiring full-form expansion before execution. | I-37:L1-L3, L13-L13 |
| I37-E002 | E/CAP | It turns each natural-language requirement into a typed EarsRequirement linked to deterministic evidence and a live status. | I-37:L5-L11, L15-L24 |
| I37-E003 | E/DATA | pattern is an enum of five EARS stereotypes: Ubiquitous, Event-driven, State-driven, Option, and Unwanted-behaviour; freeform strings are prohibited. | I-37:L15-L24, L89-L93 |
| I37-E004 | E/DATA | content_hash is blake3 of canonical requirement text and detects drift. | I-37:L18-L20 |
| I37-E005 | E/DATA | bound_tests is Vec<TestSymbol> naming tests claimed to cover the requirement. | I-37:L20-L22 |
| I37-E006 | E/DATA | mutation_coverage stores I-20's sampled score for bound tests at the latest audit. | I-37:L21-L24, L38-L46 |
| I37-E007 | E/DATA | metamorphic_relations is Vec<MrId> resolved through the existing RelationLibrary. | I-37:L22-L24, L38-L45 |
| I37-E008 | E/STATE | verified_status has Satisfied, Unverified, and Regressed; satisfaction requires mutation coverage at/above the applicable floor and all bound metamorphic relations to hold. | I-37:L23-L24, L31-L32 |
| I37-E009 | E/HONESTY | A requirement lacking verifiable evidence is shown as Unverified and can never be silently dropped. | I-37:L31-L32, L47-L55, L89-L99 |
| I37-E010 | E/INT | IntentPair/IntentPairsGate are generalized rather than rebuilt; the simplest IntentPair maps to an Ubiquitous record with one bound test and one relation. | I-37:L26-L29, L38-L45, L62-L67, L77-L83, L101-L116 |
| I37-E011 | E/INTEROP | Spec-Kit and Kiro-style YAML/Markdown-frontmatter requirement lists round-trip into typed records at planning time; imported files express intent while ELAI proves satisfaction. | I-37:L26-L29 |
| I37-E012 | E/CAP | Tier 1 explicitly provides parse_speckit_file()->Vec<EarsRequirement>. | I-37:L47-L50, L62-L64, L70-L76 |
| I37-E013 | E/FLOW | Requirements are authored/imported at the C-50 ContractSpec PLANNING gate before the TESTING_FIRST transition. | I-37:L47-L53, L62-L65, L77-L87 |
| I37-E014 | E/DATA | requirements.rs supplies EarsRequirement, EarsRequirementSet, verify_requirement, and parse_speckit_file. | I-37:L47-L50, L62-L64, L70-L76 |
| I37-E015 | E/EVENT | RequirementVerified kinds are Satisfied, Unverified, and Regressed and are registered as a live-baseline-relative +1 SSB type. | I-37:L47-L54, L70-L76, L96-L109 |
| I37-E016 | E/FLOW | Audit cycles call verify_requirement, join I-20 mutation evidence and metamorphic results, then emit RequirementVerified. | I-37:L62-L67, L84-L87, L96-L99 |
| I37-E017 | E/LIFECYCLE | Deleting a bound test and reaudit flips the affected requirement to Regressed. | I-37:L47-L55, L96-L99 |
| I37-E018 | E/IDEM | Imported Spec-Kit data round-trips idempotently on content_hash. | I-37:L96-L99 |
| I37-E019 | E/SURFACE | G-5 cockpit renders per-requirement status chips per hunk via H-52 anchoring and Tauri/Svelte pairing; cross_lang_diff.py must pass. | I-37:L31-L36, L41-L43, L47-L54, L62-L68, L84-L99 |
| I37-E020 | E/OBS | Satisfied chips point to real test and mutation rows, with joins recomputable from I-20 SSB evidence under ELAI_SEED. | I-37:L89-L99 |
| I37-E021 | E/TEST | The fixture contains three EARS requirements, bound tests, and mutation scores; it exercises Satisfied, deleted-test Regressed, and no-bound-test Unverified. | I-37:L47-L55, L96-L99 |
| I37-E022 | E/LIMIT | The stated “88% sampled mutation plus MR-3” is an example requirement expression, not a declared universal threshold. | I-37:L31-L32 |
| I37-E023 | E/DEFER | Formal-obligation closure/H-54 verifier integration is deferred until after Wave 3. | I-37:L57-L59 |
| I37-E024 | E/DEFER | Hierarchical/cross-requirement dependency graphs are research-grade and deferred until after an I-20 full-corpus run. | I-37:L58-L60 |
| I37-E025 | E/DEFER | Webhook-triggered automated Spec-Kit synchronization is deferred as infrastructure rather than correctness. | I-37:L59-L60 |
| I37-E026 | E/DET | Production paths prohibit fabricated mutation coverage, unwrap/expect, and blake2/sha2; hashes use blake3. | I-37:L89-L93 |
| I37-E027 | E/DATA | The required decision artifact is workflow/reasoning/I-37-decisions.md. | I-37:L54-L55 |
| I37-E028 | E/LIMIT | I-37 must not reimplement intent capture, metamorphic testing, ContractSpec gate machinery, cockpit anchoring, or mutation scoring. | I-37:L111-L116 |
| I37-E029 | E/EVIDENCE | The METR citation and related review-economics headline must be web-confirmed before use. | I-37:L101-L109 |
| I37-E030 | E/EVIDENCE | The motivation carries review-bottleneck figures of a 19% METR slowdown and AI-PR review waits 2.5–5.3x longer; these are evidence claims, not product thresholds. | I-37:L31-L36, L101-L108 |
| I37-A001 | A/CFG | The mutation floor used by verify_requirement is not specified; the only number is explicitly framed as an example. | I-37:L23-L24, L31-L32 |
| I37-A002 | A/INTEROP | The source broadly promises Kiro-style round-trip interop, but Tier-1 symbols name only parse_speckit_file and define no distinct Kiro parser/format contract. | I-37:L26-L29, L47-L50, L70-L76 |
| I37-A003 | A/DATA | Canonical requirement-text normalization for content_hash is not defined. | I-37:L18-L20 |
| I37-A004 | A/STATE | The source uses VERIFIED/UNVERIFIED in prose/headline but the typed enum is Satisfied/Unverified/Regressed; whether “VERIFIED” is a UI label or alias is unstated. | I-37:L1-L11, L23-L24 |
| I37-C001 | C/STATE | The orientation describes deletion as flipping to Unverified, while the numbered source's explicit typed state and fixture require Regressed. | I-37:L23-L24, L47-L55, L96-L99; orientation:L1018-L1019 |
| I37-C002 | C/INTEROP | The orientation lists Spec-Kit and Kiro imports equally; the numbered Tier-1 API explicitly implements only parse_speckit_file, leaving Kiro-specific import support ambiguous. | I-37:L26-L29, L47-L50, L70-L76; orientation:L1008-L1009 |

Category coverage: capabilities/actions/surfaces ✓; workflow/state/lifecycle ✓; schema/data ✓; flags/default/config ✓ through threshold ambiguity; integrations/events ✓; permissions/security/privacy/compliance are not explicitly modeled beyond evidence integrity; failures/fallback/recovery ✓ through visible Unverified/Regressed; determinism/idempotency ✓; concurrency/retry absent; observability/evidence ✓; tests/done gates ✓; constraints/non-goals ✓; dependencies/status/deferred ✓.

### I-48 — prompts as reviewed file artifacts

Source: docs/thefinalglance/I-plan/I-48_prompts_as_artifacts.md (68/68 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| I48-E001 | E/STATUS | I-48 is a strong-grounded Track-20 REFACTOR, effort S, Wave 2, STEERING lever and Tier-1 competitor steal. | I-48:L1-L5, L58-L60 |
| I48-E002 | E/CAP | Runtime dispatch prompt templates move from inline literals to static Markdown files under crates/<crate>/prompts and are imported with include_str!. | I-48:L5-L10, L13-L20 |
| I48-E003 | E/REVIEW | File artifacts make prompt changes diffable and code-reviewable and make I-36 compiled variants/I-47 frozen blocks pinnable. | I-48:L5-L11, L62-L66 |
| I48-E004 | E/FLOW | The first task inventories dispatch/agent/role modules for prompt-like multiline or role-framing literals above an approximate 200-character threshold. | I-48:L16-L20, L25-L29 |
| I48-E005 | E/FLOW | Extraction proceeds per crate, preserving the existing typed substitution path and auditing each batch under one-change-at-a-time discipline. | I-48:L16-L23, L58-L60 |
| I48-E006 | E/TEST | A contract scans dispatch modules and fails on new long inline prompt literals; only frozen non-prompt strings may be allowlisted. | I-48:L16-L20, L40-L49 |
| I48-E007 | E/LIMIT | Extraction is byte-preserving and may not rewrite prompt content. | I-48:L21-L23, L40-L49 |
| I48-E008 | E/DEP | No new template-engine dependency is allowed; include_str! feeds the unchanged D-1 typed substitution system. | I-48:L21-L23, L40-L49, L62-L66 |
| I48-E009 | E/LIMIT | .claude/agents/*.md is excluded because it is harness rather than runtime prompt scope. | I-48:L21-L23 |
| I48-E010 | E/SCOPE | Ship-now scope is orchestrator/providers dispatch prompts plus the ban test. | I-48:L31-L34, L36-L38 |
| I48-E011 | E/DEFER | Python-sidecar prompt strings are a separate cross-language slice and dashboard-side copy is deferred. | I-48:L31-L34 |
| I48-E012 | E/STATUS | Inline-literal count is initially unknown; inventory revises the estimate, and a blast radius above 40 files pauses for impact mapping. | I-48:L25-L29 |
| I48-E013 | E/FLOW | Production reachability is preserved because every dispatch call site assembles byte-identical prompt output from a file instead of its former literal. | I-48:L40-L46 |
| I48-E014 | E/EVENT | I-48 adds no action type; assembled prompt bytes remain in existing journaled requests, with later provenance attributable to I-36 PromptCompiled. | I-48:L50-L55, L62-L66 |
| I48-E015 | E/SURFACE | No UI changes ship because this is a build-time discipline refactor. | I-48:L36-L38 |
| I48-E016 | E/TEST | Executor and auditor prompt assemblies are compared before/after using blake3 over journaled request bodies and must be byte-identical. | I-48:L45-L55 |
| I48-E017 | E/TEST | A seeded inline-literal reintroduction must fail the ban contract. | I-48:L50-L56 |
| I48-E018 | E/REVIEW | Extracted prompt files must render as ordinary git diffs. | I-48:L50-L56 |
| I48-E019 | E/LIMIT | Partial extraction cannot “pass” by allowlisting prompt literals across half the codebase. | I-48:L47-L49 |
| I48-E020 | E/DEP | E-8 GEPA reads/writes file artifacts, I-36 compiles/diffs them, and I-47's frozen injection block becomes one; none of those systems is restated. | I-48:L62-L66 |
| I48-E021 | E/EVIDENCE | The gajae prompt-directory pattern is the competitor source and SL-10 original-read rule applies before execution even though the directory shape already exposes the discipline. | I-48:L13-L15, L58-L60 |
| I48-A001 | A/CFG | The inline-literal threshold is approximate (“~200 chars or multiline with role framing”), not a fixed contract value. | I-48:L16-L20 |
| I48-A002 | A/SCOPE | “Every dispatch prompt template” is the headline target, but current Tier 1 explicitly excludes Python and dashboard text; completion of the global headline spans later slices. | I-48:L1-L11, L31-L34 |
| I48-A003 | A/TEST | The source names executor and auditor comparison in the trigger but does not enumerate every role/provider prompt that must have a golden pre-extraction body. | I-48:L50-L55 |
| I48-C001 | C/SCOPE | The orientation presents every dispatch prompt as moved now; the numbered source limits ship-now extraction to orchestrator/providers and defers sidecar/dashboard material. | I-48:L31-L34; orientation:L1309-L1311 |

Category coverage: capabilities/actions/surfaces ✓; workflows/states ✓; schema/data/lifecycle ✓ through file layout and existing journal body; flags/default/config ✓ through threshold ambiguity; integrations/events ✓; security/privacy/compliance are not specified; failure/fallback ✓ through contract/impact pause; determinism/idempotency ✓ through byte identity; concurrency/retry/recovery absent; observability/evidence ✓; tests/done gates ✓; constraints/non-goals ✓; dependencies/status/deferred ✓.

## Cross-source interactions and lifecycle

These relational records do not add feature atoms; each resolves to the cited source atoms.

| Interaction ID | Explicit or bounded interaction | Atom references |
|---|---|---|
| IX-001 | The grounding map is a dated substrate/absence/correction contract, not a current-state oracle; every numbered executor must re-grep live seams and baselines. | IG-E001–IG-E006, IG-E024–IG-E026, IG-I001 |
| IX-002 | I-8 turns H-39's abstention scoring into an explicit tool path, uses H-49 for rollback, and attaches its watchdog to I-17; the tool/cache can precede the later integrations. | I8-E002–I8-E013 |
| IX-003 | I-8 abort outcomes feed the single calibration surface that I-23 unifies; neither plan duplicates refusal or scoring math. | I8-E004, I8-E012–I8-E013, I23-E002, I23-E017 |
| IX-004 | I-13's three layers are complementary: write-time refusal protects stored memory, condensed ToolResult protects context budget, and TraceQuery recovers exact source ranges on demand. | I13-E002–I13-E015 |
| IX-005 | I-13 extends the shipped safety/memory stack at write time and must not duplicate H-31/H-32, E-37, MINJA, G-9, or H-6. | IG-E017, IG-E051, I13-E005–I13-E008 |
| IX-006 | I-15 exact-key lookup precedes semantic lookup; I-10 supplies TTL choices, H-8 supplies store/confinement, and H-51 gates future cross-session warmup. | I15-E002–I15-E014 |
| IX-007 | I-18 chains the SSB evidence on which I-28's sealed DPO provenance relies; an unsealed record cannot enter the I-30 training-data path. | I18-E003–I18-E008, I28-E017–I28-E019 |
| IX-008 | I-18's security boundary is offline-first: local chain/stack/AI-BOM ships independently, while user-approved, anti-slopsquat-checked Rekor remains optional and fail-closed when requested. | I18-E009–I18-E014 |
| IX-009 | I-23 supplies a single tau identity to existing refusal and future I-4 search; I-4 activation is later, and G-7 owns its own planned migration. | I23-E003–I23-E012 |
| IX-010 | I-28 depends on I-4 returning SearchLadderOutcome and I-17 frozen verdicts; its first consumer-facing training artifact is gated on I-30 reader/wire-dark coordination. | I28-E001–I28-E006, I28-E016–I28-E019 |
| IX-011 | I-28's lifecycle is strictly sliced: branch credit now/default-off, pruned-verdict DPO only after FORK-1, and node/chunk credit only after PUCT/provenance. | I28-E006–I28-E023, I28-A001–I28-A003 |
| IX-012 | I-28 sequencing preserves veto supremacy: a hard-vetoed sibling may be selected only by non-L2 metadata after FORK-1, never by an invented verifier score. | I28-E016–I28-E018, I28-A001, I28-C002 |
| IX-013 | I-48 file extraction provides reviewable source artifacts; I-36 subsequently compiles per-backend variants from them, and I-47 can pin a frozen injection block as another file. | I48-E002–I48-E003, I48-E020, I36-E002–I36-E003 |
| IX-014 | I-36 serves a compiled variant only after I-32 role scoping and uses B-37 for the role/backend key; its optimizer and promotion evaluator stay outside each other's write boundaries. | I36-E006–I36-E007, I36-E013 |
| IX-015 | I-36's delivery lifecycle is compile -> fixture comparison -> gated promotion -> served hash receipt -> automatic rollback on regression. | I36-E004, I36-E008–I36-E015 |
| IX-016 | I-37's evidence lifecycle is planning import/authorship -> bound tests/MRs -> I-20 sampled coverage -> audit status -> per-hunk cockpit chip; deleting evidence creates Regressed rather than erasing the requirement. | I37-E004–I37-E020 |
| IX-017 | I-37 composes C-29 intent, C-50 planning gates, I-20 mutation evidence, RelationLibrary, and G-5/H-52 presentation without owning those subsystems. | I37-E010, I37-E013–I37-E020, I37-E028 |
| IX-018 | The chunk's hash doctrine is consistently blake3 for dead-end/query/integrity/requirement/prompt evidence; sources explicitly ban blake2/sha2 on new paths. | I8-E006, I8-E019, I15-E004, I15-E019, I18-E003, I18-E006, I18-E018, I36-E023, I37-E004, I37-E026, I48-E016 |
| IX-019 | Optional/new external dependencies never become silent defaults: I-13 YARA and I-18 cosign/Rekor need explicit approval/verification, while I-36 cloud optimization is budget-gated. | I13-E005, I18-E011–I18-E014, I36-E005, I36-E017 |
| IX-020 | Evidence ownership stays outside the model: I-28 credit is deterministic, I-36 promotion passes frozen ledger gates, I-37 coverage is journal-recomputable, and I-18 makes the resulting record tamper-evident. | I28-E007, I28-E016, I36-E009, I36-E013, I37-E020, I18-E003–I18-E008 |

## Explicit ambiguity and conflict register

This register references, rather than duplicates, the A/C atoms above.

| Register ID | Type | Unresolved or conflicting point | Atom references |
|---|---|---|---|
| AC-001 | temporal | Grounding baselines and wiring observations are dated and must not be promoted to current status. | IG-I001, IG-A002, I28-C006 |
| AC-002 | external evidence | Research identifiers, vendor figures, CycloneDX version, and Rekor/cosign details require execution-time verification. | IG-A001, I18-A001, I18-A003 |
| AC-003 | dead-end identity | The canonical AST/approach shape and exact tool-name relationship are undefined. | I8-A001–I8-A002 |
| AC-004 | rollout | I-8 lacks the named halting-bound key, and its UI is deferred despite the summary's unconditional presentation. | I8-A003, I8-C001 |
| AC-005 | guard | I-13 lacks a numeric false-positive floor, concrete Tier-1 pattern catalog, and TraceQuery path/range authorization contract. | I13-A001–I13-A003 |
| AC-006 | cache | I-15 leaves canonical query normalization, vector threshold, and final event schema open. | I15-A001–I15-A003, I15-C001 |
| AC-007 | attestation | Rekor implementation, field-migration semantics, and external schema identity remain choices; Rekor push is deferred despite broad summary wording. | I18-A001–I18-A003, I18-C001 |
| AC-008 | summary-only dependency | I-45 coordination appears only in orientation and is absent from the numbered I-18 composition contract. | I18-C002 |
| AC-009 | calibration | Local threshold derivation and the future I-4 seam are open; G-7 migration is future-owned, not current. | I23-A001–I23-A002, I23-C001 |
| AC-010 | credit model | I-28's negative selection, writer activation, provenance owner, and DpoPairExtracted registry shape are unresolved. | I28-A001–I28-A004 |
| AC-011 | refreshed I-28 correction | The summary's node/chunk/template walk, L2-scored veto negative, PhaseComplete hook, three-abort demotion, and optional heat UI conflict with the refreshed sliced plan. | I28-C001–I28-C005 |
| AC-012 | prompt compilation | Tier-1 variant provenance, kill-switch semantics, and fixture floor are open because automated compilation is deferred. | I36-A001–I36-A003, I36-C001 |
| AC-013 | requirements | I-37 leaves the mutation floor, canonical text, Kiro-specific parser, and VERIFIED/Satisfied label mapping unresolved; deletion status conflicts with the summary. | I37-A001–I37-A004, I37-C001–I37-C002 |
| AC-014 | prompt extraction | I-48's approximate threshold, global completion boundary, and golden role inventory remain open; the summary overstates immediate all-dispatch scope. | I48-A001–I48-A003, I48-C001 |

## Summary additions and corrections

The orientation file explicitly says it did not fold original numbered plans or grounding maps into its source inventory. The deltas below record what this deep read adds, qualifies, or corrects. They do not clone atoms.

### I-GROUNDING-MAP deltas

| Delta ID | Change relative to 09-I-SUMMARY_INDEX.discovery.md | Atom references |
|---|---|---|
| SD-IG-001 | ADD exact grounding method, stale-map warning, dated 221/532 constants, stale baseline list, and v97 test residue. | IG-E001–IG-E006 |
| SD-IG-002 | ADD shipped-substrate ledger for SSB, Darwin, traps, cache, context, calibration, mutation, recovery, RRA, provenance, decoding, compiled stores, tool catalog, intent, gates, crates, and wiring tools. | IG-E007–IG-E026 |
| SD-IG-003 | ADD 14 confirmed absence/build-status records for I-4/I-6/I-9/I-10/I-11/I-14/I-18/I-21/I-33/I-35/I-37/I-38/I-39 and related gaps. | IG-E027–IG-E040 |
| SD-IG-004 | ADD binding design corrections for I-7/I-10/I-11/I-13/I-14 and the cache/tool-layout tradeoffs. | IG-E041–IG-E045, IG-E049–IG-E051 |
| SD-IG-005 | ADD binding sequencing/status corrections for I-4/I-19/I-20/I-23, Track 17, checkpoint recovery, trust wiring, persona scope, and constrained decoding. | IG-E046–IG-E048, IG-E052–IG-E059 |
| SD-IG-006 | ADD exact unverified citation/evidence inventory plus dated cache, rate-limit, extended-thinking, batch, nondeterminism, benchmark, and optimizer landmines. | IG-E060–IG-E065, IG-E073–IG-E074, IG-A001 |
| SD-IG-007 | ADD exact Track-12–18 per-session substrate/production-anchor map. | IG-E066–IG-E072 |
| SD-IG-008 | QUALIFY every grounding-map status as dated/necessary-not-sufficient and leave file-token reachability unresolved. | IG-I001, IG-A002 |

### I-8 deltas

| Delta ID | Change | Atom references |
|---|---|---|
| SD-I8-001 | ADD bounded in-session LRU semantics, pre-call final prune, blake3-only rule, and cross-session persistence gate. | I8-E006–I8-E008, I8-E014, I8-E019 |
| SD-I8-002 | ADD exact tool/score/rollback/watchdog production chains and staged Wave-1 versus H-49/I-17 activation. | I8-E002–I8-E005, I8-E009–I8-E013, I8-E026–I8-E027 |
| SD-I8-003 | ADD learned-classifier/global-timeout exclusions, same-session resurrection ban, named-config requirement, decision artifact, and complete broken-arm evidence. | I8-E016–I8-E025 |
| SD-I8-004 | CORRECT the summary's unconditional ReasoningDrawer surface: it is deferred/wire-dark behind GD-4 and W-6-justified for v1. | I8-E015, I8-C001 |
| SD-I8-005 | FLAG undefined approach canonicalization, tool alias/handler naming, and configuration-key identity. | I8-A001–I8-A003 |

### I-13 deltas

| Delta ID | Change | Atom references |
|---|---|---|
| SD-I13-001 | ADD exact composition boundary with memory/safety systems, write-side Provenance registration, hard refusal/no-store semantics, and no-refilter rule. | I13-E003–I13-E009 |
| SD-I13-002 | ADD exact dual-serialization and TraceQuery production chains plus the three MemoryGuardEvent discriminators. | I13-E010–I13-E016, I13-E020 |
| SD-I13-003 | ADD YARA's “only if measured first-line scanner insufficient” gate and deferred governed threat-feed updates. | I13-E005, I13-E017–I13-E018 |
| SD-I13-004 | ADD negative-arm and full-vs-condensed 500-file artifact proof; FLAG unspecified FP threshold, path authorization, and pattern catalog. | I13-E021–I13-E025, I13-A001–I13-A003 |

### I-15 deltas

| Delta ID | Change | Atom references |
|---|---|---|
| SD-I15-001 | ADD exact canonical-query fast-path ordering, semantic miss fallback, production A-RAG/DKG seams, and broader exclusion of tool/reasoning content. | I15-E003–I15-E007, I15-E011 |
| SD-I15-002 | ADD volatile five-minute default under I-10 override and defer cross-session warming plus threshold tuning. | I15-E008, I15-E013–I15-E015 |
| SD-I15-003 | QUALIFY summary event reuse: OracleCacheHit/Miss is preferred, but SemanticCacheEvent remains a conditional edit-time choice. | I15-E009–I15-E010, I15-A001, I15-C001 |
| SD-I15-004 | ADD exact two-run/broken-arm evidence and FLAG undefined canonicalization/vector threshold. | I15-E016–I15-E020, I15-A002–I15-A003 |

### I-18 deltas

| Delta ID | Change | Atom references |
|---|---|---|
| SD-I18-001 | ADD exact previous-row-byte hash, single-writer/no-async invariant, append-lock coordination, layer inputs, and sealed immutability. | I18-E003–I18-E008, I18-E017–I18-E018 |
| SD-I18-002 | ADD exact RowChainSealed kinds, CLI first-break behavior, badge subscription/state, and complete negative row-edit/Cargo-lock tests. | I18-E015–I18-E024 |
| SD-I18-003 | QUALIFY Rekor as separately deferred push with scaffolded UI/flag, approval/anti-slopsquat dependency choice, and fail-closed requested behavior. | I18-E011–I18-E014, I18-A001, I18-C001 |
| SD-I18-004 | ADD execution-time CycloneDX/Rekor verification requirements and FLAG schema-extension wording ambiguity. | I18-E009–I18-E010, I18-A002–I18-A003 |
| SD-I18-005 | REMOVE/FLAG the orientation-only I-45 coordination claim, which the numbered source does not support. | I18-C002 |

### I-23 deltas

| Delta ID | Change | Atom references |
|---|---|---|
| SD-I23-001 | ADD typed facade methods, zero-direct-import invariant/test, exact reused-event identity, and D-16 unchanged regression gate. | I23-E003–I23-E006, I23-E011–I23-E014 |
| SD-I23-002 | QUALIFY G-7 as a future-owned migration and I-4 fork as Wave-1 scaffolding activated only with Wave-4 PUCT. | I23-E007–I23-E010, I23-C001 |
| SD-I23-003 | ADD local threshold artifact, divergent-hypothesis rule, no-new-UI details, and deferred multi-model tau support. | I23-E008–I23-E010, I23-E015–I23-E016 |
| SD-I23-004 | FLAG undefined CI-width calibration procedure and exact future I-4 seam. | I23-A001–I23-A002 |

### I-28 deltas

| Delta ID | Change | Atom references |
|---|---|---|
| SD-I28-001 | CORRECT summary DAG/node attribution to ratified default-off Slice-1 deterministic branch-grain sibling deltas. | I28-E001–I28-E011, I28-C001 |
| SD-I28-002 | ADD exact task-runner Option<SearchLadderOutcome> pass-through and placement before escalation; CORRECT PhaseComplete. | I28-E003–I28-E005, I28-C003 |
| SD-I28-003 | ADD default-off flag, wire-dark promotion row, production-caller policy, contract/schema-roundtrip gates, and refreshed registry derivation. | I28-E010–I28-E015 |
| SD-I28-004 | CORRECT L2-scored vetoed negative as forbidden; DPO selection is post-FORK-1 and must preserve frozen hard-veto semantics. | I28-E016–I28-E019, I28-A001, I28-C002 |
| SD-I28-005 | CORRECT chunk demotion/three-abort threshold to fully deferred Slice 3 pending real per-branch provenance/owner; node-grain waits for PUCT. | I28-E020–I28-E022, I28-A003, I28-C004 |
| SD-I28-006 | CORRECT optional immediate heat UI to no-dashboard unit/deferred heat; ADD I-30 reader-or-wire-dark artifact rule. | I28-E019, I28-E023, I28-A002, I28-C005 |
| SD-I28-007 | ADD detailed fail/abort/red-team/off-agent verification and plan-refresh execution boundaries. | I28-E024–I28-E033 |
| SD-I28-008 | FLAG unresolved DPO action shape and source-level temporal conflict with dated grounding constants. | I28-A004, I28-C006 |

### I-36 deltas

| Delta ID | Change | Atom references |
|---|---|---|
| SD-I36-001 | ADD exact artifact/store methods, 11-backend motive, post-role-contract routing order, cloud-only optimizer boundary, and no-reimplementation seams. | I36-E002–I36-E007, I36-E019–I36-E020, I36-E026 |
| SD-I36-002 | ADD deterministic transfer/rollback lifecycle, exact event kinds, promotion gates, and evaluator write-boundary isolation. | I36-E008–I36-E015 |
| SD-I36-003 | QUALIFY offline compilation: automated per-backend optimizer runs are Tier 2 behind a kill switch, leaving Tier-1 variant creation unspecified. | I36-E017, I36-A001–I36-A002, I36-C001 |
| SD-I36-004 | ADD UI/cross-language gate, decision artifact, web-citation caution, and FLAG undefined fixture floor. | I36-E016, I36-E019, I36-E021–I36-E025, I36-A003 |

### I-37 deltas

| Delta ID | Change | Atom references |
|---|---|---|
| SD-I37-001 | ADD exact typed fields/enums, blake3 drift key, Satisfied predicate, and content-hash idempotent round-trip. | I37-E003–I37-E008, I37-E018 |
| SD-I37-002 | ADD exact planning/audit/cockpit lifecycle, event kinds, real evidence joins, and full three-state fixture. | I37-E009–I37-E021 |
| SD-I37-003 | CORRECT deletion state from summary Unverified to source Regressed. | I37-E017, I37-C001 |
| SD-I37-004 | QUALIFY Kiro import: prose promises Kiro-style round-trip, but Tier-1 API exposes only parse_speckit_file. | I37-E011–I37-E012, I37-A002, I37-C002 |
| SD-I37-005 | ADD formal closure, dependency graph, and webhook-sync deferrals plus composition/non-reimplementation boundaries and review-bottleneck evidence scope. | I37-E023–I37-E030 |
| SD-I37-006 | FLAG example-only mutation floor, canonicalization, and VERIFIED-versus-Satisfied naming. | I37-E022, I37-A001, I37-A003–I37-A004 |

### I-48 deltas

| Delta ID | Change | Atom references |
|---|---|---|
| SD-I48-001 | ADD inventory-first workflow, approximate literal heuristic, per-crate extraction discipline, impact-map pause, and frozen non-prompt allowlist. | I48-E004–I48-E006, I48-E012, I48-E019 |
| SD-I48-002 | ADD explicit exclusions: no content rewrite, template dependency, or harness-agent edit; D-1 remains unchanged; SL-10 original competitor read still applies. | I48-E007–I48-E009, I48-E021 |
| SD-I48-003 | CORRECT global “every prompt now” scope to orchestrator/providers Tier 1 with Python/dashboard slices deferred. | I48-E010–I48-E011, I48-A002, I48-C001 |
| SD-I48-004 | ADD production byte-identity reachability, existing-journal/no-event behavior, seeded negative arm, and ordinary git-diff proof. | I48-E013–I48-E018 |
| SD-I48-005 | FLAG approximate threshold and incomplete golden role/provider inventory. | I48-A001, I48-A003 |

## Exhaustiveness, zero-omission, and duplicate audit

- Source coverage: **10/10 files**, **1,169/1,169 logical lines**, **99,202/99,202 bytes** read and reconciled; source SHA mismatches **0**.
- Atomic source ledger: **348 unique atoms** = **301 explicit**, **002 inferred**, **030 ambiguous**, **015 conflicting/corrective**.
- Per-file atom counts: I-GROUNDING-MAP **077**; I-8 **031**; I-13 **028**; I-15 **024**; I-18 **031**; I-23 **022**; I-28 **043**; I-36 **031**; I-37 **036**; I-48 **025**. Sum **348**.
- Relational interaction records: **20** (IX-001–IX-020). Summary-delta records: **053** (SD-*). Ambiguity/conflict register: **14** (AC-001–AC-014). These records reference source atoms and are not double-counted as features.
- ID audit: every atomic table row has exactly one unique ID matching IG-[EIAC]nnn or I8/I13/I15/I18/I23/I28/I36/I37/I48-[EIAC]nnn. Duplicate atomic IDs **0**; missing/blank atomic IDs **0**.
- Semantic duplicate audit: repeated premise/scope/wiring/grounding/acceptance statements were merged into one atom with multiple anchors. Interactions, conflict register, and summary deltas reference atoms instead of cloning them. Duplicate feature atoms found **0**.
- Omission audit: every source was checked against capabilities/actions/surfaces; workflows/states; schemas/data/lifecycle; flags/defaults/config; integrations/events; permissions/security/privacy/compliance; failures/fallback/retry/recovery; determinism/idempotency/concurrency; observability/evidence; tests/acceptance/done gates; constraints/non-goals; dependencies/status/deferred. Each per-file ledger names dimensions absent rather than inventing them.
- Interpretation audit: inferred, ambiguous, and conflicting/corrective atoms are visibly separated by ID/class. No inferred behavior is presented as a requirement. Orientation-only facts were not imported; unsupported or stale summary claims are corrected/flagged.
- Manifest-boundary audit: the only original sources opened were the ten exact DEEP-I-001 manifest rows. The only orientation artifact consulted was 09-I-SUMMARY_INDEX.discovery.md.
- Output metrics after finalization: output_lines=0637; output_bytes=095474.
- Canonical normalized output SHA-256: 431f87e86b6f08e253e923e3a1e1f7a1cadce4f46fc068c5fa4215c9c6ff5521.
- Hash rule: compute SHA-256 over the exact final UTF-8 bytes including the final newline after replacing only the 64 hexadecimal characters on the Canonical normalized output SHA-256 line with 64 ASCII zeroes. Fixed-width output metrics and digest substitution avoid impossible raw self-reference.
