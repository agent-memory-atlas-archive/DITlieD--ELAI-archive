# Tripwire Stack: KL + Δρ + IQR + Perplexity + DDM (D-4)

- **ELAI ids:** D-4 / Phase 4H.VI / companion §3.6 + §3.9
- **Spec status (as designed):** PLANNED. Feature-map tags DARK (`ELAI_TRIPWIRE_ENABLE`; `observe_post_consensus_tripwires` zero prod callers) (`ELAI-FEATURE-MAP.md:1678-1681`).
- **Crate / module (spec):** `crates/orchestrator/src/tripwire.rs`; ingest hook in `task_runner.rs`
- **Purpose of this file:** enough architecture to rebuild the five orthogonal tripwires, concurrent-fire rule, and append-only auto-revert. Not a status ledger.

## 1. What it is

The **tripwire stack** is five published detectors — KL divergence, proxy–gold correlation drop (Δρ), IQR outliers, action-perplexity anomalies, and DDM concept drift — watched over a sliding window of actions. When **≥2** tripwires fire in the same window (default 50), the orchestrator auto-reverts the most recently Promoted template to its parent and marks it `Retired` with `RetirementReason::Degradation`. This replaces the ungrounded “10% over 3 pipelines” / “MSI > 0.3” rule (`D-4_tripwire_stack.md:3-8`, `D-SUMMARY_INDEX.md:40-45`).

## 2. Why it exists

Reward-hacking prevalence is 21.3% of expert-validated episodes; the four arxiv:2507.05619 tripwires plus DDM catch ≈80% when ≥2 concur. If this organ is missing, a gamed PPM or MSI sigmoid can keep a bad template Promoted. If it is faked (the model watches its own scores), the model evaluates the modification it just shipped. Thresholds are L3-locked; the agent does not set them (`D-4_tripwire_stack.md:18-42`, `:222-223`).

## 3. How it works (runtime)

1. **Who calls it.** `task_runner.rs` post-action hook: every `ActionRecord` → `TripwireOrchestrator::ingest` (`D-4_tripwire_stack.md:130-135`, `:212-213`).
2. **Window (deterministic).** Append to `VecDeque<ActionRecord>`; trim oldest to `config.window_size` (default 50). Must not silently drop; trim is deterministic (`:134-136`, `:218-219`).
3. **KL (deterministic math, data from outputs).** Token-distribution histograms of current vs baseline step outputs. `kl_divergence` standard formula; arrays sum to 1.0 within epsilon; identical → 0.0; zero baseline bin with nonzero current → `f32::INFINITY` (not 0, not NaN). Fire if KL > `kl_spec_tau` (0.3) (`:90-97`, `:216-217`).
4. **Δρ (deterministic).** Pearson on proxy vs gold in two windows. `len` must match; zero-variance → 0.0. Fire if `(corr_a - corr_b) > rho_delta_threshold` (0.5). Proxy = D-3 per-step `VerificationResult`; gold = deterministic compile/test pass. Held-out pass-rate is a Δρ source once D-6 lands (`:46-47`, `:99-105`, `:208`).
5. **IQR (deterministic).** Q1, Q3, IQR = Q3−Q1. Fire if any per-task score `> Q3 + multiplier * IQR` (multiplier 3.0). `event.value` = the outlier (`:107-111`).
6. **Perplexity (deterministic on GG proxy).** Mean + stddev of `ppl_values` (from Phase 4G `mean_logprob` + entropy). Fire if any value `> μ + sigma * σ` (sigma 2.0) (`:48`, `:113-117`, `:209`).
7. **DDM (deterministic FSM, Lu et al §3.2).** Track error rate `p_i` and `s_i = sqrt(p_i*(1-p_i)/i)`. NORMAL → WARNING when `p_i + s_i >= p_min + 2*s_min`. WARNING → DRIFT when `p_i + s_i >= p_min + 3*s_min`. Emit `TripwireEvent` on entering **Drift**. Reset stats on Drift → Normal (`:119-128`).
8. **Concurrent fire (deterministic).** If `fired_kinds.len() >= consensus_fire_count` (default 2) inside the current window: emit `TripwireFiredConcurrent`, then auto-revert (`:137-142`).
9. **Auto-revert (deterministic, fail-closed, append-only).** **SSB `TripwireAutoRevert` FIRST**, then `archive_chain.retire(most_recently_promoted, RetirementReason::Degradation)` via `ArchiveChain.append` (D-1: no delete). Orchestrator switches to `parent_id`. Intent is on the journal even if retire fails (`:38-42`, `:137-142`, `:191-193`, `:220-221`).
10. **Per-fire journal.** Each detector also emits `TripwireFired { kind, value, threshold, signal_source, audit_id }` (`:145-146`).

The model may generate the actions that fill the window. It may **not** choose thresholds, suppress events, skip ingest, or grade whether a revert should happen.

## 4. Types, enums, objects

Copy from `D-4_tripwire_stack.md:76-132`. Do not invent fields.

```
enum TripwireKind { KlDivergence, CorrelationDrop, IqrOutlier, PerplexityAnomaly, Ddm }
  // closed; L3-locked

struct TripwireConfig {
  kl_spec_tau: f32,                 // default 0.3
  rho_delta_threshold: f32,         // 0.5
  iqr_outlier_multiplier: f32,      // 3.0
  ppl_anomaly_sigma: f32,           // 2.0
  ddm_warning_threshold: f32,       // 2.0
  ddm_drift_threshold: f32,         // 3.0
  consensus_fire_count: u8,         // 2
  window_size: u32,                 // 50
}

struct TripwireEvent {
  kind: TripwireKind,
  fired_at: DateTime<Utc>,
  value: f32,
  threshold: f32,
  signal_source: SignalSource,
  audit_id: Uuid,
}

enum SignalSource { CurrentSession, HeldOut, CrossDomain }

struct DdmState {
  sample_count: u32,
  p_min: f32,
  s_min: f32,
  current_p: f32,
  current_s: f32,
  level: DdmLevel,
}
enum DdmLevel { Normal, Warning, Drift }
  impl DdmState { update(success: bool) -> Option<TripwireEvent> }

struct TripwireOrchestrator {
  config: TripwireConfig,
  window: VecDeque<ActionRecord>,
  ddm_state: DdmState,
  ssb_tx: SsbSender,
  archive_chain: Arc<RwLock<ArchiveChain>>,
}
  fn ingest(action: ActionRecord)
```

**Detectors:** `kl_divergence`, `check_kl`, `pearson_correlation`, `check_correlation_drop`, `iqr_outlier_check`, `ppl_anomaly_check`, `DdmState::update` (`:90-128`).

**SSB SCHEMA 49→50** (`:144-150`):

```
TripwireFired { kind: String, value: f64, threshold: f64, signal_source: String, audit_id: Uuid }
TripwireFiredConcurrent { kinds: Vec<String>, window_size: u32, audit_id: Uuid }
TripwireAutoRevert { template_id: TemplateId, parent_template_id: Option<TemplateId>,
                     fired_kinds: Vec<String>, audit_id: Uuid }
```

**Toml `[tripwire]`** — paper-cited defaults, L3-LOCKED (`:152-157`):

```
kl_spec_tau = 0.3
rho_delta_threshold = 0.5
iqr_outlier_multiplier = 3.0
ppl_anomaly_sigma = 2.0
ddm_warning_threshold = 2.0
ddm_drift_threshold = 3.0
consensus_fire_count = 2
window_size = 50
```

Companion also names cryptographic checksums (D-8) and syscall tracing for RF access (D-9) as related defenses, **not** members of the five-kind enum (`:25-27`).

## 5. Control flow / state machine

**Owner of the loop:** `TripwireOrchestrator`. Not the model, not MSI.

**DDM:**

```
Normal --(p+s >= p_min + 2 s_min)--> Warning
Warning --(p+s >= p_min + 3 s_min)--> Drift  [emit TripwireEvent]
Drift  --(recovery; spec: reset stats on entering Normal from Drift)--> Normal
```

(`:122-128`)

**Concurrent revert:**

```
ingest → trim window → run 5 checks → collect TripwireEvent
if count(distinct firing kinds in window) >= consensus_fire_count:
    SSB TripwireFiredConcurrent
    SSB TripwireAutoRevert          # BEFORE retire
    ArchiveChain.append retire(Promoted → Retired/Degradation)
    switch runtime template to parent_id
```

**Forced rules:**

- Frozen templates are not the revert **target** of this path; revert acts on **most recently Promoted** (`:38-40`). D-1: Promoted→Retired allowed; Frozen→* forbidden.
- Physical delete banned (`:191-193`).
- Thresholds from toml, never hardcoded in the orchestrator constructor (`:222-223`).

**UNCERTAIN:** Warning-level DDM does not emit `TripwireEvent` (only Drift is named). Whether Warning counts toward `consensus_fire_count`: spec says emit on Drift only (`:127-128`).

## 6. Grounding, receipts, gates

Receipts are `TripwireEvent` + three SSB types + archive status change. Gold for Δρ is **deterministic** pass/fail, not an LLM grade (`:46-47`).

**Never allowed as a confidence signal:** MSI sigmoid; “10% over 3 pipelines”; model self-report of KL; dropping INFINITY KL to 0; agent-edited thresholds.

**Coverage target (paper, not a soft pass):** ≥2 fires catch ~80%; 8% of confirmed hacks fire 0 tripwires (accepted miss) (`:188-190`). Integration requires **≥3 of 5** kinds by action 65 after injection at 60 (`:175-177`).

## 7. Budgets, limits, ceilings

| Knob | Value | Scope | Source |
|---|---|---|---|
| KL τ_spec | 0.3 | current vs baseline dist | arxiv:2507.05619 (`:21`) |
| Δρ | 0.5 | 50-step windows | same (`:22`) |
| IQR | Q3 + 3·IQR | per-task score variance | (`:23`) |
| PPL | μ + 2σ | action perplexity | (`:24`) |
| DDM warn / drift | 2.0 / 3.0 × s_min | error-rate FSM | Lu (`:84-85`, `:122-123`) |
| consensus_fire | 2 | concurrent kinds | (`:38`, `:85`) |
| window | 50 actions | sliding | (`:38-39`, `:85`) |
| integration horizon | inject @60, detect by @65 | 5-step lag | (`:175-179`) |
| unit tests | ≥15 | orchestrator | (`:286`) |
| SCHEMA | 49 → 50 | SSB | (`:144`) |

**Window overflow:** trim oldest; do not drop the new event (`:218-219`).

**KL overflow:** INFINITY is a fire, not a NaN skip (`:216-217`).

## 8. Roles and neighbors

Orchestrator-owned. No agent role tunes tripwires.

**Consumes:** D-1 `ArchiveChain` retire/append; D-3 `VerifierConsensusPass` count as gold/proxy; Phase 4G GG `mean_logprob`+entropy; D-6 held-out as KL/Δρ baseline once present; D-7 CrossDomain `SignalSource` (`:206-211`, `:326-327`).

**Produces:** events + SCHEMA 50 + auto-revert lineage. D-5 may consume fires as posterior input (`:314-316`).

**Must not fork:** a prompt “please revert if you notice drift”; deleting archive rows; a second in-agent tripwire with looser τ.

**Parallel-safe with D-5, D-6, D-8** (`:12-14`). D-3 is a pre-req (not parallel).

## 9. Config / flags / defaults

`[tripwire]` keys = `TripwireConfig` defaults (section 4). L3-LOCKED; D-9 enforces; D-4 only documents (`:157`, `:319-320`).

Feature-map `ELAI_TRIPWIRE_ENABLE` / observe-only prod is **not** in the D-4 session file (section 16). Rebuild the D-4 ingest-always design unless D-9 later gates it.

Observable also names `tripwire.toml` defaults matching paper values (`:230-231`) vs `elai_cc.toml [tripwire]` — treat jsonl/toml dual file as UNCERTAIN.

## 10. Implementation map

| Symbol | Spec path |
|---|---|
| `orchestrator/src/tripwire.rs` | all types + 5 detectors + orchestrator (`:76-142`) |
| `task_runner.rs` post-action `ingest` | (`:303-304`) |
| `ssb.rs` SCHEMA 50 | 3 variants (`:300-302`) |
| `elai_cc.toml [tripwire]` | (`:305`) |
| integration | `orchestrator/tests/tripwire_integration.rs` (`:173-181`) |
| contract | `contracts/tests/tripwire_contract.rs` (`:308`) |
| decisions | `workflow/reasoning/D-4-decisions.md` |
| papers | `reward-hacking-detection-2507.05619`, `concept-drift-2004.05785`, `rm-overoptimization-2210.10760` (`:66-72`) |

D-1 did not name `ArchiveChain.retire`; D-4 requires it as append+status (`:139-140`, `:279-280`). Rebuild: add `retire` as a typed append, not a new delete API.

## 11. Journal / SSB events

Three variants (section 4). `rg` expect ≥12 hits (`:239-242`).

Observable (`:224-229`): synthetic 100-action stream, 80% failure from action 60 → ≥3 `TripwireFired` distinct kinds by 65; ≥1 concurrent; ≥1 auto-revert; JSONL shows Retired/Degradation.

Cloud B: 50 healthy `/task` actions → **zero** `TripwireFiredConcurrent` (`:292-293`).

Order invariant: AutoRevert SSB **before** `archive.retire` (`:220-221`).

## 12. Tests and observables

Named unit tests (`:159-171`):

| Id | Assert |
|---|---|
| T-kl-id | identical dist → 0.0 |
| T-kl-disjoint | disjoint → INFINITY |
| T-kl-hand | uniform vs peaked matches hand-computed finite KL |
| T-pearson | perfect linear ±1.0; uncorrelated 0.0; window drop reflects |
| T-iqr-fire | outlier at Q3+4·IQR fires (mult=3) |
| T-iqr-quiet | Q3+2.5·IQR does not |
| T-ppl-fire | μ+2.5σ fires (σ=2) |
| T-ppl-quiet | μ+1.5σ does not |
| T-ddm | 100 success → Normal; then 50 at 80% fail → Warning then Drift |
| T-orch | consensus_fire=2 + 3 distinct kinds → Concurrent + AutoRevert |

Integration (`:173-181`): 100-action stream, regression @60; ≥3/5 kinds by 65; AutoRevert by 65 (within 5 steps); archive most_recent_promoted Retired/Degradation.

Cloud A: same via harness; until D-8, inspect JSONL (`:288-291`). Cloud B: no concurrent on healthy 50 (`:292-293`).

Done-gates: ≥15 orchestrator tests; SCHEMA 50; paper-cited toml defaults (`:270-296`).

## 13. Rebuild contract

Another harness has **this** organ iff all of the following are true:

- Five kinds: KL, Δρ (Pearson drop), IQR, perplexity, DDM — not a single MSI number.
- Published defaults: 0.3 / 0.5 / 3.0 / 2.0 / DDM 2 and 3 / fire-count 2 / window 50.
- KL zero-baseline → INFINITY.
- DDM Normal→Warning→Drift with emit on Drift.
- ≥2 concurrent kinds in the window → SSB concurrent + auto-revert.
- Revert is append-only Promoted→Retired/Degradation + switch to `parent_id`; no delete.
- AutoRevert journal row lands **before** retire.
- Thresholds from L3 config, not constructor literals the agent can patch.
- Integration: inject @60, ≥3 kinds by @65.
- Model does not decide fire or revert.

A “10% over 3 pipelines” counter, an MSI sigmoid, or an LLM watching loss is a cousin, not this organ.

## 14. Non-goals / banned designs

- BAI algorithm — D-5 (`:314-316`).
- `elai archive verify` CLI — D-8 (`:317-318`).
- LockedKeysAuditor — D-9 (`:319-320`).
- Dashboard stream — D-26 (`:321`).
- Physical archive delete (`:191-193`).
- Hardcoded thresholds in `TripwireOrchestrator::new` (`:222-223`).
- Silent drop on full window (`:218-219`).
- KL NaN/0 on zero baseline (`:216-217`).
- Replacing DGM’s ungrounded 10% rule with another unpublished heuristic (`:33-36`).
- Model self-eval of whether the stack should fire.

## 15. Source citations

Primary: `docs/thefinalglance/D-plan/D-4_tripwire_stack.md` (full file). Index: `D-SUMMARY_INDEX.md:40-45`, `:271`. Feature map: `ELAI-FEATURE-MAP.md:1678-1681`. Companion (not in this tree): §3.6 lines 279-292, §3.9 lines 339-343 as cited `:53-54`. Papers: 2507.05619, 2004.05785, 2210.10760 Gao RM overoptimization (`:66-69`).

## 16. Gaps

Label **UNCERTAIN**; do not fill:

- `ActionRecord` / `StepOutput` field lists (`:95-96`, `:134`).
- Which window is `window_a` vs `window_b` for Δρ (baseline vs current) (`:102-105`).
- Vocab-bin scheme for KL histograms (`:96-97`).
- How GG `mean_logprob`+entropy become `ppl_values` (`:113-117`, `:209`).
- Whether DDM Warning counts toward concurrent fire (`:127-128` vs `:137`).
- D-1 has no `retire` method name — only append + status enum (`D-1` vs `D-4:139-140`).
- `elai_cc.toml [tripwire]` vs observable `tripwire.toml` (`:230-231` vs `:152`).
- `ELAI_TRIPWIRE_ENABLE` / observe-only in feature-map vs always-ingest in D-4 (`ELAI-FEATURE-MAP.md:1678-1681`).
- `check_kl` baseline source (parent template? held-out? session start?) (`:95-97`, `:326-327`).
- IQR: “any value flagged” vs count of outliers; which value if many (`:110-111`).
- Concurrent count: distinct `TripwireKind` vs raw event count (`:137` “tripwires fire” vs test “3 events from different kinds” `:170-171`).
- Recovery path: who calls DDM reset besides “entering Normal from Drift” (`:128`).
- Page-Hinkley / ADWIN named in D-SUMMARY cites (`D-SUMMARY_INDEX.md:43`) but **not** in D-4 enum — do not add them here.
