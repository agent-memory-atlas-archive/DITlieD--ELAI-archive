# G-Plan Index — the trust READ/EVIDENCE/RUNTIME/MEASUREMENT side, wiring-enforced (9 sessions)
Date: 2026-06-02
Source synthesis: workflow/reasoning/ELAI-TRUST-MAXIMIZATION-2026-06-02.md (the 9 grounded
  initiatives R1..R9 + their adversarial-verifier verdicts, run wf_826d66d4-0e6).
Grounding files: ELAI-DIRECTION-2026-06-01.md, VIBESEC-{RESEARCH,TOOLS-MATRIX,PAPERS}-2026-06-01.md,
  workflow/papers/rtbas-interface.md, workflow/papers/fides-interface.md.
Predecessors: B-plan, C-plan, D-plan, E-plan, F-plan (F-1..F-10 = the WRITE-side gates this plan
  composes with; F-2 owned by another session).

Predecessor reading (mandatory before any G-N execution):
  workflow/reasoning/ELAI-TRUST-MAXIMIZATION-2026-06-02.md (the why + the per-initiative scope + the
    verifier landmines in section 5 — DO NOT re-introduce them)
  docs/thefinalglance/F-plan/F-0_INDEX.md (the W-1..W-10 wiring doctrine this plan INHERITS verbatim)
  docs/thefinalglance/F-plan/F-SUMMARY_INDEX.md (the F-gates whose verdicts the G-gates consume)
  This file (G-0_INDEX.md), then G-SUMMARY_INDEX.md.

## Why G-plan exists (read this before anything else)

The F-plan built the WRITE side: ten in-process gates that decide whether a vuln / secret / slop /
fabricated-PASS / CWE-regression LANDS. The trust-maximization research (run wf_826d66d4-0e6, 8 axes,
hostile-verified) found the strictly-complementary whitespace is the READ / EVIDENCE / RUNTIME /
MEASUREMENT side: nobody on the market turns gate verdicts into (a) a reviewer-facing cockpit, (b) a
re-executable run artifact, or (c) a measured, reproducible, anti-gameable safety-delta number. The
recurring credibility anchor is the SSB append-only journal (ssb.rs:1-5 "SQLite WAL, append-only,
no UPDATE/DELETE") + ELAI_SEED determinism — it is what turns "trust us" into "replay us," the one
thing every competitor (Endor Agent Security League, CodeRabbit/Greptile leaderboards, the
claude-replay viewer) lacks.

THE CENTRAL FINDING (spot-checked vs elai.map + source 2026-06-02): ELAI ALREADY SHIPS most of the
trust substrate but WIRE-DARK. provenance.rs (Provenance + Provenanced<T> + from_web/from_tool_output/
from_file/from_sandbox), capability_gate.rs (allow_tool_with_origin + DESTRUCTIVE_TOOLS),
conformal_calibration.rs (decide/claim_score/tau/TauUpdater), refusal_sla.rs, mcp_schema_pin.rs
(pin_or_verify/hash_tool_list), sandbox/researcher_net.rs (allowed_hosts) ALL EXIST. The fastest trust
wins are ACTIVATION, not invention. G-plan's single job: light up that substrate and bind it to a
reproducible artifact + a measured number, under the SAME hard, executed wiring done-gate the F-plan
uses, so none of it can ship wire-dark again.

## Status legend
- PLANNED — file written, work not started (all 9 G-N at generation)
- IN PROGRESS — active session
- DONE — passes the FULL done-gate (wiring_contract_check.py --run-trigger PASS; cargo-green is
  necessary but NOT sufficient)
- DEFERRED — RunPod / external dependency gated (named per G-N)

================================================================================
THE WIRING DOCTRINE — INHERITED W-1..W-10 (canonical text: F-0_INDEX.md) + G-DELTAS
================================================================================

Every G-N INHERITS the F-plan doctrine W-1..W-10 VERBATIM (do not re-litigate; F-0_INDEX.md is canon):
  W-1  no-wire-dark done-gate: DONE only when wiring_contract_check.py --bn-file <G-N>.md --run-trigger
       returns PASS (produces exist as DEF, consumes resolve to CALL/REF, reachable_from reached by BFS,
       stub_ban clean, AND observable_effect actually RAN + emitted the SSB row + artifact). cargo-green != DONE.
  W-2  no default-off env gate as the terminal state (kill-switch DISABLE only; default ON; observable_effect
       runs with NO env override).
  W-3  reachable from a REAL production entrypoint (CLI subcommand / FSM transition / PreToolUse-PostToolUse
       hook / Tauri command); a #[cfg(test)]-only caller is wire-dark.
  W-4  frontend_pair mandatory for any dashboard-surfaceable SSB signal; cross_lang_diff.py PASS.
  W-5  observable_effect is a LIVE end-to-end trigger, not a unit test; RunPod-gated work ships a
       deterministic fixture harness exercising the REAL path + a separately-gated live checkbox.
  W-6  >=1 ACCEPTANCE item verified THROUGH the dashboard (dashboard-smoker), or N/A with one-line justification.
  W-7  no hallucinated integration points: every consumes_symbol / reachable_from anchor / MODIFY target
       grep-confirmed in elai.map AT AUTHORING TIME; the validator re-greps.
  W-8  produces/consumes symmetry: every produced symbol has an in-plan caller.
  W-9  new SSB action_types flow the action_type golden lock (Pattern-2 single-variant-with-Kind where it
       applies); SCHEMA bump named, never hard-coded.
  W-10 tiered-gate: any axis needing network/subprocess/RunPod at gate time is SPLIT — the in-process Tier-1
       is the fail-closed done-gate; the network/subprocess Tier-2 is separately gated and MUST NOT advertise
       fail-closed if it is fail-open.

G-DELTA rules (additional, specific to the trust/read/measurement nature of this plan — all BLOCKING):
  GD-1  COMPOSE-NOT-RESTATE. Every G-N either fills a READ / EVIDENCE / RUNTIME / MEASUREMENT gap the F-plan
        does not, or COMPOSES-WITH an F-gate by CONSUMING its verdict (e.g. F-4 stdout_hash, F-1 source_blake3,
        F-9 dep_name_hash, F-10 delta_blake3). Re-implementing or re-stating any F-1..F-10 (esp. F-2, owned
        elsewhere) is a P0 incident logged to gotchas.md. Each G-N names the F-gate verdict it consumes.
  GD-2  REPRODUCIBLE-METRIC. Any number a G-N emits (DSR, safety-delta, FP-rate, confidence) MUST itself be
        recomputable from the SSB event stream under a pinned ELAI_SEED — the metric is auditable too, not
        just the gate. A number that cannot be re-derived from the journal is wire-dark by construction.
  GD-3  ABLATION-FRAMING. Any safety/performance number is framed gate-ON-vs-gate-OFF (the gate's marginal
        contribution, same model), NEVER as a SWE-bench-Verified rank, a model-capability claim, or a
        perfect-code promise. A green replay proves the run was REPRODUCED, not RIGHT (DFAH arXiv:2601.15322).
  GD-4  FP-FRICTION RELEASE GATE. No G-N gate flips default-ON until its false-positive / over-block rate is
        MEASURED and bounded on >=3 real orchestrate runs. This is the CodeGate-death lesson (archived
        2025-06-05) and is empirically grounded: RTBAS's global-join variant collapses utility to 33% on the
        Slack suite (over-tainting), vs 90%+ with dependency-screened join (arXiv:2502.08966, VERIFIED). A gate
        a reviewer learns to ignore destroys the review-time thesis. The metric IS the FP early-warning.
  GD-5  NO-FABRICATED-GUARANTEE. A fail-open / RunPod-pending / cloud-opaque path NEVER advertises a guarantee
        it does not deliver — carry the honest state (runpod_pending; reproducible-action-stream-not-tokens for
        the cloud lane; fail-open-with-advisory-row). Worse than no gate is a gate that implies safety it lacks.

DONE DEFINITION (FSM may not reach COMPLETE without all): (1) cargo check --workspace GREEN, (2) auditor PASS
  (sole test runner), (3) wiring_contract_check.py --run-trigger PASS, (4) cross_lang_diff PASS if a
  frontend_pair is present, (5) the W-6 dashboard E2E item, (6) the GD-4 FP-rate bound recorded, (7)
  memory-writer ran.

================================================================================
THE 9 SESSIONS (1:1 with the trust-max roadmap R1..R9; G-N = R-N identity)
================================================================================

  G-N   title                                            lever                    grounding  effort  wave
  G-1   harness-wide taint propagation (data-plane)       cant-wreck              STRONG     L       1
  G-2   reproducible-run artifact (RRA) [KEYSTONE]        auditable-reproducible  STRONG     M       2
  G-3   deterministic runtime blast-radius gates          cant-wreck              STRONG     M       2
  G-4   live MCP rug-pull / line-jump re-pin gate         cant-wreck              STRONG     M       1
  G-5   reviewer cockpit + Diff-Surface-to-Review metric  verified-before-seen    STRONG     M       2
  G-6   SAFEDELTA competitive safety-delta benchmark      auditable-reproducible  STRONG     XL      3
  G-7   per-change confidence producer + Dual-LLM quarant honest-limits+cant-wreck STRONG    L       3
  G-8   signed in-toto attestation + Article-50 verifier  auditable-reproducible  STRONG     L       2
  G-9   SSB/log secret+PII redaction gate + DB-file TTL    auditable-reproducible  MODERATE   M       2

Each G-N's "Existing state" block names the F/B/C/D/E plans + the wire-dark ELAI symbols it composes with
(grep-confirmed in elai.map at authoring time; the authored G-N re-greps to confirm — W-7).

================================================================================
POST-G-9 EXTENSIONS (added 2026-06-02; NOT 1:1 with R1..R9 — derived from a gap analysis + elai.map grep)
================================================================================
  G-N   title                                            lever                     grounding effort wave
  G-10  model-drift regression canary (exogenous)        honest-limits+cant-wreck  STRONG    M      3
        Closes the seam left after G-9: ELAI watches its OWN changes (self-mod / GEPA / per-refine CWE-class /
        goal-drift) but NOTHING watches whether the model UNDERNEATH it silently changed. COMPOSES-WITH D-7
        canary.rs (reuse families+delta+band), G-2 RunArtifact (system_fingerprint+model_pin identity input),
        F-4 stdout_hash (malformed-output signal); re-implements none (GD-1). Detection default-ON; the auto-pin
        action is human-gated + wire-dark-allowlisted (GD-4). Local lane = fingerprint-confirmed, cloud lane =
        behavioral-suspected (Anthropic system_fingerprint opaque per G-2 §6) (GD-5). Provenance: r/ClaudeAI
        "not sexy work" corroboration (tier-5) + grep confirming no model-version gate + ModelSwap/TierModelSwap
        wire-dark. File: G-10_model_drift_regression_canary.md.

================================================================================
SCHEMA_VERSION BUMP SITES (baseline per CLAUDE.md; currently ~199 / TOTAL_VARIANTS ~476 after the F-plan
Tier-1 run per memory project_fplan_tier1_run — NEVER hard-code; each G-N names "baseline + N")
================================================================================
  G-1   +taint_flow (Pattern-2 single-variant; ingest-site Kind: Web/ToolOutput/File/Sandbox)
  G-2   +run_artifact_emitted (RRA root blake3 + evidence-manifest hash)
  G-3   +blast_radius_blocked (Pattern-2; Kind: FsEscape/EgressDenied), full pre-AND-post-expansion command
  G-4   +mcp_rug_pull_blocked (before/after tool-description diff)
  G-5   +review_surface_scored (DSR ratio + vouched/un-vouched line counts)
  G-6   +safety_delta_measured [DEFERRED behind F-2b LIVE keystone; RunPod-gated]
  G-7   +claim_confidence_scored, +trifecta_quarantined (Pattern-2 if shared payload)
  G-8   +provenance_attestation_emitted (in-toto predicate digest)
  G-9   (no new action_type — redaction mutates the write path; +secret_redacted advisory optional)
  G-10  +model_drift_suspected (Pattern-2 single-variant; Kind: FingerprintChanged / CanaryRegressed / MalformedOutputSpike); baseline+1  [Post-G-9 ext]

================================================================================
SEQUENCING (cheap-and-high-grounding first; the headline last)
================================================================================
  WAVE 1 (activation, low risk, high grounding, no deps): G-4 (MCP re-pin — extends shipped pin_or_verify,
    no deps) + G-1 (taint propagation — lights up shipped provenance.rs/capability_gate.rs; unlocks G-3 + G-7)
    + the patch-N citation fixes (capability_gate.rs CaMeL title/author; conformal_calibration.rs UniCR id).
  WAVE 2 (the trust spine made shippable): G-2 (RRA, the keystone) -> then G-3 (blast-radius), G-5
    (cockpit + DSR), G-8 (attestation), G-9 (journal redaction) ALL hang off G-2's evidence manifest.
  WAVE 3 (the headline, gated): G-7 (confidence producer + Dual-LLM, needs G-1's live taint) + G-6
    (SAFEDELTA, needs G-2 + the RunPod-pending F-2b LIVE keystone). NO external safety-delta claim ships
    before ELAI's own single-harness dynamic floor (F-2b) is measured.

THE ONE HEADLINE MOVE: couple G-2 (RRA) with G-6 (SAFEDELTA) — the artifact makes the benchmark
independently replayable; the benchmark gives the artifact a published number. Together = ELAI-DIRECTION
R1 (a real measured number on ELAI's own engine, made FALSIFIABLE) + R2 (the anti-Replit distribution demo).

================================================================================
HARD RULES INHERITED FROM CLAUDE.md + F-0 (apply to every G-N)
================================================================================
- blake3 only (never blake2/sha2 for non-RFC uses). provenance.rs already uses label_blake3_prefix.
- No .unwrap()/.expect() in production paths (test cfg + main entrypoint allowed).
- Anti-slopsquatting: verify name+repo+version+date+license BEFORE any dep. Per-G-N dep flags: G-8
  cosign/sigstore (opt-in only; local blake3-Merkle is the default, hand-rolled); G-9 NO Presidio/ggshield
  dep (local hand-rolled redactor); G-4 mcp-scan dist-channel UNVERIFIED — verify before wiring.
- One change at a time; the auditor is the sole test runner; the main LLM never runs tests.
- Grounded-by-default: every external-behaviour claim cites the trust-max synthesis + its primary source;
  UNVERIFIED claims labelled. The carried DO-NOT-CITE landmines below are BINDING.

================================================================================
CARRIED VERIFIER LANDMINES (from trust-max §5 — DO NOT RE-INTRODUCE; full text in that doc)
================================================================================
- PRvL (arXiv:2508.05545) Presidio 0.83/0.88/0.85 + GPT-4o numbers are REFUTED (not in that paper). Use the
  qualitative "PII detection is high-recall/low-precision" framing only; re-source from Presidio docs (G-9).
- EUR 35M/7% is the Art-5 PROHIBITED-PRACTICES tier; Art-12 logging / Art-50 marking carry EUR 15M/3% (Art-99).
  Do NOT pair 35M/7% with a logging/marking compliance argument (G-8/G-9).
- CyberSecEval 3 (arXiv:2408.01605) is OFFENSIVE-cyber-focused — NOT the insecure-code-GENERATION source; the
  autocomplete/instruct insecure-code suite is CyberSecEval 1/2 (arXiv:2404.13161) (G-6).
- "CVE-2026-30623 in Anthropic's MCP SDK" is WRONG (that CVE is LiteLLM). Anthropic's SDK has no CVE; say
  "the stdio command-injection family; Anthropic declined a protocol fix" (G-4).
- Thinking Machines nondeterminism experiment used Qwen3-235B-A22B-Instruct-2507, NOT Qwen3-8B (G-2).
- Do NOT ground a BspEngine move on "no production caller" — REFUTED (task_runner.rs:5018 is a non-test
  caller); the real gap is recover_from_checkpoint (RESUME path) is a stub (G-2).
- mcp-scan is Invariant Labs, NOT Snyk; its dist-channel (pip/npm/cargo) UNVERIFIED — anti-slopsquat first (G-4).
- in-codebase patch-N (fix while touching those files, NOT a roadmap initiative): capability_gate.rs:8-9 cites
  CaMeL with wrong title/author (ID 2503.18813 ok; real title "Defeating Prompt Injections by Design", Google
  DeepMind/ETH); conformal_calibration.rs:12 cites arXiv:2503.01345 for UniCR (verifier: real UniCR = 2509.01455,
  itself a shaky preprint — re-confirm both IDs before patching).

================================================================================
SESSION LOG (populated as G-Ns ship)
================================================================================
(Empty at generation. Each shipped G-N adds a row: context-compiler -> integration-explorer ->
spec-grounder -> executor -> auditor -> fixer (if BLOCKs) -> wiring-contract-validator (RUN_TRIGGER PASS
REQUIRED) -> dashboard-smoker (if frontend_pair) -> memory-writer -> COMMITTING.)

================================================================================
NEXT STEP
================================================================================
Start WAVE 1: G-4 (MCP re-pin, no deps) + G-1 (taint propagation, unlocks G-3/G-7) co-shipped, plus the two
patch-N citation fixes. Then G-2 (RRA keystone) opens WAVE 2. Respect W-1..W-10 + GD-1..GD-5 on every session.

End of G-0_INDEX.md.
