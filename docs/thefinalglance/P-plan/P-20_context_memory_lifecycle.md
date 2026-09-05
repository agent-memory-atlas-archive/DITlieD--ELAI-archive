# P-20: context/memory lifecycle hardening (pre-compaction flush + on-demand index)

STATUS: PLANNED (WAVE OC-1; EXTENSION of H-50/H-51 + H user-memory + L familiarity + H MCP defer-load; PROOF_SPINE: none; execute after the proof spine per A-7)

- Track OPENCLAW-STEAL | lever no-context-lost-uncached | grounding STRONG (openclaw flush + on-demand index source-verified; H anchors PLAN-asserted, executor re-greps W-7) | nature EXTENSION | effort M | WAVE OC-1 | PROOF_SPINE none
- Premise: ELAI's context spec is strong on invariants, thin on lifecycle mechanics. Add a pre-compaction memory-flush (ordered, journaled), a versioned on-demand skill/rules index, Dreaming-style promotion-gate prior art, and the per-provider cache-control mapping table.

## Context — why this matters
Grounded in workflow/reasoning/openclaw-vs-elai-comparison-2026-07-11.md 2.6 + 2.7 + 2.8 + steals S3/S7/S16/S18. openclaw: a PRE-COMPACTION MEMORY FLUSH (a silent NO_REPLY turn writes durable notes BEFORE compaction, once per cycle, soft threshold 4000 tokens); a self-built system prompt with a byte-identical stable/volatile cache boundary; SKILL.md dirs with on-demand loading (the prompt carries only a compact index: name/description/path + a sha256 VERSION marker ~24 tokens/skill; the model reads the file when needed, re-reads on version change; budget compact-degrade); Dreaming (opt-in background consolidation promoting short-term recall through score/recall/diversity gates); cacheRetention mapped per provider (Anthropic cache_control 5m/1h; OpenAI prompt_cache_key; Gemini cachedContents — tested knowledge of what each provider actually honors). The flush invariant: "no compaction may destroy context that was never offered to durable memory."

## Existing ground truth (B..Q coverage; PLAN-asserted; executor re-greps W-7)
- H-50/H-51 context lifecycle + sleep-time handoff (H-SUMMARY_INDEX.md:885): regime-aware in-session context + cross-session grounded topic-digest. The flush is IN-session, distinct from H-51's cross-session handoff. EXTEND.
- H two-tier user memory (H-SUMMARY:500): bi-temporal preferences + validate-before-use facts — the flush TARGET.
- L familiarity surface (L-SUMMARY): user-authorable rules/hooks/skills — the on-demand index consumer.
- H MCP defer-loading: the same lazy-load pattern — ONE mechanism for skills + rules + MCP.
- H cache-stable layout + 1h TTL: the per-provider cache-control table implements this thesis.

## Steal/build (REVISED scope, W-10 tiered)
- SHIP NOW (Tier-1): (a) a pre-compaction memory-flush as an ORDERED journaled step (writes durable notes BEFORE compaction, once per cycle) — the invariant "no compaction destroys context never offered to durable memory"; (b) a versioned on-demand skill/rules index (compact prompt index: name/description/path + a sha256 VERSION marker; read-file-on-need; re-read on version change; budget compact-degrade).
- DEFER (Tier-2): Dreaming's promotion gates (score/recall/diversity) as prior art for H memory governance; the per-provider cache-control mapping table into H caching.
- EXCLUDED as vaporware: a second memory store (ride H's two-tier memory; A-1/N-2); a flush that fires AFTER compaction (defeats the invariant).

## Reads (mandatory before coding)
- workflow/reasoning/openclaw-vs-elai-comparison-2026-07-11.md (2.6/2.7/2.8; steals S3/S7/S16/S18)
- docs/thefinalglance/H-plan/H-51_*.md (context lifecycle), H user-memory unit, L familiarity, H MCP defer-load
- crates/orchestrator/src/ (the compaction path + the skill/rules index loader)

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/context/precompact_flush.rs (an ordered flush hook: durable notes written BEFORE the compaction row)
  - crates/orchestrator/src/context/ondemand_index.rs (versioned skill/rules index: sha256 marker + read-on-need + budget-degrade)
consumes_symbols:
  - crates/orchestrator/src/context/lifecycle.rs::CompactionPass (H-51; EXTEND with the flush ordering — do NOT re-implement)
  - crates/orchestrator/src/memory/user_memory.rs::DurableMemory (H two-tier memory; the flush target)
reachable_from:
  - crates/orchestrator/src/context/lifecycle.rs (the compaction path invoking the flush before compacting) [PRODUCTION compaction path]
  - a #[cfg(test)] or contracts/ caller NEVER satisfies this (R7)
stub_ban:
  - NO compaction that fires before its flush (the flush is an ordered pre-step; a post-compaction flush is a FAIL)
  - NO full skill body loaded into the prompt before the skill is invoked (compact index until needed)
  - NO second memory store (ride H's two-tier memory; A-1/N-2)
  - NO .unwrap()/.expect() on the context path; NO blake2/sha2 (blake3/sha256 for the version marker per the source contract)
frontend_pair:
  N/A with justification: internal context machinery; no user surface beyond the existing cost/memory panels (W-6 N/A). Justified.
observable_effect:
  trigger: "elai session drive-to-compaction --fixture tests/fixtures/context/long_session.json"
  expect_ssb: "memory_flush_event"
  expect_artifact: "a memory_flush_event row lands BEFORE the compaction row (ordered), carrying the durable notes; the skill index shows only compact entries until a skill is needed then the file is read once (trace); NEGATIVE: a compaction that fires before its flush, or a full skill body loaded before the skill is invoked, FAILS"

## Wargame (plan-standard unit shape)
EXPECTED_OBSERVATION: driving a session to the compaction threshold lands a memory_flush_event row BEFORE the compaction row (ordered), carrying durable notes; the skill index shows only compact entries until a skill is invoked, then that file is read exactly once (trace).
ON_FAIL: the flush lands after compaction, or a full skill body is preloaded (CAUSE: the flush hook is not ordered ahead of the compaction pass, or the index eager-loads bodies). COUNTER-MOVE: sequence the flush strictly before CompactionPass; make the index carry only name/description/path + sha256 marker; the negative tests (post-flush / eager-load) must FAIL.
FORKS: none (the flush is an in-session ordered step; the cross-session handoff is H-51's separate concern and is not duplicated here).
RECON_NEEDED: confirm the exact H-51 CompactionPass + H DurableMemory symbols to extend + the openclaw flush threshold shape. Settling check: `grep -n "CompactionPass\|DurableMemory" crates/orchestrator/src/` + source-read openclaw pre-compaction flush.
ABORT: if the flush would require a SECOND durable memory store (A-1/N-2 violation), STOP and flag — extend H's two-tier memory, never fork.
VERIFY: the load-bearing proof is the off-agent memory_flush_event row ordered before the compaction row, driven by the real session compaction path. Run:
```bash
python .claude/tools/plan_gate_check.py --file docs/thefinalglance/P-plan/P-20_context_memory_lifecycle.md   # authoring gate (exit 0)
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/P-plan/P-20_context_memory_lifecycle.md   # execution done-gate
```
REDTEAM: surviving-attack seed — a session that would silently lose late context to compaction is defeated by the ordered flush (the flush is a required pre-step; no compaction proceeds without it). Attack that would succeed if unpatched: a huge skill corpus bloating the prompt — PATCH: the on-demand index keeps ~24 tokens/skill until invocation; the body loads once on need. Deep red-team DEFERRED to execution pre-flight.

## INTEGRATION_SMOKE
An independent agent drives a real long multi-turn session past the compaction threshold TWICE on a HARD input (tool-heavy history), asserting: each compaction is preceded by a flush row, no tool-call/result pair is split, and the on-demand index loads a skill body only on first invocation. Toy short session does NOT satisfy R10. (Composes with P-19's compaction split-rule.)

## Grounding / citations
- Source: workflow/reasoning/openclaw-vs-elai-comparison-2026-07-11.md 2.6/2.7/2.8, steals S3/S7/S16/S18 (openclaw flush + index source-verified there).
- ELAI ground truth (PLAN-asserted, executor re-greps W-7): H-50/H-51, H user-memory, L familiarity, H MCP defer-load per the indexes cited.
- CLAUDE.md hard rules: no second memory store (A-1/N-2); MCP defer-loading (one lazy-load mechanism); blake3; no .unwrap on prod.

## PRODUCTION_HOOK
crates/orchestrator/src/context/lifecycle.rs — the compaction path invokes precompact_flush before CompactionPass; the on-demand index loader replaces eager skill/rules loading on the prompt-build path. Both wired in this changeset.

## Next
Closes the context-lifecycle mechanics gap (openclaw ships the ordering; ELAI has the invariants). Pairs with P-19 (compaction split-rule) and H-51 (cross-session handoff, the flush's out-of-session sibling).

End of P-20.
