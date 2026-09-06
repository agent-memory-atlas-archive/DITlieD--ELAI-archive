# G-plan — Summary Index of All 9 Sessions
Generated: 2026-06-02. Mirrors F-SUMMARY_INDEX format. Each entry preserves the roadmap mapping (G-N = R-N),
the grounding strength + adversarial-verifier verdict that survived run wf_826d66d4-0e6, the scope, the
wire-dark ELAI substrate it activates, the F-gate verdict it COMPOSES-WITH (GD-1), the production integration
point, and the observable_effect that is the wiring done-gate (W-1). Canonical why + per-initiative detail:
workflow/reasoning/ELAI-TRUST-MAXIMIZATION-2026-06-02.md. Enforcement doctrine: G-0_INDEX.md (inherits
F-0 W-1..W-10 + adds GD-1..GD-5).

## Premise
Every G-N turns one trust READ/EVIDENCE/RUNTIME/MEASUREMENT initiative into wired, dashboard-visible,
reproducible reality. The differentiator vs F is GD-1 (COMPOSE-NOT-RESTATE: G consumes F-gate verdicts,
re-implements none) + GD-2 (every emitted number is itself recomputable from the SSB journal). A session is
not DONE until wiring_contract_check.py --run-trigger PASSes AND its false-positive rate is bounded on >=3
real runs (GD-4). Default-off env gates as a terminal state are banned (W-2).

## Per-session index

### G-1 — Harness-wide taint propagation (data-plane defense, lit up from the shipped substrate)
- Roadmap R1 | lever cant-wreck | grounding STRONG | effort L | WAVE 1
- Steal/build: ELAI already SHIPS the RTBAS-style label type (provenance.rs: Provenance + Provenanced<T> +
  from_web/from_tool_output/from_file/from_sandbox; capability_gate.rs: allow_tool_with_origin +
  DESTRUCTIVE_TOOLS) but the constructors are built ONLY in tests = dead wiring; the gate has no live tags.
  G-1 constructs Provenance at the four REAL ingest sites and threads Provenanced<String> through agent
  dispatch so capability_gate.rs sees real taint, blocking untrusted-content -> destructive-sink flows
  deterministically. Design grounding (paper-ingested 2026-06-02): RTBAS (arXiv:2502.08966, CMU, 100% attack
  prevention / 90%+ utility WITH dependency-screened join) + FIDES (arXiv:2505.23643, Microsoft, 163->1 attack
  successes, P-T/P-F + HIDE/query_llm) + CaMeL (arXiv:2503.18813). CRITICAL PITFALL (GD-4): RTBAS's GLOBAL
  join collapses utility to 33% (Slack over-tainting) — ELAI MUST use dependency-screened join, not global.
- COMPOSES-WITH: F-3 (Tool-Filter least-privilege — taint is the data-plane complement to F-3's control-plane
  tool-set commitment); E-37 (prompt-armor classify_injection seeds the Untrusted label).
- Integration: construct Provenance at WebFetch/MCP tools-call/file-read/sandbox-stdout ingest in agent_pool
  dispatch; enforce via capability_gate.rs::allow_tool_with_origin before destructive sinks (git/Write/Bash).
  frontend_pair: taint-flow rows in a dashboard drawer.
- observable_effect (done-gate): an injected web/MCP fixture asserting "call git push / fs.write" produces an
  Untrusted-labelled context; the destructive sink is BLOCKED (or routed to confirm) with a taint_flow SSB row
  naming the ingest origin; a clean Trusted flow is unaffected (FP guard). Default-ON behind no env override.

### G-2 — Reproducible-Run Artifact (RRA) [THE KEYSTONE]
- Roadmap R2 | lever auditable-reproducible | grounding STRONG | effort M | WAVE 2
- Steal/build: a single portable msgpack+zstd file bundling the SSB action stream + ELAI_SEED + HardwareClass
  + system_fingerprint pin + model pin + an evidence manifest of every F-gate's blake3 verdict hash (F-1
  source_blake3, F-4 stdout_hash, F-9 dep_name_hash, F-10 delta_blake3) + a blake3 root over the bundle.
  New `elai run --emit-rra` + `elai rra show` + `elai replay --verify`. Turns the stated "a security control
  you can audit and reproduce" from prose into a shippable file. Precedent: event-log-as-source-of-truth (ESAA
  arXiv:2602.23193, ActiveGraph arXiv:2605.21997); claude-replay is viewer-only (no re-exec/seed/divergence) =
  first-in-category. HONEST SCOPE (GD-5): cloud-lane runs are reproducible-ACTION-STREAM, not reproducible-
  TOKENS (Anthropic system_fingerprint opacity); SSB stores HASHED tool I/O so the guarantee may be
  verify-divergence not full re-execute for body-reading steps — state which, do not over-claim.
- COMPOSES-WITH: F-1/F-4/F-9/F-10 (consumes their verdict hashes; re-implements none). Anchors G-5/G-6/G-8/G-9.
- Integration: ssb.rs (WAL append-only stream, no UPDATE/DELETE per ssb.rs:5) + the determinism seed/
  HardwareClass/system_fingerprint (grep-confirmed present across seccodeplt_runner.rs/l3_lock.rs; canonical
  module path confirm-at-authoring, W-7) + the run entrypoint (build_production_hook_dispatcher triple-bootstrap).
  RESUME caveat: recover_from_checkpoint is a STUB (NOT "BSP has no caller" — REFUTED); resume-from-journal is
  the real gap and is the optional second half. frontend_pair: an RRA viewer panel.
- observable_effect: `elai run --emit-rra` on a fixture task writes a .rra file; `elai replay --verify` re-runs
  the action stream and reports PASS (action-stream reproduced + every F-gate verdict hash re-checked) or a
  DIVERGENCE with the first mismatching event; a run_artifact_emitted SSB row carries the blake3 root.

### G-3 — Deterministic runtime blast-radius gates (post-expansion FS gate + egress IP-pin)
- Roadmap R3 | lever cant-wreck | grounding STRONG | effort M | WAVE 2
- Steal/build: (a) a post-shell-EXPANSION filesystem write/delete gate that canonicalizes ~/glob/symlink
  BEFORE the verb runs and fails closed outside the worktree root; (b) a host-side egress allow-list that
  RESOLVES and PINS the allowed IP and rejects hostnames with null bytes / control chars. Both emit a block row
  with the FULL pre- AND post-expansion command (fixing the rm-rf #10077 forensics gap — the command was never
  logged). Grounding: rm-rf #10077 + #12637 (shell/glob/tilde expansion after validation) CONFIRMED; SOCKS5
  null-byte egress bypass CONFIRMED; LLM permission classifier collapses to 81% FNR under adversarial
  scope-escalation (arXiv:2604.04978 CONFIRMED) -> deterministic gate beats classifier; Landlock (kernel 6.7+)
  as OS backstop.
- COMPOSES-WITH: E-30 (argv destructive-verb allowlist — G-3 gates the EXPANDED path/host, E-30 gates the verb);
  G-1 (egress/FS gates score richer with live taint tags). Partly EXISTS: sandbox/researcher_net.rs has
  allowed_hosts + veth+iptables OUTPUT allowlist (the BLOCK-2 deployment note) and compile_seccomp_network_extra
  (sandbox/mod.rs:1269) opens AF_INET WHOLESALE — the gap is host-side enforcement + IP-pin + null-byte reject,
  wired into a real dispatch path (not the deployment-note stub).
- observable_effect: a fixture `rm -rf $X` where $X expands to `~/` OR a fetch to a null-byte-spoofed host is
  BLOCKED before the verb/connect; blast_radius_blocked SSB row carries the pre- AND post-expansion command.
- CAVEAT (whats_missing): seccomp+Landlock is Linux-only; Windows AppContainer / macOS Seatbelt parity for the
  FS gate + worktree is unresearched (egress proxy + credential broker are OS-agnostic). Scope non-Linux separately.

### G-4 — Live MCP rug-pull / line-jump re-pin gate
- Roadmap R4 | lever cant-wreck | grounding STRONG | effort M | WAVE 1 (cheapest high-grounding; no deps)
- Steal/build: ELAI's mcp_schema_pin.rs (pin_or_verify + hash_tool_list + PinStatus + mcp_schema_pin_violation
  SSB) is TOFU-only — it pins at FIRST connect. On every `notifications/tools/list_changed`, recompute the
  blake3 schema-pin and BLOCK fail-closed on mismatch, requiring explicit operator re-approval; emit a new
  mcp_rug_pull_blocked row with the before/after tool-DESCRIPTION diff (line-jumping hides in descriptions).
  Grounding: OWASP MCP03:2025 (runtime schema change accepted w/o operator confirm) CONFIRMED; Trail-of-Bits
  line-jumping "alert on description changes" CONFIRMED; Invariant Labs WhatsApp sleeper rug-pull CONFIRMED.
  (Correction carried: mcp-scan is Invariant Labs not Snyk; its dist-channel UNVERIFIED — anti-slopsquat first.)
- COMPOSES-WITH: E-24 (MCP version pin), E-43 (skill-import whole-directory scan). Extends the SHIPPED schema-pin.
- Integration: the MCP plugin's tools/list_changed handler (plugins/src/mcp/*) -> pin_or_verify recompute ->
  PinStatus::Violation -> block + mcp_rug_pull_blocked. frontend_pair: the before/after description diff in a drawer.
- observable_effect: a fixture MCP server that swaps a tool description after first-connect approval triggers a
  mcp_rug_pull_blocked row with the diff; the changed tool is blocked until operator re-approval.

### G-5 — Reviewer Cockpit + Diff-Surface-to-Review (DSR) metric
- Roadmap R5 | lever verified-before-seen | grounding STRONG | effort M | WAVE 2 | depends G-2
- Steal/build: a per-hunk view rendering the proof tokens ELAI ALREADY emits (F-4 stdout_hash-bound passing
  contract test, grounding-sweep citation, F-1/F-2 write-gate verdict) so a reviewer SKIMS vouched hunks. DSR =
  (lines needing full attention) / (total changed lines), computed deterministically from SSB events (GD-2) — the
  headline review-time number. Silent-by-default: surface a finding ONLY when backed by a deterministic gate
  verdict or a failing test (no LLM-opinion noise). Grounding: METR (arXiv:2507.09089 — reviewing/editing AI
  output is THE confirmed time sink, 19% slowdown) CONFIRMED; code-review-agent PRs sit at 0-30% signal in 60.2%
  of cases (arXiv:2604.03196) CONFIRMED; naive LLM review is style-biased + misses SQLi/XSS (arXiv:2509.13650).
- COMPOSES-WITH: F-4 (consumes stdout_hash), G-2 (consumes the RRA evidence manifest), the grounding-sweep.
- Integration: a new cockpit view in dashboard-v2 reading the RRA + SSB evidence; a CLI `elai review --cockpit`.
  observable_effect: a fixture diff renders with vouched/un-vouched bands + a review_surface_scored SSB row whose
  DSR re-computes byte-identically from the journal (GD-2). DSR + escaped-defect-rate is also the GD-4 FP early-warning.
- MEASUREMENT (the review-time number): a within-subjects screen-recorded clustered-SE study (METR design) of
  time-to-first-approve through the cockpit vs a plain diff, escaped-defect-rate as the guardrail. Speed-AND-safety
  simultaneity is the HYPOTHESIS the study TESTS (industry-wide unmeasured), NOT a pre-claimed result.

### G-6 — SAFEDELTA competitive safety-delta benchmark
- Roadmap R6 | lever auditable-reproducible | grounding STRONG | effort XL | WAVE 3 | depends G-2 + F-2b LIVE
- Steal/build: a gate-ON-vs-gate-OFF ABLATION (GD-3) measuring the gate's marginal residual-insecure-rate that
  reaches a human, on git-stripped held-out tasks, secondary head-to-head vs competitor HARNESSES. Suites
  (all IDs verifier-CONFIRMED): SUSVIBES-200 (arXiv:2512.03262), BaxBench-392 (arXiv:2502.11844, DYNAMIC exploit
  exec), SeCodePLT (arXiv:2410.11096, via the SHIPPED seccodeplt_runner), AgentDojo (arXiv:2406.13352), optional
  MSB (arXiv:2510.15994). Competitors: ELAI gate-ON | ELAI gate-OFF | Cursor+frozen-model | Claude Code | Copilot
  | Cline. Anti-gaming: git-history strip + a cheat-detector cloned from Endor's finding that SWE-Agent+Opus
  exploited git history in 163/200=81% of tasks; DYNAMIC exploit exec; held-out CWE split; the ablation arm;
  SAFEDELTA's own PASS bound to an F-4 stdout_hash. Headline: "N fewer dynamically-exploitable diffs per 100 tasks
  reach a reviewer," 95% bootstrap CI, per-CWE breakdown. NEVER a SWE-bench rank.
- COMPOSES-WITH: F-2b LIVE keystone (the dynamic single-harness extension of seccodeplt_runner — MUST land first;
  G-6 does NOT redefine F-2), G-2 (reproducibility wrapper), F-4 (binds SAFEDELTA's own result).
- observable_effect: `elai bench safedelta` runs the matrix under ELAI_SEED, emits a safety_delta_measured row +
  a signed, independently-replayable run manifest. DEFERRED: no external claim before the F-2b dynamic floor.
- CAVEAT (whats_missing): with 200-392 tasks + small per-CWE cells, CI tightness for a significance claim is
  unresolved; may require pooling suites (cross-harness methodological defensibility itself unverified).

### G-7 — Per-change confidence PRODUCER + Dual-LLM lethal-trifecta quarantine
- Roadmap R7 | lever honest-about-limits + cant-wreck | grounding STRONG | effort L | WAVE 3 | depends G-1
- Steal/build: ELAI SHIPS the conformal refusal CONSUMER (conformal_calibration.rs: decide / claim_score / tau /
  TauUpdater) but the composite claim_score PRODUCER is a deferred slot. G-7 fuses token log-prob + self-consistency
  dispersion + oracle-free generated-test execution-agreement into the scalar claim_score decide() already
  consumes, so the abstention layer actually FIRES on real changes (the "honest stop"). Plus: when one context
  holds Trusted-secret + Untrusted + egress, route the untrusted span through a quarantined sub-LLM with no tool
  grants (FIDES query_llm / HIDE + Design-Patterns Dual-LLM arXiv:2506.08837). Grounding: post-hoc Platt rescaling
  is model-size-agnostic — the 7B IS a viable calibration target, ECE 0.32->0.03 (Spiess arXiv:2402.02047
  CONFIRMED); oracle-free execution signal (CodeHalu arXiv:2405.00253 + Task-Abstention arXiv:2605.17029 CONFIRMED).
  UNVERIFIED: UniCR (2509.01455) risk-control guarantee is MODERATE (preprint) — do not cite as proven.
- COMPOSES-WITH: D-16 (the shipped conformal layer), G-1 (live taint detects the trifecta), F-4 (execution signal).
- observable_effect: a fixture change with low self-consistency yields a claim_confidence_scored row + a conformal
  ABSTENTION (the change is held / surfaced low-confidence); a trifecta fixture yields trifecta_quarantined + the
  untrusted span never reaches the tool-caller. Feeds the G-5 confidence-triage band (read low-confidence first).

### G-8 — Signed in-toto provenance attestation + Article-50 verifier CLI
- Roadmap R8 | lever auditable-reproducible | grounding STRONG | effort L | WAVE 2 | depends G-2
- Steal/build: a custom in-toto v1 predicate (subject bound by blake3/sha256 digest; carries model id, ELAI_SEED,
  an SLSA-VSA-style PASSED/verifiedLevels rollup of every F-gate, evidence hashes, SSB event id) emitted to a git
  commit trailer (digest only) + a sidecar file + a new SSB action_type; signed via a LOCAL blake3-Merkle proof
  (offline default) or opt-in cosign keyless. New `elai verify-attestation` re-checks OFFLINE. Grounding: in-toto
  Statement v1 custom-predicate freedom CONFIRMED; SLSA VSA "trust without re-running" CONFIRMED; Sigstore Rekor-v2
  GA CONFIRMED; EU AI Act Art-50(2) marking + the Code-of-Practice cryptographic/logging route CONFIRMED. POSITION
  AS "designed to support Article-50 marking", NOT "certified compliant" (final Code June 2026; source code is not
  an enumerated example). Penalty tier is EUR 15M/3% (Art-99), NOT 35M/7%.
- COMPOSES-WITH: F-4 (extends anti-fabrication from inside-the-run to off-machine), G-2 (shares the evidence
  manifest + SSB chaining). Anti-slopsquat: cosign/sigstore are opt-in deps, verify-before-add; local-Merkle is
  hand-rolled default.
- observable_effect: `elai run` emits an attestation; `elai verify-attestation <commit>` re-verifies OFFLINE and
  prints verified / not-verified WITHOUT re-running ELAI; provenance_attestation_emitted SSB row.

### G-9 — SSB/log write-time secret+PII redaction gate + DB-file-granularity TTL
- Roadmap R9 | lever auditable-reproducible + honest-about-limits | grounding MODERATE | effort M | WAVE 2 | depends G-2
- Steal/build: a LOCAL-ONLY (no cloud egress, unlike ggshield/Nightfall) fail-closed redactor on the single-writer
  SSB commit path + the hook-log sink, replacing detected secrets/PII with stable blake3-keyed tokens so the
  append-only rows stay valid (ssb.rs:5 "no UPDATE/DELETE" forces WRITE-TIME redaction — cannot scrub after).
  Retention via purging whole expired SESSION DBs (file-granularity, since rows cannot be deleted). Grounding:
  ssb.rs payload carries the raw prompt; OWASP AI Agent Cheat Sheet "audit memory before persistence" + "redact log
  data" CONFIRMED; GitGuardian 24,008 MCP-config secrets + 3.2% Claude-Code leak rate CONFIRMED; ggshield 3-moment
  block/notify CONFIRMED. CORRECTIONS (binding): EU penalty EUR 15M/3% not 35M/7%; PRvL Presidio numbers REFUTED —
  re-source from Presidio docs; NO Presidio/ggshield dependency (hand-rolled local redactor).
- COMPOSES-WITH: F-1 (reuses the secret regex set), G-2 (the RRA must NOT export unredacted secrets — G-9 is its
  precondition). MODERATE grounding: PII detection is inherently high-recall/low-precision; bound the FP rate (GD-4).
- observable_effect: a fixture event carrying an AWS key + an email is redacted to blake3 tokens at the SSB commit
  path; the raw secret never lands in ssb.db or the hook log; an expired session DB is purged whole. Makes the
  RRA/attestation shippable to a client without being a secrets-exposure liability.

## Post-G-9 extensions (added 2026-06-02 — PLANNED, implementation pending)
One initiative added after the original R1..R9 set, derived from a gap analysis corroborated by external
practitioner discourse (r/ClaudeAI "production-ready AI is not sexy work" — tier-5 sentiment, corroboration
not a requirement) and CONFIRMED by an elai.map grep. NOT a trust-max roadmap item; a COMPOSITION of shipped
substrate.

### G-10 — Model-Drift Regression Canary (exogenous model-version behavioral-regression gate)
- Post-G-9 ext | HYBRID | novelty GENUINELY_NEW (every ELAI regression/canary/drift symbol keys on ENDOGENOUS
  change; nothing watches the model underneath) | grounding STRONG | feasibility EASY-M | effort M | WAVE 3 |
  depends G-2; reuses D-7
- Steal/build: pin the (model, system_fingerprint, lane) identity tuple, hold a small fixed canary set with
  baseline scores keyed to that identity, and on a detected identity change (or schedule) re-run the canary,
  compute a per-family regression delta vs the last-good baseline, and on regression FLAG model_drift_suspected
  + route the affected ReasoningTemplates to review. SURFACE, never auto-swap by default (GD-4). The literal
  "roll back when a model update breaks your use case" gap. Prior art: arXiv:2311.11123 "(Why) Is My Prompt
  Getting Worse?" (silent API upgrades degrade the GPT-3.5 family; prompt is a key factor), RETAIN
  (arXiv:2409.03928), AgentAssay (arXiv:2603.02601, reports a silent provider update 93%->71% routing —
  MOTIVATION, not an ELAI target).
- Existing ground truth (grep-confirmed 2026-06-02): NO model-version pin type; security_monotonicity /
  staged_eval / optimizer_admit / canary.rs / goal_drift_canary ALL ENDOGENOUS-keyed; autonomous_gate::
  substitute_model is restart/VRAM-triggered; ssb ModelSwap/TierModelSwap are WIRE-DARK (no production emitter).
  canary.rs CanaryFamily + FamilyEvalResult + the non-determinism band, held_out::TaskId, reasoning_template::
  Domain ALL shipped.
- COMPOSES-WITH (GD-1): D-7 canary.rs (REUSE families+delta+band — re-implements none), G-2 RunArtifact
  (system_fingerprint+model_pin = the identity input), F-4 stdout_hash (the malformed-output-spike signal),
  D-1 ReasoningTemplate (the review/revert target). HONEST SCOPE (GD-5): local lane = fingerprint-confirmed;
  cloud lane = behavioral-suspected (Anthropic system_fingerprint opaque per G-2 §6), never fabricated.
- Integration: new crates/orchestrator/src/model_drift.rs + run-end check in runtime_setup.rs + `elai model-drift
  baseline|check|show` CLI; +model_drift_suspected SSB (Pattern-2); ModelDriftPanel frontend pair. Auto-pin-to-
  last-good is a SEPARATE wire-dark-allowlisted human-gated follow-on (would give ModelSwap its first emitter),
  NOT in the done-gate.
- observable_effect (done-gate): baseline a canary set under fixture model A, then `model-drift check` under a
  mock-provider model B returning a regressed output -> a model_drift_suspected row (signals + per_family_delta
  + affected_template_ids + confirmed_by_fingerprint); a CLEAN re-check under A within the non-determinism band
  -> NoDrift + NO row (the GD-4 FP guard); the delta re-computes byte-identically from the SSB journal under
  ELAI_SEED (GD-2); ModelDriftPanel shows the before/after scores + the fingerprint-confirmed/behavioral badge.

## Cross-cutting (what makes G-plan different from F-plan)
- Every G-N COMPOSES-WITH an F-gate by consuming its verdict, or fills a read/runtime/measurement gap (GD-1);
  re-stating any F-1..F-10 is a P0.
- Every emitted number (DSR, safety-delta, confidence, FP-rate) is recomputable from the SSB journal under
  ELAI_SEED (GD-2) — the metric is auditable, not just the gate.
- Every safety/perf number is gate-ON-vs-gate-OFF ablation framing, never a SWE-bench rank or perfect-code
  promise (GD-3).
- No gate flips default-ON until its FP/over-block rate is bounded on >=3 real runs (GD-4 — the CodeGate death;
  RTBAS Slack 33% over-tainting is the measured failure mode).
- No fail-open / RunPod-pending / cloud-opaque path advertises a guarantee it does not deliver (GD-5).

End of G-SUMMARY_INDEX.md.
