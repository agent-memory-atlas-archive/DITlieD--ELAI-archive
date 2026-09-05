STEAL ANALYSIS — omnigent / google-agents-cli / agent-control-plane
Date 2026-06-25. Lens: spec-eval AS-SPECIFIED + paper-to-plan phase 4.
Inputs: 3 sonnet paper-ingestor (mode=source) runs, interface maps at
  workflow/papers/{omnigent,agents-cli,agent-control-plane}-interface.md.
RAG was DOWN this session (reranker crash); maps are on disk, NOT RAG-indexed.
Outcome chosen by user: STEAL-ANALYSIS ONLY (no plan files) + LEDGER RIDERS.

META FINDING
After grounding against ELAI's actual security/sandbox code (not the feature
map one-liners), NONE of the 3 repos yields a clean NEW plan. Two are rejects
(Google-ecosystem-niche; Pi-driver strictly simpler than orchestrate). The one
with a real cluster (omnigent security/sandbox) downgrades to 2-3 NARROW riders
once measured against the real tree, because the feature map UNDERSTATED ELAI's
sandbox surface. This re-confirms the standing pattern: ELAI's improvement lever
is activation-not-invention of its own DARK/PARTIAL backlog; external harnesses
keep validating it rather than extending it.

GROUNDING CORRECTION (why the omnigent agent overstated)
The agent had only the dispatch summary, which leaned on the map line
"TIER-C ... process-level only with no network isolation". The real tree:
- crates/orchestrator/src/sandbox/mod.rs: SandboxBackend {Firecracker, Gvisor,
  AppContainer (Windows), Microsandbox} + select_backend degradation. Windows +
  Linux already covered; macOS Seatbelt explicitly absent.
- crates/orchestrator/src/sandbox/researcher_net.rs: veth pair + iptables OUTPUT
  allowlist, allowed_hosts -> resolved IPs pinned, REJECT-not-DROP (Linux Tier-2).
- crates/safety/src/blast_radius.rs::check_egress_host: NUL/control reject then
  resolved-IP subset of pinned allow-set, cross-platform Tier-1 (std::net only),
  fail-closed. check_fs_target: post-expansion worktree containment.
- secret handling: providers/src/runtime/secret.rs (SecretString),
  orchestrator/src/auth_broker_emit.rs (I-50, token never journaled),
  orchestrator/src/vibegate_secrets.rs + privacy/secret_scan.rs (write+output scan),
  J-30 BYOK (0600 key file). Tier-C agents have NO net at all (stronger than allowlisting).

=====================================================================
REPO 1 — omnigent (omnigent-ai/omnigent)
  Apache-2.0 (Databricks). HEAD 2026-06-25 (today). 1009 test files. reliability HIGH.
  Targets claude-opus-4-7/4-8, claude-sonnet-4-6. NOT stale.
  Interface map: workflow/papers/omnigent-interface.md
=====================================================================
C1 cross-platform sandbox (bwrap / macOS sandbox-exec / Windows Job Object)
  ELAI subsystem: TIER-C / SandboxBackend (sandbox/mod.rs).
  VERDICT: ALREADY-SHIPPED for Linux+Windows (Firecracker/Gvisor/Microsandbox/
  AppContainer). RIDER(macOS): macOS Seatbelt/sandbox-exec backend is genuinely
  absent in-tree (blast_radius.rs comment confirms). Minor EXTENSION of the
  SandboxBackend enum; LOW priority (ELAI dev = Windows + remote Linux 3090;
  macOS surface is only E-51 edge-inference). Record, do not author.

C2 L7 egress MITM proxy + per-path allowlist DSL (GET api.github.com/**)
  ELAI subsystem: G-3 BLAST-RADIUS RUNTIME GATES (safety/blast_radius.rs) +
  researcher_net.rs.
  VERDICT: EXTENSION (finer granularity), MARGINAL value. ELAI egress is L3/L4
  (host -> resolved-IP subset of pinned set), cross-platform Tier-1 + Linux veth
  Tier-2. omnigent adds L7 (method+path) via an in-process CA MITM proxy. The
  path-granularity is rarely the bottleneck for ELAI's threat model (host/IP +
  G-1 taint + capability_gate already gate the destructive sinks), and a MITM CA
  is heavy machinery. RIDER on G-3 only; do not author. The DNS-safe host
  validator (reject NUL/%/CRLF/brackets/IPv6-literal) is already ELAI's
  check_egress_host behavior (NUL/control reject) — confirmation, not a steal.

C3 secretless credential proxy (real secret stays in parent; injected on egress;
   oa_cred_<rand> placeholder for clients that need a local cred; 403 if placeholder
   sent to any other host)
  ELAI subsystem: J-30 BYOK + I-50 AUTH BROKER + J-63 LOCAL-LANE CLAUDECEPTION
  (providers/runtime/secret.rs, auth_broker_emit.rs).
  VERDICT: NEW pattern but NARROW surface. ELAI's architecture keeps secrets in
  the orchestrator and gives Tier-C agents NO network, so the "agent needs a
  cred but must not hold it" case only exists on the network-capable lanes:
  the J-63 generated-app broker and the vendor-CLI bridges. There the
  egress-injection + per-host placeholder-scoping pattern is a clean hardening.
  RIDER on J-30/I-50/J-63 (record as a design option: "secrets never enter the
  sandboxed process; egress proxy injects, scopes the placeholder per-host").
  Do not author as its own plan; it folds into J-63's security spine if/when J-63
  is built.

C4 CEL fail-closed policy hook (PreToolUse/PostToolUse -> EvaluationRequest POST,
   30s retry, fail-CLOSED to DENY on server-unreachable; cel-expr terminates)
  ELAI subsystem: J-71 CEDAR POLICY-AS-CODE SPIKE; autonomous_gate.rs; R1/R2
  missing=FAIL doctrine.
  VERDICT: ALREADY-PLANNED + ALREADY-DOCTRINE. CEL is the sibling of Cedar (J-71
  spike, may be declined); fail-closed-to-DENY = ELAI's missing=FAIL gate
  doctrine. PromptPolicy sub-executor-LLM-as-evaluator = ALREADY-REJECTED
  (TD-3, LLM never adjudicates a gate). CEL-vs-Cedar is one data point for the
  J-71 spike, not a steal.

C5 conversation fork with XML sentinel injection (<omnigent_fork_history>...,
   sentinel-neutralize user text, clear preamble only after successful inject)
  ELAI subsystem: H-49 REVERSIBLE EDITS / SESSION DAG fork; I-11 PABU handoff.
  VERDICT: RIDER on H-49. The sentinel-neutralization + clear-after-success
  hardening is a small robustness detail; the fork-history-as-injected-message
  is H-49 / PABU territory. Record, do not author.

C6 per-turn brain-model selection + shadow/advise mode (cost_control.plan JSON v3)
  VERDICT: ALREADY-SHIPPED. = H-7 CALIBRATED ROUTING + E-31 tier matrix +
  RouteDecisionEvent shadow mode (ELAI_ROUTE_DECISION_SSB_ENABLE observe-only).

C7 GOTCHA (intel, not a build): the Antigravity (agy 1.0.8) PreToolUse hook does
   NOT fire before tool execution — enforcement is post-hoc/observational.
  RIDER on J-64 / AGENT-CLI BRIDGE: do not assume PreToolUse fires across all
  vendor CLIs; the ELAI hook-on-vendor-CLI integration must treat some vendor
  hooks as post-hoc audit only. Useful for the CLI-bridge + ACP transport design.

OMNIGENT NET: 0 new plans. Riders: macOS-Seatbelt-backend (LOW, SandboxBackend),
  L7-path-egress (MARGINAL, G-3), secretless-egress-cred-injection (J-30/I-50/J-63),
  fork-sentinel-hardening (H-49), vendor-CLI-PreToolUse-post-hoc intel (J-64).
  DO-NOT-COPY: nothing flagged.

=====================================================================
REPO 2 — google/agents-cli
  Google. v0.5.1, last commit 2026-06-22. niche. reliability HIGH.
  = Google Cloud ADK DevOps lifecycle CLI (scaffold -> eval -> deploy on Vertex AI).
  Interface map: workflow/papers/agents-cli-interface.md
=====================================================================
- GEPA multi-backend prompt optimizer (GEPA/ProTeGi/BayesianSearch/MetaPrompt/
  PromptWizard menu): ALREADY-REJECTED. ELAI E-8 GEPA canonical; determinism
  ledger already says "dspy REJECTED as framework swap, GEPA canonical, no newer
  optimizer found". The 6-backend menu is a reference list only.
- A2A transport (Google agent2agent, AgentCard at /.well-known/agent.json):
  RIDER (weak). ELAI already ships ACP transport (J-64) + MCP bridge + gemini_cli
  bridge. A2A is a competing Google-specific protocol; at most a future ADDITIVE
  transport sibling of J-64 IF Google-ecosystem demand appears. Not a unit.
- 3-target cloud deploy (agent_runtime/cloud_run/gke) + per-target flag rejection:
  REJECT (off-wedge). ELAI is not a cloud-deploy tool (B-37 is inference routing).
- ADK per-case agent-cache-flush (asyncio.Lock cross-loop), npx skills install +
  12h drift check, experiment-flag NamedTuple registry: REJECT/covered. Flag
  registry is weaker than wire-dark-allowlist.toml (type/owner/expiry/promotion/
  smoke + R9); npx path banned by bun-not-node; cache-flush is ADK-internal.
AGENTS-CLI NET: 0 new plans. REJECT. One weak rider: A2A as a possible future
  J-64-sibling transport, Google-ecosystem-gated. Same class as the prior
  goose-topology / deer-flow "Google-niche, strictly simpler, no steal" verdicts.

=====================================================================
REPO 3 — coleam00/agent-control-plane
  MIT. last commit 2026-06-15. reliability MEDIUM (read, not run).
  = Bun/TS harness driving the external Pi coding agent in a loop + Neon Postgres
  + React dashboard. orchestrated mode (LLM decides tasks/round) + ralph mode.
  Interface map: workflow/papers/agent-control-plane-interface.md
=====================================================================
The agent flagged 5 "not covered"; 4 of 5 are covered or stronger in ELAI once
the reject ledger (which the agent could not see) is applied:
- LLM-dynamic per-round task decomposition: ALREADY-REJECTED in spirit. This is
  the Arbor "the loop is a prompt" pattern ELAI deliberately did NOT port
  (plan-first deterministic DAG + AutonomousGate/BubbleUp/ReplanConcept). Same
  verdict class as deer-flow/RecursiveMAS "ELAI is plan-first, ahead on verification".
- awaiting_approval mid-loop human gate (park at max_iterations, resume w/ extra
  budget): ALREADY-SHIPPED = H-37 PLAN-APPROVAL + H-38 COST-CONFIRM +
  AutonomousGate/InteractiveGate + B-22 50-baby-step circuit breaker.
- per-role tool-surface restriction at subprocess call time (--tools): ALREADY-
  SHIPPED, ELAI STRONGER = phase-4A AgentDispatchConfig.tool_filter (whitelist
  enforced at the IPC layer) + F-3 Phase1 Tool Filter (locks researcher tools
  before first untrusted byte). The agent's "no runtime restriction" claim is wrong.
- ACP_FAKE_PI control-token deterministic integration harness ([steps=N][fail]
  [fanout=K] tokens in the goal string, on-disk counters, full integration suite
  without a real LLM): capability ALREADY-COVERED = phase-3I DST (madsim +
  MockTestRunner + HTTP-VCR + IPC fault injection: packet loss/latency/SIGKILL).
  RIDER (testing-DX only): the control-token-in-goal-string ergonomic is a nice
  authoring pattern for ELAI's sidecar/agent integration tests; record, not a unit.
- parent_run_id row-level lineage (worker run records orchestrator run UUID):
  the ONE marginal item. ELAI has SESSION-DAG parent_id + agent-lifecycle rows +
  J-2 session_fork + J-10 provenance-hash lineage, so spawn lineage exists, but
  an EXPLICIT orchestrator-run -> worker-run edge on the SSB agent-dispatch row
  may be a small gap. VERIFY-RIDER (codegraph the agent_pool dispatch SSB row for
  a parent field); not a plan on its own.
AGENT-CONTROL-PLANE NET: 0 new plans. CONFIRMATION/REJECT. Riders: control-token
  test-fake (testing DX), parent_run_id explicit-edge (VERIFY). "Pi-driver,
  strictly simpler than orchestrate" — same class as goose/deer-flow/RecursiveMAS.

=====================================================================
OUTCOME
=====================================================================
NEW plans authored: 0 (user chose steal-analysis-only).
Recorded riders (feature-map Section 4 ledger upsert, this date):
  omnigent: macOS-Seatbelt SandboxBackend (LOW); L7 path-egress on G-3 (MARGINAL);
    secretless egress credential injection on J-30/I-50/J-63 (NARROW, fold into J-63);
    fork-sentinel-hardening on H-49; vendor-CLI-PreToolUse-post-hoc intel on J-64.
  agents-cli: REJECT; A2A future-J-64-sibling transport (Google-gated) rider.
  agent-control-plane: REJECT/CONFIRMATION; control-token test-fake (testing DX);
    parent_run_id explicit-edge VERIFY-rider.
DO-NOT-COPY: none surfaced.
NEXT (if user opts in later): (1) the secretless-egress-cred pattern is the single
  highest-value survivor and naturally folds into J-63 authoring; (2) one quick
  codegraph verify of the agent-dispatch SSB row for parent lineage.
