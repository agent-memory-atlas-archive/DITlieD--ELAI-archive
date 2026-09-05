# G-7: Per-change confidence PRODUCER + Dual-LLM lethal-trifecta quarantine

STATUS: PLANNED (WAVE 3; depends G-1 live taint; lights up the shipped D-16 conformal consumer)

Objective: ELAI SHIPS the conformal refusal CONSUMER (conformal_calibration.rs: decide / tau / TauUpdater,
the D-16 deliverable) but the composite `claim_score` it consumes is a DEFERRED slot — decide() takes
claim_score as an input PARAMETER and the module's own out-of-scope block (conformal_calibration.rs:30-35)
states the seed corpus carries a "pre-composed" predicted_score, so nothing in code PRODUCES a live
per-change confidence. G-7 builds that producer: fuse the zero-cost intrinsic GGSignals
(sidecar_client.rs:160 — token mean/min log-prob + entropy + generation_novelty), self-consistency
dispersion, and oracle-free generated-test execution-agreement (the F-4 TestOutcomePayload.stdout_hash
signal) into the scalar claim_score that decide() already consumes, so the abstention layer actually FIRES
on real changes (the "honest stop"). Plus the Dual-LLM half: when one context holds Trusted-secret +
Untrusted + a destructive/egress sink (the lethal trifecta), route the untrusted span through a quarantined
sub-LLM with NO tool grants (the FIDES query_llm / HIDE primitive, paper-only today — no code match).
Calibration via post-hoc Platt rescaling is model-size-agnostic (the 7B is a viable target, ECE 0.32->0.03,
Spiess arXiv:2402.02047 CONFIRMED). Estimated effort: ~2 sessions (~10-14 h). In-process / fail-closed at
the abstention boundary; the quarantined sub-LLM is a local inference call, no RunPod at gate time.

Pre-requisites: D-16 conformal_calibration.rs SHIPPED (grep-confirmed 2026-06-02: decide:534, tau_path_default:303,
compute_tau:487, TauUpdater:331, update_tau:380, DEFAULT_EPSILON:55). G-1 taint SHIPPED-after-G-1 (provenance.rs
Provenance:131 / Provenanced<T>:196 / is_untrusted:187,225; capability_gate.rs DESTRUCTIVE_TOOLS:57 /
allow_tool_with_origin:84). GGSignals SHIPPED (sidecar_client.rs:160, A-17/A-25). F-4 TestOutcomePayload.stdout_hash
SHIPPED (ssb.rs:9887). fides-interface.md present (workflow/papers/fides-interface.md, FIDES=arXiv:2505.23643).
Parallel-safe with: G-2, G-4, G-8 (disjoint files). NOT parallel-safe with G-1 (shares provenance/capability_gate)
or any plan editing conformal_calibration.rs / ssb.rs write path in the same window.
Source: workflow/reasoning/ELAI-TRUST-MAXIMIZATION-2026-06-02.md R7. Doctrine: G-0_INDEX.md (W-1..W-10 + GD-1..GD-5).

## Context

Per the trust-max synthesis R7: ELAI's "honest stop" depends on a calibrated per-change confidence that the
conformal abstention layer can threshold. The layer SHIPS (D-16): decide(claim_score: f64, tau: f64) ->
RefusalDecision (conformal_calibration.rs:534) is a PURE CONSUMER — NaN fail-closed to Abstain, clamp to [0,1],
nonconformity = (1 - p), Abstain when nonconformity > tau; tau is the (1 - epsilon) empirical-quantile threshold
(compute_tau:487, DEFAULT_EPSILON 0.1 at :55) maintained by TauUpdater (:331, update_tau:380, atomic tmp+rename,
floor >= 500 via with_min_corpus_size:362). But claim_score is never PRODUCED in code: it is a function
parameter to decide()/decide_and_emit() and a struct field on ConformalRefusalDecidedPayload, and the module's
out-of-scope block (conformal_calibration.rs:30-35) explicitly defers "[conformal] composite-weights loading,"
noting "the seed corpus already carries predicted_score as a pre-composed scalar." So the abstention layer never
fires on a REAL change because no live signal feeds it. That is the G-7 producer gap.

The fusion inputs already land in code. GGSignals (crates/providers/src/sidecar_client.rs:160) carries
mean_logprob (:162), min_logprob (:164), entropy_mean (:166), entropy_max (:168), generation_novelty (:173) —
"extracted from token logprobs and entropy during generation at zero additional inference cost" (A-17/A-25);
generation_novelty is the self-consistency/dispersion proxy. The oracle-free execution signal is the F-4
TestOutcomePayload.stdout_hash (ssb.rs:9877/:9887, blake3 16-char prefix of captured test stdout) — generated-test
execution-agreement (CodeHalu arXiv:2405.00253, Task-Abstention arXiv:2605.17029). G-7 fuses these three into a
single claim_score, calibrated by post-hoc Platt rescaling (model-size-agnostic; the 7B is a viable calibration
target, ECE 0.32->0.03, Spiess arXiv:2402.02047 CONFIRMED), and feeds decide() so the conformal layer actually
abstains on low-confidence changes.

The Dual-LLM half closes the lethal trifecta data-plane: when one context holds a Trusted-secret + an Untrusted
span (provenance.rs is_untrusted:187/:225) + a destructive/egress sink (DESTRUCTIVE_TOOLS:57), the untrusted span
is routed through a quarantined sub-LLM with NO tool grants (FIDES query_llm / HIDE, Design-Patterns Dual-LLM
arXiv:2506.08837). query_llm and HIDE are documented in workflow/papers/fides-interface.md (:56 / :45) but have
ZERO code matches — G-7 BUILDS the primitive; it is not an existing symbol.

ELAI ground truth (grep+source-confirmed 2026-06-02): the conformal layer is the D-16 deliverable
crates/orchestrator/src/conformal_calibration.rs (header :1 "D-16 — UniCR conformal-refusal calibration"). decide
(:534) is a pure consumer; claim_score is an input param never produced (out-of-scope :30-35). tau plumbing:
tau_path_default:303, compute_tau:487, TauUpdater:331 / new:349 / with_min_corpus_size:362 (floor 500) / update_tau:380.
Fusion inputs: GGSignals (sidecar_client.rs:160 — mean_logprob:162, min_logprob:164, entropy_mean:166, entropy_max:168,
generation_novelty:173); F-4 TestOutcomePayload.stdout_hash (ssb.rs:9877/:9887). Trifecta anchors (crates/safety):
provenance.rs Provenance:131 / Provenanced<T>:196 / is_untrusted:187,225; capability_gate.rs DESTRUCTIVE_TOOLS:57 /
allow_tool_with_origin:84. FIDES query_llm:56 / HIDE:45 in workflow/papers/fides-interface.md — paper-only, ZERO
code matches. SSB Pattern-2 precedents: RouteDecisionEvent (ssb.rs:2595, 43rd), TestIntegrityEvent (:2745, 50th),
ToolFilterEvent (:2758, 51st); flat-beacon alt WriteTimeSlopsquatBlocked (:2771). SCHEMA_VERSION authoring-baseline=205,
TOTAL_VARIANTS authoring-baseline=482; G-7-corrected 2026-06-04: the SHIPPED end-state is SCHEMA_VERSION=216
(ssb.rs:2972) / TOTAL_VARIANTS=493 (tests/action_type_lock.rs:605) — the baseline drifted as intervening plans landed;
G-7 added exactly the two variants below. CITATION NOTE: the shipped D-16 code cites
UniCR as arXiv:2503.01345 (conformal_calibration.rs:12,52,526) — that ID is WRONG (it is a prompt-robustness paper);
the real UniCR is arXiv:2509.01455 (a single-author non-peer-reviewed preprint), fixed by the carried patch-N.
G-7 cites UniCR as 2509.01455 and labels its risk-control guarantee MODERATE/unproven.

## Existing state (B/C/D/E/F coverage)
- D-16 conformal_calibration.rs (crates/orchestrator) — SHIPPED CONSUMER. decide:534 (pure; claim_score input param),
  tau_path_default:303, compute_tau:487, TauUpdater:331 / update_tau:380, DEFAULT_EPSILON:55. claim_score PRODUCER is
  the deferred slot (out-of-scope :30-35). G-7 builds the producer; re-implements no part of decide()/tau (GD-1).
- GGSignals (crates/providers/src/sidecar_client.rs:160) — SHIPPED. mean_logprob:162 / min_logprob:164 / entropy_mean:166 /
  entropy_max:168 / generation_novelty:173 (Option<f64>, A-17/A-25, zero-cost). The log-prob + dispersion fusion inputs.
- F-4 TestOutcomePayload.stdout_hash (ssb.rs:9877/:9887) — SHIPPED. The oracle-free generated-test execution-agreement
  signal G-7 fuses (also TestIntegrityEventPayload carries stdout_hash, ssb.rs:25170).
- G-1 taint (crates/safety) — provenance.rs is_untrusted:187/:225, Provenanced<T>:196; capability_gate.rs
  DESTRUCTIVE_TOOLS:57, allow_tool_with_origin:84. The lethal-trifecta detector G-7's quarantine hooks into.
- FIDES (workflow/papers/fides-interface.md, arXiv:2505.23643) — query_llm:56 + HIDE:45 are PAPER primitives; ZERO
  code matches. G-7 builds the quarantined-sub-LLM call (no tool grants) from them.
- ssb.rs (crates/orchestrator) — Pattern-2 precedents RouteDecisionEvent:2595 / TestIntegrityEvent:2745 / ToolFilterEvent:2758;
  flat-beacon alt WriteTimeSlopsquatBlocked:2771. G-7-corrected 2026-06-04: SHIPPED SCHEMA 216 (ssb.rs:2972),
  TOTAL_VARIANTS 493 (tests/action_type_lock.rs:605) — both G-7 variants present in the enum cascade + str map (:9450/:9452) + parse (:10150/:10152) + SQL CHECK (:4756/:4761).

## Reads (mandatory before coding)
- workflow/reasoning/ELAI-TRUST-MAXIMIZATION-2026-06-02.md R7 + GD-5 (honest-limits: UniCR is MODERATE/unproven)
- workflow/papers/fides-interface.md (query_llm:56 + HIDE:45 — the quarantined-LLM primitive to build) + rtbas-interface.md (the trifecta)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\conformal_calibration.rs (full — decide/tau/TauUpdater + the out-of-scope claim_score deferral :30-35)
- F:\.ELAI_workflow\.elai_cc\crates\providers\src\sidecar_client.rs:155-205,500-540 (GGSignals + the CompletionResponse parse/map)
- F:\.ELAI_workflow\.elai_cc\crates\safety\src\provenance.rs:180-230 (is_untrusted) + capability_gate.rs:50-120 (DESTRUCTIVE_TOOLS + allow_tool_with_origin)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\ssb.rs:2595,2745,2758,9877 (Pattern-2 precedents + the F-4 stdout_hash)
- CLAUDE.md HARD RULES: blake3, no .unwrap()/.expect() in prod, Pattern-2 SSB, SCHEMA baseline+N, anti-slopsquatting.

## Tasks (WAVE 3)

1. **Add the claim_score PRODUCER (~160 LOC in a new crates/orchestrator/src/claim_confidence.rs).** `pub fn
   compute_claim_score(gg: &GGSignals, self_consistency: f64, exec_agreement: f64, platt: &PlattParams) -> f64`
   fusing the three signals into [0,1]: (a) the intrinsic GGSignals (mean_logprob/min_logprob/entropy normalized);
   (b) self-consistency dispersion (generation_novelty + a k-sample agreement ratio); (c) oracle-free execution
   agreement (fraction of generated tests whose stdout_hash matches across runs — the F-4 signal). `pub struct
   PlattParams { a: f64, b: f64 }` + `pub fn platt_rescale(raw: f64, p: &PlattParams) -> f64` (post-hoc Platt,
   model-size-agnostic, Spiess). The output is the SCALAR conformal_calibration.rs::decide already consumes — G-7
   wires compute_claim_score's output INTO decide() at the live change-evaluation site (W-8). NO `.unwrap()`/`.expect()`;
   a missing signal degrades conservatively (lower confidence), never silently to 1.0.

2. **Wire the producer into the live abstention path (~50 LOC delta).** At the per-change evaluation boundary,
   call compute_claim_score then decide(claim_score, tau); on RefusalDecision::Abstain the change is HELD / surfaced
   low-confidence (the honest stop) and a `claim_confidence_scored` SSB row is emitted carrying the score + the
   per-signal breakdown + the conformal verdict. Default-ON (W-2): kill-switch ELAI_CLAIM_CONFIDENCE_DISABLE only.
   This is W-8 — compute_claim_score (produced) feeds the SHIPPED decide() in this plan.

3. **Add the Dual-LLM quarantine (~120 LOC in a new crates/orchestrator/src/quarantine.rs).** `pub fn
   detect_trifecta(ctx: &TaintedContext, pending_tool: &str) -> bool` = (context holds a Trusted-secret) AND (any
   span is_untrusted, provenance.rs:187) AND (pending_tool in DESTRUCTIVE_TOOLS, capability_gate.rs:57). On a true
   trifecta, route the untrusted span through `pub async fn query_quarantined(span, expected_type) -> QuarantinedResult`
   — a sub-LLM call with NO tool grants (the FIDES query_llm primitive) whose output is type-constrained and
   re-labelled, so the untrusted span NEVER reaches the tool-caller. Emit `trifecta_quarantined`. NO tool-grant
   leak (stub_ban): the quarantined call's result cannot carry an executable tool invocation.

4. **Add `claim_confidence_scored` + `trifecta_quarantined` SSB action_types (~120 LOC in ssb.rs, W-9).** Two
   single-variant Pattern-2 action_types mirroring RouteDecisionEvent (ssb.rs:2595) / ToolFilterEvent (:2758):
   `ClaimConfidenceScoredPayload { claim_score: f64, logprob_component: f64, self_consistency_component: f64,
   exec_agreement_component: f64, tau: f64, conformal_verdict: String, abstained: bool }` and
   `TrifectaQuarantinedPayload { untrusted_origin_blake3: String (blake3), sink_tool: String, secret_present: bool,
   quarantined: bool }`. sql_str + parse + CHECK + round-trip golden + action_type_lock per type. G-7-corrected
   2026-06-04: SHIPPED SCHEMA_VERSION 216 (ssb.rs:2972), TOTAL_VARIANTS 493 (tests/action_type_lock.rs:605) — the two
   variants landed on the then-current baseline (214/491 per G-7-decisions.md); the +2 ALL_VARIANTS rows + golden regen + the SQL CHECK rows (:4756/:4761) all shipped.

5. **Default-ON kill-switch + honest-limits framing (GD-5) (~30 LOC).** ELAI_CLAIM_CONFIDENCE_DISABLE kill-switch
   only; default ON; observable_effect runs with NO override (W-2). The decision doc + the SSB row label the UniCR
   risk-control guarantee as MODERATE/unproven (single-author preprint arXiv:2509.01455; the distribution-free
   guarantee derives from peer-reviewed conformal risk control, Vovk/Gammerman/Shafer + Angelopoulos arXiv:2208.02814).
   NO over-claim of a calibration guarantee on the 7B (Platt is a heuristic rescale, not a proof).

6. **Frontend pair: ConfidenceDrawer (~260 LOC svelte + ~130 LOC tauri) (W-4).** New
   crates/dashboard-v2/src/lib/drawers/ConfidenceDrawer.svelte listing claim_confidence_scored rows (score + per-signal
   bars + abstained flag) and trifecta_quarantined rows (origin + sink + quarantined); Tauri `get_claim_confidence` +
   `get_trifecta_quarantines`. cross_lang_diff.py PASS. DrawerKey 'confidence'.

7. **Decision doc workflow/reasoning/G-7-decisions.md (~120 LOC).** Records: (a) the 3-signal fusion
   (logprob + self-consistency + execution-agreement) feeding the SHIPPED decide(); (b) Platt rescaling is
   model-size-agnostic (Spiess) but a HEURISTIC, not a guarantee; (c) UniCR is MODERATE/unproven (2509.01455
   single-author preprint), conformal risk-control proper is the peer-reviewed grounding; (d) the Dual-LLM
   trifecta = is_untrusted + secret + DESTRUCTIVE_TOOLS, quarantined via the FIDES query_llm primitive (built, not
   imported); (e) G-7 composes with G-1 (taint), F-4 (execution signal), feeds the G-5 confidence-triage band.

## WIRING_CONTRACT
produces_symbols:
  # G-7-corrected 2026-06-04 (see G-7-decisions.md): shipped module is claim_confidence.rs (not the
  # paper-era confidence_producer.rs name); the 3 fusion-signal fns + the SSB adapter are added below.
  - crates/orchestrator/src/claim_confidence.rs::compute_claim_score
  - crates/orchestrator/src/claim_confidence.rs::platt_rescale
  - crates/orchestrator/src/claim_confidence.rs::PlattParams
  - crates/orchestrator/src/claim_confidence.rs::logprob_confidence
  - crates/orchestrator/src/claim_confidence.rs::self_consistency_agreement
  - crates/orchestrator/src/claim_confidence.rs::exec_agreement_ratio
  - crates/orchestrator/src/quarantine.rs::detect_trifecta
  - crates/orchestrator/src/quarantine.rs::query_quarantined
  - crates/orchestrator/src/quarantine.rs::TrifectaComponents
  - crates/orchestrator/src/quarantine.rs::QuarantineDispatcher
  - crates/orchestrator/src/trifecta_quarantine_ssb_adapter.rs::emit_trifecta_quarantined
  - crates/orchestrator/src/task_runner.rs::emit_claim_confidence_scored
  - crates/orchestrator/src/ssb.rs::SsbActionType::ClaimConfidenceScored
  - crates/orchestrator/src/ssb.rs::ClaimConfidenceScoredPayload
  - crates/orchestrator/src/ssb.rs::SsbActionType::TrifectaQuarantined
  - crates/orchestrator/src/ssb.rs::TrifectaQuarantinedPayload
  - crates/dashboard-v2/src-tauri/src/commands_g7.rs::get_claim_confidence
  - crates/dashboard-v2/src-tauri/src/commands_g7.rs::get_trifecta_quarantines
  - crates/dashboard-v2/src/lib/drawers/ConfidenceDrawer.svelte
consumes_symbols:
  # G-7-corrected 2026-06-04 (see G-7-decisions.md): the producer wire is the REPLACEMENT of the fake
  # claim_score placeholder feeding the ALREADY-SHIPPED conformal decide() (NOT a new consumer of decide()).
  - crates/orchestrator/src/task_runner.rs::dispatch_conformal_refusal_per_claim_hook (:11453) — the SHIPPED
    producer-wire site; the fake placeholder is REPLACED by compute_claim_score fed passing_stdout_hashes(test_rows)
    (real F-4 signal, NON-constant) at the call :11558; feeds the UNCHANGED conformal decide() via agent_pool::dispatch_conformal_refusal (:11578)
  - crates/orchestrator/src/conformal_calibration.rs::decide (the SHIPPED consumer claim_score feeds; UNCHANGED, GD-1)
  - crates/orchestrator/src/conformal_calibration.rs::compute_tau (reused for the producer row's tau_for_row, :11572)
  - crates/providers/src/sidecar_client.rs::GGSignals (mean_logprob/min_logprob/entropy/generation_novelty, :160) — LOCAL-lane log-prob only; cloud uses test+self-consistency (DD1, GD-5); local log-prob threading DEFERRED
  - crates/orchestrator/src/task_runner.rs::passing_stdout_hashes (F-4 stdout_hash execution signal threaded as test_rows; replaces the constant score)
  - crates/orchestrator/src/quarantine.rs::is_egress_tool / context_holds_secret (G-9 secret detector bridge, argv-scope per FIX-3) / tainted_context (G-1 taint, untrusted leg)
  - crates/orchestrator/src/ssb.rs::RouteDecisionEvent (Pattern-2 precedent) / ToolFilterEvent (Pattern-2 precedent)
reachable_from:
  # G-7-corrected 2026-06-04 (see G-7-decisions.md): shipped entrypoints below.
  - the live per-change evaluation hook task_runner.rs::dispatch_conformal_refusal_per_claim_hook (:11453) ->
    compute_claim_score (:11558, real test signal) -> conformal_calibration.rs::decide (UNCHANGED) -> Abstain ->
    change held + emit_claim_confidence_scored (task_runner.rs:11633) SSB row  [PRODUCTION abstention path]
  - the destructive-sink dispatch production_agent_rpc.rs::translate_tool_request -> enforce_trifecta_quarantine_gate
    (:286, called from :1694 with ("bash","git_commit",&parts,ssb)? BEFORE the git_commit dispatch) -> detect_trifecta
    (secret[G-9 argv-scope] + untrusted[G-1] + egress) -> on trifecta: emit_trifecta_quarantined
    {quarantined=false, sub_llm_verdict="not_run_blocked"} + Err (HARD BLOCK)  [PRODUCTION dispatch path].
    NOTE: prod is BLOCK-ONLY. The live no-tool sub-LLM routing (query_quarantined / QuarantineDispatcher) is BUILT as a
    producer-side primitive but is NOT driven in prod (the live client is not threaded to this free-fn chokepoint);
    routing it is a DOCUMENTED DEFERRED follow-up. The load-bearing security property (the untrusted span never reaches
    the real tool) holds via the Err refusal, NOT via a sub-LLM that ran.
  - crates/dashboard-v2/src-tauri/src/commands_g7.rs::get_claim_confidence / get_trifecta_quarantines -> ConfidenceDrawer.svelte  [dashboard]
stub_ban:
  - NO fabricated confidence: a missing GGSignal / execution signal LOWERS the score (conservative), never defaults to 1.0
  - NO claim_score hard-coded constant: it is computed by compute_claim_score from live signals (a constant is a BLOCK)
  - NO tool-grant leak from the quarantined sub-LLM: query_quarantined's result cannot carry an executable tool call
  - NO default-off ENABLE flag (only ELAI_CLAIM_CONFIDENCE_DISABLE kill-switch; default ON, W-2)
  - NO over-claimed UniCR guarantee: the row/doc label it MODERATE/unproven (arXiv:2509.01455 single-author preprint)
  - blake3 only (untrusted_origin_blake3); no blake2/sha2
  - do NOT cite query_llm as an existing code symbol (it is paper-only; G-7 BUILDS query_quarantined)
  - cite UniCR as arXiv:2509.01455 (NOT 2503.01345, which is the wrong ID being fixed by the carried patch-N)
frontend_pair:
  svelte: crates/dashboard-v2/src/lib/drawers/ConfidenceDrawer.svelte
  tauri: crates/dashboard-v2/src-tauri/src/commands_g7.rs::get_claim_confidence + get_trifecta_quarantines
observable_effect:
  trigger: "cd .elai_cc && cargo run -p elai-cli -- <orchestrate a fixture change with low self-consistency / a
    failing generated test; AND a second fixture whose context holds a secret + an injected untrusted span asking
    to call a destructive tool; run with NO env override>"
  expect_ssb: "a claim_confidence_scored row (low claim_score, abstained=true, per-signal breakdown) AND a
    trifecta_quarantined row (untrusted_origin_blake3 populated, sink_tool, quarantined=true) in the live ssb.db"
  expect_artifact: "the low-confidence change is HELD / surfaced low-confidence (the conformal Abstain fired); the
    untrusted span NEVER reaches the tool-caller; a high-confidence clean change is NOT abstained (FP guard);
    ConfidenceDrawer renders both rows"

## DETERMINISTIC_CHECK
# G-7-corrected 2026-06-04 (see G-7-decisions.md): the gate runs with NO opt-in flag (R3 — --run-trigger is
# not a real flag on wiring_contract_check.py and an opt-in on the gate is banned); the contract tests live in
# crates/orchestrator/tests/ (NOT contracts/); the producer wire grep targets task_runner.rs.
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/G-plan/G-7_confidence_producer_dual_llm.md
rg "pub fn compute_claim_score|pub fn platt_rescale|pub struct PlattParams|pub fn logprob_confidence|pub fn self_consistency_agreement|pub fn exec_agreement_ratio" .elai_cc/crates/orchestrator/src/claim_confidence.rs
rg "pub fn detect_trifecta|pub async fn query_quarantined|pub trait QuarantineDispatcher|pub struct TrifectaComponents" .elai_cc/crates/orchestrator/src/quarantine.rs
rg "compute_claim_score" .elai_cc/crates/orchestrator/src/task_runner.rs   # producer wired into the conformal hook (dispatch_conformal_refusal_per_claim_hook) at the live site, feeding the SHIPPED decide()
rg "enforce_trifecta_quarantine_gate" .elai_cc/crates/orchestrator/src/production_agent_rpc.rs   # trifecta gate fired BEFORE git_commit dispatch (:1694); BLOCK-only in prod
rg "ClaimConfidenceScored|claim_confidence_scored|TrifectaQuarantined|trifecta_quarantined" .elai_cc/crates/orchestrator/src/ssb.rs   # enum + str + parse + SQL CHECK (:4756/:4761)
rg "claim_confidence_scored|trifecta_quarantined" .elai_cc/crates/orchestrator/tests/action_type_lock.golden
rg -c "ELAI_CLAIM_CONFIDENCE_ENABLE" .elai_cc/crates/orchestrator/src/claim_confidence.rs   # MUST be 0 (kill-switch DISABLE only)
rg -c "2503.01345" .elai_cc/crates/orchestrator/src/claim_confidence.rs                     # MUST be 0 (use 2509.01455)
rg -c "blake2|sha2" .elai_cc/crates/orchestrator/src/claim_confidence.rs                    # MUST be 0
test -f .elai_cc/crates/dashboard-v2/src/lib/drawers/ConfidenceDrawer.svelte
test -f .elai_cc/crates/orchestrator/tests/g7_confidence_quarantine_contract.rs   # the Tier-1 in-process fixture (W-10)
python .claude/tools/cross_lang_diff.py
test -f workflow/reasoning/G-7-decisions.md
```

## ACCEPTANCE
- [ ] W-1 done-gate: wiring_contract_check.py --run-trigger PASS (produces exist, consumes resolve, reachable_from
  reached by BFS, stub_ban clean, observable_effect RAN + emitted both rows + the conformal Abstain fired + the
  untrusted span was quarantined). cargo-green != DONE.
- [ ] W-2 default-ON: the low-confidence change is abstained with NO env override; ELAI_CLAIM_CONFIDENCE_DISABLE=1
  disables it; no default-off ENABLE flag exists.
- [ ] W-3 production entrypoint: compute_claim_score feeds the live conformal_calibration.rs::decide (:534) at the
  per-change evaluation site, and detect_trifecta fires from the live destructive-sink dispatch (not test-only).
- [ ] W-6 dashboard E2E: ConfidenceDrawer renders a claim_confidence_scored row + a trifecta_quarantined row via dashboard-smoker.
- [ ] GD-2: the claim_score re-computes byte-identically from the per-signal SSB breakdown under a pinned seed
  (the metric is auditable, not just the gate).
- [ ] GD-5 honest-limits: a contract test asserts the row/doc label UniCR (arXiv:2509.01455) MODERATE/unproven and
  does NOT claim a proven 7B calibration guarantee; Platt is labelled a heuristic rescale.
- [ ] FP guard: a high-confidence clean change is NOT abstained; a Trusted-only flow does NOT trigger the quarantine.
- [ ] claim_confidence_scored + trifecta_quarantined round-trip via the SSB roundtrip contract + action_type_lock
  golden; G-7-corrected 2026-06-04: SHIPPED SCHEMA_VERSION 216, TOTAL_VARIANTS 493 (the +2 landed on the live baseline).
- [ ] cargo check --workspace GREEN; auditor PASS on -p elai-orchestrator -p elai-providers -p contracts.
- [ ] No new dependency (conformal_calibration/GGSignals/provenance/blake3 in-tree; anti-slopsquatting — verify before any add).

## WIRING DONE-GATE (clause by clause)
- W-1: wiring_contract_check --run-trigger is ACCEPTANCE 1; compute_claim_score (produced) is consumed by the live
  decide() call (W-8) reached by the per-change evaluation path (W-3).
- W-2: only ELAI_CLAIM_CONFIDENCE_DISABLE; default ON; observable_effect runs with no override.
- W-3: the live per-change abstention path + the destructive-sink dispatch; not test-only.
- W-4: both action_types are human-surfaceable -> ConfidenceDrawer + get_claim_confidence/get_trifecta_quarantines; cross_lang_diff PASS.
- W-5: observable_effect is a runnable orchestrate command -> live ssb.db rows + a held change + a quarantined span. In-process.
- W-6: ACCEPTANCE verifies both rows THROUGH ConfidenceDrawer via dashboard-smoker.
- W-7: conformal_calibration.rs (decide:534, TauUpdater:331), GGSignals (sidecar_client.rs:160), TestOutcomePayload
  (ssb.rs:9877/:9887), provenance.rs is_untrusted:187, capability_gate.rs DESTRUCTIVE_TOOLS:57, ssb.rs Pattern-2
  precedents :2595/:2758 grep-confirmed 2026-06-02; query_llm confirmed PAPER-ONLY (zero code matches) and NOT cited
  as a code symbol; the producer wire site is confirm-at-authoring + the validator re-greps.
- W-8: compute_claim_score (produced) feeds decide() (Task 2); detect_trifecta/query_quarantined (produced) wired in
  the dispatch (Task 3); both payloads emitted + consumed by the Tauri commands (Task 6). No produced symbol caller-less.
- W-9: TWO Pattern-2 single-variant action_types (mirror RouteDecisionEvent/ToolFilterEvent); action_type golden +
  cross_lang_diff; G-7-corrected 2026-06-04: SHIPPED SCHEMA 216 / TOTAL_VARIANTS 493 (golden ordinals 0069 claim_confidence_scored + 0464 trifecta_quarantined).
- W-10: Tier-1 in-process at the abstention boundary (signal fusion + decide()) is the fail-closed done-gate via the
  in-process g7_confidence_quarantine_contract.rs fixture; genuinely fail-closed (missing signal -> lower confidence,
  trifecta -> hard BLOCK). G-7-corrected 2026-06-04: in PROD the trifecta path is BLOCK-ONLY (the dispatch is refused
  with Err before the real tool runs); the quarantined no-tool sub-LLM (query_quarantined / QuarantineDispatcher) is
  BUILT as a producer primitive but NOT driven in prod (live client not threaded to the free-fn chokepoint) — that live
  routing is a DOCUMENTED DEFERRED follow-up, NOT a claim that a sub-LLM ran. Tier-2 (a live agent run firing a real
  abstention / a real trifecta BLOCK, W-5) is separately tracked + RunPod-gated.
- GD-1: COMPOSES-WITH D-16 (decide/tau, consumed) + G-1 (taint) + F-4 (execution signal); re-implements none. GD-3: no
  SWE-bench/capability claim; the confidence is a per-change abstention signal. GD-5: UniCR labelled MODERATE/unproven; no
  fabricated 7B calibration guarantee.

## Tests to add
# G-7-corrected 2026-06-04 (see G-7-decisions.md): the SHIPPED contract test is ONE combined in-process file at
# crates/orchestrator/tests/ (NOT crates/contracts/tests/) — it is the Tier-1 fail-closed done-gate fixture (W-10).
- crates/orchestrator/tests/g7_confidence_quarantine_contract.rs (~SHIPPED): (a) a low-self-consistency change -> low
  claim_score -> decide() Abstain + claim_confidence_scored; (b) a high-confidence clean change -> Answer (not abstained,
  FP guard); (c) a missing GGSignal/execution signal LOWERS the score (never 1.0); (d) compute_claim_score re-computes
  identically from the per-signal breakdown under a pinned seed (GD-2); (e) platt_rescale monotone; (f) score VARIES with
  the F-4 test signal (the floor-fake-replacement proof); (g) secret + untrusted + egress -> detect_trifecta true ->
  query_quarantined (in-process via a CapturingDispatcher fixture) -> the untrusted span NEVER reaches a tool-caller;
  (h) a Trusted-only / missing-leg flow -> detect_trifecta false (FP guard); (i) both Pattern-2 variants round-trip.

## Files touched
# G-7-corrected 2026-06-04 (see G-7-decisions.md): shipped module = claim_confidence.rs; the wire site is
# task_runner.rs (the conformal hook), NOT conformal_calibration.rs; the SSB emit lives in a dedicated adapter;
# the contract test is one combined file under crates/orchestrator/tests/.
- CREATE: crates/orchestrator/src/claim_confidence.rs (compute_claim_score + platt_rescale + logprob_confidence + self_consistency_agreement + exec_agreement_ratio + PlattParams)
- CREATE: crates/orchestrator/src/quarantine.rs (detect_trifecta + TrifectaComponents + query_quarantined + QuarantineDispatcher + is_egress_tool + context_holds_secret)
- CREATE: crates/orchestrator/src/trifecta_quarantine_ssb_adapter.rs (emit_trifecta_quarantined)
- CREATE: crates/dashboard-v2/src-tauri/src/commands_g7.rs (get_claim_confidence + get_trifecta_quarantines)
- CREATE: crates/dashboard-v2/src/lib/drawers/ConfidenceDrawer.svelte
- CREATE: crates/orchestrator/tests/g7_confidence_quarantine_contract.rs (the Tier-1 combined contract fixture)
- CREATE: workflow/reasoning/G-7-decisions.md
- MODIFY: crates/orchestrator/src/lib.rs (pub mod claim_confidence; pub mod quarantine; pub mod trifecta_quarantine_ssb_adapter;)
- MODIFY: crates/orchestrator/src/task_runner.rs (the producer wire: dispatch_conformal_refusal_per_claim_hook :11453 REPLACES the fake claim_score placeholder with compute_claim_score fed passing_stdout_hashes(test_rows) at :11558; emit_claim_confidence_scored :11633; the SHIPPED conformal decide() consumer is UNCHANGED, GD-1)
- MODIFY: crates/orchestrator/src/production_agent_rpc.rs (the trifecta gate: enforce_trifecta_quarantine_gate :286, called from :1694 BEFORE the git_commit dispatch; BLOCK-only in prod with Err; live sub-LLM routing DEFERRED)
- MODIFY: crates/orchestrator/src/ssb.rs (ClaimConfidenceScored + TrifectaQuarantined variants + payloads + sql_str + parse + SQL CHECK :4756/:4761 + golden)
- MODIFY: crates/orchestrator/tests/action_type_lock.rs (TOTAL_VARIANTS -> 493 + 2 ALL_VARIANTS rows) + tests/action_type_lock.golden (2 rows: ordinals 0069 + 0464)
- MODIFY: crates/dashboard-v2/src/lib/api/types.ts + DrawerHost.svelte (DrawerKey 'confidence' + 3-label sync)

## Citations / Grounding
- workflow/reasoning/ELAI-TRUST-MAXIMIZATION-2026-06-02.md R7 (the gap + grounding). Post-hoc Platt rescaling
  model-size-agnostic, ECE 0.32->0.03 = Spiess arXiv:2402.02047 (CONFIRMED) — the 7B is a viable calibration target.
  Oracle-free execution signal = CodeHalu arXiv:2405.00253 + Task-Abstention arXiv:2605.17029 (CONFIRMED). Dual-LLM
  quarantine = Design-Patterns arXiv:2506.08837 + FIDES query_llm/HIDE (arXiv:2505.23643, workflow/papers/fides-interface.md:56/:45).
  UniCR = arXiv:2509.01455 (Krasniqi, single-author non-peer-reviewed preprint — MODERATE/unproven; the distribution-free
  guarantee derives from peer-reviewed conformal risk control, Vovk/Gammerman/Shafer + Angelopoulos arXiv:2208.02814).
  NOTE: the shipped D-16 code mis-cites UniCR as arXiv:2503.01345 (a prompt-robustness paper) at conformal_calibration.rs:12/52/526
  — fixed by the carried patch-N; G-7 cites 2509.01455.
- ELAI ground truth (grep+source-confirmed 2026-06-02): conformal_calibration.rs decide:534 (pure consumer, claim_score
  input param; producer deferred :30-35), tau_path_default:303, compute_tau:487, TauUpdater:331/:380, DEFAULT_EPSILON:55;
  GGSignals sidecar_client.rs:160 (mean_logprob:162/min_logprob:164/entropy_mean:166/entropy_max:168/generation_novelty:173);
  TestOutcomePayload.stdout_hash ssb.rs:9877/:9887; provenance.rs is_untrusted:187/:225 / Provenanced:196; capability_gate.rs
  DESTRUCTIVE_TOOLS:57 / allow_tool_with_origin:84; FIDES query_llm:56/HIDE:45 (paper-only, zero code matches); ssb.rs
  Pattern-2 RouteDecisionEvent:2595 / TestIntegrityEvent:2745 / ToolFilterEvent:2758. G-7-corrected 2026-06-04: those
  were the 2026-06-02 authoring anchors (SCHEMA 205 / TV 482); the SHIPPED end-state is SCHEMA_VERSION 216 (ssb.rs:2972) /
  TOTAL_VARIANTS 493 (tests/action_type_lock.rs:605), and the producer wire is task_runner.rs:11453/11558 (the conformal
  hook), feeding the UNCHANGED conformal decide() (GD-1) — NOT a :30-35 producer slot in conformal_calibration.rs.
- CLAUDE.md HARD RULES: blake3, no .unwrap()/.expect() in prod, Pattern-2 SSB, SCHEMA baseline+N, anti-slopsquatting.

## Open questions / risks
- TOP failure mode: an over-confident claim_score that suppresses a needed abstention (the gate stops firing when it
  should). MITIGATION: conservative degradation on missing signals; the FP guard works in BOTH directions (a clean
  change is not abstained AND a low-confidence change IS); measure the abstention rate against the D-17 honest-refusal
  SLA band (5%-50%) before trusting it. UNVERIFIED until measured: ELAI's own per-change calibration error (no baseline).
- Platt rescaling needs calibration data (the D-16/D-30 corpus); on a cold corpus the rescale is identity-ish — document
  that the producer is honest about an un-calibrated cold start (low-confidence-by-default), not falsely confident.
- The self-consistency signal needs k samples per change (cost); confirm at authoring whether k-sampling is affordable
  on the local lane or whether generation_novelty alone is the dispersion proxy. Record the k choice in the decision doc.
- UNVERIFIED: UniCR (arXiv:2509.01455) is a single-author non-peer-reviewed preprint — its risk-control guarantee is
  MODERATE; do NOT claim ELAI inherits a proven conformal guarantee. The peer-reviewed grounding is conformal risk
  control proper (Angelopoulos arXiv:2208.02814).

## Next
G-7 feeds the G-5 confidence-triage band (a reviewer reads low-confidence changes first). G-2's RRA journals the
claim_confidence_scored + trifecta_quarantined rows into the replayable artifact (the abstention decisions are part of
the reproducible record). G-1's live taint is the trifecta detector's input; F-3's Tool-Filter (control-plane) + G-7's
quarantine (data-plane Dual-LLM) together are the full lethal-trifecta defense. The claim_score producer also unblocks
the D-17 honest-refusal SLA (a real per-change confidence to threshold the refusal rate against).

End of G-7.
