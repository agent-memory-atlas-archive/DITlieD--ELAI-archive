# F-5: Cross-subsystem staleness-quarantine + bi-temporal contradiction cascade

Objective: lift SHARD's genuinely-unique-vs-all-rivals primitive — time-decay staleness with quarantine PLUS the cross-subsystem self-healing cascade `handle_stale_memory` (invalidate -> demote dependents -> broadcast) — into ELAI as a WIRED cross-subsystem cascade. E-5 already ships the bi-temporal mechanism inside `crates/memory/` (FactStore supersede + DecayScheduler + `audit_conflicts`); the genuine gap (per synthesis C5) is cross-subsystem SCOPE: when a memory fact is quarantined by the decay tick, NOTHING today propagates that staleness into the DKG (`concept_stale` is hard-coded DEFAULT 1, never populated) or into the Cosmos artifact worker. F-5 wires the memory decay tick to emit a `StalenessQuarantineBroadcast` SSB row that DKG + Cosmos consumers act on, so a quarantined fact demotes the concepts that cite it within one tick — and it makes that wiring a HARD done-gate (W-1) so the cascade cannot ship wire-dark.
Estimated effort: 2 sessions (~9-12 h), Tier-3 (governance + composition). Phase 1 (audit) is the prerequisite gate; Phase 2 (cascade) is the wired deliverable.
Pre-requisites: B-30 (DKG hardening — PREREQ for the DKG schema half: F-5 Phase 2 touches `dkg_nodes.concept_stale` population, which B-30 must have stabilised first). E-5 (memory crate bi-temporal: FactStore supersede + DecayScheduler + `audit_conflicts` + `MemoryFactSuperseded` SSB — F-5 EXTENDS this, does not re-implement it; the synthesis correction is explicit that ELAI is NOT missing bi-temporal). B-42 (Cosmos artifact decay — F-5's Cosmos consumer attaches to the existing `cosmos_worker.rs` staleness path, no new decay engine).
Parallel-safe with: F-6 (learned-skill library — both are Tier-3 governance; F-6 touches `crates/orchestrator/src/skill_trust.rs`, no overlap with F-5's memory/knowledge crates). NOT parallel-safe with any plan editing `crates/knowledge/src/dkg/mod.rs` concurrently (concept_stale population is the contended surface — coordinate the edit window with B-30).
Source synthesis: workflow/reasoning/VIBESEC-SHARD-STEAL-AND-BUILD-2026-06-01.md section 4 item C5 (mechanism + ELAI ground truth + REVISED SCOPE + measurable + the always-1 latent hazard); section 1 (corrections — ELAI is NOT missing bi-temporal, do not re-introduce that claim); section 3 (SHARD = blueprint, not adoption); section 5 (Tier-3 sequencing: after B-30, concept_stale retrieval audit FIRST).

## Context

Per synthesis C5: SHARD's staleness primitive is `staleness = (now - last_verified)/expected_lifetime`; flag at >1.0, quarantine at >2.0, then `handle_stale_memory` runs a cross-subsystem self-healing cascade (invalidate the stale node -> demote its dependents -> broadcast the invalidation so other subsystems re-rank). SHARD's `handle_stale_memory` was fetched real (composition.py, 2026-06-01); Graphiti's bi-temporal non-destructive contradiction invalidation was confirmed in `edge_operations.py` (arXiv:2501.13956 §2.2.3). The borrow target is the CROSS-SUBSYSTEM cascade, not the per-subsystem decay (ELAI already has three isolated decays: memory `decay_tick`, Cosmos `cosmos_worker` status, rules 90-day confidence decay).

ELAI ground truth (code-read this session, W-7 confirmed in elai.map + source):
- `crates/memory/src/decay.rs::decay_tick` (line 88) returns `DecayTickStats`; it already advances trust-decay per tick but emits NO cross-subsystem signal.
- `crates/memory/src/decay_scheduler.rs::spawn_decay_scheduler` (line 101) wraps `decay_tick` in a background loop with an `on_tick: Option<DecayTickObserver>` hook (line 105) — the exact, real attach point for a quarantine-broadcast emitter. Note: `spawn_decay_scheduler` is currently referenced only from a COMMENT (live_cli.rs:1783) and tests; the LIVE production decay path is the in-line `decay_tick` call at `live_cli.rs:1789` (post-reflect tick). F-5 wires the broadcast into the in-line path so it is reachable from a real entrypoint TODAY (W-3), and also extends the scheduler observer for runs that own a long-lived runtime.
- `crates/memory/src/contradiction.rs::audit_conflicts` (line 72) is the existing bi-temporal contradiction closer (auto_supersede path). F-5 reuses its verdict; it does NOT add a new contradiction detector.
- `crates/knowledge/src/dkg/mod.rs:36` — `concept_stale INTEGER DEFAULT 1` is hard-coded ALWAYS-1; the only writer is the INSERT at line 369 which always passes the default. This is the sharp latent hazard the synthesis flagged: a retrieval path may treat `concept_stale=1` as a constant and silently break when the first real `0`/`2` is written.
- `crates/knowledge/src/cosmos_worker.rs::status_for` / `CosmosStatus` (line 122/811) + `tick` — the Cosmos artifact staleness path F-5's second consumer attaches to.
- `crates/cli/src/dkg_lifecycle.rs::ensure_dkg_or_rebuild` (C-38, line 12) is the real production DKG entrypoint (wired at live_cli.rs:1722-1732), emitting via `crates/knowledge/src/knowledge_ssb_sink.rs::KnowledgeSsbSink` (the `emit_dkg_rebuild_started` pattern, line 191). F-5's DKG consumer rides this same sink.
- `crates/orchestrator/src/ssb.rs::MemoryFactSuperseded` (variant line 5320, payload line 15891) is the E-5 Pattern-2 reference shape F-5's new variant mirrors.

The genuine gap (C5, survived refutation): cross-subsystem SCOPE + a unified quarantine broadcast. NOT the bi-temporal mechanism (correction section 1: ELAI already ships bi-temporal — do not claim otherwise). The measurable: quarantine a fact -> within one tick, `concept_stale` flips on all DKG concepts citing it AND a grounding-cache miss on re-query, proven by a mock-clock contract test.

## Existing state (B/C/D/E coverage)

- E-5 (SHIPPED): `crates/memory/` bi-temporal FactStore supersede + `Mem0Dispatcher` A.U.D.N. + `DecayScheduler` + `audit_conflicts` auto_supersede + `MemoryFactSuperseded` SSB. SCOPED TO `crates/memory/` only — no cross-subsystem emit. F-5 EXTENDS (does not re-implement).
- B-30 (PREREQ): DKG hardening. F-5 Phase 2 population of `concept_stale` must land AFTER B-30 stabilises `dkg_nodes`. Confirmed contended surface: `crates/knowledge/src/dkg/mod.rs`.
- B-42 (SHIPPED): Cosmos artifact tier with isolated decay (`cosmos_worker.rs` status_for / CosmosStatus / tick). F-5's Cosmos consumer rides the existing status path; NO new decay engine (no new dep, per CLAUDE.md).
- C-38 (SHIPPED): `dkg_lifecycle.rs::ensure_dkg_or_rebuild` production DKG entrypoint + `KnowledgeSsbSink`. F-5 DKG consumer attaches here (W-7 confirmed).
- Rules crate: isolated 90-day confidence decay (out of F-5 scope; the synthesis lists it as a THIRD isolated decay, not a consumer for v1).
- DETAILED_SUMMARY_01:119 records `concept_stale` always-1 / never-populated. F-5 Phase 1 is the audit that de-risks populating it.

## Reads (mandatory before coding)

- `F:\.ELAI_workflow\workflow\reasoning\VIBESEC-SHARD-STEAL-AND-BUILD-2026-06-01.md` section 4 item C5 + section 1 + section 3 + section 5 (the mechanism, the corrections, the SHARD-is-blueprint framing, the Tier-3 sequencing)
- `F:\.ELAI_workflow\.elai_cc\crates\memory\src\decay.rs:40-120` (decay_tick + DecayTickStats — the per-tick surface to emit from)
- `F:\.ELAI_workflow\.elai_cc\crates\memory\src\decay_scheduler.rs:35-150` (spawn_decay_scheduler + DecayTickObserver — the observer attach point)
- `F:\.ELAI_workflow\.elai_cc\crates\memory\src\contradiction.rs:60-145` (audit_conflicts — reuse the verdict, do not re-detect)
- `F:\.ELAI_workflow\.elai_cc\crates\knowledge\src\dkg\mod.rs:25-76 + 360-380` (SCHEMA_SQL concept_stale + the INSERT writer — the always-1 hazard surface)
- `F:\.ELAI_workflow\.elai_cc\crates\knowledge\src\dkg_adapter.rs:60-150 + 440-465` (the DKG read/SELECT consumers — Phase-1 audit targets)
- `F:\.ELAI_workflow\.elai_cc\crates\knowledge\src\cosmos_worker.rs:90-145 + 800-820` (CosmosStatus / status_for — the Cosmos consumer attach point)
- `F:\.ELAI_workflow\.elai_cc\crates\cli\src\dkg_lifecycle.rs:1-120` (ensure_dkg_or_rebuild + KnowledgeSsbSink usage — the production DKG entrypoint)
- `F:\.ELAI_workflow\.elai_cc\crates\knowledge\src\knowledge_ssb_sink.rs:135-220` (KnowledgeSsbSink trait + emit_dkg_rebuild_started — the SSB emit pattern to extend)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\ssb.rs:5319-5321 + 15885-15898` (MemoryFactSuperseded variant + payload — the Pattern-2 reference)
- `F:\.ELAI_workflow\.elai_cc\crates\cli\src\live_cli.rs:1770-1810` (the in-line post-reflect decay_tick production caller — the W-3 entrypoint)
- `F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src\lib\drawers\KnowledgeDrawer.svelte` + `AntiStalenessDrawer.svelte` (the frontend consumers to extend)
- arXiv:2501.13956 Graphiti §2.2.3 (bi-temporal contradiction invalidation — PRIMARY, cited; ELAI already ships the equivalent, used here only as the cross-subsystem-cascade reference)
- SHARD composition.py `handle_stale_memory` (fetched 2026-06-01 per synthesis — the cascade shape; blueprint not adoption)

## Tasks

### PHASE 1 (audit — the always-1 retrieval hazard; lands BEFORE any population, gates Phase 2)

1. **Audit every `concept_stale` retrieval consumer (~0 LOC product code; a written audit + a contract test).** Grep the knowledge crate for every read of `dkg_nodes.concept_stale` and every SELECT over `dkg_nodes`; record in `workflow/reasoning/F-5-concept-stale-audit.md` which consumers assume `concept_stale=1` as a constant filter. Confirmed read surface so far: `crates/knowledge/src/dkg_adapter.rs` SELECTs (lines 63/140/441/451/462) + `crates/knowledge/src/dkg/mod.rs` SELECTs. Output: a per-consumer table {file:line, reads concept_stale Y/N, assumes-always-1 Y/N, ranking-affecting Y/N}.

2. **Add a retrieval contract test with `concept_stale=0` BEFORE any population logic lands (~120 LOC at `crates/contracts/tests/f5_concept_stale_retrieval_contract.rs`).** Build a fixture DKG with two nodes (one `concept_stale=1`, one `concept_stale=0`), run each audited retrieval path, and ASSERT the retrieval ranking/inclusion is correct (a `concept_stale=0` node must NOT silently drop from results, and a quarantined `concept_stale=2` node must demote, not vanish). This test MUST exist and pass against the CURRENT code (pre-population) so the Phase-2 population cannot silently change ranking — that is the whole point of Phase 1.

### PHASE 2 (the wired cascade — bounded fan-out, default-ON)

3. **Add `StalenessQuarantineBroadcastPayload` + `SsbActionType::StalenessQuarantineBroadcast` (~90 LOC in `crates/orchestrator/src/ssb.rs`).** Mirror the `MemoryFactSuperseded` Pattern-2 shape (variant at ssb.rs:5320, payload at ssb.rs:15891). Payload fields: `fact_id: String` (the quarantined fact), `staleness_ratio: f64` (>2.0 = quarantine), `dependents_demoted: u32` (DKG concepts whose `concept_stale` flipped this tick), `cosmos_artifacts_flagged: u32`, `tick_ms: u64`, `broadcast_id: String` (blake3 prefix of `fact_id || tick_ms` — blake3 ONLY per CLAUDE.md). Wire `sql_str`, the `from_str` parse arm, the `SsbPayload` variant, the SQL CHECK constraint, the golden-lock row, and a round-trip golden fixture. Single Pattern-2 variant (no multi-arm Kind needed — one broadcast shape); SCHEMA_VERSION baseline+1 (baseline 191 per CLAUDE.md; the golden file is at TOTAL_VARIANTS 470 today — bump named "baseline+1" / TOTAL_VARIANTS 470 -> 471).

4. **Emit the broadcast from the memory decay tick (~110 LOC: new `crates/memory/src/quarantine_broadcast.rs` + wiring delta).** New `pub fn collect_quarantined(stats: &DecayTickStats, store: &FactStore, now_ms: u128) -> Vec<QuarantineRecord>` returning facts whose `staleness_ratio > 2.0` this tick, reusing the existing decay math in `decay.rs` (NO new staleness formula — read the existing trust-decay + last_observed). New `QuarantineRecord { fact_id, staleness_ratio }`. Reuse `audit_conflicts` (contradiction.rs:72) verdict for the contradiction-driven quarantine case (invalidate-on-conflict, not just age) — do NOT add a second contradiction detector (synthesis correction: ELAI already has bi-temporal). The emitter is invoked from BOTH the in-line production tick (`live_cli.rs:1789`, the W-3 live path) and the `spawn_decay_scheduler` observer (decay_scheduler.rs:105) so long-lived runtimes also fire it.

5. **DKG consumer: populate `concept_stale` on broadcast (~120 LOC in `crates/knowledge/src/dkg/mod.rs` + a new `apply_staleness_broadcast` fn).** New `pub fn apply_staleness_broadcast(&self, broadcast: &StalenessQuarantineView) -> Result<u32, DkgError>` that, for each quarantined `fact_id`, finds DKG concepts citing it (via the `concept` column / `dkg_edges`) and UPDATEs `concept_stale=2` (quarantine) or `concept_stale=0`->`1`? — NO: set `concept_stale=2` for quarantined-citing concepts and close the contradicted concept edge bi-temporally (mark the edge row, non-destructive, mirroring `audit_conflicts`). Returns the count of demoted concepts. Wired into the C-38 production path: `ensure_dkg_or_rebuild` (dkg_lifecycle.rs) gains a post-rebuild hook that drains pending broadcasts; the live `KnowledgeSsbSink` carries the demotion count. CRITICAL: this is the population logic the Phase-1 contract test (Task 2) gates — it MUST NOT land until Task 2 is green against pre-population code.

6. **Cosmos consumer: flag artifacts on broadcast (~70 LOC in `crates/knowledge/src/cosmos_worker.rs`).** New `pub fn flag_on_staleness_broadcast(&mut self, broadcast: &StalenessQuarantineView) -> u32` that, for each quarantined fact whose artifact is cited by a Cosmos manifest entry, transitions that artifact's `CosmosStatus` to a stale state via the EXISTING `status_for` path (no new status enum variant — reuse the shipped Stale state). Returns the flagged count. Reuses the B-42 worker tick; no new decay engine.

7. **BOUNDED fan-out queue (~80 LOC in `crates/memory/src/quarantine_broadcast.rs`).** Per synthesis: a BOUNDED fan-out queue (explicit channel capacity) to prevent scheduler-tick pile-up under artifact churn. New `QuarantineBroadcastQueue` over a `tokio::sync::mpsc` with an explicit `capacity` const (named constant, no magic number — `const QUARANTINE_QUEUE_CAPACITY: usize = 256;`). On full: drop-oldest + emit a `dropped_count` field on the next broadcast (back-pressure visible, never silent — counterfactual guard: if the queue silently drops, a quarantine is lost and a stale concept stays live; the `dropped_count` makes loss observable). Uses `tokio::sync` (CLAUDE.md: tokio::sync::Mutex in async). NO new external dep.

8. **Frontend pair: extend `KnowledgeDrawer.svelte` + `commands_f5.rs` (~130 LOC svelte + ~90 LOC tauri).** New Tauri command `get_staleness_quarantine` (in new `crates/dashboard-v2/src-tauri/src/commands_f5.rs`, following the `commands_e5.rs::get_memory_facts` pattern — `#[tauri::command] pub async fn`) returning a typed `StalenessQuarantineDto { broadcasts: Vec<...>, dependents_demoted, cosmos_flagged }`. `KnowledgeDrawer.svelte` (or a new `StalenessQuarantineDrawer.svelte` sibling of `AntiStalenessDrawer.svelte`) consumes it via `invoke<StalenessQuarantineDto>('get_staleness_quarantine')` (import from `'@tauri-apps/api/core'` per MemoryFactsDrawer.svelte:14), rendering the quarantine list + dependent-demotion counts. Register the command in `src-tauri/src/lib.rs` invoke_handler. `cross_lang_diff.py` MUST pass (Rust DTO <-> TS type parity).

9. **CLI entrypoint: `elai dkg staleness-tick` subcommand (~60 LOC in `crates/cli/src/dkg_lifecycle.rs` or the dkg CLI surface).** A real runnable production command that forces one decay tick, drains the bounded queue, applies the DKG + Cosmos consumers, and prints the demotion summary — this is the `observable_effect.trigger` (W-5 live end-to-end). It calls the SAME production code path the background tick uses (not a mock).

10. **Decision doc at `workflow/reasoning/F-5-decisions.md`** covering: (a) ELAI is NOT missing bi-temporal — F-5 adds cross-subsystem SCOPE only (synthesis correction 1, do not regress); (b) the always-1 concept_stale hazard + why Phase 1 audit gates Phase 2; (c) bounded queue capacity choice + drop-oldest-with-visible-count rationale; (d) default-ON with a kill-switch env var only (`ELAI_STALENESS_CASCADE_KILL=1` disables; default wired ON per W-2); (e) reuse of `audit_conflicts` instead of a second contradiction detector; (f) SHARD = blueprint, the cascade shape borrowed, not the code.

## WIRING_CONTRACT

produces_symbols:
  - crates/orchestrator/src/ssb.rs::StalenessQuarantineBroadcast
  - crates/orchestrator/src/ssb.rs::StalenessQuarantineBroadcastPayload
  - crates/memory/src/quarantine_broadcast.rs::collect_quarantined
  - crates/memory/src/quarantine_broadcast.rs::QuarantineBroadcastQueue
  - crates/memory/src/quarantine_broadcast.rs::QuarantineRecord
  - crates/knowledge/src/dkg/mod.rs::apply_staleness_broadcast
  - crates/knowledge/src/cosmos_worker.rs::flag_on_staleness_broadcast
  - crates/dashboard-v2/src-tauri/src/commands_f5.rs::get_staleness_quarantine
consumes_symbols:
  - crates/memory/src/decay.rs::decay_tick
  - crates/memory/src/decay_scheduler.rs::spawn_decay_scheduler
  - crates/memory/src/decay_scheduler.rs::DecayTickObserver
  - crates/memory/src/contradiction.rs::audit_conflicts
  - crates/knowledge/src/knowledge_ssb_sink.rs::KnowledgeSsbSink
  - crates/cli/src/dkg_lifecycle.rs::ensure_dkg_or_rebuild
  - crates/knowledge/src/cosmos_worker.rs::status_for
  - crates/orchestrator/src/ssb.rs::MemoryFactSuperseded
reachable_from:
  # in-line post-reflect production tick at live_cli.rs:1789; collect_quarantined -> StalenessQuarantineBroadcast SSB row
  - crates/cli/src/live_cli.rs::decay_tick
  # C-38 production DKG entrypoint; queue drain -> apply_staleness_broadcast -> concept_stale UPDATE
  - crates/cli/src/dkg_lifecycle.rs::ensure_dkg_or_rebuild
  # Tauri command reachable from KnowledgeDrawer (NEW file, created by F-5)
  - crates/dashboard-v2/src-tauri/src/commands_f5.rs::get_staleness_quarantine
stub_ban:
  - "TODO: wire staleness cascade" literal absent
  - "// cross-subsystem later" literal absent
  - No default-OFF terminal state: the cascade default MUST be ON; `ELAI_STALENESS_CASCADE_KILL` is a kill-switch only (W-2). A `default-off` or `_ENABLE=1`-gated terminal state for this feature is a BLOCK.
  - No LLM-bypass branch: the quarantine decision is deterministic (staleness ratio + audit_conflicts verdict); no prompt/model call may override or disable the cascade.
  - No silent queue drop: QuarantineBroadcastQueue drop-oldest MUST surface a `dropped_count`; a silent drop is a BLOCK.
  - No blake2/sha2 for broadcast_id (blake3 only, CLAUDE.md).
frontend_pair: crates/dashboard-v2/src/lib/drawers/KnowledgeDrawer.svelte (Svelte consumer, renders quarantine list + dependent-demotion counts) <- crates/dashboard-v2/src-tauri/src/commands_f5.rs::get_staleness_quarantine (Tauri command)
observable_effect:
  trigger: "cd .elai_cc && cargo run -p elai-cli -- dkg staleness-tick --workspace ."
  expect_ssb: "staleness_quarantine_broadcast"
  expect_artifact: "F-5-staleness-tick-*.json glob OR stdout matching /dependents_demoted: \\d+/ with concept_stale flipped to 2 on >=1 dkg_nodes row"

## DETERMINISTIC_CHECK

```bash
# W-1 DONE-GATE (the literal gate line — must return PASS / exit 0):
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/F-plan/F-5_staleness_quarantine_cascade.md --run-trigger

# Produced symbols exist as DEFs
rg "fn collect_quarantined|struct QuarantineBroadcastQueue|struct QuarantineRecord" .elai_cc/crates/memory/src/quarantine_broadcast.rs
rg "fn apply_staleness_broadcast" .elai_cc/crates/knowledge/src/dkg/mod.rs
rg "fn flag_on_staleness_broadcast" .elai_cc/crates/knowledge/src/cosmos_worker.rs
rg "fn get_staleness_quarantine" .elai_cc/crates/dashboard-v2/src-tauri/src/commands_f5.rs

# SSB action type + payload + golden lock
rg "StalenessQuarantineBroadcast\b" .elai_cc/crates/orchestrator/src/ssb.rs
rg "staleness_quarantine_broadcast" .elai_cc/crates/orchestrator/src/ssb.rs
rg "staleness_quarantine_broadcast" .elai_cc/crates/orchestrator/tests/action_type_lock.golden

# Consumed anchors are real (W-7 — Rust symbols confirmed in elai.map; concept_stale confirmed via source grep only, it is a SQL schema string not a Rust symbol)
rg "pub fn decay_tick" .elai_cc/crates/memory/src/decay.rs
rg "pub fn spawn_decay_scheduler" .elai_cc/crates/memory/src/decay_scheduler.rs
rg "pub fn audit_conflicts" .elai_cc/crates/memory/src/contradiction.rs
rg "concept_stale" .elai_cc/crates/knowledge/src/dkg/mod.rs

# Phase-1 retrieval audit + the pre-population contract test exist
test -f workflow/reasoning/F-5-concept-stale-audit.md
test -f .elai_cc/crates/contracts/tests/f5_concept_stale_retrieval_contract.rs

# Kill-switch is a kill-switch (default ON): the env var name appears but no _ENABLE default-off gate
rg "ELAI_STALENESS_CASCADE_KILL" .elai_cc/crates/memory/src/quarantine_broadcast.rs

# Frontend cross-language parity (W-4)
python .claude/tools/cross_lang_diff.py
```

## ACCEPTANCE

- [ ] W-1 WIRING DONE-GATE: `python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/F-plan/F-5_staleness_quarantine_cascade.md --run-trigger` returns PASS (exit 0) — every produces_symbol is a DEF, every consumes_symbol has a real CALL/REF, every reachable_from anchor reached by BFS, stub_ban finds zero forbidden literals, AND the observable_effect trigger ran and emitted the row + artifact.
- [ ] W-2 DEFAULT-ON: `cargo run -p elai-cli -- dkg staleness-tick --workspace .` runs with NO env override set and emits the `staleness_quarantine_broadcast` row + flips `concept_stale=2` on >=1 citing concept. Setting `ELAI_STALENESS_CASCADE_KILL=1` disables it (kill-switch proven distinct from terminal default-off).
- [ ] W-6 DASHBOARD E2E: through the dashboard, open the KnowledgeDrawer (staleness/knowledge panel), observe the quarantine list populate with the dependent-demotion count after a `dkg staleness-tick` run — verified by the dashboard-smoker agent (svelte-check + cargo check + manual tauri:dev observation).
- [ ] Phase-1 gate: `f5_concept_stale_retrieval_contract.rs` passes against PRE-population code (proves no retrieval consumer assumes concept_stale=1 as a constant); it still passes after Task 5 population lands.
- [ ] Measurable (synthesis C5): quarantine a fact via mock-clock -> within one tick, `concept_stale` flips on all DKG concepts citing it AND a grounding-cache miss occurs on re-query — proven by the mock-clock contract test.
- [ ] Bounded queue: under a churn fixture exceeding `QUARANTINE_QUEUE_CAPACITY`, drops are visible via `dropped_count` on the next broadcast (no silent loss).
- [ ] SSB: `StalenessQuarantineBroadcast` round-trips via the SSB roundtrip contract; golden lock + action_type_lock GREEN; SCHEMA_VERSION baseline+1 (TOTAL_VARIANTS 470 -> 471) named in the bump.
- [ ] `cross_lang_diff.py` PASS (KnowledgeDrawer DTO <-> commands_f5 type parity).
- [ ] `cargo check --workspace` GREEN; auditor (sole test runner) reports `-p elai-memory -p elai-knowledge -p elai-orchestrator -p contracts` PASS.
- [ ] No new external dependency (anti-slopsquatting per CLAUDE.md — Cosmos + DKG + queue all use shipped crates).
- [ ] NOT RunPod-gated: this F-N is fully headless/deterministic (mock-clock). No separate live-run checkbox required (no model inference on the gate path).

## WIRING DONE-GATE

How F-5 satisfies W-1..W-10 (clause by clause — the implementer cannot skip wiring):
- W-1 (no-wire-dark done-gate): the DONE definition is `wiring_contract_check.py --run-trigger` PASS, NOT cargo-green. The `dkg staleness-tick` trigger must actually emit a live `staleness_quarantine_broadcast` row in ssb.db + flip a real `concept_stale` value. The wiring-contract-validator agent runs this before COMPLETE.
- W-2 (no default-off terminal state): the cascade is wired ON by default in the live decay tick; `ELAI_STALENESS_CASCADE_KILL` is a kill-switch (disable-only). stub_ban forbids an `_ENABLE`-gated default-off terminal state; ACCEPTANCE verifies the trigger runs with NO env override.
- W-3 (production entrypoint): reachable from the in-line `decay_tick` at `live_cli.rs:1789` (live REPL production tick), from `ensure_dkg_or_rebuild` (C-38 production DKG path), and from the `dkg staleness-tick` CLI subcommand — all real, none test-only.
- W-4 (frontend pair): `KnowledgeDrawer.svelte` consumes the `get_staleness_quarantine` Tauri command for a dashboard-surfaceable SSB signal; `cross_lang_diff.py` must pass.
- W-5 (live observable_effect): trigger is a runnable `cargo run -p elai-cli -- dkg staleness-tick` producing a live ssb.db row + an inspectable JSON/stdout artifact — never a unit test. Fully deterministic (mock-clock), so no RunPod fixture-vs-live split is needed.
- W-6 (dashboard checkpoint): ACCEPTANCE item verifies the quarantine list + demotion count THROUGH the KnowledgeDrawer via dashboard-smoker.
- W-7 (no hallucinated integration points): every consumes_symbol + reachable_from anchor + MODIFY target was greped in elai.map / source this session — `decay_tick` (elai.map DEF_FN decay.rs:88), `spawn_decay_scheduler` (elai.map DEF_FN decay_scheduler.rs:101), `DecayTickObserver` (elai.map TYPE_REF decay_scheduler.rs:88 type alias), `audit_conflicts` (elai.map DEF_FN contradiction.rs:64), `MemoryFactSuperseded` (elai.map variant ssb.rs:5320), `concept_stale` (SQL schema string in dkg/mod.rs:36 + :369; NOT a Rust symbol, confirmed via source grep only — not in elai.map by design), `cosmos_worker::status_for` (elai.map DEF_FN cosmos_worker.rs:729), `ensure_dkg_or_rebuild` (elai.map DEF_FN dkg_lifecycle.rs:127), `KnowledgeSsbSink` (elai.map trait knowledge_ssb_sink.rs) all confirmed real.
- W-8 (produces/consumes symmetry): every produced symbol has an in-plan caller — `collect_quarantined` called from the live tick (Task 4); `QuarantineBroadcastQueue` drained by `apply_staleness_broadcast` (Task 5) + `flag_on_staleness_broadcast` (Task 6); `get_staleness_quarantine` called by KnowledgeDrawer (Task 8); the SSB variant emitted by Task 4 and consumed by Task 8.
- W-9 (SSB golden lock + named bump): `StalenessQuarantineBroadcast` is a single Pattern-2 variant flowing through `action_type_lock.golden`; SCHEMA_VERSION named "baseline+1" (191 baseline per CLAUDE.md; golden TOTAL_VARIANTS 470 -> 471); cross_lang_diff + action_type_lock green.
- W-10 (tiered-gate): F-5 is entirely in-process (SQLite UPDATE + bounded in-memory mpsc queue) — NO network, NO subprocess, NO RunPod on the gate path. The fail-closed hot path is the deterministic quarantine decision; there is no fail-open Tier-2 axis to split out. Declared TIER-1-pure for the gate (the plan's Tier-3 ranking is sequencing/effort, not a network/subprocess axis).

## Tests to add

- `crates/contracts/tests/f5_concept_stale_retrieval_contract.rs` (~120 LOC, Task 2): retrieval ranking correct with concept_stale in {0,1,2}; must pass pre- AND post-population.
- `crates/memory/tests/quarantine_broadcast_cascade.rs` (~140 LOC): mock-clock quarantine -> collect_quarantined returns the fact -> broadcast emitted; the measurable (concept_stale flips on citing concepts within one tick + grounding-cache miss).
- `crates/memory/tests/quarantine_queue_bounded.rs` (~80 LOC): churn beyond QUARANTINE_QUEUE_CAPACITY surfaces dropped_count; no silent loss.
- `crates/knowledge/tests/dkg_apply_staleness_broadcast.rs` (~110 LOC): apply_staleness_broadcast UPDATEs concept_stale=2 + closes the contradicted edge bi-temporally (non-destructive).
- `crates/knowledge/tests/cosmos_flag_on_broadcast.rs` (~70 LOC): flag_on_staleness_broadcast transitions a cited artifact to the existing Stale CosmosStatus.
- `crates/orchestrator/tests/ssb_staleness_quarantine_roundtrip.rs` (~60 LOC): StalenessQuarantineBroadcast payload round-trips; golden fixture.

## Files touched

- CREATE: `crates/memory/src/quarantine_broadcast.rs` (~270 LOC: collect_quarantined + QuarantineRecord + QuarantineBroadcastQueue + QUARANTINE_QUEUE_CAPACITY + kill-switch read)
- CREATE: `crates/dashboard-v2/src-tauri/src/commands_f5.rs` (~90 LOC: get_staleness_quarantine + StalenessQuarantineDto)
- CREATE: `crates/dashboard-v2/src/lib/drawers/StalenessQuarantineDrawer.svelte` OR extend KnowledgeDrawer.svelte (~130 LOC)
- CREATE: `crates/contracts/tests/f5_concept_stale_retrieval_contract.rs` (~120 LOC)
- CREATE: `crates/memory/tests/quarantine_broadcast_cascade.rs` (~140 LOC)
- CREATE: `crates/memory/tests/quarantine_queue_bounded.rs` (~80 LOC)
- CREATE: `crates/knowledge/tests/dkg_apply_staleness_broadcast.rs` (~110 LOC)
- CREATE: `crates/knowledge/tests/cosmos_flag_on_broadcast.rs` (~70 LOC)
- CREATE: `crates/orchestrator/tests/ssb_staleness_quarantine_roundtrip.rs` (~60 LOC)
- CREATE: `workflow/reasoning/F-5-concept-stale-audit.md` (~80 LOC)
- CREATE: `workflow/reasoning/F-5-decisions.md` (~120 LOC)
- MODIFY: `crates/orchestrator/src/ssb.rs` (~90 LOC: StalenessQuarantineBroadcast variant + payload + sql_str + parse arm + CHECK + golden row)
- MODIFY: `crates/orchestrator/tests/action_type_lock.golden` (~1 row: staleness_quarantine_broadcast; TOTAL_VARIANTS 470 -> 471)
- MODIFY: `crates/memory/src/lib.rs` (~4 LOC: re-export quarantine_broadcast symbols)
- MODIFY: `crates/memory/src/decay_scheduler.rs` (~20 LOC: observer fires collect_quarantined for long-lived runtimes)
- MODIFY: `crates/cli/src/live_cli.rs:1770-1810` (~40 LOC: in-line post-reflect tick drains collect_quarantined + emits broadcast — the W-3 live path)
- MODIFY: `crates/knowledge/src/dkg/mod.rs:360-380` (~120 LOC: apply_staleness_broadcast + concept_stale UPDATE + bi-temporal edge close)
- MODIFY: `crates/knowledge/src/cosmos_worker.rs` (~70 LOC: flag_on_staleness_broadcast)
- MODIFY: `crates/cli/src/dkg_lifecycle.rs` (~60 LOC: dkg staleness-tick subcommand + queue-drain post-rebuild hook)
- MODIFY: `crates/dashboard-v2/src-tauri/src/lib.rs` (~3 LOC: register get_staleness_quarantine in invoke_handler)
- MODIFY: `crates/dashboard-v2/src/lib/drawers/DrawerHost.svelte` + `types.ts` (~10 LOC: DrawerKey if a new drawer is added)

## Citations / Grounding

- synthesis VIBESEC-SHARD-STEAL-AND-BUILD-2026-06-01.md §4 C5 (mechanism + ELAI ground truth + REVISED SCOPE + always-1 hazard) — PRIMARY for this plan's scope.
- synthesis §1 (correction: ELAI is NOT missing bi-temporal — F-5 adds cross-subsystem SCOPE only; do not re-introduce the "missing bi-temporal" claim) — VERIFIED correction, binding.
- synthesis §3 (SHARD = composition-interface BLUEPRINT, not adoption; `handle_stale_memory` real in composition.py fetched 2026-06-01) — borrow the cascade shape, not the code.
- synthesis §5 (Tier-3 sequencing: after B-30; concept_stale retrieval audit FIRST) — binding sequencing.
- arXiv:2501.13956 Graphiti §2.2.3 bi-temporal contradiction invalidation (confirmed in edge_operations.py per synthesis) — PRIMARY reference for the bi-temporal edge-close pattern; ELAI already ships the equivalent (`audit_conflicts`), used here as the cross-subsystem reference only.
- ELAI ground truth (code-read + elai.map, this session, W-7): decay.rs:88, decay_scheduler.rs:101/105, contradiction.rs:72, ssb.rs:5320/15891, dkg/mod.rs:36/369, cosmos_worker.rs:122/811, dkg_lifecycle.rs:12, knowledge_ssb_sink.rs:191, live_cli.rs:1789.
- DETAILED_SUMMARY_01:119 (concept_stale always-1 / never populated) — the latent hazard source.
- UNVERIFIED / labelled: SHARD's exact staleness thresholds (>1.0 flag, >2.0 quarantine) are SHARD's constants; ELAI's `staleness_ratio` reuses the EXISTING decay math in decay.rs — the >2.0 quarantine cut is a design choice to confirm in F-5-decisions.md, not a ported guarantee. The "grounding-cache miss on re-query" measurable assumes a grounding cache keyed on concept freshness; confirm the cache key reads concept_stale during Phase-1 audit (Task 1) before relying on it.

## Open questions / risks

- The always-1 hazard (PRIMARY risk): if any retrieval consumer treats `concept_stale=1` as a constant (e.g. `WHERE concept_stale=1` as an implicit "all rows" filter), the first real `0`/`2` silently changes ranking. Phase-1 audit (Task 1) + the pre-population contract test (Task 2) are the mitigation and MUST land before Task 5. This is non-negotiable per synthesis §5 + §7.
- B-30 dependency: `dkg_nodes` schema must be B-30-stable before Task 5 touches it. If B-30 is not shipped, F-5 Phase 2 is BLOCKED (Phase 1 audit can still proceed). Coordinate the `dkg/mod.rs` edit window.
- Bounded-queue back-pressure: drop-oldest loses the OLDEST quarantine, which may be the most-stale (most important). Counterfactual (CSQ): if the queue drops a quarantine silently, a stale concept stays live and a downstream grounding could cite stale knowledge. Mitigation: visible `dropped_count` + a dashboard surface; the capacity (256) is a starting constant to tune against real churn in F-5-decisions.md.
- `spawn_decay_scheduler` is currently wire-dark (only a comment references it). F-5 wires the broadcast into the LIVE in-line tick (live_cli.rs:1789) so it is reachable TODAY; the scheduler-observer wiring (Task 4) is additive for long-lived runtimes and must not be the SOLE caller (would re-create the wire-dark failure F-plan exists to prevent).
- Cosmos consumer reuses the existing Stale `CosmosStatus` — confirm B-42's status enum has a Stale state reachable from `status_for` (greped: `CosmosStatus::Fresh` matched at cosmos_worker.rs:811; verify the Stale variant in the read before Task 6).
- No sidecar boot, no RunPod, no network — F-5 is fully deterministic. The HARDWARE-HEAVY sidecar rule does not apply.

## Next

F-6 (learned-skill library) composes: SkillTrustTier staleness demotion (C6c) is a NATURAL consumer of the same `StalenessQuarantineBroadcast` — a quarantined fact that a skill depends on can demote that skill's trust tier via the same cascade. F-7's CBR template ranking can consume the demotion signal as a negative-utility input. The Graphiti LLM-judged contradiction detection (synthesis §4 C8 alternative) is the higher-maturity upgrade for the contradiction half on ELAI's cloud lane, RUNPOD-PENDING. E-29 observability: every `staleness_quarantine_broadcast` row flows through the OTLP exporter for off-machine archival.

End of F-5_staleness_quarantine_cascade.md.
