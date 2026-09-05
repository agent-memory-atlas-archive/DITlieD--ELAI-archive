# K-12: Context-losslessness enforcement + handoff-faithfulness gate — prove "lossless on what matters"

STATUS: PLANNED (WAVE KW-3; S-M; BUILD gate; EXTENSION(H-49/I-11/G-2/SessionDAG) — two deterministic gates over BUILT code)

Objective: ELAI's context design is a two-layer model — a cryptographically lossless RECORD (append-only SSB journal; trim→stub→recover_stubs with a blake3 integrity check, session_dag.rs:1539, WIRED) under a lossy-but-reversible VIEW. Two grounded gaps keep it from CLAIMING "lossless on what matters" with a PROOF instead of a hope. R1: check_lossless_compaction (compact.rs:545, H-49 RV-2) currently WARNS when unrecoverable_count>0 instead of BLOCKING — "we emit a warning but do not break it"; the record is only lossless if archival actually ran first. K-12 flips it FAIL-CLOSED: refuse the trim (or force the SSB capture-hook to run first) when unrecoverable_count>0 (wiring R1 missing=FAIL). R2 [the one genuinely-new idea]: a HANDOFF-FAITHFULNESS GATE — extend the I-11 BeliefState schema (belief_state.rs; BeliefState::new wired at escalation_waterfall.rs:239, the local→cloud handoff) with critical_atoms{kind, value, must_preserve} (file path, symbol, test name, line span, the bug location, a safety constraint); after any cross-model handoff / I-5 pristine-rebuild (pristine_path_filter.rs) / trim, deterministically assert EVERY must_preserve atom is present in the projected context by exact string/symbol match against the SSB record — missing = FAIL. This is the deterministic "Critical Atom Recall" gate (TD-3, the LLM never adjudicates). HARD BOUNDARY: NO generative summarization (HD-2 ban — busts cache + hallucinates), NO ciphers (IC-3); the gate is exact-presence-check, never semantic similarity ("semantic similarity alone is NOT lossless" — the deep-research report's rule). GD-1: do NOT re-implement recover_stubs, BeliefState, or the RRA verify — COMPOSE them. KD-1 (number/proof-before-claim). Source: workflow/reasoning/context-compression-losslessness-2026-06-25.md PART 7/8 (R1/R2) + PART 6 (Context Codec critical-atom measurement, tier-2 corroboration).

Estimated effort: ~1 session (~S-M). Both legs in-process at compaction/handoff time; no network.
Pre-requisites (re-grep live per W-7): session_dag.rs:1539 recover_stubs (blake3(payload)==stub.summary_hash; StubRecoveryFailed on mismatch — WIRED, 1 prod caller in live_cli.rs); compact.rs:545 check_lossless_compaction (H-49 RV-2, WARN-only today); belief_state.rs BeliefState + belief_state_event + i11_belief_state_contract.rs; BeliefState::new at escalation_waterfall.rs:239 (BeliefStateEventKind::Condensed); pristine_path_filter.rs + i5_pristine_context_contract.rs (the pristine rebuild it gates); G-2 rra.rs verify_rra.
Parallel-safe with: K-11, J-80, J-81, J-82 (disjoint files).
Source: context-compression-losslessness-2026-06-25 PART 7 (code-grounded reality, CORRECTS the map's stale PLANNED labels for I-5/I-11/H-50) + PART 8 R1/R2. Doctrine: K-0_INDEX.md (KD-1).

## Context
Why this matters: the user's exact ask — "lossless on what matters, drop the bloat, move just-the-findings between small models" — is ELAI's RV-2 doctrine ("lossless store / lossy view"). The record-side losslessness is real and WIRED (hash-verified recover_stubs). What is missing is the PROOF that the lossy VIEW preserved the decision-relevant facts: there is no gate asserting the must-preserve atoms survived a trim/handoff, and the compaction safety check only WARNs. The independent deep-research report (PART 6, tier-2/WEAK — corroboration, never a GRC grounding source) re-derives ELAI's design feature-for-feature (hybrid state ledger = structured handoff + retrieval pointers + commitment atoms; deterministic-not-generative) and names exactly the missing piece: Critical Atom Recall / round-trip recoverability as a deterministic gate. R2 adds precisely that. Distillation stays STRUCTURAL (typed belief-state + symbol pointers), NEVER generative — soft/latent compression (AutoCompressor 2305.14788, ICAE 2307.06945) is cross-model-INCOMPATIBLE and model-bound (PART 5, paper-verified), the same disqualification ELAI applied to LatentMAS/RecursiveMAS.

## Existing state (code-grounded 2026-06-25, CORRECTS stale map labels — W-7 anchors, executor re-greps)
- session_dag.rs:1539 recover_stubs() [WIRED] — trimmed msgs → stubs {ssb_seq, summary_hash}; recovery asserts blake3(payload)==summary_hash, else StubRecoveryFailed. Byte-identical hash-verified recovery against the journal. 1 prod caller (live_cli.rs) + recover_single_stub.
- compact.rs:545 check_lossless_compaction() [PARTIAL, WARN-only] — H-49 RV-2 fail-closed PRECONDITION counts unrecoverable (no SSB stub); is_safe = unrecoverable==0, but it tracing::warn's instead of BLOCKING. R1 flips this.
- belief_state.rs BeliefState + belief_state_event [BUILT, map said PLANNED] — BeliefState::new wired at escalation_waterfall.rs:239; i11_belief_state_contract.rs. R2 extends the schema.
- pristine_path_filter.rs [BUILT, map said PLANNED] — i5_pristine_context_contract.rs asserts pristine_rebuild_zero_traceback_bytes + emits ContextPruned + a contrastive constraint. The rebuild R2 gates for atom survival.
- G-2 rra.rs verify_rra [WIRED]; SSB append-only journal [WIRED] (the ground-truth record).

## Reads (mandatory before coding)
1. _grounding/K-GROUNDING-MAP.md; workflow/reasoning/context-compression-losslessness-2026-06-25.md PART 7 + PART 8 (R1/R2) + PART 6 (Context Codec atoms, tier-2).
2. compact.rs (check_lossless_compaction); session_dag.rs (recover_stubs); belief_state.rs + escalation_waterfall.rs (BeliefState handoff); pristine_path_filter.rs (the rebuild).
3. F-0_INDEX W-1 (missing evidence = FAIL); G-0 GD-1/GD-2/GD-5; H Track-8 RV-1..RV-4; I-0 IC-2/IC-3 (deterministic reversible only); CLAUDE.md blake3-only/no-.unwrap.

## Tasks (WAVE KW-3 — enforce the record, prove the view)
1. R1 — fail-close check_lossless_compaction (~30 LOC): when unrecoverable_count>0, force the SSB capture-hook to run first; if still >0, REFUSE the trim (Err, not warn). wiring R1 missing=FAIL. NO behavior change when unrecoverable==0. Honest residue: external side-effects stay surfaced from the saga log (RV-4), never auto-reverted.
2. R2 — extend BeliefState with critical_atoms (~40 LOC): add critical_atoms: Vec<CriticalAtom{kind: AtomKind, value: String, must_preserve: bool}> to the I-11 schema; populate from the handoff source (the orchestrator declares them, TD-3 — the LLM never sets must_preserve). NO new belief machinery; extend the existing struct.
3. R2 — the faithfulness gate (~60 LOC): after a handoff / pristine-rebuild / trim, for every atom with must_preserve=true assert exact string/symbol presence in the projected context, verified against the SSB record; missing = FAIL (refuse + emit). Exact-match only — NEVER semantic similarity. Deterministic, LLM-free.
4. Wire R1 into the live compaction path and R2 into escalation_waterfall.rs:239 + the pristine-rebuild seam (~25 LOC). Default-ON behind ELAI_LOSSLESS_GATE_DISABLE (W-2), OBSERVE-then-enforce (GD-4: bound the FP rate over >=3 real runs before BLOCK-by-default).
5. workflow/reasoning/K-12-decisions.md.

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/compact.rs — check_lossless_compaction returns Err on unrecoverable>0 (fail-closed, R1)
  - crates/orchestrator/src/belief_state.rs — CriticalAtom{kind,value,must_preserve} on BeliefState (R2 schema extension)
  - crates/orchestrator/src/handoff_faithfulness.rs — the Critical-Atom-Recall gate (exact-presence assert over the SSB record)
consumes_symbols:
  - session_dag.rs::recover_stubs (the hash-verified record — COMPOSE, never re-implement)
  - belief_state.rs::BeliefState + escalation_waterfall.rs:239 (the handoff seam); pristine_path_filter.rs (the rebuild); rra.rs::verify_rra (round-trip check)
reachable_from:
  - the live compaction path in live_cli.rs → check_lossless_compaction (PRODUCTION compaction)
  - escalation_waterfall.rs:239 BeliefState handoff → handoff_faithfulness gate (PRODUCTION local→cloud handoff)
  - a #[cfg(test)] / contracts/ caller NEVER satisfies this (R7)
stub_ban:
  - NO generative/LLM summarization (HD-2) and NO ciphers/abbrev substitution (IC-3) — deterministic only
  - NO semantic-similarity match (exact string/symbol presence ONLY; "semantic similarity is not lossless")
  - NO LLM setting must_preserve (TD-3 — orchestrator declares it)
  - NO re-implementation of recover_stubs / BeliefState / verify_rra (GD-1 P0); NO .unwrap()/.expect() on prod; blake3 not sha2
frontend_pair:
  N/A with justification — the gates are runtime fail-closed checks with no dashboard surface of their own; their verdicts already flow to the SSB the existing context/trust panels read. Justified.
observable_effect:
  trigger: "drive a compaction that would drop an un-archived message + a local→cloud handoff that omits a must_preserve atom (integration smoke, R10, on a HARD multi-turn input)"
  expect_ssb: "+1 HandoffFaithfulnessGated (Pattern-2; Kind: AtomVerified / AtomMissing / CompactionRefused) OR reuse BeliefStateEvent +Kind; baseline+0..1 from LIVE ~267/577 (re-grep, W-9)"
  expect_artifact: "the compaction with unrecoverable>0 is REFUSED (Err, not warn); NEGATIVE arm: a handoff that drops the bug-location atom MUST FAIL (not pass with a warning)"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/K-plan/K-12_losslessness_enforcement_handoff_faithfulness_gate.md --run-trigger
rg "must_preserve|CriticalAtom" .elai_cc/crates/orchestrator/src/belief_state.rs          # R2 schema present
rg "Err\(|refuse|unrecoverable" .elai_cc/crates/orchestrator/src/compact.rs               # R1 fail-closed path
rg -c "tracing::warn.*unrecoverable" .elai_cc/crates/orchestrator/src/compact.rs          # MUST be 0 (warn replaced by Err)
rg -ci "semantic.*similar|cosine" .elai_cc/crates/orchestrator/src/handoff_faithfulness.rs # MUST be 0 (exact match only)
test -f workflow/reasoning/K-12-decisions.md
```

## ACCEPTANCE
- [ ] R1: check_lossless_compaction REFUSES (Err) when unrecoverable>0; no warn-and-proceed remains.
- [ ] R2: every must_preserve atom is asserted present by exact match against the SSB record; missing=FAIL.
- [ ] HD-2/IC-3: no generative summarization, no ciphers; distillation stays structural.
- [ ] TD-3: the LLM never sets must_preserve and never adjudicates the gate.
- [ ] GD-1: recover_stubs/BeliefState/verify_rra consumed, not re-implemented.
- [ ] W-2: default-ON behind ELAI_LOSSLESS_GATE_DISABLE; GD-4 FP bound on >=3 runs before BLOCK-by-default.
- [ ] cargo check --workspace GREEN; auditor PASS on -p orchestrator. No new dependency.

## WIRING DONE-GATE (clause by clause)
- W-1: done = the integration smoke refuses an unsafe compaction + fails an atom-dropping handoff (not cargo-green).
- W-2: kill-switch ELAI_LOSSLESS_GATE_DISABLE; observe-then-enforce.
- W-3: extends BUILT code (compact.rs/belief_state.rs), not a summary-only port.
- W-4: frontend_pair N/A (justified above).
- W-5/W-10: in-process at gate time; no network leg.
- W-6: N/A (no dashboard surface); justified.
- W-7: every anchor re-grepped at edit (the map's I-5/I-11/H-50 PLANNED labels are stale — code exists).
- W-9: SSB prefer-reuse BeliefStateEvent or +1, named from the LIVE baseline.
- GD-2: the gate verdict is recomputable from the SSB journal under ELAI_SEED.
- GD-5: cloud reproducibility stays action-stream; the atom gate works on the action record, not tokens.

## Tests to add
- crates/contracts/tests/k12_losslessness_gate_contract.rs (~LOC): (a) compaction with unrecoverable>0 returns Err (not warn); (b) a handoff carrying a must_preserve bug-location atom present in the projection PASSES; (c) a handoff that drops that atom FAILS; (d) the gate uses exact match (a paraphrase of the atom does NOT satisfy it); (e) recover_stubs hash-mismatch still raises StubRecoveryFailed (composition unbroken); (f) ELAI_LOSSLESS_GATE_DISABLE downgrades to observe.

## Files touched
- MODIFY: .elai_cc/crates/orchestrator/src/compact.rs (~30 LOC: R1 fail-close)
- MODIFY: .elai_cc/crates/orchestrator/src/belief_state.rs (~40 LOC: CriticalAtom schema)
- CREATE: .elai_cc/crates/orchestrator/src/handoff_faithfulness.rs (~80 LOC: the gate)
- MODIFY: .elai_cc/crates/orchestrator/src/escalation_waterfall.rs + live_cli compaction wire (~25 LOC)
- CREATE: crates/contracts/tests/k12_losslessness_gate_contract.rs (~80 LOC)
- CREATE: workflow/reasoning/K-12-decisions.md

## Citations / Grounding
- context-compression-losslessness-2026-06-25.md PART 7 (recover_stubs blake3-verified; check_lossless_compaction WARN-only; I-5/I-11/H-50 BUILT) + PART 8 R1/R2 + PART 6 Context-Codec critical-atom measurement (tier-2/WEAK, corroboration only) + PART 5 (AutoCompressor 2305.14788 / ICAE 2307.06945 cross-model-incompatible, paper-verified — why distillation stays structural).
- ELAI ground truth: session_dag.rs:1539, compact.rs:545, belief_state.rs + escalation_waterfall.rs:239, pristine_path_filter.rs, rra.rs (all re-grepped at edit, W-7).
- CLAUDE.md: blake3-only, no .unwrap on prod, deterministic-reversible-only (IC-3), HD-2 no-resummarization, TD-3 LLM-advisory.

## Open questions / risks
- Atom EXTRACTION (who declares critical_atoms): the orchestrator computes them deterministically from the task/objective + DKG symbols (TD-3); the bug location comes from the I-5 contrastive constraint + the failing test's anchor. Decide the exact source set at authoring; OBSERVE-first to bound false-FAILs (GD-4).
- R1 interaction with H-50 local window-minimization: the local lane minimizes aggressively (context_policy.rs); the fail-close must force-archive before the minimize trim, not block legitimate minimization. Verify against the live trim path.
- MAP FIX (apply at upsert): I-5/I-11/H-50 PLANNED→PARTIAL/BUILT (code + contracts exist; FINISH-grade wiring audit pending — that audit is the context-compression R4 rider, not K-12).

## Next
K-12 turns ELAI's "lossless on what matters" from doctrine into an enforced, proven property: the record is fail-closed lossless and the view carries a deterministic faithfulness proof. Pairs with J-34 (the measured per-model degradation knee = the measurement spine under R2) and J-72 (in-domain retriever adaptation keeps the re-fetch accurate). Closes the context-compression run's two real gaps; the rest (R3/R4) is activation/measurement, not invention.

End of K-12.
