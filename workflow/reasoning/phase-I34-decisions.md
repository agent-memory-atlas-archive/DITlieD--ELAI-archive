phase I-34 decisions (CRANE hybrid grammar pinning)

WHAT:
Implements three hardening layers on the shipped crane_adaptive flag (IS-12):
(1) CRANE runtime logic in providers/src/decoding.rs: a descriptor struct + helper returning
    the segmentation policy (free thinking tokens between commit points, constraint on final
    output only per IC-18).
(2) Grammar pin manifest + blake3 load-time verification in providers/src/grammar_store.rs.
    Mirrors the E-24 mcp_schema_pin.rs pattern.
(3) Post-parse schema re-validation: re_validate_output() in providers/src/decoding.rs returns
    Result<(), ValidationError>; the orchestrator-layer caller emits GrammarGateEvent rows.
(4) GrammarGateEvent SSB variant (Pattern-2, 3-arm Kind: CranePassed/PinViolation/PostParseRejected).
    SCHEMA_VERSION 233->234, TOTAL_VARIANTS 542->543 across all 7 lockstep sites.
(5) Orchestrator grammar_gate.rs: caller that holds SsbWriter + decoded output, emits events,
    routes PostParseRejected to D-16 refusal seam.
(6) CLI fixture: elai bench crane --crane-fixture <path> --seed <n> deterministic harness.
(7) Frontend chip: pin_status field added to get_grammar_compilations response;
    GrammarCompilePanel.svelte shows a pin-status chip (OPTION A -- extend existing panel).
(8) Contract test: crates/contracts/tests/i34_grammar_gate_contract.rs.

KEY DECISIONS:

DECISION-1 (D-16 seam / PostParseRejected routing):
  re_validate_output() lives in providers (no orchestrator dep in providers).
  The CALLER in the orchestrator layer (grammar_gate.rs) receives Err(ValidationError) and:
    (a) emits GrammarGateEvent(PostParseRejected) SSB row, then
    (b) calls dispatch_conformal_refusal at agent_pool.rs:2972.
  The CalibrationCorpus is NOT available inside providers crate; the orchestrator layer
  already holds one at the dispatch site (consistent with agent_pool.rs:2972's existing callers).
  New orchestrator module: crates/orchestrator/src/grammar_gate.rs -- the gate caller.
  This keeps the providers -> orchestrator dep edge clean (one-way, no circular dep).
  Grounding: exploration file D-16 SEAM section, decoding.rs header comment at :11-14.

DECISION-2 (frontend chip location):
  OPTION A selected: extend get_grammar_compilations response (commands_e35.rs) to include
  pin_status: String field. Avoids new GrammarPinPanel.svelte file + new panel ID in types.ts.
  Lowest blast radius; both sides (Rust + Svelte) update atomically for cross_lang_diff.py pass.
  The Svelte component is LOCAL types only (no stub.ts entry needed -- panel calls invoke directly).
  Grounding: exploration file VALIDATION WARNINGS #1 "option (a) is lower blast radius; lean: (a)".

DECISION-3 (CRANE runtime body scope):
  The CRANE runtime body behind crane_adaptive_eligible is a DESCRIPTOR STRUCT (not live inference).
  Adds CraneAdaptiveConfig::from_mode(&DecodingMode) -> Option<CraneAdaptiveConfig> returning the
  segmentation policy (thinking_tokens_permitted: bool, constrain_final_only: bool).
  No live inference runs in this session (W-10 tiered: GPU leg is PENDING checkbox).
  The fixture harness (bench_crane.rs) drives the gate and verifies SSB row shape without inference.
  Grounding: IC-18 "free thinking tokens between structural commitments, constraint on FINAL output only";
             exploration file GAPS/RISKS "crane_adaptive_runtime fn: exact signature TBD".

DECISION-4 (grammar_store.rs location):
  New file: crates/providers/src/grammar_store.rs -- most natural home (providers already owns decoding).
  GrammarPinManifest stored under .elai/grammar_pins/ (separate from .elai/mcp/pins/ per E-24 precedent).
  blake3 only (never blake2/sha2 per CLAUDE.md hard rule).
  toml already present in providers/Cargo.toml (I-51 added it; workspace = true).
  Grounding: exploration file PATTERNS TO FOLLOW: "TOML pin record | mcp_schema_pin.rs:56-70".

DECISION-5 (claim_score for PostParseRejected):
  dispatch_conformal_refusal requires claim_score: f64. For PostParseRejected, the score is 0.0
  (complete failure to comply with the grammar -- hardest possible refusal from the conformal
  calibration's perspective). This mirrors the landmine #7 posture ("silently non-compliant outputs
  are the measured failure mode").
  Grounding: context-package note "likely 0.0 = full refusal"; agent_pool.rs:2972 signature.

EXPECTED TEST OUTCOME:
  cargo check -p elai_providers -p elai_orchestrator: exit 0
  cargo test -p elai_contracts --test i34_grammar_gate_contract: PASS (fixture leg, pin violation,
    post-parse rejection, schema lock 2/2)
  cargo test -p elai_orchestrator --test action_type_lock: PASS (TOTAL_VARIANTS = 543, new variant present)
  cargo test -p elai_contracts --test e45_cross_language_drift_contract: PASS (542->543 pin)
  svelte-check: 0 errors (pin_status field added to response type)

DECISION-6 (Tier-1 delivered scope — clarified after A59 audit):

  CraneAdaptiveConfig is a DESCRIPTOR STRUCT, not a runtime. Decision-3 above already scoped
  this correctly. To be explicit: Tier-1 ships the eligibility gate, pin manifest verification,
  post-parse re-validation, GrammarGateEvent SSB variant, the grammar_gate.rs orchestrator module,
  and the bench fixture. The live sidecar mask integration (Tier-2) is deferred.

DECISION-7 (production hook — F-3 outcome: PENDING, not forced):

  call_grammar_gate has zero production callers as of the A61-I34-FIX session. The correct
  insertion point is the orchestrator-layer function that receives a decoded structured/tool-call
  response string from the sidecar or cloud provider and already holds both an SsbWriter and
  a CalibrationCorpus. Based on grep of agent_pool.rs, task_runner.rs, and run_subcommand.rs,
  no such seam exists in the current code — the sidecar decode response handling path has not
  been extended to pass decoded_output + schema_json to any post-decode hook yet.

  The hook MUST land at: agent_pool.rs (or a new grammar_gate_hook module it calls), at the
  point AFTER the sidecar/cloud client returns a successful structured response and BEFORE the
  result is emitted as an SSB row, gated behind:
    crane_adaptive_eligible(decoding_mode)  AND
    grammar_manifest_present(project_root)  (absent manifest = feature inert, zero behavior change)
  When both gates pass, call call_grammar_gate(...) and route PostParseRejected through
  dispatch_conformal_refusal at agent_pool.rs:2972 (already holds CalibrationCorpus).

  This requires identifying the precise async fn in the agent_pool/sidecar dispatch loop that
  holds both the decoded output string and an Ssb + CalibrationCorpus reference. That function
  exists but requires a dedicated exploration session to identify safely without destabilizing
  existing tests. Registering as WIRE-DARK pending a follow-on patch.

UNKNOWNS / PENDING:
  [ ] CRANE runtime leg against live sidecar -- requires sidecar boot (Tier-2 separate session)
  [ ] XGrammar Python-side CRANE mask layer integration -- sidecar_xgrammar_hook.rs followups block
  [ ] Cloud lane parity for CranePassed chip -- cloud uses Tool API JsonSchema, not EBNF; chip
      shows honest "not-applicable" state for cloud lane per lane-parity D-023
  [ ] Production hook for call_grammar_gate -- WIRE-DARK pending follow-on patch; exact site:
      the async fn in agent_pool.rs / sidecar dispatch path that receives decoded structured
      output and holds Ssb + CalibrationCorpus (see DECISION-7 above)
