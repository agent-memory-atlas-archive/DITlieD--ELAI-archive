H-9 DECISION DOC — cost cockpit dashboard (the believe-the-number instrument)
session s-66424 | 2026-06-03 | orchestrate H-plan WAVE 1 | composes-with H-1, C-5, C-25

WHAT
The cloud spend ledger is dark END-TO-END (CORRECTION C10): the SOLE production cloud client ctor never
chains .with_ssb_writer, so the `if let Some(ssb)` CloudSpend block (grounding_client.rs:887-969) never fires
and CloudPanel renders ALL-ZEROS. H-9 attaches the writer (the keystone) + surfaces the measured cache
efficiency. This is the instrument that makes H-1/H-2 falsifiable.

GROUNDED FINDINGS (3 explorers, live-verified)
- THE GAP: task_runner.rs:5840 builds the sole prod cloud client via CloudGroundingClient::from_env_async()
  with NO .with_ssb_writer. `writer: &Arc<Ssb>` is ALREADY in drive_task's signature (:5436) and passed as
  Some(writer) to research_decompose at :5851. So the fix is one chained call.
- H-1 COMPOSITION: measure_cache_hit_rate is ALREADY in that same `if let Some(ssb)` block (grounding_client.rs:956,
  shipped by H-1). Attaching the writer lights BOTH CloudSpend AND H-1's prompt_cache_hit_rate monitor for free.
- CloudPanel.svelte ALREADY renders cache_read/cache_creation tokens (:251-259) + day/task spend; the cloud_spend
  reader (get_cloud_spend -> cloud_spend_summary, ssb_viewer.rs:4742) is CORRECT (C-4 rewrote it). ALL-ZEROS is
  NOT a read bug — it is the missing producer attach. Once rows exist, CloudPanel renders non-zero.
- Bindings are HAND-WRITTEN stub.ts + types.ts (NOT tauri-specta — the compat doc was wrong). CloudSpendInfo
  (types.ts:1923) already carries cache_creation_tokens + cache_read_tokens.
- The prompt-cache hit-rate read path EXISTS: get_critic_decision_state (commands_e10.rs:60) projects the
  PromptCacheHitRate tile (challenger_score=hit rate); CriticDecisionPanel.svelte already renders it. So the
  hit-rate is NOT dark at the command layer.

KEY DECISIONS
D1 (the keystone): chain `.with_ssb_writer(Arc::clone(writer))` on the from_env_async cloud client at
   task_runner.rs:5840. Mirrors the GD-1 reference pattern (runtime_setup.rs:382 TemplateCrystallizer threads a
   cloned Arc<Ssb> at construction). This is the one change that lights the dark ledger + H-1's monitor in prod.
D2 (surface the measured cache efficiency): CloudPanel.svelte gains a client-derived `cacheHitRate` =
   cache_read / (cache_read + cache_creation) (the day-aggregate cache-read fraction — the MEASURED number, HD-1)
   rendered as a new "Cache hit rate" row. Uses the EXISTING CloudSpendInfo fields; no new DTO, no new binding,
   no new command. Zero-guard (0 tokens => 0%).
D3 (no schema bump): H-9 adds NO action_type — it reuses CloudSpend (already golden-locked) + the existing
   CriticDecisionEvent/PromptCacheHitRate row from H-1. SCHEMA untouched (live ~210/487; the G run advanced it).

BOUNDED SCOPE / DEFERRED (honest)
DEFERRED to Tier-2 (each needs a NEW DTO/command/source the explorers flagged as genuine GAPS):
- per-RESPONSE input/output/cached breakdown (CloudSpendPayload rows are aggregated away by cloud_spend_summary;
  needs a new recent_cloud_spend_rows viewer + command + TS type).
- precise $/RESOLVED-task (needs a "resolved task" definition + count source; task_usd is the active-task
  approximation shown today).
- cache-EXPIRY timer (needs last-cache-write ts + the TTL constant persisted; not on the DTO).
- a stand-alone CostCockpitPanel component in a new window (extending CloudPanel delivers the same numbers at
  lower risk; the Devin-style per-turn component is the Tier-2 surface).
- CloudSpend on the MAIN cloud-completion path (today CloudSpend fires only on the cloud-GROUNDING path;
  handle_infer_request has no Ssb in scope — the same GAP-2 plumbing H-1 hit; full coding-call $/task is a
  follow-on). H-9 lights the grounding-path ledger + the cache monitor + the cockpit UI.

DONE-GATE (Tier-1; live cloud run = deferred Tier-2, sidecar-gated)
crates/contracts/tests/h9_cost_cockpit_contract.rs (source-wiring): the .with_ssb_writer attach is present at the
from_env_async cloud client in task_runner.rs; CloudPanel renders a cache hit rate. cargo check -p elai-orchestrator
clean (SCOPED — NOT --workspace; the G-2 run_subcommand.rs:453 error breaks the workspace, not mine). dashboard-smoker:
svelte-check exit 0 + cargo check -p elai-dashboard-v2 exit 0 (W-6). Test EXECUTION + live render DEFERRED.

RISK
R1 task_runner.rs:5840 chaining: from_env_async().await.map_err(..)? yields Self; .with_ssb_writer(self)->Self
   chains cleanly. Arc + writer in scope (orchestrator file; writer:&Arc<Ssb> in drive_task sig). Low risk.
R2 measure.rs:994 is another from_env_async caller (a CLI canary, no Ssb in scope) — leave it; only the
   task_runner drive_task site gets the attach.
R3 CloudPanel cacheHitRate divide-by-zero — guarded (0 tokens => 0).

DEFERRED-from-G note: the shared tree carries G's uncommitted work + a G-2 compile error (run_subcommand.rs:453)
that breaks `cargo check --workspace` — NOT H-9's; auditor uses scoped -p. NO commit (accumulate to run end).
