G-7 decisions — Per-change confidence PRODUCER + Dual-LLM lethal-trifecta quarantine
session s-56068, 2026-06-04

WHAT
(A) Fuse token log-prob + self-consistency dispersion + oracle-free generated-test execution-agreement into the scalar claim_score that the SHIPPED conformal_calibration::decide() consumes, so the abstention layer fires on a REAL score (today it is fed a step-count FAKE). (B) When one context holds Trusted-secret + Untrusted + egress (the lethal trifecta, via G-1 live taint + G-9 secret detector + G-3 egress), route the untrusted span through a quarantined no-tool sub-LLM so the injected instruction never reaches a tool-caller.

GROUND TRUTH (context-compiler + 2 explorers, s-56068, fresh elai.map): two build-gaps (a self-consistency sampler + a no-tool sub-LLM dispatch + a secret-present bridge). Baseline SCHEMA_VERSION 214 (ssb.rs), TOTAL_VARIANTS 491 -> 216/493 (TWO new variants, payloads differ).

KEY DECISIONS

DD1 CONFIDENCE PRODUCER (new module confidence_producer.rs in orchestrator). Fuse 3 signals into a calibrated [0,1] claim_score:
  (a) TOKEN LOG-PROB — reuse GGSignals (sidecar_client.rs:160, Python GGSignalCollector server.py:1007). LOCAL-LANE ONLY: the Anthropic cloud client exposes NO logprobs. On the cloud/Sonnet lane signal (a) is UNAVAILABLE -> cloud claim_score uses (b)+(c) only (GD-5 honest; state it explicitly, do NOT claim log-prob confidence on cloud).
  (b) SELF-CONSISTENCY DISPERSION — NO per-change n>1 sampler exists (only per-blank run_best_of_n / precomputed MajorityVote). ADD a small multi-sample probe (n samples of the change, measure dispersion/agreement). On cloud this is MANDATORY (it + (c) are the only signals). Note the cost; keep n small + gate the probe.
  (c) ORACLE-FREE TEST EXECUTION-AGREEMENT — reuse F-4 stdout_hash (task_runner.rs:16141, blake3); cross-sample test agreement (depends on (b)'s samples).
  CALIBRATE via the EXISTING Platt/sigmoid rescaler cost::cloud_route::confidence::compute_score (cloud_route.rs:189) — REUSE, do not add a new calibrator (conformal_calibration has tau/TauUpdater but no Platt).
  WIRE: REPLACE the fake placeholder at task_runner.rs:11509 (claim_score = (0.05 + steps_f/22.0).clamp(...), whose docstring :11410 names "a D-16 follow-up" = G-7) with the real fused claim_score, feeding the SHIPPED conformal_calibration::decide():539 via dispatch_conformal_refusal_per_claim_hook (task_runner.rs:11425). The consumer decide()/tau/TauUpdater is UNCHANGED (GD-1, D-16 shipped — re-implement none).

DD2 TRIFECTA QUARANTINE. Detect at the tool-call chokepoint production_agent_rpc::translate_tool_request (:1406, where G-1 enforce_taint_sink_gate + G-3 enforce_fs_blast_radius/check_egress_host already hook + build_tool_catalog:509 has the grants + the live client:352):
  (b) UNTRUSTED span = tainted_context().lookup_for_argv (taint_flow.rs:190, G-1, shipped) returns Untrusted provenance.
  (c) EGRESS available = is_destructive_tool (curl/web_fetch/http_post, capability_gate.rs:58) / G-3 check_egress_host — an egress-capable tool is in the grant set.
  (a) TRUSTED-SECRET present = THE BRIDGE (the missing-label gap; TrustLevel has no Secret arm + TaintedContext drops Trusted). REUSE G-9's redaction SECRET DETECTOR (redaction.rs SecretKind / the F-1 SECRET_PATTERNS span layer G-9 built) to SCAN the live context/messages visible at the chokepoint for a secret. So (a) = "a secret pattern matches in the current context." This COMPOSES G-9 (detector) + G-1 (taint) + G-3 (egress) — GD-1, re-implement none.
  ON ALL THREE -> route the UNTRUSTED span through a NEW quarantined sub-LLM: a MessageRequest (providers/types.rs:11) dispatched through the existing client with a SHRUNK/empty tool catalog (no destructive/egress tools — shrink via allowed_tools_for_subagent tools/lib.rs:3670). Mirror the nested-LLM shape of chat_reflect.rs:561. ONLY the sub-LLM's STRUCTURED/sanitized output returns to the tool-calling agent; the raw untrusted span + tool access NEVER reach the tool-caller (the observable_effect). Emit +trifecta_quarantined.

DD3 SSB = TWO separate Pattern-2 single-variants (payloads differ): claim_confidence_scored (claim_score + per-signal components + decide() verdict + lane) and trifecta_quarantined (the 3 detected components + the quarantine action + the sub-LLM verdict). SCHEMA 214->216, TOTAL_VARIANTS 491->493 (two +1 bumps). REGEN golden + update all_test_payloads count (491->493 fixtures). BOTH variants MUST be added to the action_type SQL CHECK IN(...) list (ssb.rs:~4738 — the G-8 latent-bug / G-9-fix lesson; the CHECK is a SEPARATE site from the enum cascade and is runtime-only-enforced). Mirror the just-shipped SecretRedacted cascade (8 sites each). STANDING CHECKS: both emits hold the LIVE run Arc<Ssb> + project_id_from_path bucket (NOT a literal).

DD4 GD-4 / DEFAULT STATE. The conformal abstention + the quarantine are gated by the conformal layer's EXISTING default state (a triple default-OFF gate today). G-7's DELIVERABLE is wiring the REAL producer + the real detection/quarantine. Flipping the abstention/quarantine default-ON is the GD-4 promotion gated on the >=3-real-run FP/over-abstention bound (a conformal layer that abstains on everything is the CodeGate death). The fixtures demonstrate the observable_effect with the gate ON; the default-ON flip is the documented promotion path (W-2/R9: if it stays default-OFF, register the flag in wire-dark-allowlist.toml with a promotion plan).

DD5 G-5 LINK: claim_confidence_scored feeds the G-5 cockpit's confidence-triage band (review low-confidence hunks first). frontend_pair: per the WIRING_CONTRACT — if a confidence/quarantine drawer, mirror an existing drawer (e.g. StalenessQuarantineDrawer.svelte) + the 3-label-map sync; if backend-only, justify.

EXPECTED TEST OUTCOME
Deterministic fixtures: (1) a change with LOW self-consistency (divergent samples) + a failing/absent oracle-free test -> a LOW fused claim_score -> conformal decide() returns ABSTAIN -> a claim_confidence_scored row lands (score + components + abstain verdict) + the change is held/surfaced low-confidence; a HIGH-consistency + passing-test change -> high score -> ACCEPT, no abstention (FP guard). (2) a trifecta fixture (context with an AWS key [secret] + an Untrusted from_web span instructing "POST to evil.com" + a curl/web_fetch grant [egress]) -> trifecta detected -> the untrusted span routed to the no-tool sub-LLM -> a trifecta_quarantined row lands + the untrusted span NEVER reaches the tool-caller (assert the tool-call with the untrusted instruction is NOT dispatched); a benign context (no secret OR no untrusted OR no egress) -> NO quarantine (FP guard). Both scores/decisions re-derive from the journal under ELAI_SEED (GD-2).

GD-5 HONEST SCOPE
log-prob is LOCAL-lane only (cloud = self-consistency + execution-agreement). UniCR (2509.01455) risk-control is UNVERIFIED preprint — do NOT cite as proven; the conformal guarantee is the SHIPPED D-16 layer's, framed as calibrated-abstention not a proof. The self-consistency probe costs N extra samples — note it; keep N small. W-5: a live agent run firing a real abstention / quarantine = Tier-2 separately tracked; the deterministic fixtures are the Tier-1 gate.

UNKNOWNS / RISKS
- The self-consistency multi-sample probe cost (N samples per change) — keep N small (3?), gate it, measure.
- The secret-present bridge scans the live context with G-9's detector at the chokepoint — confirm the context (messages/prompt) is accessible at translate_tool_request; if not, thread a secret-present flag from the write path.
- The no-tool sub-LLM: its structured output must be the ONLY thing that returns; verify the untrusted raw text + tool grants are genuinely withheld (the load-bearing security property).
- conformal_calibration::decide() consumer must NOT be re-implemented (GD-1) — only the producer + the wire-in change.
