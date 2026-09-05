# I-GROUNDING-MAP — Tracks 12-18 (the whole I-plan folder)
Grounded 2026-06-10, session s-iplan-author-2026-06-10. Method: live source greps against
F:\.ELAI_workflow\.elai_cc (read-only; a concurrent H session may hold the tree — no map regen, no cargo)
+ the 13-file docs/prefableresearch corpus read end-to-end + the B..H summary indexes + the
wire-remediation handoff + the s-plan-audit-2026-06-04 FINISH master index. elai.map was NOT regenerated
(stale vs live H edits); every anchor below that is load-bearing was grep-confirmed in SOURCE, not the map.
Detail files cite these IDs instead of re-asserting facts. Executors re-grep every anchor at EXECUTING (W-7).

================================================================================
CANONICAL CONSTANTS (live at grep time 2026-06-10 — re-read at edit, W-9 / R-c)
================================================================================
SCHEMA_VERSION = 221           (.elai_cc/crates/orchestrator/src/ssb.rs:2981; SCHEMA_VERSION_PUBLIC :2986)
TOTAL_VARIANTS = 532           (.elai_cc/crates/orchestrator/tests/action_type_lock.rs:601; ALL_VARIANTS :610)
STALE numbers seen in prior docs (do NOT quote): 191/464 (CLAUDE.md), 207/484, 220/497, 224/501 (H headers),
234/511, 236/513, 240/517, 209/520, 522 (handoff snapshots). The integer moves between sessions; every I-N
quotes its bump as "baseline+N from LIVE x/y as of <edit date> (ssb.rs / action_type_lock.rs); re-read at edit".
Known stale test: ssb_schema_v97_payloads::schema_version_is_97 asserts ==97 (pre-existing, flagged in the
wire-remediation handoff; fix is a patch-N, not an I-N).

================================================================================
IS-n — SUBSTRATE SHIPPED (grep-confirmed live this session; compose, never restate — GD-1)
================================================================================
IS-1  Live SSB constants as above; ssb.rs golden-locked via action_type_lock.rs (append-only discipline ssb.rs:5).
IS-2  derived_alpha = blake3(config bytes) fitness placeholder — darwin_admit_loop.rs:202 (H-43 replaces; I-31
      consumes H-43's execution-grounded fitness, never the byte-hash).
IS-3  score_candidate_for_traps keyword grep — trap_tasks.rs:123 (H-44 replaces; I-24/I-31 compose H-44).
IS-4  config_archive.rs insert_variant:198 / select_parent:274 / select_parent_with:283 (Darwin archive; I-28/I-31).
IS-5  Anthropic cache plumbing: providers/src/types.rs:58-130 (CacheControl, ephemeral(), "at most 4 markers"
      :61); RuntimeAdapter path anthropic_api.rs:137/163 still cache_control:None (the H-1 gap is OPEN at grep
      time). I-10 composes H-1/H-2; if H-1 ships first I-10 is the TTL/economics/guard layer only.
IS-6  context_budget.rs: update_breakpoints:1507 (blake3 prefix stability), adaptive_reduce:1551,
      ContextSlot enum :1079 — now NINE slots incl ContextSlot::Memory (H-31 note at :10). I-6 adds the
      anchor slot LAST; I-5/I-14 must preserve bp stability (HD-2).
IS-7  canary.rs decide:430 (banded family deltas, D-7/G-10); calibration_bootstrap.rs warm_start_tau (:12
      docstring) + conformal_calibration consumers across task_runner/agent_pool (file-level grep) — D-16/D-30
      substrate for I-23.
IS-8  metamorphic_testing.rs RelationLibrary:181 (+ named financial/trading relations per H-22/H-26 plans);
      mutation_testing.rs MutationRunner:354 + EquivalentMutantCache (blake3-keyed, 30-day TTL per C-28 spec).
      I-20 extends BOTH; I-24 extends the relation library.
IS-9  bsp/engine.rs recover_from_checkpoint:958 + test :1242 — EXISTS (the G-2-era "stub" framing is STALE;
      see IC-15). Production reachability unverified this pass; executor re-traces R7.
IS-10 rra.rs build_rra:384 (+ verify_rra per H-35 cites); attestation.rs exists (in-toto/sigstore mentions) —
      G-2/G-8 shipped surface. I-18 EXTENDS with per-row chaining + Rekor push + AI-BOM; restates none.
IS-11 Taint/provenance substrate: capability_flow.rs taint_text:234; crates/safety/src/provenance.rs:131
      Provenance (a dedicated safety crate now exists). G-1 liveness (real ingest-site constructors) is the
      composition seam for I-13/I-35/I-40.
IS-12 providers/src/decoding.rs DecodingMode::Grammar{ebnf_path, crane_adaptive} :73-82 — the crane_adaptive
      flag ships but the CRANE runtime is absent (":81 today it only affects crane_adaptive_eligible").
      I-34 builds the runtime leg.
IS-13 compiled_pipeline.rs TemplateCrystallizer:293 + CompiledPipelineStore:669 (C-25 shipped) — the
      Compiled-AI zero-token execution seam for I-12's curator + the I-36 compiled-prompt store precedent.
IS-14 production_agent_rpc.rs build_tool_catalog:657 (+ the 3-arm convert/dispatch pattern per H-4/H-13 plans)
      — the tool-array surface I-10 must keep prefix-stable and I-35 gates.
IS-15 intent_pair.rs IntentPair:86 / IntentPairSet:163; evidence.rs IntentPairsGate:658 (C-29 shipped) —
      the substrate I-37's EARS requirement objects extend.
IS-16 H Track-3/4 gate substrate EXISTS as files: trial_ledger.rs, holdout_budget.rs, deflation_gate.rs,
      gate_self_validation.rs, drift_interlock.rs, leakage_gate.rs, trading_null_gate.rs, result_manifest.rs
      (+ crates/rules/seed/repro/result_manifest.toml). ID-6 binds weight/prompt promotions to these.
IS-17 Workspace crates beyond the original 9: safety, memory, eval-corpus, ssb-otel-export (E-29), web_search,
      web_validator, agent, ipc-types, scaffold, dashboard-v2 (+ parked dashboard, .bkp_old).
IS-18 Tokens present at file level (wiring depth NOT verified this pass; executors re-trace R7):
      streaming_validator (runtime_builder.rs, runtime_client.rs), messages/batches (telemetry_types.rs,
      task_runner.rs — H-5 surface), get_relevant_facts (memory/retrieval.rs), conformal_calibration
      (task_runner.rs, agent_pool.rs).
IS-19 Wiring doctrine + tooling live: wiring_contract_check.py (静 + --run-trigger), gate_freeze_check.py,
      wire-dark-allowlist.toml (~79 active rows post r2), failure_taxonomy.py, impact_radius.py. The
      wire-remediation handoff (2026-06-07b) records: NO clean wiring flips remain; the static tally for
      D/E/F/G is 34 wired / 21 dark / 50 partial / 3 empty-contract; remaining work is feature-eng / GPU /
      new-deps. The I-plan is exactly that next layer of feature engineering.

================================================================================
IN-n — NET-NEW CONFIRMED (absence checked this session; name-greps are necessary-not-sufficient — R5)
================================================================================
IN-1  NO MCTS module (no mcts*.rs under orchestrator/src; "mcts" text hits are comments/AB-MCTS config keys).
      I-4 is a genuine BUILD — and deliberately a LADDER, not a day-1 tree (IC-6).
IN-2  NO typed BeliefState module (I-11 BUILD).
IN-3  NO AnchorPane / privileged-last slot (ContextSlot has 9 slots, none is an anchor pane) (I-6 BUILD).
IN-4  NO sprint_contract module (I-21 BUILD; composes H-37/H-38 gate seams).
IN-5  NO per-row chain_prev_hash in ssb rows (attestation exists; row-chaining absent) (I-18 BUILD).
IN-6  NO compression middleware (no LLMLingua/compressor/api_middleware module) (I-14 BUILD, deterministic-first).
IN-7  NO EARS requirement object (intent_pair.rs is the nearest substrate) (I-37 BUILD).
IN-8  NO per-backend compiled-prompt artifact store (I-36 BUILD; CompiledPipelineStore is the storage precedent).
IN-9  NO QDELTA harness (G-6 SAFEDELTA harness exists for the SECURITY delta; the QUALITY sibling is absent)
      (I-38 BUILD).
IN-10 NO `elai plan status` repo-derived plan-state command (I-39 BUILD).
IN-11 NO CEFL candidate-externalization / I6Q rationale-quality / commit-reveal entropy primitives (I-33 BUILD).
IN-12 NO temporal tool-SEQUENCE rule layer (FSM phase tool-masks + E-30 argv allowlist + F-3 tool-set commitment
      exist = per-call gates; ordering-over-history enforcement is absent) (I-35 BUILD).
IN-13 NO cache-economics planner / TTL-tier selector / cache-hit hard-guard (cache_check.rs exists per H-1 as a
      wire-dark monitor; the guard + planner are absent) (I-10 BUILD on H-1/H-2).
IN-14 NO error-amplification (alpha_t) harness (I-9 BUILD; DriftComparator + C-28 mutation are the components).

================================================================================
IC-n — DESIGN-DOC CORRECTIONS (authors MUST use these, not the source docs' originals)
================================================================================
IC-1  Consolidated I-7 "Batch API parallel expansion + speculative local racing" is ARCHITECTURALLY WRONG for
      live loops: Anthropic/OpenAI batch lanes run 10-45 min with 24 h SLA and cancels do not refund queued
      work. CORRECTED I-7 = cascading escalation (local-first waterfall, condense-then-escalate); the Batch
      API stays confined to offline lanes (Cosmos/research/bench — already H-5's scope). Source:
      Review_ADECO-Adversarial-Critique §1/§4-2 + Research_RateLimits (batch semantics).
IC-2  Consolidated I-14's LLMLingua-2-first is DEMOTED: an XLM-R-class encoder costs local VRAM and is neural
      where a deterministic cut exists. CORRECTED I-14 = LSP-slice + tree-sitter AST-fold first (bodies ->
      "/* elided */", signatures intact), LLMLingua-2 as a gated Tier-2 for prose-only payloads. Source:
      critique §1/§3 + §4-1.
IC-3  ADECO "N-Gram Abbreviation (CompactPrompt) for code symbols" is KILLED — cipher tokens break BPE semantic
      priors ("neural poison"). Numeric quantization for tabular payloads survives as a narrow Tier-2 option.
IC-4  ADECO "cache micro-heartbeat" is DEMOTED to last-resort fallback: TTL is a max not a guarantee (LRU under
      pressure). CORRECTED primary = explicit 1 h TTL tiering on the frozen prefix + 5 m on volatile tail +
      burst-amortized scheduling (generate K wide, validate locally, reconnect in a burst). Source: critique §4-3
      + Review_ELAI-Competitive 1.3 (TTL tiering; the 2026-03-06 default-TTL flip).
IC-5  Consolidated I-10 says "OVERRIDE H-4's array mutation". TOO STRONG: H-4's Tool-Search deferral and I-10's
      prefix invariance are BOTH levers with opposite cache effects. CORRECTED: decide per catalog size by
      MEASURED (cache-hit x token) cost from the journal — small catalog inline+frozen (prefix-stable), large
      catalog deferred (H-4) with the deferral structure itself byte-stable. No assumption ships.
IC-6  Consolidated I-4 (MCTS, XL, BLOCKING) is RE-SCOPED to a ladder: v1 = best-of-N with deterministic
      verifier RANKING (selection-by-receipts), v2 = tournament, v3 = PUCT tree with virtual loss. Both
      reviews converge on this; every rung yields user value + training data. The XL tree never blocks Track 14/15.
IC-7  Consolidated Track 17 order is RE-SEQUENCED: I-30 rejection-sampling SFT (on verifier-ranked best-of-N
      trajectories) FIRST; I-31 GRPO LAST with BLOCKING prerequisites (I-24 canaries, H-28 reward-tampering
      gate, H-17/H-18 ledger promotion, KL penalty, evaluator outside the loop's write boundary). LoRA target =
      the canonical local model (Qwen2.5-Coder per CLAUDE.md); any MoE retarget (Qwen3-Coder-Next-class) is a
      RunPod-gated EVALUATION, never an assumed swap (memory: baseline_findings rejected prior swaps).
IC-8  PABU/BeliefState is a commoditized pattern (Letta/MemGPT). ELAI's differentiator = typed + DKG-linked +
      journaled (ID-3) + HONESTLY MEASURED (periodic full-history-vs-BeliefState re-run, delta to the ledger).
IC-9  CoW isolation (I-2): overlayfs/btrfs is Linux-only. CORRECTED cross-platform posture: Linux overlayfs;
      Windows/macOS = pooled warm git worktrees with shared target-dir; cloud sandboxes (E2B-class) = optional
      adapter behind the same trait, NOT a dependency. Composes E-15 Firecracker pool + C-11.
IC-10 Consolidated I-13's "OWASP memory guard" must COMPOSE the shipped H-31/H-32 auditor-objectivity firewall +
      E-37 PromptArmor + memory-crate MINJA defenses; net-new = write-time deterministic scan + dual
      serialization + TraceQuery. A YARA engine is a NEW DEP (anti-slopsquat + user approval) — regex/aho-corasick
      first.
IC-11 Consolidated I-18's premise "G-2 RRA lacks real-time chaining" understates shipped state: rra.rs +
      attestation.rs ship (IS-10). I-18 = EXTENSION (per-row prev-hash chain, 5-layer stack hash, opt-in Rekor,
      AI-BOM per OWASP AISVS AC.8), restating neither.
IC-12 Consolidated I-19's premise "agents vibe-code with no TDD gate" ignores TESTING_FIRST FSM gating + E-13
      anti-over-mock + F-1 VibeGate. I-19 = the symbol-level PreToolUse leg only (src-edit requires matching
      test-symbol delta), with a coverage-delta refactor exemption; FP-bounded before default-ON (GD-4).
IC-13 Consolidated I-23 "implement Brier calibration" — conformal_calibration + calibration_bootstrap SHIPPED
      (IS-7). I-23 = UNIFICATION (one calibration impl consumed by D-16 refusal, G-7 claim_score, and the
      I-4 search prior), not new math.
IC-14 Consolidated I-20's premise "C-28 re-evaluates the entire AST per check" is PARTLY STALE: the
      EquivalentMutantCache ships. The real gaps = diff-cone mutant selection, time-boxed SAMPLED scoring with
      a confidence interval, and the surviving-mutant -> regeneration Directive loop.
IC-15 G-2-era claim "recover_from_checkpoint is a STUB" is STALE: bsp/engine.rs:958 implements it with a test
      (:1242). Remaining honest gap = production reachability + resume-from-journal UX (H-35/H-51 own the UX).
IC-16 Trust-review A1's "wire the trust spine first" is ALREADY SUBSTANTIALLY EXECUTED per the wire-remediation
      handoff (bedrock reached; D trust stack built, default-OFF static-dark pending R9 rows + runtime leg +
      GPU). The I-plan therefore does NOT restate spine-wiring sessions; it sequences its own waves so that
      trust-reinforcing items (Wave 1) precede search/flywheel (Waves 4-5), and it binds I-31 to the spine's
      gates (ID-6).
IC-17 Persona prompting: REMOVE narrative personas from factual/code-generation roles (measured -3.6 to -5.3 pp
      MMLU; -0.65 MT-Bench coding); KEEP hostile-critic/safety personas where alignment-dependent (auditor
      style compliance, anti-spiral lateral thinking) — the research supports exactly that split. I-32 encodes it.
IC-18 Constrained decoding is NOT free: instruction-tuned models degrade on generation tasks (structure
      snowballing, arXiv:2604.06066); reasoning under tight grammars costs 10-15 pp. CORRECTED posture (I-34):
      CRANE-style grammar augmentation (free thinking tokens between structural commitments), constraints on
      FINAL output only for reflection loops, post-parse schema re-validation because engines emit silently
      non-compliant outputs (JSONSchemaBench: XGrammar 38 silent failures), and grammars treated as a pinned
      control-plane artifact (CDA: EnumAttack 96.2% / DictAttack 75.8-99.5% ASR).

================================================================================
WEB / EXTERNAL VERIFICATION (carried from the source dossiers — dated)
================================================================================
The 13 docs/prefableresearch files are themselves web-grounded research outputs (2026-06-09 era). This
authoring session did NOT independently WebSearch-verify their arXiv IDs / vendor numbers. Per
feedback_paper_citation_verify (B-40/B-42/E-40 prior bad-cite history), every arXiv ID below MUST be
WebSearch-confirmed before it headlines any gate or number. IDs carried: CaMeL 2503.18813, FIDES 2505.23643,
MAST 2503.13657, scaling-agents 2512.08296, sycophancy-propagation 2604.02668, Compiled-AI 2604.05150,
AgentSpec 2503.18666, Agent-C 2512.23738, Pro2Guard 2508.00500, CRANE 2502.09061, structure-snowballing
2604.06066, Let-Me-Speak-Freely 2408.02442, XGrammar 2411.15100, JSONSchemaBench 2501.10868, CFGzip
2605.04066, SELF-DISCOVER 2402.03620, LLMLingua-2 (MSFT), PABU/ACON (26.9% steps), ACE (+10.6%), DSPy/MIPROv2
(+13%), StateFlow COLM-2024 (+13-28%, 3-5x), persona-eval set (MMLU deltas), METR 2507.09089.
Dated economics landmines (carry verbatim into I-0 §CARRIED WEB LANDMINES): 2026-03-06 Anthropic default cache
TTL flipped 1h->5m (set ttl explicitly); TPM = input + max_tokens (clamp to schema size); 60 RPM enforced
sub-second (~1 rps); 1h-TTL write = 2x base (breakeven ~3 reads), 5m = 1.25x (breakeven 1 read); Opus 4.7+
rejects manual thinking budget_tokens (effort parameter only); batch lanes 10-45 min/24 h SLA; temp=0 is NOT
deterministic (~15% variance; MoE 18-75%); SWE-bench Verified retired by OpenAI (contamination; K-Prize
contamination-free top ~7.5%); OPRO-style self-optimization fails under ~100B (optimizer = cloud lane only).

================================================================================
PER-TRACK SESSION -> ANCHOR TABLE (the symbols each detail file leans on)
================================================================================
Track 12: I-1 task_runner dispatch + hook_dispatcher + IS-11 safety crate | I-2 IS-9 bsp engine + E-15/C-11
  sandbox plans + git worktree | I-3 ssb.rs pragmas (Phase 3B spec) + checkpoint write sites.
Track 13: I-4 SessionDag (session_dag.rs, H-49 plan) + IS-8 verifiers + IS-14 catalog | I-5 IS-6
  context_budget + E-19 probe | I-6 IS-6 ContextSlot | I-7 IS-18 batches + H-5/H-7 plans | I-8 IS-7 conformal
  + D-16/D-17 + H-39 abstention | I-9 IS-8 mutation + determinism.rs DriftComparator (H-23 cite).
Track 14: I-10 IS-5 + IN-13 + H-1/H-2/H-4/H-9 plans | I-11 IN-2 + IS-6 + H-7 routing | I-12 IS-13 + Phase 3F
  rules lifecycle + E-8 GEPA | I-13 IS-11 + memory crate (H-31/H-32) + E-37 | I-14 IN-6 + E-52 LSP plan +
  knowledge tree-sitter grammars | I-15 IS-13 store precedent + C-30/H-8 oracle cache | I-16 python sidecar
  server.py + E-26/E-27/E-28 plans.
Track 15: I-17 D-3 verifier_adapters + D-11 checkers + F-1/F-9 gates + IS-8 | I-18 IS-10 + ssb.rs append-only |
  I-19 hooks run_pre_tool_use (F-1 seam) + knowledge grammars | I-20 IS-8 + impact_radius.py/elai_graph calls |
  I-21 H-37 gate.rs run_interactive_gate + D-13 adversary | I-22 D-2 PpmClient seam + H-49 undo-tree |
  I-23 IS-7 | I-24 IS-8 RelationLibrary + H-44 TrapBehaviorDetected (compose).
Track 16: I-25 H-7 routing seams (escalation_policy.rs, cloud_mode.rs) + E-31 RouteDecisionEvent | I-26 H-42
  serial-depth + E-39 cost gate | I-27 E-49 metrics + benchmark.rs | I-28 IS-4 archive + Phase 4H | I-29 C-32
  MiniCritic + E-48 research orchestrator + media/CRDT absent (BUILD).
Track 17: I-30 IS-10 RRA + IS-16 ledger gates + E-55 SFT-on-rollouts plan | I-31 IS-2/IS-3 (via H-43/H-44) +
  IS-16 + D-6 held-out + E-55.
Track 18: I-32 Phase 2C agent configs + .claude/agents personas | I-33 rules engine predicates + D-16 |
  I-34 IS-12 + E-35/E-18 XGrammar substrate + C-37 | I-35 IN-12 + E-30 + F-3 + FSM tool masks | I-36 IN-8 +
  IS-13 + B-37 PerRoleRuntime + E-8 | I-37 IS-15 + C-50 ContractSpec + G-5 cockpit + H-52 anchoring |
  I-38 IN-9 + G-6 safedelta_runner + D-6/E-54 corpus + D-20 memorization probe | I-39 IN-10 + SSB viewer +
  wiring fragments + F-13 archive | I-40 IS-10/IS-11 + G-2 .rra format + B-37 adapters + mcp bridge.

End of I-GROUNDING-MAP.
