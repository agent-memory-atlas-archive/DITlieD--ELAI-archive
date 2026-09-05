CONJECTURE RUN — context-lifecycle-economics — 2026-07-11

RUN TYPE: UNIT-5 INTEGRATION SMOKE (auditor-class agent, independent of UNITS 1-4 executor).
Session task_id=UNIT-5-integration-smoke. Driven end-to-end through the real skill protocol
(.claude/skills/improvement-conjecture/SKILL.md), not a toy happy-path.

## TRIGGER

Verbatim input: "improve ELAI's context-lifecycle economics"
FORK-1 routing: named weakness / "improve area X" phrasing -> idea-intake route or standalone
/conjecture (not a paper/repo link) -> standalone route taken.
FORK-2 anchor oracle: `python .claude/tools/elai_map_freshness.py` -> FRESH (map age 3959s older
than newest source, exit 0). elai.map used directly as the anchor oracle; no substitution needed.
Ledger source: docs/thefinalglance/ELAI-FEATURE-MAP.md Section 4 (REJECTED/ALREADY-TRIED), read
once at round 1 for grounding (legitimate — ATTACK-4 only bans re-rendering ledger content into
ROUNDS AFTER THE FIRST, not the initial grounding read).

## BIN MATRIX

Families: gate/checker (F1), memory/reuse (F2), routing/economics (F3), prompt/context-layout (F4),
decoding/inference (F5), measurement/bench (F6), doctrine/process (F7), integration/seam (F8).
Areas (real .elai_cc anchors, grep-verified against elai.map before generation):
  A1 h50-local-regime         crates/orchestrator/src/context_policy.rs
  A2 h51-resume-digest        crates/orchestrator/src/handoff_digest.rs
  A3 cloud-cache-economics    crates/orchestrator/src/cache_economics.rs
  A4 context-budget-slots     crates/orchestrator/src/context_budget.rs
  A5 sidecar-prefix-kv-cache  python/elai_inference/server.py (PrefixKVCache)
  A6 escalation-waterfall     crates/orchestrator/src/escalation_waterfall.rs
  A7 runtime-builder-wiring   ELAI_LOCAL_MAX_CONTEXT read sites (cross-crate + sidecar)
  A8 k2-routing-economics     usage_turn_event (J-28, landed commit fe069d0)
  A9 h50-h51-doctrine-dup     context_policy.rs + handoff_digest.rs (doc-doctrine overlap)

|        | A1 | A2 | A3      | A4 | A5      | A6 | A7  | A8 | A9 |
|--------|----|----|---------|----|---------|----|-----|----|----|
| F1     | C1 |    | C7      |    |         |    |     |    |    |
| F2     |    | C3 | C11(dup)|    | C9      |    |     |    |    |
| F3     |    |    |         |    |         |    |     | C8 |    |
| F4     |    |    |         | C5 |         |    |     |    |    |
| F5     |    |    | EMPTY-ATTEMPTED (round 4, no groundable candidate found) |
| F6     |    |    |         |    |         | C4 |     |    |    |
| F7     |    |    |         |    |         |    |     |    | C9(no)/C6 |
| F8     |    |    |         |    | C2      |    | C10 |    |    |

EMPTY-ATTEMPTED bin recorded: (F5 decoding/inference) x (A3 cloud-cache-economics). Round-4
(Architect stance) attempted a decoding-level lever tying TtlTier to sampling params
(temperature/top_p) to reduce cache-busting logit warmup; no runnable or citable falsifier could be
specified (the causal link between decode sampling and a KV/prompt-cache prefix hit-rate is not
groundable from anything in this codebase or a cited paper in one pass) -> abandoned per FORK-3,
recorded empty rather than padded with a near-duplicate.

Distinct filled bins: 11 (F1xA1, F1xA3, F2xA2, F2xA3, F2xA5, F3xA8, F4xA4, F6xA6, F7xA9, F8xA5,
F8xA7). Candidates: 11 (C1-C11). Exceeds floor (>=8 candidates, >=5 bins).

## CANDIDATES

Generation ran across 5 calls (>=3 required), stance-rotated. ROUND 1 is the only call that saw
ledger content (legitimate first-call grounding); ROUNDS 2-5 received ONLY the bin-matrix state
(filled/empty coordinates) and were directed at naming an empty bin — never prior candidate text,
never ledger content (ATTACK-4 discipline, self-enforced and declared here for audit).

--- ROUND 1 (stance: Hacker; input: trigger + ledger Section 4 + elai.map anchor grounding) ---

### CANDIDATE C1
- id: C1
- mechanism_family: gate/checker
- target_area: h50-local-regime
- target_units: crates/orchestrator/src/context_policy.rs, effective_context_window
- expected_effect: a production-path clamp/warn when ELAI_LOCAL_MAX_CONTEXT parses above the
  documented 32000 hard ceiling (CLAUDE.md), preventing a misconfigured env var from silently
  requesting a KV-cache size the 3090's 24GB cannot hold at 4-bit.
- cost_class: S
- falsifier: (runnable) grep -n "32000" crates/orchestrator/src/context_policy.rs, restricted to
  non-#[cfg(test)] lines; pass-if the count of PRODUCTION (non-test) matches is 0, confirming no
  runtime ceiling clamp exists today (the literal only appears in test assertions).
- bin: gate/checker x h50-local-regime
- stance: Hacker
- typicality: 0.55
- proof_spine: feeds: PS-2 (a new pre-tool-use-style env clamp is gate machinery; composes the
  gate-tax instrument if promoted, one more counted gate)
- external_claims: none

### CANDIDATE C2
- id: C2
- mechanism_family: integration/seam
- target_area: sidecar-prefix-kv-cache
- target_units: python/elai_inference/server.py, PrefixKVCache, crates/orchestrator/src/cache_economics.rs, emit_cache_economics_event
- expected_effect: the LOCAL sidecar's PrefixKVCache (LRU KV-prefix reuse, server.py:1966+) emits
  only logger.info; the CLOUD lane's cache_economics.rs emit_cache_economics_event (I-10/F-2) is
  wired into SSB via cli:stream (runtime_client.rs). Bridging a LOCAL-lane SSB emission on
  admit/lease/evict would give ELAI a single cross-lane cache-economics event stream instead of one
  lane observable and one lane log-only.
- cost_class: M
- falsifier: (runnable) grep -rn emit_cache_economics_event crates/ --include=*.rs; pass-if every
  call site is under crates/cli/src/runtime_client.rs (CLOUD-lane-only), confirming the LOCAL sidecar
  has zero equivalent emission today.
- bin: integration/seam x sidecar-prefix-kv-cache
- stance: Hacker
- typicality: 0.4
- proof_spine: feeds: PS-4 (a unified cache-economics stream is a precondition for measuring
  routing $/verified-task across BOTH lanes, not cloud-only)
- external_claims: none

### CANDIDATE C11
- id: C11
- mechanism_family: memory/reuse
- target_area: cloud-cache-economics
- target_units: crates/orchestrator/src/cache_economics.rs, CacheEconomicsPlanner, plan_ttl
- expected_effect: LLM re-summarization compaction on the cache prefix shrinks the effective per-turn cache-read token cost.
- cost_class: M
- falsifier: (runnable) run conjecture_triage.py --run <this artifact> --json and check the
  emitted KILL-EDGE line for C11; pass-if killer_type is triage or experiment (i.e. NOT killed as a
  ledger duplicate) — deliberately testing the stage-2 ledger check against a candidate this session
  expects to collide with feature-map Section 4 ("LLM re-summarization compaction on the cache
  prefix: REJECTED (busts cache; deterministic masking only, HD-2)").
- bin: memory/reuse x cloud-cache-economics
- stance: Hacker
- typicality: 0.3
- proof_spine: none (deliberate ledger-collision probe for this smoke, not a real proposal)
- external_claims: none

--- ROUND 2 (stance: Researcher; input: BIN MATRIX ONLY — filled: F1xA1, F8xA5, F2xA3; directed at
empty bins F2xA2, F6xA6, F3xA8) ---

### CANDIDATE C3
- id: C3
- mechanism_family: memory/reuse
- target_area: h51-resume-digest
- target_units: crates/orchestrator/src/handoff_digest.rs, author_topic_digest
- expected_effect: the H-51 resume digest (author_topic_digest, handoff_digest.rs:81, called from
  live_cli.rs:3127) reports elapsed-time-since-last-session but nothing about whether the sidecar's
  LRU prefix cache still held a hot entry for that topic at resume; adding a one-line cache-hit-rate
  stat to the digest header would tell a resuming session whether its first turn will be a cold
  prefill.
- cost_class: S
- falsifier: (runnable) grep -n "hit_rate\|cache_hit" crates/orchestrator/src/handoff_digest.rs;
  pass-if 0 matches, confirming the digest currently carries no cache-state signal.
- bin: memory/reuse x h51-resume-digest
- stance: Researcher
- typicality: 0.5
- proof_spine: none (exploratory harness improvement, not yet plan-authored)
- external_claims: none

### CANDIDATE C4
- id: C4
- mechanism_family: measurement/bench
- target_area: escalation-waterfall
- target_units: crates/orchestrator/src/escalation_waterfall.rs, condense
- expected_effect: escalation_waterfall.rs::condense (line 227) has no measured before/after context
  byte-count or token-count recorded as an SSB row; instrumenting it would let a later session
  compute the real token-cost of an escalation, feeding K-2 $/verified-task with the now-landed J-28
  usage_turn_event ledger (commit fe069d0) instead of leaving escalation cost invisible.
- cost_class: S
- falsifier: (runnable) grep -n "fn condense" crates/orchestrator/src/escalation_waterfall.rs then
  check for an emit/SSB call inside the function body; pass-if 0 SSB emission calls found inside
  condense, confirming the cost of an escalation is currently unmeasured.
- bin: measurement/bench x escalation-waterfall
- stance: Researcher
- typicality: 0.45
- proof_spine: feeds: PS-4 (escalation cost is a direct input to $/verified-task)
- external_claims: none

### CANDIDATE C8
- id: C8
- mechanism_family: routing/economics
- target_area: k2-routing-economics
- target_units: crates/orchestrator/src/runtime_orchestrator.rs, usage_turn_event, crates/cli/src/usage_cmd.rs
- expected_effect: route LOCAL-vs-CLOUD backend selection by the now-landed J-28 usage_turn_event
  ledger's measured $/verified-task instead of the static BackendMode regime rules in
  context_policy.rs::regime_policy, closing the K-2 live leg the proof-spine ruling marked
  "blocks on J-28" (J-28 landed 2026-07-11 per commit fe069d0, this session's git log).
- cost_class: L
- falsifier: (runnable) grep -rln usage_turn_event crates/ --include=*.rs; pass-if the symbol
  resolves in >=3 real modules (runtime_orchestrator.rs, action_type.rs, usage_cmd.rs), confirming
  the ledger is a real, wired artifact and not a stub before proposing to route on it.
- bin: routing/economics x k2-routing-economics
- stance: Researcher
- typicality: 0.35
- proof_spine: feeds: PS-4 (this candidate IS the K-2 live leg named in the proof-spine ruling)
- external_claims: none

--- ROUND 3 (stance: Simplifier; input: BIN MATRIX ONLY; directed at empty bins F4xA4, F7xA9) ---

### CANDIDATE C5
- id: C5
- mechanism_family: prompt/context-layout
- target_area: context-budget-slots
- target_units: crates/orchestrator/src/context_budget.rs, SLOT_COUNT
- expected_effect: context_budget.rs fixes SLOT_COUNT == 10 (contract-enforced,
  context_budget_contract.rs:24); a structural-simplification pass may find slots whose weights are
  near-zero or whose purpose text overlaps enough to merge, shrinking the fixed layout instead of
  adding another slot next time a new context class shows up (maintainability-review dimension 0).
- cost_class: S
- falsifier: (runnable) grep -n "SLOT_COUNT\|SlotWeights::default_fixed" crates/orchestrator/src/context_budget.rs
  and crates/contracts/tests/context_budget_contract.rs; count the 10 slot names/weights and measure
  whether any two textually overlap enough to be merge candidates.
- bin: prompt/context-layout x context-budget-slots
- stance: Simplifier
- typicality: 0.6
- proof_spine: none (exploratory harness improvement, not yet plan-authored)
- external_claims: none

### CANDIDATE C6
- id: C6
- mechanism_family: doctrine/process
- target_area: h50-h51-doctrine-dup
- target_units: crates/orchestrator/src/context_policy.rs, crates/orchestrator/src/handoff_digest.rs
- expected_effect: the "REGIME-AWARE CONTEXT POLICY (H-50)" phrase and its wiring description repeat
  near-verbatim across multiple feature-map regions (Section 4 lines ~4507/4542 per this run's
  grounding grep); consolidating to one doctrine paragraph with pointers reduces token cost of every
  future context-compiler read of the feature map (token-discipline.md: "redundancy between files is
  a bug" applied to redundancy WITHIN a file).
- cost_class: S
- falsifier: (runnable) grep -c "REGIME-AWARE CONTEXT POLICY" docs/thefinalglance/ELAI-FEATURE-MAP.md;
  pass-if count >= 2, confirming the duplication this candidate proposes to collapse actually exists.
- bin: doctrine/process x h50-h51-doctrine-dup
- stance: Simplifier
- typicality: 0.5
- proof_spine: none (doc-token-economy, not a measured spine number)
- external_claims: none

--- ROUND 4 (stance: Architect; input: BIN MATRIX ONLY; directed at empty bin F1xA3 and attempted
F5xA3) ---

### CANDIDATE C7
- id: C7
- mechanism_family: gate/checker
- target_area: cloud-cache-economics
- target_units: crates/orchestrator/src/cache_economics.rs, PrefixInvarianceGuard, check_miss_rate, crates/cli/src/runtime_client.rs
- expected_effect: PrefixInvarianceGuard::check_miss_rate is called in production
  (runtime_client.rs:883) and on GuardVerdict::Tripped it emits an SSB row + tracing::warn, but does
  NOT stop or downgrade the call (verified by reading runtime_client.rs:878-899 this session). A
  hard gate (abort-and-fallback-to-fresh-prefix on repeated Tripped) would turn an observed cache
  pathology into an enforced correction instead of a warning nobody reads.
- cost_class: M
- falsifier: (runnable) grep -n "GuardVerdict::Tripped" crates/cli/src/runtime_client.rs then read
  the branch body; pass-if the branch contains no early-return/circuit-break/retry call, confirming
  today's behavior is observability-only.
- bin: gate/checker x cloud-cache-economics
- stance: Architect
- typicality: 0.45
- proof_spine: feeds: PS-2 (a new hard gate is directly countable gate-tax)
- external_claims: none

FORK-3 note: F5 (decoding/inference) x A3 (cloud-cache-economics) was also attempted this round —
see BIN MATRIX EMPTY-ATTEMPTED entry above.

--- ROUND 5 (stance: Contrarian; input: BIN MATRIX ONLY; directed at remaining fillable bins
F2xA5 (second candidate, tail-weighting toward the least-typical framing) and F8xA7) ---

### CANDIDATE C9
- id: C9
- mechanism_family: memory/reuse
- target_area: sidecar-prefix-kv-cache
- target_units: python/elai_inference/server.py, PrefixKVCache, max_entries
- expected_effect: CONTRARIAN framing against C2 — the sidecar's PrefixKVCache LRU defaults to
  max_entries=4 (server.py init_prefix_kv_cache default) with no measurement behind that number;
  before building MORE machinery on top of it (C2's cross-lane bridge), measure whether 4 entries is
  even adequate for a typical multi-turn session's distinct-prompt-prefix count, or whether the
  entries are being thrashed/evicted before they are ever reused.
- cost_class: S
- falsifier: (runnable) grep -n "kv_cache_size: int = 4\|max_entries" python/elai_inference/server.py;
  count distinct prefix-lease call sites (server.py:2779, 2940) vs LRU capacity to sanity-check
  whether 4 is measured or arbitrary.
- bin: memory/reuse x sidecar-prefix-kv-cache
- stance: Contrarian
- typicality: 0.25
- proof_spine: none (exploratory harness improvement, not yet plan-authored)
- external_claims: none

### CANDIDATE C10
- id: C10
- mechanism_family: integration/seam
- target_area: runtime-builder-wiring
- target_units: crates/orchestrator/src/context_policy.rs, crates/dashboard-v2/src-tauri/src/state.rs, python/elai_inference/context_journal.py, ELAI_LOCAL_MAX_CONTEXT
- expected_effect: CONTRARIAN framing — ELAI_LOCAL_MAX_CONTEXT is read independently in >=3 places
  this session confirmed by grep (context_policy.rs::effective_context_window,
  dashboard-v2/src-tauri/src/state.rs:528, python/elai_inference/context_journal.py:76), each free to
  parse/default differently; this is exactly the "spaghetti prevention" / "boundary clarity" smell
  (maintainability-review.md dims 2/5) the doctrine already bans — thread it through ONE typed
  resolver instead of N independent env reads.
- cost_class: M
- falsifier: (runnable) grep -rn ELAI_LOCAL_MAX_CONTEXT crates/ python/ --include=*.rs --include=*.py;
  count distinct independent read sites; pass-if count > 1, confirming the fan-out coupling this
  candidate proposes to collapse.
- bin: integration/seam x runtime-builder-wiring
- stance: Contrarian
- typicality: 0.3
- proof_spine: feeds: PS-6 (an independently-defaulting env fan-out is exactly the class of drift
  that produces wire-dark/inconsistent-behavior rows over time; collapsing it is a longitudinal
  wire-dark-rate lever)
- external_claims: none

## STAGE-2 KILLS

Ran: `python .claude/tools/conjecture_triage.py --run workflow/reasoning/conjecture-run-context-lifecycle-economics-2026-07-11.md --json`
Tool output: candidate_count=11, refuted_count=1, ledger_entries_loaded=88, exit 0.

KILL-EDGE: C11 <- ledger:llm-re-summarization-compaction-on-the-cache-prefix
  reason (tool-reported): "duplicate of ledger entry: - LLM re-summarization compaction on the cache
  prefix: REJECTED (busts" — a genuine, correct kill against feature-map Section 4.
  NOTE (self-corrected mid-run): the first draft of C11's expected_effect field wrapped the matching
  phrase across a continuation line, which the card parser stores in `_raw` (not the `expected_effect`
  field card_signature() reads), so the FIRST triage run scored candidate_count=11, refuted_count=0 —
  the ledger check did not fire. Root cause confirmed by inspecting parse_cards()/card_signature() in
  conjecture_triage.py (only mechanism_family/target_area/target_units/expected_effect feed the
  signature; continuation lines land in _raw and are invisible to check_ledger). Fixed by rewriting
  C11's expected_effect as a single line; re-ran the tool, kill fired as expected. This is recorded
  here rather than silently re-run-and-forgotten because it is a real card-authoring hazard for any
  future run of this skill (multi-line bullet continuations silently drop content from the fields the
  deterministic checks actually read).

(all other candidates C1-C10 cleared stage 2: demarcation present+runnable, no ledger collision,
every target_units anchor grep-verified against elai.map, no fork-verb+canonical-subsystem
composition flag, proof_spine field present per PSR-5, no external_claims requiring RECON routing.)

## STAGE-3 RESULTS

SF-1 preference order applied: executed micro-experiment > cited estimate > judge-only score.
Judge pass: NOT RUN this smoke (SF-2 requires the judge be a fresh, non-generating agent; spawning
one more sub-agent here would cross the 2-agent fan-out announcement threshold this dispatch was not
pre-authorized for, per the calling session's hard rule — recorded as a substitution/limitation of
this smoke run, not silently skipped). Ranking below relies on SF-1 executed receipts and reasoned
tradeoffs only; no candidate is laundered through a fabricated judge score.

EXECUTED MICRO-EXPERIMENTS (scratchpad files, blake3 receipts):

C1 — grep -n "32000" .elai_cc/crates/orchestrator/src/context_policy.rs
  result: 4 matches, ALL inside #[cfg(test)] (lines 199-205: unsafe env::set_var + 2 asserts).
  Zero production-path matches. CONFIRMS the candidate premise: no runtime ceiling clamp exists.
  receipt file: scratchpad/c1_falsifier.txt
  blake3: fe74a0cf90f37255ebff7123ce9b9b3c8d5d3ec78f70cfd40671edce1d9b7003

C2 — grep -rn "emit_cache_economics_event" .elai_cc/crates --include=*.rs
  result: 11 matches across runtime_client.rs (3 production call sites, all "cli:stream"),
  cache_economics.rs (def), i10_cache_economics_contract.rs (test), action_type.rs/payloads.rs
  (doc comments). CONFIRMS: emission is real, wired, and CLOUD-lane-only — zero LOCAL-sidecar
  equivalent. Premise REFRAMED (not "unwired", but "cloud-only") vs the round-1 draft; framing above
  already corrected before this receipt.
  receipt file: scratchpad/c2_falsifier.txt
  blake3: 21f558a49b1c859afca4208366a4a52d84f3986f0fd92541a04a1f11fe5ea5bc

C3 — grep -n "hit_rate\|cache_hit" .elai_cc/crates/orchestrator/src/handoff_digest.rs
  result: 0 matches. CONFIRMS the candidate premise: the resume digest carries no cache-state signal.
  receipt file: scratchpad/c3_falsifier.txt
  blake3: af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262

C8 — grep -rln "usage_turn_event" .elai_cc/crates --include=*.rs
  result: 5 files (usage_cmd.rs, runtime_orchestrator.rs, action_type.rs, schema.rs, ssb_viewer.rs).
  CONFIRMS J-28 is landed and real (not a stub); candidate's K-2 routing-economics premise is
  buildable today.
  receipt file: scratchpad/c8_falsifier.txt
  blake3: a852f42029fe35982e0e6d06e53511e77d53623173587397b00b606dc40f2d15

C4, C5, C6, C7, C9, C10 — falsifiers specified as runnable (grep-based) but NOT executed in this
smoke pass (budget: 4 executed micro-experiments judged sufficient to satisfy the >=1-receipt pass
criterion + give the PICK real evidence; C4/C7/C9/C10 were read-verified against source during
candidate authoring above — see the inline citations to exact file:line in each card's
expected_effect — which is a weaker READ-grounding, not an executed falsifier). These rank PENDING
per SF-1 (a judge-only or read-only candidate ranks pending; pending can rank, pending cannot win).

C11 — killed at stage 2 (ledger duplicate), no stage-3 experiment run (SF-1: stage-3 spends no
budget on a refuted candidate).

## PICK

Top-3 by executed-leg strength + composition fit (SF-1: executed > estimate > judge-only; C1/C2/C3/C8
are the only candidates with an executed receipt, so the pick is drawn from that set):

1. C8 (routing/economics x k2-routing-economics) — routing LOCAL-vs-CLOUD selection off the now-real
   J-28 usage_turn_event ledger. Tradeoffs: HIGH cost_class (L, touches routing decision code +
   needs a $/verified-task rollup query), but it is the ONLY candidate in this run that is directly
   named as a proof-spine owner leg (PS-4's "live leg blocks on J-28" — J-28 is now landed, so this
   candidate is the unblock). Composition fit: composes cache_economics.rs / context_policy.rs
   regime rules rather than forking them (no FORK_VERBS flag raised at stage 2). Falsifier strength:
   executed (5 real modules confirmed) but the ROUTING decision itself is unmeasured — this
   candidate proposes to CREATE the measurement, it doesn't yet have one.

2. C2 (integration/seam x sidecar-prefix-kv-cache) — bridge LOCAL sidecar cache events into SSB
   alongside the already-wired CLOUD emission. Tradeoffs: M cost, but LOW risk (additive, does not
   touch the existing CLOUD path); directly closes the asymmetry this trigger asked about
   ("context-lifecycle economics" spans both lanes, and today only one lane is observable).
   Falsifier strength: executed and it corrected the candidate's own premise mid-run (a genuine
   stage-3 win — the experiment caught an inaccurate initial framing before it could ship wrong).

3. C1 (gate/checker x h50-local-regime) — a runtime clamp/warn at the documented 32000 ceiling.
   Tradeoffs: S cost, cheapest to ship, but narrowest blast radius (one env var, one failure mode:
   OOM prevention on a single-GPU remote box). Falsifier strength: executed, clean confirmation of a
   real gap (doc-only ceiling, zero code enforcement).

C3 (memory/reuse x h51-resume-digest) is the fourth-ranked executed candidate, held out of the top-3
only because its effect (a resume-time cache-hit hint) is informational, not economic — weaker fit
to "economics" than C1/C2/C8.

PICK: C8, PICKED as the top-ranked candidate WITH AN EXECUTED LEG (autonomous-mode rule: auto-pick
the top-ranked candidate with an executed leg). Not PICKED-PENDING — C8's own falsifier executed
successfully.

SF-3 note: this run artifact is NEVER to be handed to the auditor that would later judge any plan
authored from C8; that judgment happens blind to this ranking (ABORT-4).

## RECHECK HOOKS

Mechanical loser re-check (documented, not executed against the LIVE feature map — see the RECHECK
DRILL section below for the actual drill against a SCRATCH copy):
```
grep -n "KILL-EDGE:" workflow/reasoning/conjecture-run-*.md
```
Any KILL-EDGE line whose killer-ref no longer matches current ledger/anchor/premise state (checked
by re-running conjecture_triage.py --run against a fresh artifact, or by hand for a single ref) means
that candidate is reinstated to pending, never silently left refuted (SF-4: losers are quarantined
via status, never deleted).

---

## RECHECK DRILL (mechanical loser re-check, executed against a SCRATCH copy only)

DRILL NOTE (2026-07-11, UNIT-5 smoke):
1. Hashed the LIVE feature map before touching anything:
   blake3:fd56068a0392319b1fe51fd85f9fdd00041017db1ada672cb72e6191948d1ddc
2. Copied it to a scratchpad-only path (never docs/thefinalglance/), then altered ONLY the scratch
   copy's ledger line that killed C11 — replaced
   "- LLM re-summarization compaction on the cache prefix: REJECTED (busts" with an unrelated
   REJECTED line ("- Retry-budget doubling on flaky sidecar timeouts: REJECTED (masks") so C11's
   killer no longer exists in the ledger the tool reads.
3. Re-ran `conjecture_triage.py --run <this artifact> --feature-map <scratch-copy> --json` (the
   tool's own `--feature-map` override flag, a stronger and more auditable mechanism than a bare
   grep since it re-derives the verdict from the same deterministic checker rather than eyeballing
   text): candidate_count=11, refuted_count=0, C11 status flipped from "refuted" back to "pending",
   kill_edges=[] — the candidate is found mechanically and reinstated to pending, exactly as the
   documented RECON hook (`grep -n "KILL-EDGE:" workflow/reasoning/conjecture-run-*.md`, then
   correlate the killer-ref against current ledger state) predicts a later session would observe.
4. Re-hashed the LIVE feature map after the drill: blake3:fd56068a0392319b1fe51fd85f9fdd00041017db1ada672cb72e6191948d1ddc
   — byte-identical to step 1. `git status --porcelain -- docs/thefinalglance/ELAI-FEATURE-MAP.md`
   returned empty (no modification recorded by git). The live ledger was never touched.

Result: PASS. The recheck mechanism works as specified — a changed/removed killer reinstates the
candidate to pending without any manual re-authoring of the run artifact, and the live ledger stays
untouched by the drill.
