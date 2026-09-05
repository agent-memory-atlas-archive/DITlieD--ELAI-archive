# J-55: model-strength modality router — win token-efficiency AND agentic autonomy by routing the ACTION SURFACE (CodeAct vs constrained tool-call) per model, never one-size-fits-all

STATUS: PLANNED (split WAVE-1 offline half + DEFERRED-gated inference half; wargame-refit 2026-08-27 to pass plan-gate 10/10. Design RATIFIED 2026-08-11 — seam RESTORED in text, not redesigned. Baseline gate was FAIL (P1 P3 P4 P5 P6 P7 P10 fail; P2 P9 pass). Refit work-product + fresh grounding evidence: conductor/opus_walk/evidence/J-55/plan-refit.md) [KEYSTONE of the win-both design]

Objective: the "agentic needs tools, tools cost tokens, efficient means no tools" tension is false — per-turn token cost is three INDEPENDENT sinks (tool-schema, action-envelope, edit-output) attacked separately. J-55 ships a ModalityRouter that resolves the ACTION MODALITY per (model, role): CodeAct (the model emits code that imports tools as functions — collapses the schema + envelope sinks) for capable lanes; constrained single-tool-call under XGrammar + prompt-compiled minimal schemas (I-36/C-25) for the weak local lane; a UNIVERSAL SEARCH/REPLACE-diff edit path (E-35/E-18/H-3) regardless of modality (the edit-output sink, model-independent); progressive tool disclosure (H-4/H-34/J-24) on the schema'd path. KEYSTONE rationale (grounded): naive "CodeAct everywhere" LOSES on ELAI's own canonical Qwen2.5-Coder-7B local lane — open-source models collapse to ~12% on multi-tool code-mode (arXiv:2402.01030), viability appears ~14B (arXiv:2509.04508), and the 7B lane is below that threshold. So the ROUTER is the load-bearing unit, not CodeAct. Hard boundary: J-55 does NOT author CodeAct, SEARCH/REPLACE, the adherence bench, prompt compilation, or the tier router — all exist as E-35/J-51/I-36/E-31; J-55 is the deterministic dispatcher that SELECTS among them and the J-34-style bench that grounds the selection. GD-1 do-NOT-re-implement: E-35 CodeAct+SEARCH/REPLACE matcher; J-51 ModelCapabilityProfile.tool_call_adherence; I-36 CompiledPromptStore; E-31 tier routing; H-4/H-34/J-24 progressive disclosure; J-34 calibration harness.

Estimated effort: ~11 h total → UNIT W1 ~7 h (offline, buildable NOW); UNIT D1 ~4 h (executes only after its named gates open). In-process at dispatch (a pure decision over ModelCapabilityProfile + tier); the done-gate calibration bench is network/sidecar-gated for the live per-model crossover numbers.
Pre-requisites (re-grep per W-7): E-35 STRUCTURED OUTPUT SUBSTRATE (CodeAct + SEARCH/REPLACE matcher); J-51 TOOL-CALL ADHERENCE FLOOR (ModelCapabilityProfile.tool_call_adherence — PLANNED and NOT yet in code, re-verified zero hits 2026-08-27; if not yet shipped, J-55 consumes its profile field as the gating input and is sequenced AFTER it); E-31 tier matrix (RuntimeRoutingPolicy); I-36 CROSS-MODEL PROMPT COMPILATION (CompiledPromptStore); H-4 Tool Search + H-34 MCP defer; J-34 EMPIRICAL CONTEXT CALIBRATION (the bench pattern to mirror).
Parallel-safe with: J-54 (disjoint files — modality vs inference backend).
Source: steal-free-api-rotation-2026-06-20.md S8; this session's spec-eval reasoning + oracle run oracle-agentic-token-efficiency-codeact-toolsearch-2026. Doctrine: J-0_INDEX.md.

## Context
Why this matters (external evidence, arXiv IDs WebSearch-verified 2026-06-20, scope-tagged per HD-1):
- CodeAct (executable code actions) beats JSON/text action formats by up to ~20% success with ~30% fewer actions on multi-tool tasks — but this is a FRONTIER-MODEL result; open-source models (Mistral-70B class) collapse to ~12.2% on M3ToolEval in code-mode (arXiv:2402.01030, ICML 2024, VERIFIED). [DESIGN, large-model-scoped]
- the ~14B threshold: 14B code-tuned models (Qwen2.5-Coder-14B / Phi-4-14B) approach GPT-4o on multi-agent code tasks only WITH progressive sub-task fine-tuning (arXiv:2509.04508 ProST, VERIFIED). zero-shot code-mode at 3-7B is essentially unproven. [DESIGN, directional]
- grammar-constrained decoding enforces format but costs 3.6-8.2x latency on 7-9B models, and offline prompt-optimization beats per-inference grammar on the combined accuracy+latency metric (arXiv:2605.02363 "When Correct Isn't Usable", VERIFIED). [DESIGN — directly motivates leaning on I-36 prompt-compilation over heavy grammar on the local lane]
Correctness boundary: the modality decision is DETERMINISTIC (TD-3) — it reads ModelCapabilityProfile + tier, the LLM never picks its own modality. Mechanism: ModalityRouter::resolve(model, role) -> ActionModality; the executor's emission/parse path branches on it; the done-gate is a measured per-(model,lane) crossover (J-34 pattern), not a folklore threshold. The three sinks: CodeAct collapses tool-schema + action-envelope; SEARCH/REPLACE collapses edit-output (model-independent, always on); progressive disclosure caps the schema'd path (Tool Search ~77K->8.7K, H-4, vendor-reported).

## Existing state (B/C/D/E/F/G/H/I/J coverage)
- E-35 STRUCTURED OUTPUT SUBSTRATE [PARTIAL]: XGrammar + CodeAct + SEARCH/REPLACE; SearchReplaceMatcher not yet wired into the dispatcher (feature-map §1.9). J-55 SELECTS it; the matcher wiring is E-35's deliverable (compose, do not author).
- J-51 TOOL-CALL ADHERENCE FLOOR [PLANNED]: per-local-model semantic adherence bench -> ModelCapabilityProfile.tool_call_adherence -> role gate (feature-map §1.9). The gating INPUT for J-55. PLAN-asserted; sequenced after J-51.
- E-31 tier matrix [PARTIAL]: RuntimeRoutingPolicy + RouteDecisionEvent (cloud_mode.rs:284 / escalation_policy.rs). The tier signal (local-7B vs cloud) feeding J-55.
- I-36 CROSS-MODEL PROMPT COMPILATION [DARK(ELAI_PROMPT_COMPILE_DISABLE, default-ON)]: per-(role,backend) CompiledPromptStore (feature-map §1.9). Supplies the minimal compiled schemas for the constrained-call lane (the prompt-optimization leg that beats grammar latency, 2605.02363).
- H-4 Tool Search [PARTIAL] + H-34 MCP defer + J-24 context-frugal tools [PLANNED]: progressive disclosure on the schema'd path.
- J-34 EMPIRICAL CONTEXT CALIBRATION [PLANNED]: `elai bench context-fill` measured-curve pattern. J-55's done-gate mirrors it (a modality-crossover bench).
- B-10 ModelCapabilityProfile [WIRED]: the per-model record the modality decision hangs off.

## Reads (mandatory before coding)
1. steal-free-api-rotation-2026-06-20.md S8 (the win-both design + the three-sink framing + CB-tags).
2. .elai_cc/crates/orchestrator (the executor dispatch / emission-format seam) + E-35 CodeAct/SEARCH-REPLACE surface + the XGrammar mask site.
3. J-51 ModelCapabilityProfile.tool_call_adherence field slot; E-31 RuntimeRoutingPolicy (escalation_policy.rs); I-36 CompiledPromptStore (build_system_prompt_for_role substitution).
4. J-34_empirical_context_calibration.md (the bench shape to mirror — deterministic gate verdicts, never LLM-judge).
5. CLAUDE.md: TD-3 (LLM never adjudicates the route), W-2 kill-switch, no .unwrap in prod.

## GROUNDING (verified seams — recon complete, re-verified this refit)
RECON_NEEDED: none open — every load-bearing seam below was probed directly on 2026-08-27 at HEAD b9efaf5e (commands + raw outputs archived in conductor/opus_walk/evidence/J-55/plan-refit.md). Execute against these; do not re-plan them.
- PRODUCTION SEAM (the hook): `.elai_cc/crates/orchestrator/src/dispatcher.rs:228` = `pub fn build_dispatch_context(fsm_state: FsmState, config: &BackendConfig, all_tool_names: &[String], priority_map: &BTreeMap<String,u8>, profile: Option<&ModelCapabilityProfile>, phase_slot: Option<&PhaseSlot>) -> Result<DispatchContext, DispatchError>`; re-exported via `crates/orchestrator/src/lib.rs:1643`.
- LIVE CALLER 1 of 2: `crates/orchestrator/src/task_runner.rs`, inside `async fn drive_task` (decl at :6494). Controller-verified line was :6687 at verification time; HEAD-drift re-check places the identical production call at :6742 (`match crate::dispatcher::build_dispatch_context(`), lines :6687..:6741 being its arg-assembly preamble (DISPATCH_CONTEXT_BASELINE_TOOLS -> priority_map -> fsm_state). Same function, same single production call — claim PRESERVED, line drift recorded.
- LIVE CALLER 2 of 2: `crates/cli/src/runtime_client.rs:301`, inside `RuntimeClient::stream` (`match elai_orchestrator::build_dispatch_context(` through the lib.rs re-export), feeding per-turn sidecar tool filtering. Line EXACT at HEAD.
- CENSUS: every other reference to build_dispatch_context under crates/ is a doc comment or a #[cfg(test)] body (dispatcher.rs self-tests :388+, crates/contracts/tests/*) — R7 TEST ROOTS that never satisfy W-5/production-caller law.
- GREENFIELD: `rg -n "ModalityRouter|ActionModality|modality_router" crates/` under .elai_cc → ZERO matches (exit 1, 2026-08-27). Also zero: `modality_crossover`, `ModalityRouted`, `tool_call_adherence`. All J-55 symbols are genuinely new; J-51's adherence field does not exist yet.
- INFERENCE GATES (probed 2026-08-27): `curl http://127.0.0.1:8181/health` → connection refused (exit 7); `curl http://127.0.0.1:4000/health` → connection refused (exit 7). Both DEAD → the measured half of the done-gate is DEFERRED-gated as UNIT D1; it stays IN this plan with its gates named.
- PATH NOTE: Rust workspaces live under `.elai_cc/`; bare `crates/...` paths throughout match prior plan convention and resolve under `.elai_cc/crates/...`.

## VERIFY (off-agent authority — the author NEVER self-certifies DONE)
Off-agent pass artifacts required; author-run outputs are iteration feedback, never authority:
```
python3 .claude/tools/plan_gate_check.py --file docs/thefinalglance/J-plan/J-55_model_strength_modality_router.md
#   off-agent verifier reruns this; PASS tail appended to conductor/opus_walk/evidence/J-55/plan-refit.md
python3 .claude/tools/production_caller_check.py --files crates/orchestrator/src/modality_router.rs
#   EXECUTE-time auditor rerun per R-8; any DARK symbol = BLOCKED return
python3 .claude/tools/python_wire_check.py --files <changed .py>
cargo check -p orchestrator -p cli
git diff --stat
```
- VERIFY artifact 1 (this refit): frozen plan_gate_check PASS tail stored by the OFF-AGENT verifier / opus controller in conductor/opus_walk/evidence/J-55/plan-refit.md. This author ran the checker only to iterate the text.
- VERIFY artifact 2 (EXECUTE waves): auditor-run production_caller_check / python_wire_check outputs over the changeset; RUNNER-UP forbidden (self-run-as-authority = syntax-error severity).
- VERIFY artifact 3 (D1 when unblocked): SSB stream excerpt + bench crossover JSON from real lanes, captured off-agent, checked against the keystone negative arm.

## REDTEAM (recorded attacks; auditors replay them)
- RT-1 falsify-greenfield attack: attempted to break the plan by finding pre-existing ModalityRouter/ActionModality/modality_router symbols anywhere under crates/ (which would silently convert task W1.1 into a collision refactor). Result: zero matches, exit 1 (2026-08-27) → greenfield HOLDS; consequence accepted: the new pub symbols WILL trip production_caller_check unless wired — hence the hook is pinned to the two verified callers, not left to executor discretion.
- RT-2 falsify-two-callers attack: enumerated EVERY build_dispatch_context reference in crates/. Production calls exist ONLY in drive_task (task_runner.rs, now :6742) and RuntimeClient::stream (runtime_client.rs:301). Everything else is #[cfg(test)] or docs. Attack implication recorded: an EXECUTOR claiming W-5 satisfied via dispatcher.rs self-tests or crates/contracts/tests fixtures is a DARK-DONE — the audit must accept only the two production sites.
- RT-3 route-spoof attack (design-level; replay fixture for D1): steer the Qwen2.5-Coder-7B local lane into CodeAct by forging profile params/adherence upstream of resolve(). Analysis: profiles enter via context_budget::detect_model_capability_profile(model, vram_tier) inside RuntimeClient::stream — derived from configuration, never from LLM output; resolve is deterministic TD-3 with the cloud-tier check ordered first; contract negative arm pins sub-14B ⇒ never CodeActChosen. Residual logged honestly: a mis-provisioned endpoint whose TIER reads cloud while serving a 7B model would mis-route — pinned as a D1 live negative fixture rather than waved away.
- FORMAT redteam of this refit itself: baseline FAIL reproduction matched the brief exactly (8 scored points failing) before editing; gate was treated as frozen (never edited, R-3).

## FORKS
- FORKS: none — design RATIFIED 2026-08-11; this refit restores the ratified seam in text and adds cadence only. Resolutions locked here so no defensible alternative remains open for an executor to re-litigate:
  1. Former open fork "J-51 sequencing" is CLOSED by ground truth: tool_call_adherence verified absent from code on 2026-08-27 ⇒ UNIT W1 resolves CodeAct on params+tier alone with directional NAMED CONSTANTS (labelled directional until replaced); adherence folds in as a second signal when J-51 lands — additive, exactly the mitigation this plan already carried.
  2. Wave split W1/D1 is availability bookkeeping driven by probed-dead endpoints (:8181/:4000), not a design change; D1 stays in-plan, gated.

## ABORT_CONDITIONS
- A1 SEAM REGRESSION: build_dispatch_context moved/renamed/re-signatured such that resolve() can no longer hang off the two verified production sites → STOP + BLOCKED handoff. Never invent entrypoint #3; production wiring IS the deliverable (R-8).
- A2 COMPOSE BREAK: E-35 SearchReplaceMatcher/CodeAct emission surface or I-36 CompiledPromptStore unavailable at authoring time → wave tasks W1.2/W1.3 cannot compose without violating GD-1 (re-implementing) → STOP + handoff; stubbing/faking them is forbidden.
- A3 RECORD LOSS: B-10 ModelCapabilityProfile gone/unusable → STOP (resolve has nothing data-driven to read).
- A4 FROZEN-GATE DISAGREEMENT: plan_gate_check stays FAIL after bounded scoped iteration due to apparent checker bug/drift → STOP + handoff. The gate is frozen to every worker; editing it is out of scope forever.
- A5 BUILD LAW: cargo is invoked ONLY as plain `cargo` (depot shim; TEIKOKU_DEPOT_OFFLOAD=remote). Absolute-path cargo/rustc, `rustup run`, CARGO_TARGET_DIR overrides are banned and watchdog-killed ON SIGHT; a Killed build is never retried; shim failure = BLOCKED with shim output. opus_land's gate owns the authoritative GREEN verdict; local checks are best-effort feedback only.
- A6 GIT: any destructive-git temptation → R-1 STOP (git is read-only for workers; orchestrator mutates git).
- A7 SCOPE: any demand to touch files outside this plan + the designated evidence file → R-3 STOP + handoff.
- A8 ENVIRONMENT FLIP: if :8181/:4000 come alive mid-flight, W1 is unaffected; D1 scheduling is an ORCHESTRATOR decision (new TaskSpec), not a unilateral start.

## UNITS (re-scope: former six tasks → TWO halves, availability-driven)

### UNIT W1 — WAVE-1 OFFLINE half (buildable NOW; zero inference dependency)
Carries former tasks 1, 2, 3, 5(+tests, decision doc). Thresholds ship as directional NAMED CONSTANTS; the bench CODE and every live-turn proof are DEFERRED-gated into D1.

- EXPECTED_OBSERVATION (W1.1 router core, ~140 LOC): `.elai_cc/crates/orchestrator/src/modality_router.rs` exports `enum ActionModality { CodeAct, ConstrainedToolCall, TextFallback }` and pure `ModalityRouter::resolve(profile: &ModelCapabilityProfile, role: AgentRole, tier: Tier) -> ActionModality`; deterministic TD-3 rules — tier=cloud OR (local AND params>=CODEACT threshold) -> CodeAct; local weak (<threshold; adherence conjunct folds in post-J-51) -> ConstrainedToolCall; no-tool model -> TextFallback. All thresholds NAMED CONSTANTS labelled directional until D1 seeds measured ones via the new `ModelCapabilityProfile.modality_crossover` Option field (None => constants). Offline pass = compile + contract arms green with NO model attached.
  - ON_FAIL: ModelCapabilityProfile shape differs from B-10 assumptions → STOP per ABORT_CONDITIONS A3; never guess fields.
- EXPECTED_OBSERVATION (W1.2 emission wiring, ~90 LOC): resolve() is invoked from BOTH verified production sites' turn-prep — beside drive_task's build_dispatch_context call (task_runner.rs, now :6742) and beside RuntimeClient::stream's (:301) — branching the emission/parse path: CodeAct → E-35 code surface (tools as imports, no per-turn JSON schemas); ConstrainedToolCall → single tool-call under XGrammar + I-36 compiled MINIMAL schema (+ H-4/H-34 progressive disclosure), never the full registry; whole branch behind ELAI_MODALITY_ROUTER_DISABLE (default-ON router). Surfaces are CALLED, not authored (GD-1).
  - ON_FAIL: a site cannot take the branch without out-of-scope signature churn → STOP + handoff; wiring ONLY ONE lane and calling it wired fails R-8/R-10.
- EXPECTED_OBSERVATION (W1.3 universal edit path, ~40 LOC): every EDIT operation routes through the E-35 SearchReplaceMatcher regardless of ActionModality; whole-file write provably not taken when a diff applies.
  - ON_FAIL: matcher API mismatch → STOP per A2 (compose break); re-implementing the matcher is forbidden.
- EXPECTED_OBSERVATION (W1.4 SSB arm, ~50 LOC): `SsbActionType::ModalityRouted` 3-arm { CodeActChosen{model_fp,role}, ConstrainedCallChosen{model_fp,role,schema_tokens}, EditFormatChosen{diff:bool} } lands pattern-2 golden-lock lockstep (LIVE SCHEMA baseline 268/577 @2026-06-15 ssb.rs:1508 — re-read at edit per W-9, ModalityRouted = baseline+1).
  - ON_FAIL: golden-lock mismatch → fix within unit; baseline drifted upstream → re-read and re-pin before proceeding.
- EXPECTED_OBSERVATION (W1.5 contracts + decision doc): crates/contracts/tests/j55_modality_router_contract.rs (~120 LOC) green: (a) repeated resolve() identical; (b) cloud→CodeAct; (c) sub-14B local→ConstrainedToolCall (KEYSTONE GUARD); (d) no-tool→TextFallback; (e) constrained lane schema_tokens bounded (I-36 minimal, not registry); (f) edit diffs, not whole-file; (g) NEGATIVE arm: Qwen2.5-Coder-7B profile NEVER yields CodeActChosen. workflow/reasoning/J-55-decisions.md records the FORKS closures + RT-3 residual.
  - ON_FAIL: any arm red → fix producers within W1 file set; a red keystone arm means the resolve RULE regressed → return to W1.1, rerun all arms.
- INTEGRATION_SMOKE (W1 — REAL entrypoints, zero inference): from crates/contracts/tests/j55_*, drive `elai_orchestrator::build_dispatch_context` at its production six-arg signature (pattern proven by budget_parity.rs / dashboard_sidecar_provider_wired_contract.rs), feed the resulting profile+tier into resolve(), assert per-arm branch selection including ELAI_MODALITY_ROUTER_DISABLE=1 legacy-fallback equality; round-trip ModalityRouted through the Ssb writer/reader lockstep. Live-lane smoke deliberately lives in D1.

### UNIT D1 — DEFERRED-gated inference half (remains IN PLAN; gates named below)
Carries former task 4 + all live-turn proof halves of acceptance.
GATES: GATE-LOCAL = healthy sidecar serving canonical Qwen2.5-Coder-7B at http://127.0.0.1:8181 (probed DEAD 2026-08-27, connection refused); GATE-CLOUD = reachable OpenAI-compatible proxy/endpoint at http://127.0.0.1:4000 (probed DEAD 2026-08-27, connection refused). Unblock = ORCHESTRATOR issues a fresh D1 TaskSpec after both probes drive a real turn end-to-end.

- EXPECTED_OBSERVATION (D1.1 crossover bench, ~160 LOC): `elai bench modality` (J-34 pattern) measures per-(model,lane) {CodeAct, ConstrainedToolCall} on tool-success + tokens + latency over deterministic fixtures on LIVE lanes; writes crossover into ModelCapabilityProfile.modality_crossover; verdicts deterministic (tool-call correctness + token count), NEVER LLM-judge (TD-3). Local legs need GATE-LOCAL; cloud legs GATE-CLOUD.
  - ON_FAIL: endpoint flaps mid-run → mark the run INVALID; never seed thresholds from partial curves.
- EXPECTED_OBSERVATION (D1.2 threshold seeding): measured crossover re-bases the W1.1 named constants; labels flip directional→measured with date+numbers recorded in decisions doc.
  - ON_FAIL: measured curve contradicts a directional constant materially → ship the MEASURED value and log the discrepancy; hand-tuning constants toward folklore is forbidden.
- EXPECTED_OBSERVATION (D1.3 live proofs + dashboard): local-7B turn emits ModalityRouted::ConstrainedCallChosen{schema_tokens}; cloud turn emits CodeActChosen; keystone NEGATIVE driven live — the spoofed sub-14B-on-cloud-tier fixture (RT-3 replay) FAILS when it yields CodeActChosen; ContextStatePanel/RuntimePanel badge + schema-token readout render from real rows.
  - ON_FAIL: wrong arm on ANY lane → STOP + BLOCKED with the raw SSB excerpt; a routing regression outranks shipping.
- INTEGRATION_SMOKE (D1): end-to-end via REAL entrypoint — one agentic turn through RuntimeClient::stream vs GATE-LOCAL and one vs GATE-CLOUD; assert ModalityRouted rows in the live SSB stream; archive bench JSON + stream excerpts off-agent.

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/modality_router.rs::{ActionModality, ModalityRouter::resolve}
  - resolve wired at BOTH verified production dispatch sites (drive_task ; RuntimeClient::stream)
  - the `elai bench modality` subcommand + ModelCapabilityProfile.modality_crossover field (code W1; VALUES D1-gated)
  - crates/orchestrator/src/ssb.rs::SsbActionType::ModalityRouted (3-arm)
consumes_symbols:
  - E-35 SearchReplaceMatcher + CodeAct surface (selected, never authored)
  - J-51 ModelCapabilityProfile.tool_call_adherence (folds in post-J-51; verified absent today)
  - E-31 RuntimeRoutingPolicy / Tier (cloud-vs-local signal)
  - I-36 CompiledPromptStore (minimal compiled schema, constrained lane)
  - H-4 Tool Search + H-34 MCP defer (disclosure on schema'd path)
  - J-34 bench harness pattern (calibration shape; deterministic gate)
  - B-10 ModelCapabilityProfile (the record resolve reads)
reachable_from:
  - crates/orchestrator/src/task_runner.rs — async fn drive_task turn-prep beside build_dispatch_context (controller-cited :6687, HEAD :6742) (PRODUCTION caller 1 of 2)
  - crates/cli/src/runtime_client.rs:301 — RuntimeClient::stream per-turn prep (PRODUCTION caller 2 of 2)
  - `elai bench modality` CLI subcommand (PRODUCTION bench entrypoint seeding the crossover, D1)
  - a #[cfg(test)] caller NEVER satisfies this (R7)
stub_ban:
  - NO LLM-adjudicated modality choice (TD-3)
  - NO hardcoded "always CodeAct" / "always JSON" — data-driven off ModelCapabilityProfile
  - NO magic-number thresholds — NAMED CONSTANTS, re-based only by the D1 bench
  - NO full-tool-registry sends on the ConstrainedToolCall lane
  - NO whole-file write when SEARCH/REPLACE applies
  - NO re-implementing CodeAct/SEARCH-REPLACE/adherence-bench/prompt-compile (compose E-35/J-51/I-36)
  - NO .unwrap()/.expect() on resolve/dispatch prod paths
frontend_pair:
  ContextStatePanel (or RuntimePanel) modality badge + per-turn schema-token count, Tauri read of the ModalityRouted stream (minimal surface, justified; visual proof = D1.3).
observable_effect:
  trigger: "after D1 unblock — one local turn then one cloud turn"
  expect_ssb: "local-7B turn → ConstrainedCallChosen{schema_tokens}; cloud turn → CodeActChosen"
  expect_artifact: "`elai bench modality` writes per-(model,lane) crossover into modality_crossover; NEGATIVE (fails the gate): sub-14B lane emitting CodeActChosen"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/J-plan/J-55_model_strength_modality_router.md --run-trigger
python .claude/tools/production_caller_check.py --files crates/orchestrator/src/modality_router.rs   # resolve WIRED, not DARK
rg -n "ModelCapabilityProfile|tool_call_adherence|modality_crossover" crates/orchestrator/src/modality_router.rs   # data-driven
rg -c "always.?codeact|CodeAct.*default|magic" crates/orchestrator/src/modality_router.rs   # MUST be 0
test -f workflow/reasoning/J-55-decisions.md
```

## ACCEPTANCE
- [ ] W-1 [live-turn half → D1]: resolve on a REAL agentic turn; proof = ModalityRouted SSB row, captured off-agent.
- [ ] W-2 [W1]: ships ON behind ELAI_MODALITY_ROUTER_DISABLE (kill-switch, default-ON), never an opt-in flag.
- [ ] TD-3 [W1]: route deterministic over profile+tier; LLM has no input (contract-tested).
- [ ] keystone guard [W1 contract arm g][D1 live replay]: Qwen-7B (sub-14B) never routed to CodeAct.
- [ ] thresholds [W1 directional constants][D1 measured re-base]: NAMED CONSTANTS seeded by `elai bench modality`, folklore excluded.
- [ ] universal edit path [W1]: SEARCH/REPLACE taken when a diff applies; whole-file not taken (contract-tested).
- [ ] cargo check --workspace GREEN via plain `cargo` depot shim (opus_land gate owns the verdict); auditor PASS on -p orchestrator only.
- [ ] No new dependency.

## WIRING DONE-GATE (clause by clause)
- W-1: entrypoint = the two verified dispatch sites → resolve; artifact = ModalityRouted row (offline contract smoke now; live row D1).
- W-2: ELAI_MODALITY_ROUTER_DISABLE kill-switch, default-ON. Met.
- W-3: no opt-in flag emits PASS. Met.
- W-4: measures objective (chosen format actually emitted + schema_tokens moved), not stdout — measured legs D1.
- W-5: production_caller_check.py vs the two prod sites; test roots rejected (RT-2).
- W-6: modality badge + schema-token readout ships W1; visual proof on live rows D1.
- W-7: E-35/J-51/E-31/I-36/H-4/J-34/B-10 anchors re-grepped at authoring + validation.
- W-8: gate files frozen; plan never edits the checker.
- W-9: SCHEMA baseline re-read at edit (ModalityRouted = baseline+1).
- W-10: in-process resolve now; calibration bench tiered behind D1 gates.
- GD-1: composes E-35/J-51/E-31/I-36/H-4/J-34; re-implements none.
- GD-5 honest residue: paper thresholds are 2024-2025-vintage, directional-only; the D1 bench is the load-bearing proof. No residue from this refit: grounding refreshed 2026-08-27, drift of ONE cited line recorded inline rather than silently carried.

## Tests to add
- crates/contracts/tests/j55_modality_router_contract.rs (~120 LOC): arms (a)-(g) as W1.5; D1 adds bench-determinism fixtures.

## Files touched
[W1] CREATE: .elai_cc/crates/orchestrator/src/modality_router.rs (~290 LOC)
[W1] MODIFY: .elai_cc/crates/orchestrator/src/task_runner.rs (drive_task turn-prep calls resolve beside :6742) (~45 LOC)
[W1] MODIFY: .elai_cc/crates/cli/src/runtime_client.rs (stream per-turn prep calls resolve beside :301) (~45 LOC)
[W1] MODIFY: .elai_cc/crates/orchestrator/src/ssb.rs (ModalityRouted 3-arm + golden lock) (~50 LOC)
[W1] CREATE: .elai_cc/crates/contracts/tests/j55_modality_router_contract.rs (~120 LOC)
[W1] CREATE: workflow/reasoning/J-55-decisions.md (~40 LOC)
[D1] MODIFY: orchestrator/cli bench subcommand `elai bench modality` + modality_crossover seeding (~160 LOC) — DEFERRED-gated
[D1] MODIFY: dashboard-v2 ContextStatePanel/RuntimePanel (badge + schema-token readout) + Tauri read (~70 LOC) — visual leg gated, code may land W1

## Citations / Grounding
- arXiv:2402.01030 (CodeAct, ICML 2024); arXiv:2509.04508 (ProST ~14B threshold); arXiv:2605.02363 (grammar latency) — all WebSearch-VERIFIED 2026-06-20, HD-1-scoped; oracle run oracle-agentic-token-efficiency-codeact-toolsearch-2026.
- ELAI ground truth: items in Existing state + GROUNDING above; GROUNDING re-probed 2026-08-27 @ HEAD b9efaf5e with commands archived in conductor/opus_walk/evidence/J-55/plan-refit.md.
- CLAUDE.md: TD-3, W-2 kill-switch, Qwen2.5-Coder-7B canonical sub-14B lane.
- LIVE SCHEMA baseline 268/577 @2026-06-15 (ssb.rs:1508); ModalityRouted = baseline+1; re-read at edit (W-9).

## Former open questions → CLOSED
- J-51 sequencing: CLOSED as FORKS item 1 (params+tier first, adherence additive later).
- Bench needs sidecar: superseded by D1 gating clause (probes recorded, dates pinned).
- 14B staleness: unchanged answer — the gate reads measured modality_crossover once D1 seeds it; until then directional labels are explicit.

## Next
J-55 stays the keystone of win-both: capable lanes get CodeAct's schema/envelope savings, the 7B local lane gets constrained-call reliability, the universal SEARCH/REPLACE edit path banks output-token wins everywhere. Pairs with J-54 (WHICH model serves; J-55 decides HOW it acts). EXECUTE order: UNIT W1 now; UNIT D1 strictly after both gates open, under a fresh orchestrator TaskSpec.

End of J-55.
