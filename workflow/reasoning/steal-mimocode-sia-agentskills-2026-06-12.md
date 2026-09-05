STEAL ANALYSIS - MiMoCode + sia + agent-skills - 2026-06-12
Run: paper-to-plan, 3 repos, focus=MiMo-Code (harness peer).
Sources: docs/.source/MiMo-Code-INDEX.md (+ workflow/papers/mimo-code-part-1..8.md,
  mimo-code-interface.md), docs/.source/sia-INDEX.md (+ sia-interface.md),
  docs/.source/agent-skills-INDEX.md (+ agent-skills-interface.md).
ELAI context: docs/thefinalglance/ELAI-FEATURE-MAP.md (2026-06-12, live SCHEMA 231/542).
Citations verified: arXiv:2605.27276 (sia paper) WebSearch-confirmed 2026-06-12
  (arxiv.org/abs/2605.27276, repo hexo-ai/sia is the paper's official code).
Reliability: all three reviewed source = MEDIUM.
Ledger check: REJECTED/ALREADY-TRIED ledger swept; no candidate below re-proposes a
  ledger entry. Adjacent ledger lines honored: B7 generator-side reviewer decline
  (CLAIM-strip below is auditor-side, allowed), conductor-oss workflow-engine cluster
  (MiMo QuickJS workflow runtime NOT proposed), Mem0/Zep platform-swap decline (only
  mechanics proposed), TD-3 LLM-never-load-bearing (goal/judge demoted to advisory),
  HD-3 no force-JSON on reasoning roles (structured-return only on op-emission).

================================================================
A. MiMo-Code (Xiaomi MiMoCode, opencode fork) - the focus repo
================================================================

A1. 11-section checkpoint template with per-section token budgets + dedup validator
  What: checkpoint.md schema (intent/next-action/directives/task-tree/current-work/
    files/discoveries/errors/live-resources/decisions/notes), ~11K total budget,
    section budgets 500-3000 tok, prior-titles dedup on discoveries;
    session/checkpoint-templates.ts, checkpoint.ts.
  ELAI: CROSS-SESSION SLEEP-TIME HANDOFF H-51 [PARTIAL] (handoff_digest.rs, tiered
    load ~20-27K->~1-3K) + REGIME-AWARE CONTEXT POLICY H-50 [PARTIAL].
  Verdict: EXTENSION(H-50/H-51). The typed-section + per-section-budget + dedup
    validator shape is sharper than H-51's free-form digest; directly upgrades
    handoff_digest.rs output contract.

A2. Section-aware budgeted read (skeleton always survives)
  What: budgeted-read.ts:65-118 - all section headers + index lines preserved
    unconditionally, body filled to budget in section order, file:line resumption
    hint, 0.95 safety factor.
  ELAI: H-51 tiered load; token-discipline anti-read-big is harness prose, not a
    runtime read primitive.
  Verdict: EXTENSION(H-51). The read primitive H-51's tiered loader should use.

A3. Tool-pair boundary guard before any compaction/trim boundary
  What: boundary.ts:34-77 pure backward walk; un-orphans tool_use/tool_result pairs,
    then same-message.id thinking-block siblings; zero I/O.
  ELAI: SESSION DAG 3-pass lossless trim [WIRED], E-25 ladder [DARK], H-6 masking
    [PARTIAL]. None of the audited descriptions state a tool-pair invariant walk.
  Verdict: EXTENSION(H-6/E-25/SessionDag). A correctness invariant every ELAI trim/
    mask boundary must run; cheap, deterministic, contract-testable.

A4. 4-level overflow pressure scale driving auto-compaction + protected classes
  What: compaction.ts - pressure 0/<50 1/50-70 2/70-85 3/>=85 of usable
    (= input_limit - min(20K, max_output)); PRUNE_PROTECT=40K tail; "skill" tool
    results never erased; recent-turn floor 25%.
  ELAI: H-50 fixes the flat-100K/64K gates and selects policy by BackendMode but has
    no graded pressure ladder; E-25 has 6 compaction tiers (response side).
  Verdict: EXTENSION(H-50). pressureLevel() becomes the trigger input to H-50's
    ContextPolicy; protected-class list maps to ELAI's skill/anchor slots.

A5. Microcompaction: clear tool-result blobs newer than boundary, never delete rows
  What: insertRebuildBoundary microcompacts COMPACTABLE_TOOL_NAMES output blobs
    in-place; DB rows survive.
  ELAI: RV-2 lossless-store/lossy-view (H-49) + SessionDag stubs-recoverable trim.
  Verdict: ALREADY-PLANNED(H-49)/ALREADY-SHIPPED(SessionDag). Confirmation only; no
    new unit.

A6. Background checkpoint-writer subagent + 1-slot newest-wins queue + ForkContext
  What: dedicated writer child session (only writer of checkpoint.md/MEMORY.md),
    tool-whitelisted, writes PARENT paths, pending queue evicts older requests
    (newest is superset), frozen (system, tools, watermark) ForkContext snapshot at
    spawn for prefix-cache reuse; checkpoint*.ts + actor/spawn.ts:56-92.
  ELAI: memory-writer agent (harness MEMORIZING) exists; H-51 authors digests; no
    runtime background-writer with newest-wins coalescing or fork-context snapshot.
  Verdict: EXTENSION(H-51). The writer-isolation + coalescing queue + snapshot are
    the runtime mechanics H-51's digest authoring lacks.

A7. Deterministic completion gate: task-DB reconciliation downgrades self-reported done
  What: TaskGate.decide() (actor/spawn.ts:402-437, task/gate.ts) - after finalText,
    query DB for non-terminal tasks owned by the actor; if found, one re-entry or
    downgrade reportedStatus to "partial" with incomplete IDs.
  ELAI: stub registry + Stop hook (harness); B-22 AutoAdvanceEvaluator EvidenceCheck;
    F-4 can't-lie gate (test outcomes). No runtime "open task rows veto agent-claimed
    DONE" check at agent-return time.
  Verdict: EXTENSION(B-22/anti-spiral seam). Deterministic, off-agent (DB rows), TD-3
    clean. High value per line of code.

A8. Goal/judge stop-condition via independent LLM call
  What: goal.ts evaluate() separate generateObject judge before honoring stop.
  ELAI doctrine: TD-3 LLM is advisory never load-bearing; D-13 bans same-model
    self-critique as a gate.
  Verdict: REJECT(doctrine) as a gate; the deterministic sibling A7 is the steal.
    At most an advisory lens the auditor already provides.

A9. FTS5 relative score floor + fingerprint reconcile
  What: memory/service.ts:52-133 - phrase-quoted OR tokens (CJK-safe), bm25 inverted,
    floor = 0.15*topScore (relative, corpus-size stable), 3x over-fetch cap 50;
    reconcile.ts:46-87 fingerprint = "{size}-{mtimeMs}", upsert-on-change,
    prune-on-delete.
  ELAI: USER MEMORY SUBSTRATE E-5 [PARTIAL], MEMORY HUB H-33 [PARTIAL], J-21 memory
    validation [PLANNED]; harness local-rag has its own index.
  Verdict: EXTENSION(E-5/H-33) - retrieval-mechanics rider: relative floor +
    fingerprint reconcile into crates/memory retrieval and the J-21 retrieval leg.
    NOT a platform swap (ledger-clean).

A10. Auto-dream/distill sleep-time consolidation gating
  What: auto-dream.ts:38-110 - enabled flag + project-age >= interval + last-session-
    title DB check + 10s spawn-gap; dream 7d consolidates MEMORY.md, distill 30d
    extracts skills from month of history (manual packaging, agent never self-edits
    skills at runtime - part-6 finding).
  ELAI: H-51 (sleep-time handoff) + I-12 evidence-gated playbook curator (promote on
    >=k SSB-verified outcomes -> skills export). I-12 is STRICTER (evidence-gated,
    not interval-gated).
  Verdict: ALREADY-PLANNED(H-51/I-12). Rider only: the interval + project-age +
    spawn-gap trigger guards are a sensible cadence layer for I-12/H-51 consolidation
    jobs; record, do not author a unit.

A11. Cache marker placement (4 markers incl. conversation midpoint)
  What: provider/transform.ts::applyCaching():260-329.
  ELAI: PROMPT CACHING [WIRED] + H-2 cache-stable layout 2 breakpoints [PARTIAL] +
    I-10 prefix-invariance guard [PLANNED].
  Verdict: ALREADY-PLANNED(H-2). Rider: evaluate midpoint + before-last-user marker
    positions inside H-2; no unit.

A12. Versioned event sourcing + seq-continuity fence (x-sync header causal wait)
  What: sync/index.ts typed projectors in DB tx, seq continuity on replay; fence
    middleware exposes per-aggregate seq to clients.
  ELAI: SSB single-writer broadcast-after-commit [WIRED]; I-3 throughput/checkpoint
    upgrade; J-22 payload-schema lock.
  Verdict: ALREADY-PLANNED(I-3/J-22). Rider: subscriber-side seq-continuity check
    (missing seq = BLOCK not skip) folds into J-22's writer-isolation contract test.

A13. Structured-output precedence over finalText for subagent returns
  ELAI: Tier-C agents already speak 15-message typed postcard IPC; harness agents are
    prose by R-5 design; HD-3 limits grammars to op-emission.
  Verdict: REJECT(covered by Tier-C IPC).

A14. Permission merge chain (compositional rulesets, last-match wins)
  ELAI: RULES ENGINE [WIRED] (rules as data, lifecycle, verdicts) + E-30 argv
    allowlist + per-agent whitelists.
  Verdict: REJECT(covered).

A15. QuickJS workflow sandbox / Effect.ts runtime / TUI / models.dev catalog /
  OAuth accounts / share sync
  Verdict: REJECT(not worth porting - synthesizer Section 5; conductor-oss cluster
    already covers workflow engines in the ledger).

================================================================
B. sia (hexo-ai) - self-improving harness loop, arXiv:2605.27276
================================================================

B1. Generation loop (meta writes agent -> subprocess runs task -> evaluate.py scores
  -> feedback rewrites)
  ELAI: DARWINIAN LOOP [DARK] + DGM VERIFICATION FILTERS H-43..H-48 [PARTIAL] +
    F-11 one config-only generation + GEPA E-8. ELAI's loop is stricter (frozen
    tests, traps, lineage scoring).
  Verdict: ALREADY-PLANNED/SHIPPED(H-43 family). Confirmation of the architecture;
    no unit.

B2. evaluate.py / results.json subprocess contract
  What: evaluator invoked `evaluate.py --gen-dir`, writes results.json INTO gen_dir,
    exit 0 = ran; missing results.json = evaluation did not run (fail-closed);
    EVALUATION_GUIDE.md + orchestrator.py:635-961.
  ELAI: H-43 execution-grounded fitness needs exactly this off-agent artifact shape;
    wiring-enforcement R1/R2 doctrine match (missing evidence = FAIL).
  Verdict: EXTENSION(H-43). Adopt the contract shape (subprocess + artifact-in-dir +
    exit code + missing=never-ran) as the fitness-evaluator ABI.

B3. Task-pack layout (data/public/task.md + evaluate.py + data/private/ + reference/)
  What: layout.py TaskLayout; language-agnostic directory convention; sia-format
    packs runnable by any orchestrator.
  ELAI: EVAL CORPUS E-54 [DARK] (ingest_task has no CLI entrypoint), D-6 held-out
    store, G-6 SAFEDELTA harness.
  Verdict: EXTENSION(E-54/H-43). Adopting the pack convention gives H-43 fitness
    tasks + E-54 ingest a concrete on-disk format and free interop with sia packs.

B4. Orchestrator-written append-only context.md from extracted metrics (off-agent)
  Verdict: ALREADY-SHIPPED doctrine match (off-agent artifact; SSB is stronger).
    Confirmation only.

B5. Weights mode (tinker RL), GRPO notes
  Verdict: ALREADY-REJECTED/DEFERRED (E-55 RL ladder, I-31 GRPO last; ledger).

B6. Execution-log size guards / trajectory preview truncation / multi-traj
  first-3-inline
  Verdict: REJECT(covered by existing truncation + budgeted-read steal A2).

================================================================
C. agent-skills (addyosmani) - skills collection
================================================================
(Analyzing for design steals only; NO skill import - any future import goes through
 /skill-audit + skill-imports.log per skill-import-policy.)

C1. simplify-ignore hook: model-invisible protected blocks
  What: hooks/SIMPLIFY-IGNORE.md - PreToolUse(Read) swaps annotated blocks for
    content-hashed placeholders before the model sees the file; PostToolUse(Edit|
    Write) restores; model edits around what it cannot see.
  ELAI: no equivalent anywhere (no-unsolicited-refactoring is prose-only); J-family
    harness hooks are the attach point (JD-1..JD-4, rag/hooks seam).
  Verdict: NEW (harness, J-family). Deterministic enforcement of "do not touch this
    block"; protects perf-critical/intentionally-nonstandard Rust+sidecar code.

C2. sdd-cache hook: ETag/304-revalidated WebFetch cache
  What: hooks/SDD-CACHE.md - HEAD with If-None-Match/If-Modified-Since; 304 = cache
    hit IS a fresh verification (not TTL staleness); never caches without validators;
    exit 2 = hit signal.
  ELAI: no WebFetch cache; grounded-by-default forces re-fetching the same docs.
  Verdict: NEW (harness, J-family). 304-not-TTL keeps grounding semantics while
    killing redundant fetches.

C3. CLAIM-stripped reviewer dispatch (doubt-driven-development)
  What: skills/doubt-driven-development/SKILL.md - reviewer receives ARTIFACT+
    CONTRACT, never the author's CLAIM; + doubt-theater detector (2 cycles zero
    actionable findings = validating not doubting); + 3-cycle cap.
  ELAI: auditor clean-context (H-3 rationale) + D-13 cross-family + I-21 zero shared
    context. None specify CLAIM-stripping as input shaping.
  Verdict: EXTENSION(auditor dispatch protocol, harness; echoes I-21). Amendment to
    how orchestrating skills compose auditor prompts.

C4. Out-of-scope mandatory line + checkable stop + non-yes rejection (interview-me)
  ELAI: /grill skill already does one-question interrogation; prompt-transform
    restates tasks.
  Verdict: EXTENSION(harness prompt-transform//grill) - rider-grade amendments.

C5. Fan-out 4-question validation checklist (orchestration-patterns.md)
  What: before parallel fan-out: independent? different KIND of finding per agent?
    merge fits main context? wait justifies parallelism?
  Verdict: EXTENSION(orchestrate/auto-orchestrate skills) - pre-dispatch gate line.

C6. "NOTICED BUT NOT TOUCHING" required disclosure in worker returns
  Verdict: EXTENSION(agent-dispatch-preamble R-5) - one-line amendment.

C7. Severity labels on review comments / browser trust-boundary / define-questions-
  first observability / source-driven-development / context trust taxonomy
  Verdict: REJECT(covered) - auditor BLOCK/WARN/INFO, G-1/F-3 taint, wiring-
    enforcement observability, grounded-by-default + source-tiers already cover.

================================================================
VERDICT TALLY
================================================================
NEW: 2 (C1 simplify-ignore, C2 sdd-cache - both harness J-family)
EXTENSION: 10 (A1,A2,A3,A4,A6,A7,A9 runtime; B2,B3 fitness/eval ABI; C3..C6 harness
  amendments counted as one composite)
ALREADY-SHIPPED/PLANNED (confirmation/rider only): A5,A10,A11,A12,B1,B4
REJECT: A8 (TD-3), A13/A14/A15, B5 (ledger), B6, C7

PLAN UNITS AUTHORED (fewest-files grouping):
  J-25 MiMoCode context+memory mechanics (A1+A2+A3+A4+A6+A7+A9; riders A5/A10/A11/
    A12 recorded inside)  [.elai_cc runtime]
  J-26 sia evaluator ABI + task-pack format (B2+B3; riders B1/B4)  [.elai_cc runtime]
  J-27 harness steals: protected-blocks hook + 304 web cache + dispatch amendments
    (C1+C2+C3+C4+C5+C6)  [.claude harness, JD evidence rules]
OUTCOME (2026-06-12): CREATED docs/thefinalglance/J-plan/
  J-25_mimocode_context_memory_mechanics.md,
  J-26_sia_evaluator_abi_task_packs.md,
  J-27_harness_protected_blocks_webcache_dispatch.md;
  J-0_INDEX.md + J-SUMMARY_INDEX.md updated to 27 sessions (RSW-1/RSW-2 waves, SSB
  bump sites, declines); ELAI-FEATURE-MAP.md upserted (3 PLANNED entries, J ledger
  line, 3-REPO STEAL VERDICTS ledger block, header date).
