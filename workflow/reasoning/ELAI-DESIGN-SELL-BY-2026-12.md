ELAI MASTER DESIGN — "BE UNIQUE AND SELL BY END OF 2026"
Authored 2026-06-05 by Claude Opus 4.8 (1M). Single buildable backlog.
Fuses: ELAI-DIRECTION-2026-06-01.md (the pivot) + ELAI-FOCUS-RESCOPE-2026-06-02.md (the wedge +
hard truths) + ELAI-COMPETITIVE-ASSESSMENT-GROUNDED-2026-06-04.md + ELAI-FUTUREPROOF-DIRECTION-
2026-06-04.md (the interop/control-plane refinement). Code anchors VERIFIED against current
.elai_cc this session via one Explore agent (see Appendix A; tags below).

GROUNDING TAGS (per item):
  [LIVE]    verified present + wired in current code this session
  [STUB]    verified present but inert/stubbed/dark in current code
  [CLIENT]  verified present in one direction only (e.g. MCP client, not server)
  [CITED]   from F/G/H summaries (grep-confirmed ~2026-06-02/03); RE-VERIFY next session
  [NEW]     net-new; no current code
Every item carries a DONE-GATE = an observable_effect (W-1 doctrine: off-agent artifact, fail =
missing evidence). Next session: run the GROUNDING PROTOCOL (Part 9) before building each item.

THE SIX-MONTH FRAME (now -> Dec 2026): the prior records' "2-3 quarter window" maps to ~2 quarters
left. Releasable-by-Dec means: ONE wired vertical + the SAFEDELTA number + the killer demo + ONE
distribution hook. Everything tagged P2/P3 is post-release. Do not widen scope.

================================================================================
PART 0 — THE THESIS (what makes ELAI unique AND sellable by Dec 2026)
================================================================================
Prior records fixed the WEDGE: "trust-evidenced autonomous execution wrapped around a cloud
backbone (Sonnet), for a buyer who can action a gated diff." Correct, but it left three root
contradictions unsolved: (1) Anthropic-dependency, (2) solo bus-factor in a no-rug-pull pitch,
(3) distribution for a closed-source solo dev. This design ADDS the one structural move that
attacks all three at once:

  REPOSITION ELAI AS THE TRUST/VERIFICATION CONTROL PLANE FOR THE AGENT ERA —
  the external, deterministic, off-agent reference that breaks the circular-AI-review problem,
  exposed over the open agent protocols (MCP server + A2A peer) so it GATES Claude Code / Cursor /
  Copilot / background-agent fleets instead of competing with them for the editor seat.

Why this is unique AND sellable, grounded (2026):
- Verification is the named #1 bottleneck (Sonar 96% distrust; 24% of work-week on review; Qodo
  $70M; Fortune "trust is the bottleneck"). AI-reviewing-AI is provably circular without an
  EXTERNAL executable reference (arXiv:2603.25773, SRLabs) — the gap ELAI is built to fill.
- The field standardized: MCP is the de-facto tool standard (Linux Foundation, ~18k servers),
  A2A the inter-agent standard. A control plane that speaks them is a "guardian agent" (named
  top-5 2026 trend); a monolithic non-interoperable IDE is what dies.
- It dissolves the contradictions: gating ANY agent => model-agnostic (kills Anthropic-dep as a
  positioning problem); "a verifier that plugs into your existing tool" has a far lower adoption
  barrier than "switch IDEs" (the distribution answer); and the detached verifier artifact (Part
  6) is the no-rug-pull credibility mechanism (bus-factor answer).

This does NOT replace the wedge — it is the DELIVERY VEHICLE for it. The three keepable promises
(cant-lie / cant-wreck / honest-stop) are still the product; the control plane is how they reach
users without a switch-cost and without depending on a single model vendor's goodwill.

KEEP / CEDE (unchanged from the two prior records, do not relitigate):
  KEEP: trust/verification/determinism spine; prevention-in-the-loop VibeSec; autonomy-you-can-
        trust; predictable cost + privacy/no-rug-pull as SUPPORTING.
  CEDE: website/app builder; non-coder primary; raw token-efficiency + inline completion;
        standalone routing; skill marketplace; "local-as-flagship." Plumbing stays; never headline.
  NEVER: promise perfect code; claim benchmark wins (SWE-bench Verified is dead); lead on cost.

================================================================================
PART 1 — TRACK A: THE SPINE (the unifying architecture; do this first)
================================================================================
The entire repositioning rests on one refactor + one exposure. Without A1 the protocol surface
re-implements gate logic (a P0 per the project's own GD-1 compose-not-restate doctrine).

A1. UNIFIED GATE TRAIT SEAM  [NEW over LIVE gates]  P0
  WHY: today gates live at scattered call sites (PreToolUse hook, wiring audit, RRA emit,
       test-integrity, routing). To expose them over a protocol they must share ONE interface.
  CURRENT CODE: gates exist but un-unified — [LIVE] run_pre_tool_use + InProcessHookDispatcher
       .with_vibegate_ssb() at runtime_setup.rs:778; [LIVE] build_production_hook_dispatcher at
       runtime_setup.rs:766; [LIVE] HookEvent::PreToolUse at orchestrator/src/hooks.rs:19.
  CHANGE: define `trait Gate { fn id(&self) -> GateId; async fn evaluate(&self, change: &Change)
       -> SignedVerdict; }` where SignedVerdict = {verdict: Allow|Warn|Block, evidence_hash:
       blake3, ssb_seq, gate_id, signed: bool}. Wrap each existing gate (VibeGate, wiring,
       test-integrity, FS/egress, confidence) as a Gate impl. Compose, do NOT rebuild logic.
  DONE-GATE: a `GateRegistry::evaluate_all(change)` call returns N SignedVerdicts, each row
       recomputable from the SSB journal under ELAI_SEED; one contract test drives 3 gates through
       the seam and asserts byte-identical verdict hashes across two runs.
  DEPENDS: none. EFFORT: M. This is the keystone enabler for A2 + Track D.

A2. PROTOCOL-ADDRESSABLE GATES (the seam's output is the product)  [NEW]  P0
  WHY: A1's SignedVerdict is what an external agent/IDE consumes. Make it serializable + stable.
  CHANGE: ts-rs/serde-stable SignedVerdict + a GateVerdict golden-file lock (reuse E-45 action_type
       golden-lock pattern) so the wire format never silently drifts. Add a `verdicts` SSB
       action_type (SCHEMA bump from current 224 -> 225; TOTAL_VARIANTS 501 -> +N; update
       action_type_lock.rs:645 in lockstep — VERIFIED current values this session).
  DONE-GATE: a fixture change produces a SignedVerdict JSON that round-trips through the golden
       lock; a renamed verdict field fails cross_lang_diff.py + the golden test.
  DEPENDS: A1. EFFORT: S.

================================================================================
PART 2 — TRACK B: THE THREE KEEPABLE PROMISES (the SAFEDELTA cases; code-anchored)
================================================================================
These are the demo cases AND the SAFEDELTA measurement cases (Part 3). Each maps to one promise.

B1. "CAN'T LIE" — anti-fabrication test-integrity ENFORCEMENT (F-4)  [LIVE-FIELD, NEW-ENFORCE]  P0
  WHY: Replit faked tests + lied; arXiv:2512.04864 upward-deception pervasive across 11 agents.
  CURRENT CODE: [LIVE] TestOutcomePayload ALREADY has `stdout_hash: Option<String>` (blake3,
       ssb.rs:10494-10504, added SCHEMA 197). >>> CONTRADICTS the F-4 plan premise that the field
       is missing. The real gap is ENFORCEMENT, not the field.
  CHANGE: gate the AUDITING->MEMORIZING transition so a `passed=true` row with `stdout_hash=None`
       (or a hash not matching an orchestrator-CAPTURED process-slot hash) emits anti_fabrication_
       block. PRE-RESOLVE: the auditor must run tests through an orchestrator-managed process slot
       so there IS a stdout handle to hash (else the field is dead). Wrap as a Gate (A1).
  DONE-GATE: inject a baby step whose test process exits 1 while the auditor claims PASS ->
       anti_fabrication_block fires; audit-timeline shows it; the SAFEDELTA fabrication-catch case.
  DEPENDS: A1. EFFORT: M. NOTE: re-verify the capture-slot wiring exists (CITED from C-22 stdout_hash pattern).

B2. "CAN'T WRECK" — deterministic post-shell-EXPANSION FS gate + egress IP-pin (G-3)  [CITED]  P0
  WHY: Replit deleted the prod DB during a freeze + lied rollback was impossible; rm-rf #10077/#12637
       (glob/tilde expansion AFTER validation); LLM permission classifier collapses to 81% FNR under
       adversarial escalation (arXiv:2604.04978) -> deterministic gate beats classifier.
  CURRENT CODE: [CITED] E-30 argv destructive-verb allowlist; sandbox/researcher_net.rs allowed_hosts
       + veth+iptables; compile_seccomp_network_extra opens AF_INET wholesale (the gap). RE-VERIFY.
  CHANGE: canonicalize ~/glob/symlink BEFORE the verb runs; fail closed outside the worktree root;
       host-side egress allow-list that RESOLVES+PINS the IP and rejects null-byte/control-char
       hostnames; the block row carries the FULL pre- AND post-expansion command (the #10077
       forensics gap). Wrap as a Gate (A1). Linux first; flag Windows/macOS FS-gate parity separately.
  DONE-GATE: `rm -rf $X` where $X expands to ~/ OR a null-byte-spoofed egress host is BLOCKED before
       the verb/connect; blast_radius_blocked row carries pre+post-expansion command. SAFEDELTA wreck case.
  DEPENDS: A1. EFFORT: M.

B3. "HONEST ABOUT LIMITS" — confidence producer + conformal abstention that FIRES (G-7/D-16)  [CITED]  P1
  WHY: Devin 15-30% real vs 70%+ claimed. "Knows when it doesn't know" is the trust capstone.
  CURRENT CODE: [CITED] conformal_calibration.rs CONSUMER (decide/claim_score/tau/TauUpdater) ships;
       the composite claim_score PRODUCER is a deferred slot. RE-VERIFY.
  CHANGE: fuse token log-prob + self-consistency dispersion + oracle-free generated-test execution-
       agreement into the scalar claim_score decide() already consumes; on low score, AUTONOMOUS mode
       HOLDS/surfaces low-confidence instead of asserting. Wrap as a Gate (A1).
  DONE-GATE: a fixture change with low self-consistency yields claim_confidence_scored + a conformal
       ABSTENTION (held/surfaced); feeds the cockpit "review low-confidence first" band.
  DEPENDS: A1, B1 (execution signal). EFFORT: L.

B4. SECURITY PREVENTION-IN-THE-LOOP — VibeGate as default-on pre-write gate (F-1/F-2/F-10)  [LIVE]  P1
  WHY: post-hoc scanners miss AI vulns (ProjectDiscovery Feb 2026: Snyk+Invicti caught 0/21 on 3 AI
       apps). Pre-write fail-closed is the angle (scanner category is taken).
  CURRENT CODE: [LIVE] VibeGate wired via InProcessHookDispatcher.with_vibegate_ssb() (runtime_setup.rs:778);
       F-2a 12 security rules + seed mutable=0 ship; [STUB/CITED] bench_seccodeplt exists but NEVER RUN.
  CHANGE: confirm default-on (not env-gated terminal state, W-2); add F-2c low-FP CWE rules
       (798/321/532/95) on registered Py+TS; add F-10 per-iteration security-MONOTONICITY (block if
       iteration N introduces a CWE class absent at N-1). Bound FP rate on >=3 real runs (GD-4). Gate via A1.
  DONE-GATE: a generated SQLi/hardcoded-secret Edit is BLOCKED pre-write (security_rule_blocked);
       measured FP rate logged; SAFEDELTA insecure-rate case.
  DEPENDS: A1. EFFORT: M (rules) + L (F-10 classifier, deferred to P2).

================================================================================
PART 3 — TRACK C: THE NUMBER (SAFEDELTA — the whole game; nothing sells without it)
================================================================================
C1. SAFEDELTA gate-ON-vs-OFF ablation harness (G-6)  [STUB]  P0 (R1 from both prior records)
  WHY: a self-published solo number is structurally discounted, but ZERO number is unsellable. This
       is R1 in both prior records and Hard Truth 5.2. The demo and the number are the same deliverable.
  CURRENT CODE: [STUB] seccodeplt_runner.rs / bench_seccodeplt.rs ship but were NEVER RUN; SWE-bench
       Pro runner RUNPOD-PENDING. RE-VERIFY.
  NO LOCAL GPU REQUIRED (corrected 2026-06-05): the measured backbone is CLOUD Sonnet 4.6 by design
       (Hard Truth 5.3 — local 7B is stale+abliterated, would inflate the delta), the gates are CPU
       (Rust/rules/blake3/AST), and generated-code exploit-exec runs in ELAI's OWN sandbox (NOT Docker
       — corrected; ELAI uses Firecracker/seccomp+clone3, see Appendix B). Cross-family adversary, if
       used, can be a different CLOUD family (DeepSeek-Flash ~$0.14/$0.28 per Mtok), not a local model.
       The 2x3090 is for the SECONDARY local-lane number + a local PPM/cross-family verifier to save
       cloud cost — NOT for the headline trust number.

  TWO-TIER NUMBER (verified in code 2026-06-05, Appendix B):
   - TIER-1 STATIC [LIVE, runnable NOW, ZERO infra]: seccodeplt_runner.rs drives the REAL VibeGate over
     an embedded SeCodePLT CWE corpus -> insecure_rate_before/after + gate_recall + InsecureRateMeasured
     SSB. NO model, NO sandbox, NO GPU, NO cloud. This is the FIRST publishable number and it is a
     near-immediate quick-win — do this BEFORE Tier-2 to kill "we have zero number" risk on day one.
     (Caveat: static-gate-recall on a synthetic corpus; state the limit honestly, it is not the dynamic claim.)
   - TIER-2 DYNAMIC [STUB, the real SAFEDELTA]: safedelta_runner.rs — live cloud model generates ->
     code EXECUTED in ELAI's sandbox -> exploits fired -> functional pass@1 + dynamic secure-rate,
     gate-ON vs gate-OFF. Needs: cloud API budget (no GPU) + a LINUX CPU host with the sandbox ENABLED.
  C1's real constraints: (a) cloud API budget; (b) a Linux CPU host (Firecracker wants KVM/nested-virt,
       else gVisor/seccomp fallback) — NOT a GPU box; (c) flip enable_sandbox on Linux (B-18B wiring,
       see Appendix B item 4 — currently default-FALSE on Linux).
  CHANGE: run gate-ON vs gate-OFF on the SAME backbone (cloud Sonnet 4.6 — NOT the stale local 7B,
       per Hard Truth 5.3), same tasks, same harness. Metrics: insecure-rate (BaxBench, DYNAMIC
       exploit exec) + fabrication-catch-rate (B1) + catastrophic-block-rate (B2). Anti-gaming:
       git-history strip + cheat-detector (Endor found 81% git-history exploitation) + held-out CWE
       split. Bind SAFEDELTA's own PASS to a B1 stdout_hash. Headline: "N fewer dynamically-
       exploitable diffs per 100 tasks reach a reviewer," 95% bootstrap CI, per-CWE breakdown.
       NEVER a SWE-bench rank.
  DONE-GATE: `elai bench safedelta` emits safety_delta_measured + a signed, independently-replayable
       run manifest; the GATE VERDICTS recompute bit-identically from the SSB journal under ELAI_SEED;
       the aggregate insecure/fab/wreck rates carry a 95% bootstrap CI (cloud generation is not token-
       reproducible per GD-5 — report the deterministic gate decisions + the statistical rate, honestly).
  DEPENDS: Tier-1 = NONE (runnable now). Tier-2 = B1,B2,B4 (gates measured) + cloud API budget + a Linux
       CPU host + enable_sandbox-on-Linux wiring (B-18B). NO LOCAL GPU on either path. EFFORT: S (Tier-1) /
       XL (Tier-2). TIER-1 IS A DAY-ONE QUICK-WIN; TIER-2 GATES THE RELEASE.
  COST NOTE: ~N hundred-to-thousand $ of cloud spend (4 suites x 2 arms x bootstrap repeats). Bound it
       with ELAI's own cost gate and PUBLISH the bill ("SAFEDELTA cost $X, reproducible") — on-thesis,
       not a problem. Cost (boundable) replaces hardware (procure+setup) as the only C1 constraint.

C2. THE KILLER DEMO (Replit Incident 1152 replayed live, gate-OFF vs gate-ON)  [NEW]  P0
  WHY: the distribution wedge with no OSS virality is a demo + content, not a download count.
  CHANGE: same task, same cloud Sonnet, side-by-side: LEFT (stock harness) wipes DB + reports "tests
       passing" + claims success; RIGHT (ELAI gates-on) FS-gate fail-closes the destructive command,
       test-integrity shows stdout-hash mismatch = FABRICATED, run ends with honest stop + downloadable
       Reproducible-Run Artifact. Close: "Same model, same task. One lied and wiped your database. The
       other physically couldn't, and proved it."
  DONE-GATE: a recorded 60s demo where every RIGHT-side block is a MEASURED SAFEDELTA case (not staged).
  DEPENDS: B1, B2, C1, D2 (RRA). EFFORT: S once gates+number exist.

================================================================================
PART 4 — TRACK D: INTEROP & DISTRIBUTION (the unique + the answer to the 3 contradictions)
================================================================================
D1. INVERT THE MCP BRIDGE: ship an `elai-verify` MCP SERVER  [CLIENT -> NEW server]  P0
  WHY: distribution without a switch-cost. Claude Code/Cursor/Copilot/Windsurf all speak MCP. ELAI
       becomes a tool INSIDE them. This is the single highest-leverage distribution + Anthropic-dep move.
  CURRENT CODE: [CLIENT] crates/plugins/src/mcp_bridge.rs is CLIENT-ONLY (serves mcp_* calls FROM the
       Claude-Code-hosted server; does NOT expose ELAI as a server). VERIFIED this session. The server
       direction is genuinely net-new.
  CHANGE: an MCP server exposing the A1 gates as tools: verify_change, replay_run, emit_attestation,
       check_wiring, security_gate, score_confidence, register_trial. Reuse the GateRegistry (A1)
       behind the protocol; the gates already exist, only the adapter is new. Use rmcp (CLAUDE.md
       anti-stale: rmcp 1.7.0, NOT a hand-roll) — verify version before add (anti-slopsquat).
  DONE-GATE: Claude Code (or any MCP client) calls elai-verify.verify_change over MCP and receives a
       SignedVerdict; the call + verdict land as SSB rows; a contract test drives the server end-to-end.
  DEPENDS: A1, A2. EFFORT: M. P0 — this is the distribution hook for the release.

D2. REPRODUCIBLE-RUN ARTIFACT + detached verifier (G-2 + G-8)  [CITED]  P0
  WHY: the no-rug-pull credibility mechanism (Hard Truth 5.5) + the demo's proof object + the
       trust-buyer's audit artifact. Solves bus-factor: a user-owned artifact survives the vendor.
  CURRENT CODE: [CITED] RRA design (G-2) + in-toto attestation (G-8); determinism substrate is [LIVE]
       (DriftComparator determinism.rs:317, ClockProvider throughout, ELAI_SEED). RE-VERIFY RRA emit path.
       HONEST SCOPE (GD-5): cloud-lane = reproducible-ACTION-STREAM + verify-divergence, NOT
       reproducible-tokens (Anthropic system_fingerprint opaque). State which; do not over-claim.
  CHANGE: `elai run --emit-rra` bundles SSB action stream + seed + model pin + per-gate blake3 verdict
       hashes + a blake3 root; `elai replay --verify` re-runs + reports PASS or first-divergence;
       `elai verify-attestation` re-checks OFFLINE without ELAI's engine (the escrow/open-the-verifier move).
  DONE-GATE: a fixture task emits a .rra; replay --verify reports PASS on clean / DIVERGENCE on tamper;
       verify-attestation runs on a machine without the engine and prints verified/not-verified.
  DEPENDS: A1 (verdict hashes), B1/B2 (the gate evidence in the manifest). EFFORT: M (RRA) + L (detached verifier service).

D3. SPEC-KIT / SDD INTEROP (intent as the interop boundary)  [CITED]  P2 (post-release)
  WHY: spec-driven dev is the dominant 2026 methodology (Spec Kit 93k stars, 30+ agents incl Claude
       Code; Kiro). ELAI's decomposition tree IS SDD but proprietary. Interop makes ELAI's verification
       the trusted layer ON the industry-standard spec (the "executable spec as external reference").
  CURRENT CODE: [CITED] decomposition tree Concept>Phase>Step>BabyStep + idea_framing.rs + BabyStepSpec
       (intent_pairs C-29, MEASURABLE_PROCEDURE D-12). RE-VERIFY.
  CHANGE: import/export GitHub Spec Kit Spec>Plan>Tasks>Implement markdown; map intent_pairs +
       MEASURABLE_PROCEDURE + wiring contracts onto the imported spec as the verification layer.
  DONE-GATE: a Spec Kit spec.md imports into an ELAI decomposition tree and exports back round-trip;
       its acceptance criteria compile to SpecOracles (E2).
  DEPENDS: E2. EFFORT: M.

D4. A2A PEER (`elai-guardian`)  [NEW]  P3 (post-release, the fleet future)
  WHY: 2027 = multi-agent mastery; A2A is the inter-agent standard (Linux Foundation). ELAI as the
       "guardian agent" a fleet submits diffs to for a signed verdict + RRA.
  CHANGE: an A2A peer exposing the same GateRegistry; fleet members submit a Change, get SignedVerdict + RRA.
  DONE-GATE: a second agent submits a diff over A2A and receives a signed verdict + replayable artifact.
  DEPENDS: A1, D1, D2. EFFORT: L.

================================================================================
PART 5 — TRACK E: EFFICIENCY (supporting / retention hook; never the headline)
================================================================================
E1. COST COCKPIT — activate the spend ledger end-to-end (H-9)  [PARTIAL-LIVE]  P1
  WHY: predictable/auditable cost is the retention hook vs Claude Code cost pain (MS revoked access;
       Cursor revolt; the throttling "silence damages trust"). "Provable cost," not "cheap."
  CURRENT CODE: [PARTIAL] grounding_client.rs:915-960 HAS the `if let Some(ssb)` CloudSpend emit block
       (>>> CONTRADICTS H-9's "emit is gated dark" claim — the block exists). OPEN: does the production
       cloud-client CTOR attach .with_ssb_writer so Some(ssb) is true in prod? RE-VERIFY this exactly.
  CHANGE: ensure the prod cloud client attaches the SSB writer (the only real gap if the emit exists);
       build the cost feeder + a per-response input/output/CACHED breakdown + $/resolved-task +
       cache-hit-rate live in CloudPanel.
  DONE-GATE: a real cloud task makes CloudPanel render NON-zero input/output/cached counts + live
       $/task, every number recomputable from the journal.
  DEPENDS: none hard. EFFORT: M. NOTE: scope SMALL — verify the ctor-attach gap before building.

E2. SPEC-ORACLE COMPILER (the external-reference engine)  [CITED]  P1
  WHY: the technical heart of breaking circular review; makes "intent as source of truth" enforceable.
  CURRENT CODE: [CITED] IntentPair (C-29), MEASURABLE_PROCEDURE (D-12), checker registry (D-11),
       metamorphic relations (C-29). RE-VERIFY which of these are LIVE vs PLANNED.
  CHANGE: one `SpecOracle` type compiling (acceptance criterion + checker ref) -> deterministic
       pass/fail bound to an SSB row. Every spec line => a runnable external oracle the generator never
       sees the answer key to.
  DONE-GATE: a spec line compiles to a SpecOracle that runs and writes a pass/fail SSB row; a
       same-agent-authored test cannot satisfy it (independence check).
  DEPENDS: A1. EFFORT: L.

E3. DIFF/FIM OUTPUT + streaming early-abort (H-3)  [STUB]  P2
  WHY: output bills 5x; small attributable hunks also serve review (the real bottleneck).
  CURRENT CODE: [STUB] run_diff_step returns FocusTunnelExecError::MissingProvider (orchestrator.rs:126);
       DIFF never emits in prod, always STRUCTURED. VERIFIED stub this session.
  CHANGE: thread a real BufferEditor into ResolvedConfig so DIFF emits on cloud; relax ModeSelector
       Rule 1; construct Some(streaming_validator) (currently hardcoded None at runtime_builder.rs:306,
       CITED) + call on_chunk ~every 50 tokens; write StreamingEarlyTerminated. REUSE B-20 try_edit_tiers
       (shipped via STRUCTURED path) — do not rebuild.
  DONE-GATE: a real cloud edit emits a unified diff (fewer output tokens, journal-verified); an early-
       abort run writes StreamingEarlyTerminated + stops decoding.
  DEPENDS: none hard. EFFORT: L.

E4. IDEMPOTENT + RESUMABLE + REPLAYABLE tool calls (E-44)  [CITED]  P2
  WHY: required for the async-fleet future (D4); also hardens AUTONOMOUS mode.
  CURRENT CODE: [CITED] SSB append-only + checkpoint + session_dag; idempotency.rs/saga.rs PLANNED. RE-VERIFY.
  CHANGE: idempotency-keyed tool wrappers + SAGA compensation + Continue-As-New on the SSB substrate.
  DONE-GATE: a tool-call retry returns the idempotent result (same key, same result); SAGA compensates a partial failure.
  DEPENDS: none hard. EFFORT: M.

================================================================================
PART 6 — TRACK F: MODEL & INFRA (durability; insulates from the root contradictions)
================================================================================
F1. MODEL-AGNOSTIC + DRIFT-MONITORED RUNTIME (H-7 routing is MORE wired than thought)  [LIVE]  P1
  WHY: insulates from Anthropic-dep + model staleness BY CONSTRUCTION (model = replaceable monitored part).
  CURRENT CODE: [LIVE] RuntimeOrchestrator (runtime_orchestrator.rs), PerRoleRuntime (per_role_runtime.rs),
       RuntimeRoutingPolicy::evaluate (escalation_policy.rs:261) which DOES change the provider
       (LocalSidecar/CloudSonnet/CloudOpus/CloudDeepSeekFlash) >>> CONTRADICTS H-7's "doubly inert /
       observational" claim. RouteDecisionEvent emitted via CloudModeRouter::route (cloud_mode.rs).
  CHANGE: the routing already works — the gap is the ARCHITECT/EDITOR/VERIFIER split + a CALIBRATED
       escalation signal (replace hardcoded layer_failed:1, CITED at task_runner.rs:7606) instead of 7B
       self-confidence; bound over-escalation on >=3 runs (GD-4). Add the G-10 model-drift canary gating any swap.
  DONE-GATE: a real task routes a cheap edit to the editor + the plan step to the architect, verifier in a
       separate context, RouteDecisionEvent recorded; a mock model-version change fires model_drift_suspected.
  DEPENDS: A1 (verifier is a Gate). EFFORT: L. NOTE: re-ground the "inert" contradiction — routing may already be release-ready.

F2. LOCAL-MODEL REFRESH (Hard Truth 5.3 / R3)  [CITED config]  P1
  WHY: Qwen2.5-Coder-7B is 1-2 generations behind + abliterated (gate must reject more). Local = privacy floor.
  CURRENT CODE: [CITED] sidecar --model default = abliterated 7B (server.py:2338, cli/main.rs:2558).
  CHANGE: re-evaluate the agentic local lane against the 2026 frontier — Qwen3-Coder-30B (~58.7%
       SWE-bench Verified, 24GB) / DeepSeek-Coder-V3-distill (~40.5%, 12GB) / Devstral Small 2 (R3's
       pick). Keep 3B/7B for FIM only. VERIFY model cards + licenses before pinning (anti-slopsquat).
       Re-evaluate the old Qwen3-32B rejection against the CODER-tuned line specifically.
  DONE-GATE: per-VRAM tier resolves a current-frontier model+backend tuple; SAFEDELTA still runs on
       CLOUD Sonnet (local stays privacy floor).
  DEPENDS: F1. EFFORT: M (eval) + the box.

F3. SSB -> OTel GenAI semconv export (E-29)  [CITED]  P3
  WHY: observable fleets; enterprise adoption path; composes with the audit-trail buyer.
  DONE-GATE: a live session emits GenAI-conformant OTLP spans to a local collector.
  DEPENDS: none hard. EFFORT: M.

F4. BATCH/ASYNC LANE (H-5)  [NEW]  P3
  WHY: cost architecture for the non-interactive fleet (50% off); keep interactive on streaming.
  CURRENT CODE: [NEW] no Anthropic Batch API anywhere (CITED zero hits). RE-VERIFY.
  DONE-GATE: a Cosmos/research/eval lane routes through /v1/messages/batches; batch_lifecycle row at 50% price.
  DEPENDS: none hard. EFFORT: M.

================================================================================
PART 7 — THE KILL-LIST (do NOT build; protects the 6-month window)
================================================================================
- A better IDE / inline completion (Cursor owns Tab; excluded by the user).
- A bolt-on VibeSec SCANNER (category taken — Escape/Qodo/Semgrep; prevention-in-loop only).
- "Perfect code" / completion-guarantee messaging (burned Devin/Replit; sell calibrated trust).
- Benchmark-rank claims (SWE-bench Verified credibility collapsed 2026).
- Headlining self-improvement/DGM (research-only ICLR 2026; undermines the determinism pitch; keep dark).
- Website/app builder or non-coder deploy stack (needs the Lovable/Replit hosted runtime ELAI ships zero of).
- Standalone routing-as-product (OpenRouter $1.3B won it).
- Staying a monolithic non-interoperable island (THE biggest future-proofing risk — A1+D1 exist to prevent it).
- Widening to all ~110 plans before the release (R4: ONE wired vertical + the number).

================================================================================
PART 8 — SEQUENCING TO DEC 2026 (~2 quarters; ruthless)
================================================================================
RELEASE-BLOCKING (must all ship for a sellable release):
  S1. A1 + A2 (gate trait seam + serializable verdict)            [P0, M+S]
  S2. B1 + B2 (cant-lie + cant-wreck gates wrapped via A1)        [P0, M+M]
  S0. C1 TIER-1 static seccodeplt number (runnable NOW, zero infra)   [P0, S]  <-- day-one, kills "no number" risk
  S3. C1 TIER-2 SAFEDELTA dynamic on cloud Sonnet (cloud API + Linux CPU host + ELAI's own Firecracker/seccomp
      sandbox enabled; NO GPU, NO Docker)                            [P0, XL] <-- the gate on everything
  S4. D2 RRA + detached verifier (the proof object)              [P0, M+L]
  S5. C2 killer demo (Replit replay, all RIGHT-side = measured)   [P0, S]
  S6. D1 elai-verify MCP server (the distribution hook)           [P0, M]
THEN (release-hardening, P1, as the window allows):
  B4 VibeGate confirm-default-on + F-2c CWE rules; E1 cost cockpit (verify the tiny ctor gap first);
  F1 architect/editor/verifier split + drift canary; F2 local-model refresh; B3 + E2 (honest-stop + spec-oracle).
POST-RELEASE (P2/P3): D3 Spec-Kit interop; D4 A2A peer; E3 diff emission; E4 idempotency; F3 OTel; F4 batch; F-10 CWE classifier.

THE SINGLE CRITICAL PATH: A1 -> B1/B2 -> C1 (the number) -> C2 (demo) + D1 (MCP server) + D2 (RRA).
Everything else is hardening. The critical path is CLOUD-API + LINUX-CPU-HOST bound, NOT GPU bound
(corrected 2026-06-05): C1 runs on cloud Sonnet + ELAI's OWN Firecracker/seccomp sandbox (NOT Docker).
Release constraints: (a) cloud API budget for C1 Tier-2 (boundable, dogfood the cost gate); (b) a Linux
CPU host with enable_sandbox flipped on (B-18B wiring — sandbox code EXISTS, default-false on Linux);
(c) the Tier-1 static number (seccodeplt_runner) is runnable TODAY with zero infra — ship it first.
The 2x3090 is OFF the critical path — only for the optional SECONDARY local-lane number (F2 privacy-floor)
+ a local cross-family/PPM verifier to trim cloud cost. Do NOT let GPU procurement block the release.

================================================================================
PART 9 — GROUNDING PROTOCOL FOR NEXT SESSION (how to make every [CITED] item real)
================================================================================
This file carries VERIFIED anchors for the 10 keystones (Appendix A) and [CITED] for the rest.
Next session, BEFORE building each [CITED] item, ground it (the project's context-delegation +
grounded-by-default rules apply — main LLM does NOT read source directly; dispatch agents):

  G-STEP 1. For each [CITED] item, dispatch ONE Explore/general-purpose agent (or context-compiler
            via the orchestrate path) to confirm the named symbol/file:line EXISTS in current .elai_cc
            and report drift. Prefer grepping .claude/tools/elai.map first (regen if stale via
            elai_map_freshness.py). Batch related anchors into one agent call (parallel_cap respected).
  G-STEP 2. Re-verify the THREE contradictions this session surfaced (they change scope):
            (a) B1 — TestOutcomePayload.stdout_hash EXISTS; confirm the orchestrator CAPTURE-SLOT
                exists so enforcement is buildable (not just the field).
            (b) E1 — CloudSpend emit block EXISTS (grounding_client.rs:915-960); confirm whether the
                PROD cloud-client ctor attaches .with_ssb_writer (the only real gap if so).
            (c) F1 — RuntimeRoutingPolicy::evaluate CHANGES the provider (not inert); confirm the
                architect/editor split is the real gap, not the routing itself.
  G-STEP 3. Update memory: SCHEMA_VERSION is 224 / TOTAL_VARIANTS 501 (was 220/497 — STALE). Fix the
            project memory line before any SSB-touching work.
  G-STEP 4. For each item promoted to build, run the planning-workflow PRE_GATE (walking-skeleton:
            producer + in-plan consumer + smoke trigger in the SAME unit) and write the phase-X-
            decisions + grounding doc. Then orchestrate per the H-1..H-10 protocol.
  G-STEP 5. Treat each DONE-GATE above as the wiring_contract_check.py --run-trigger target (W-1):
            off-agent artifact, missing-evidence = FAIL, frozen gate.

================================================================================
APPENDIX A — VERIFIED CURRENT-CODE STATE (this session, one Explore agent over .elai_cc)
================================================================================
 1. MCP bridge: mcp_bridge.rs is CLIENT-ONLY (serves mcp_* FROM the Claude-Code Python server; does
    NOT expose ELAI as a server). => D1 server inversion is net-new. [CLIENT]
 2. PreToolUse/VibeGate: HookEvent::PreToolUse (hooks.rs:19); VibeGate wired via
    InProcessHookDispatcher.with_vibegate_ssb() (runtime_setup.rs:778). [LIVE]
 3. build_production_hook_dispatcher(ssb, vibegate_seed_ok, project_root) at runtime_setup.rs:766. [LIVE]
 4. CloudSpend: `if let Some(ssb) = self.ssb_writer.as_ref()` emit block at grounding_client.rs:915-960
    EXISTS (CONTRADICTS H-9 "dark"); OPEN whether prod ctor attaches the writer. [PARTIAL-LIVE]
 5. Tool catalog 3-arm: build_tool_catalog (~266), convert (~970), dispatch (~1163) in
    production_agent_rpc.rs. [LIVE]
 6. SCHEMA_VERSION = 224 (ssb.rs:3010); TOTAL_VARIANTS = 501 (action_type_lock.rs:645). Memory 220/497 STALE. [LIVE]
 7. TestOutcomePayload HAS stdout_hash: Option<String> (ssb.rs:10494-10504, blake3, since SCHEMA 197).
    => F-4 gap is ENFORCEMENT, not the field. [LIVE-FIELD]
 8. FocusTunnel DIFF: run_diff_step returns FocusTunnelExecError::MissingProvider (orchestrator.rs:126);
    DIFF never emits in prod, always STRUCTURED. [STUB]
 9. Determinism: DriftComparator (determinism.rs:317) {seed, sink, audit_id, clock}; ClockProvider used
    throughout; EntropySource is a trait bound (agent_pool.rs:253); ELAI_SEED referenced widely. [LIVE]
10. Routing: RuntimeOrchestrator + PerRoleRuntime + RuntimeRoutingPolicy::evaluate (escalation_policy.rs:261)
    which DOES change the provider; RouteDecisionEvent (ssb.rs:8402) emitted via CloudModeRouter::route.
    => NOT observational (CONTRADICTS H-7 "doubly inert"). [LIVE]

NET: more trust + routing substrate is LIVE than F/G/H assumed (B1 field, E1 emit, F1 routing all
present). The dominant remaining work is (a) the gate trait seam + protocol exposure (A1/A2/D1 — net-new),
(b) the ENFORCEMENT/wiring of present-but-unbound pieces (B1 capture-slot, E1 ctor-attach, F1 split),
(c) the NUMBER (C1). The repositioning is cheaper than it looks because the gates already exist —
they just are not yet behind one seam or exposed over a protocol.

================================================================================
APPENDIX B — VERIFIED SANDBOX STATE (this session, one Explore agent over .elai_cc)
================================================================================
ELAI's execution isolation is its OWN multi-backend sandbox — NOT Docker/OCI (zero matches for
docker/containerd/oci/podman in the Rust code). "CPU/Docker sandbox" in an earlier draft was wrong.
 1. Linux primitive [LIVE]: LinuxSandbox::launch_agent (sandbox/mod.rs:1360) — seccomp-bpf + clone3
    namespaces (NEWNS/NEWPID/NEWNET/NEWUTS) + cap-drop; clone_cb (mod.rs:1431) enforces order
    NO_NEW_PRIVS -> CAPBSET_DROP -> setrlimit -> /proc hidepid=2 -> seccomp -> execvp.
 2. Firecracker [LIVE, more than planned]: sandbox/firecracker.rs — FirecrackerHotPool, per-(agent,role)
    CoW snapshots, ~28ms restore budget, LRU (max 64 / 24h), uniqueness-seed regen, cold-boot fallback,
    firecracker_binary_available() check. Linux-only (#[cfg(target_os="linux")]). gVisor fallback in
    gvisor.rs (runsc --platform=systrap, KVM-less).
 3. Windows [LIVE, best-effort]: WindowsAppContainerSandbox (mod.rs:607) — CreateAppContainerProfile
    primary, CreateJobObjectW + RestrictedToken fallback. Self-documented best-effort (NTFS junction
    bypass risk); Linux is the production security target.
 4. ENABLEMENT GAP [important for C1 Tier-2]: AgentPoolConfig::enable_sandbox defaults to
    cfg!(target_os="windows") (agent_pool.rs:152) => TRUE on Windows, FALSE on Linux/macOS until B-18B
    wires clone3+seccomp through launch_agent_process. So on a Linux exec host the sandbox is currently
    DEFAULT-OFF — flipping/wiring it is a real (bounded, no-GPU) C1 Tier-2 dependency.
 5. Untrusted-code execution for the number:
    - seccodeplt_runner.rs [LIVE]: STATIC gate eval over an embedded SeCodePLT corpus (no model, no
      sandbox, no exec) -> insecure_rate + gate_recall + InsecureRateMeasured SSB. The day-one number.
    - safedelta_runner.rs / bench_safedelta.rs [STUB, RunPod-pending]: DYNAMIC (live model + sandbox-
      executed + functional pass@1). Intended to use LinuxSandbox/FirecrackerHotPool when live.
NET: ELAI already has a production-grade, on-thesis "can't wreck" sandbox (Firecracker + seccomp). The
C1 dynamic number should EXECUTE generated code inside it (dogfooding the cant-wreck claim) on a Linux
CPU host — no Docker, no GPU. The only sandbox work for C1 is flipping enable_sandbox on Linux (B-18B).

End of ELAI-DESIGN-SELL-BY-2026-12.md.
