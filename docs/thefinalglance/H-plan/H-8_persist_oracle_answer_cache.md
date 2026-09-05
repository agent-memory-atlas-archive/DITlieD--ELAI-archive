# H-8: Persist OracleAnswerCache — idempotent grounding answers survive restart

STATUS: PLANNED (WAVE 1; in-process; BUILD — persistence only; the smallest, most isolated session)

Objective: ELAI's OracleAnswerCache (oracle_cache.rs:173, blake3-keyed, tier-derived TTL) is wired into /research|/ask
but IN-PROCESS ONLY (C9): Mutex<CacheInner>, default cap 512, LOST ON PROCESS EXIT; only the OracleCacheHit/Miss SSB
rows persist, not the answers. H-8 disk-backs the cache so idempotent grounding answers survive a restart, and confines
durable caching to idempotent grounding queries ONLY (code-gen caching is a correctness hazard — subtle context
differences yield a wrong cached answer). PromptCache (prompt_cache.rs) ALREADY persists to disk
($CLAUDE_CONFIG_HOME/cache/prompt-cache) — the design "ephemeral" framing of PromptCache is WRONG; do NOT re-implement it (GD-1).

Estimated effort: ~0.5-1 session (~3-5 h). Pure in-process; no network at gate time (W-10 Tier-1 whole).
Pre-requisites: C-30 OracleAnswerCache SHIPPED (oracle_cache.rs); the /research|/ask consumer SHIPPED (repl_command.rs:286-459);
  PromptCache disk persistence SHIPPED (prompt_cache.rs:398 — the pattern to mirror).
Parallel-safe with: all of H-1..H-7, H-9 (disjoint files). 
Source: ELAI-EFFICIENCY-TRUST-DIRECTION-2026-06-03.md §10.8 (LEVER L8); _grounding/H-GROUNDING-MAP.md C9 + the H-8 table. Doctrine: H-0_INDEX.md.

## Context

GPT Semantic Cache (arXiv:2411.05276) cut API calls 61.6-68.8% — but on FACTUAL QA, not code generation. Local prefix
caching is CLOUD-IRRELEVANT (it only helps the cheap worker's latency, never the Sonnet bill). The one cloud-relevant
response-cache lever for ELAI is persisting the idempotent grounding answers across sessions, which a journal-backed
harness does naturally and an editor with no durable journal cannot. The hard boundary (GD-5, correctness hazard):
semantic / durable caching of CODE GENERATION is UNSAFE — confine it to idempotent grounding queries (/research, /ask).
OracleAnswerCache already keys on (query + project_root) via blake3 and derives TTL per-entry (T1=30d, T2=7d, T3=24h,
T4Rejected=never); H-8 just makes that survive a restart.

## Existing state (B/C/D/E/F/G coverage)
- OracleAnswerCache (orchestrator/src/oracle_cache.rs:173) — SHIPPED but IN-PROCESS ONLY. get :220 / insert :243 /
  evict_expired :272; blake3 CacheKey over query+project_root; Mutex<CacheInner>, default cap 512; LOST ON EXIT. Real
  caller REPL /research|/ask in repl_command.rs (lazy-init get_or_insert_with :286-291; hit :301 -> OracleCacheHit SSB
  + early return; insert :443 on miss -> OracleCacheMiss SSB).
- oracle_cache field (cli/src/live_cli.rs:574 Option<Arc<OracleAnswerCache>>; None at :802/:924/:1061) — the three CLI
  ctors set None as DELIBERATE lazy-init (instantiated on first /research|/ask via repl_command.rs:288), NOT wire-dark.
- PromptCache (providers/src/prompt_cache.rs:109) — SHIPPED, ALREADY DISK-BACKED. persist_state :398 (stats.json +
  session-state.json); write_completion_entry :406 ({hash}.json under base_cache_root :467 = $CLAUDE_CONFIG_HOME/cache/prompt-cache
  or $HOME/.elai/cache/prompt-cache). completion_ttl 30s, prompt_ttl 5min. The disk-persistence pattern to MIRROR (GD-1).
- cli.clock.now_secs — the deterministic TTL clock (C-15) the cache uses; H-8 must keep TTL deterministic on reload.

## Reads (mandatory before coding)
- _grounding/H-GROUNDING-MAP.md C9 + the H-8 symbol table.
- ELAI-EFFICIENCY-TRUST-DIRECTION-2026-06-03.md §10.8 (L8): persist OracleAnswerCache; confine to idempotent grounding;
  semantic caching of code-gen is UNSAFE.
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\oracle_cache.rs (full — the struct + get/insert/evict + the CacheKey + TTL tiers)
- F:\.ELAI_workflow\.elai_cc\crates\providers\src\prompt_cache.rs:390-470 (persist_state + write_completion_entry + base_cache_root — the pattern to mirror)
- F:\.ELAI_workflow\.elai_cc\crates\cli\src\repl_command.rs:280-460 (the live consumer + the lazy-init)
- CLAUDE.md HARD RULES: blake3 (the CacheKey), no .unwrap()/.expect() in prod; GD-5 (no fabricated guarantee).

## Tasks (WAVE 1 — disk persistence)

1. **Add disk load/persist to OracleAnswerCache (~80 LOC).** Mirror PromptCache::persist_state: on lazy-init, LOAD the
   persisted entries from a deterministic cache root (e.g. $CLAUDE_CONFIG_HOME/cache/oracle-cache or $HOME/.elai/cache/oracle-cache);
   on insert and on evict_expired, WRITE the entry set to disk. Keep the blake3 CacheKey + the tier-derived TTL; on
   reload, recompute expiry against cli.clock.now_secs (TTL stays deterministic, C-15). No .unwrap on the IO path
   (a corrupt/missing cache file degrades to an empty cache, never a panic).

2. **Add the code-gen confinement guard (~30 LOC).** Ensure ONLY idempotent grounding queries (/research, /ask) are
   durable-cached; a code-generation query is never served from the durable cache (the correctness hazard). The guard is
   a hard boundary, not a heuristic — code-gen requests do not reach get/insert on the durable cache.

3. **Decision doc workflow/reasoning/H-8-decisions.md (~80 LOC).** Records: in-process-only is the gap; PromptCache
   already persists (do NOT re-implement, GD-1); the confinement to idempotent grounding (GD-5 correctness hazard);
   the deterministic TTL-on-reload (C-15); the cache root.

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/oracle_cache.rs (disk load on init + persist on insert/evict + the cache-root path)
  - crates/orchestrator/src/oracle_cache.rs (the code-gen confinement guard)
consumes_symbols:
  - crates/orchestrator/src/oracle_cache.rs::OracleAnswerCache (get/insert/evict_expired/CacheKey)
  - crates/providers/src/prompt_cache.rs::persist_state + base_cache_root (the disk pattern to MIRROR — not re-implement)
  - crates/cli/src/repl_command.rs (the live /research|/ask consumer at :286-459)
reachable_from:
  - crates/cli/src/repl_command.rs:288 (lazy-init -> disk LOAD) ; :301 (hit -> OracleCacheHit) ; :443 (insert -> disk WRITE + OracleCacheMiss)  [PRODUCTION /research|/ask]
stub_ban:
  - NO re-implementation of disk persistence (mirror PromptCache::persist_state — GD-1)
  - NO durable caching of code-generation queries (correctness hazard — confine to idempotent grounding, GD-5)
  - NO .unwrap()/.expect() on the IO path (a corrupt cache file degrades to empty, never a panic)
  - NO non-deterministic TTL on reload (recompute against cli.clock.now_secs, C-15)
  - blake3 CacheKey; no blake2/sha2
frontend_pair:
  N/A with justification — OracleAnswerCache is a CLI-side response cache with no dashboard surface. (A cache-stats line
  in the /research output is the human-visible signal.) Justified.
observable_effect:
  trigger: "cd .elai_cc && cargo run -p elai-cli -- ask '<an idempotent grounding question>'  (process A, miss+insert),
    then EXIT and re-run the same: cargo run -p elai-cli -- ask '<the same question>'  (process B)"
  expect_ssb: "process B emits an OracleCacheHit SSB row for the same query (the answer survived restart) — where
    pre-H-8 it would have been a fresh OracleCacheMiss (the in-process cache was lost on exit)"
  expect_artifact: "the answer in process B is served from the persisted cache (no fresh grounding call); a code-gen
    query is NEVER served from the durable cache (the confinement guard); the cache file exists on disk under the cache root"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/H-plan/H-8_persist_oracle_answer_cache.md --run-trigger
rg "persist|write|base_cache_root|oracle-cache" .elai_cc/crates/orchestrator/src/oracle_cache.rs   # disk persistence added
rg "OracleAnswerCache" .elai_cc/crates/providers/src/prompt_cache.rs                                # MUST be 0 (PromptCache not re-purposed; mirror only)
rg "code.?gen|is_code_generation|confine|idempotent" .elai_cc/crates/orchestrator/src/oracle_cache.rs  # the confinement guard exists
rg -c "blake2|sha2" .elai_cc/crates/orchestrator/src/oracle_cache.rs                                # MUST be 0
test -f workflow/reasoning/H-8-decisions.md
```

## ACCEPTANCE
- [ ] W-1 done-gate: --run-trigger PASS (the answer survives restart — process B is an OracleCacheHit; a code-gen query is never durable-cached).
- [ ] W-2 default-ON: persistence active with no env override.
- [ ] W-3 production entrypoint: the load/persist fires from the live repl_command /research|/ask path; not test-only.
- [ ] GD-1: PromptCache disk persistence is MIRRORED, not re-implemented (one persistence pattern, two caches).
- [ ] GD-5: code-gen queries are never durable-cached (the correctness-hazard boundary).
- [ ] TTL is deterministic on reload (C-15); a corrupt cache file degrades to empty (no panic).
- [ ] cargo check --workspace GREEN; auditor PASS on -p elai-orchestrator -p elai-cli.
- [ ] No new dependency.

## WIRING DONE-GATE (clause by clause)
- W-1: --run-trigger is ACCEPTANCE 1; the disk load (produced) is consumed at lazy-init (W-8) on the live /research path (W-3).
- W-2: persistence default-on.
- W-3: repl_command.rs:288/301/443; not test.
- W-4: N/A own pair (CLI cache); justified.
- W-5: a runnable two-process ask -> a real cross-restart cache hit; in-process.
- W-6: N/A (no dashboard surface); justified.
- W-7: OracleAnswerCache / repl_command consumer / PromptCache::persist_state grep-confirmed 2026-06-03 (C9).
- W-8: the disk load/persist (produced) is consumed by the cache get/insert on the live path.
- W-9: NO new action_type (OracleCacheHit/Miss exist).
- W-10: Tier-1 in-process (file IO is local, not network/RunPod).
- GD-1/GD-5: mirror PromptCache; confine to idempotent grounding.

## Tests to add
- crates/contracts/tests/h8_oracle_persist_contract.rs (~130 LOC): (a) an insert is written to disk and survives a new
  OracleAnswerCache instance (the cross-restart simulation); (b) TTL is recomputed deterministically on reload against a
  fixed clock (a T3=24h entry inserted 25h ago is evicted on reload); (c) a corrupt cache file degrades to an empty cache
  (no panic); (d) a code-generation query is rejected by the confinement guard (never durable-cached); (e) the blake3 CacheKey
  is stable across restart for the same (query, project_root).

## Files touched
- MODIFY: crates/orchestrator/src/oracle_cache.rs (~110 LOC: disk load/persist mirroring PromptCache + the confinement guard)
- CREATE: crates/contracts/tests/h8_oracle_persist_contract.rs (~130 LOC)
- CREATE: workflow/reasoning/H-8-decisions.md (~80 LOC)

## Citations / Grounding
- ELAI-EFFICIENCY-TRUST-DIRECTION-2026-06-03.md §10.8 (L8): GPT Semantic Cache arXiv:2411.05276 (61.6-68.8%, factual QA
  ONLY); persist OracleAnswerCache; confine to idempotent grounding; semantic caching of code-gen is UNSAFE.
- ELAI ground truth (C9): OracleAnswerCache oracle_cache.rs:173 (in-process, lost on exit; tier-derived TTL); PromptCache
  prompt_cache.rs:398 (ALREADY disk-backed — the pattern to mirror); the lazy-init repl_command.rs:288.
- CLAUDE.md HARD RULES: blake3, no .unwrap()/.expect() in prod; GD-5.

## Open questions / risks
- The cache root must respect $CLAUDE_CONFIG_HOME / $HOME like PromptCache, and must be per-project (the CacheKey already
  includes project_root, but the FILE layout should not leak one project's grounding into another).
- Staleness: a 30-day T1 grounding answer can go stale (an API/library changed). The tier TTL is the mitigation, but the
  confinement to idempotent grounding (not code-gen) is the hard safety boundary; surface the cache age in the /research output.
- Cap eviction on reload: the in-memory cap is 512; on load, respect it (evict oldest) so a large on-disk cache does not blow memory.

## Next
This closes the L8 lever. The durable grounding cache pairs with H-5 (batch research) and H-6 (memory read-back) as the
deterministic-context-efficiency cluster. With H-1..H-9 shipped, the measured $/resolved-task tuple (the believe-the-number
control) is complete and pairs with G-6 SAFEDELTA.

End of H-8.
