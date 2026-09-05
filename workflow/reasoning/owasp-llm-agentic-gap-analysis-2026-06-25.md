OWASP LLM-TOP-10 (2025) + AGENTIC-TOP-10 (2026) GAP ANALYSIS vs ELAI
Date: 2026-06-25
Author: main LLM (spec-eval AS-SPECIFIED posture), in-band, grounded.
Purpose: feed a later plan-authoring session. NET-NEW work items in Section E are
  the deliverable; Section F is the do-not-re-propose ledger; Section G is honest
  residue to verify before authoring. Plans go in NEW files (never amend old),
  default target letter = highest existing (K) OR EXTENSION units on E-37/G-1/F-3.

INPUTS / GROUNDING
- User-provided transcript = OWASP Top 10 for LLM Applications (2025 revision).
- OWASP canonical, WebFetch+WebSearch verified 2026-06-25:
  LLM 2025 list (genai.owasp.org/resource/owasp-top-10-for-llm-applications-2025):
    LLM01 Prompt Injection | LLM02 Sensitive Information Disclosure |
    LLM03 Supply Chain | LLM04 Data and Model Poisoning |
    LLM05 Improper Output Handling | LLM06 Excessive Agency |
    LLM07 System Prompt Leakage [NEW 2025] | LLM08 Vector and Embedding
    Weaknesses [NEW 2025] | LLM09 Misinformation | LLM10 Unbounded Consumption.
    (the owasp.org project landing page auto-serves the STALE v1.1/2023 list; the
    2025 list lives on genai.owasp.org — do not cite v1.1.)
  AGENTIC list (genai.owasp.org "Top 10 for Agentic Applications", Dec-2025):
    ASI01 Agent Goal Hijack | ASI02 Tool Misuse | ASI03 Identity & Privilege
    Abuse | ASI04 Agentic Supply Chain (runtime MCP/A2A poisoning) | ASI05
    Unexpected Code Execution | ASI06 Memory & Context Poisoning | ASI07 Insecure
    Inter-Agent Communication | ASI08 Cascading Failures | ASI09 Human-Agent Trust
    Exploitation | ASI10 Rogue Agents. (also: AIVSS scoring, agentic taxonomy 1.1.)
- ELAI side: ELAI-FEATURE-MAP.md read 100%; live-code verified via codegraph-elai
  (file:line cited). No OWASP string exists anywhere in the repo (grep-confirmed) —
  this is the first OWASP mapping.

FRAMING (do not lose this)
ELAI is a LOCAL-FIRST AUTONOMOUS CODING AGENT/HARNESS, not a model-as-a-service.
So the rubric splits: the AGENTIC threats (and the agentic half of the LLM list:
injection, excessive agency, output handling, supply chain, poisoning,
misinformation) hit ELAI dead-on and are what the trust spine is built for. The
SERVICE-PROVIDER threats (model inversion/extraction LLM02-tail, training-data PII
LLM02, model theft) map WEAKLY today because ELAI does not train on proprietary
data and is single-user/local — BUT the multi-seat path (J-49/J-59) RE-OPENS them,
and per-seat isolation is PLANNED only. Sequence isolation BEFORE multi-seat.

================================================================================
SECTION B — ELAI vs OWASP LLM TOP 10 (2025), design verdict AS-SPECIFIED
================================================================================
LLM01 Prompt Injection — STRONG design, PARTIAL binding. Structural defense (the
  architecturally-correct posture; OWASP itself says prose system-prompt guards
  fail): G-1 taint (crates/safety/src/provenance.rs) + capability_gate
  (crates/safety/src/capability_gate.rs) + F-3 CaMeL tool-filter + I-47 verifier-
  input hardening + I-1 supervisor (generator cannot reach gate code). The real
  binding limitation is in Section D. See WI-1.
LLM02 Sensitive Information Disclosure — COVERED for the agent threat: G-9 write-
  time secret+PII redaction (blake3-keyed, TTL, local-only), F-1 VibeGate secret
  GREP, G-3 egress IP-pin/allowlist, I-50/J-30 auth material never journaled.
  Model-inversion/extraction = N/A (no proprietary training, single-user, own
  sidecar) UNTIL multi-seat (J-59).
LLM03 Supply Chain — HEADLINE STRENGTH: F-9 hallucinated-dep write-time gate
  (existence+age+provenance incl cargo; Spracklen 19.7%), anti-slopsquatting hard
  rule, E-24 blake3 MCP schema-pin + G-4 LIVE rug-pull re-pin, E-43 skillspector
  whole-dir scan, I-18 AI-BOM 5-layer hash, G-8 in-toto attestation, G-10 model+
  fingerprint pin. Directly answers the "2M unverified HF models" point.
LLM04 Data & Model Poisoning — STRONG: RAG (F-5 staleness-quarantine cascade,
  F-12 NLI, F-8 conflict detector); memory (E-5 MINJA, J-21 PLANNED semantic-
  validation+write-quarantine, H-31 firewall+hard-filter superseded, H-32
  validate-citation-before-use); model (G-10 drift canary). ELAI never trains on
  RAG/memory => "poison snowballs into weights" structurally absent.
LLM05 Improper Output Handling — STRONG: ELAI output IS code => F-1 VibeGate code-
  vuln AST/GREP on write, F-2 CWE rules (incl CWE-95 eval-injection, CWE-798),
  SAFEDELTA bench target, I-46 bash interceptor, output never blind-trusted (F-4).
LLM06 Excessive Agency — STRONGEST AXIS (least-privilege by construction): Tier-C
  OS-process isolation (agents get NO net/LLM/tools, seccomp), G-1 capability gate
  before destructive sinks, B-22 autonomous gate (Concept=HIGH always human, 50-
  step circuit breaker), E-30 destructive-verb argv allowlist+trust dialog, I-1
  directive-supervisor plane (supervisor is the ONLY side-effect executor, holds
  credentials), orchestrate tripwires.
LLM07 System Prompt Leakage [NEW 2025] — GOOD by design-removal: I-1 supervisor
  holds credentials OUTSIDE the generator context; auth never journaled (I-50);
  G-9 redaction. No secret in the prompt to leak.
LLM08 Vector & Embedding Weaknesses [NEW 2025] — COVERED: G-1 taint on retrieved
  content, F-5 staleness quarantine, J-21 retrieval-time authorization+provenance,
  H-31 hard-filter superseded, vector circuit-breakers+content-hash cache. No
  online learning => embeddings can't bleed into weights.
LLM09 Misinformation — STRONGEST AXIS / product thesis: 6+ grounding-verification
  methods, GRC, D-16 conformal refusal, H-39 abstention scoring, H-53 citation
  pre-binding (ghost citations structurally impossible), F-4 cant-lie (blake3
  stdout-hash; auditor can't fabricate passed=true), D-13 cross-family adversary,
  G-10 drift canary, SAFEDELTA.
LLM10 Unbounded Consumption — COVERED (denial-of-wallet specifically): B-27/C-5
  hard cost ceiling (BLOCKS not warns: RouteBlocked/WarnLimitReached wired), E-39
  cost-economics ceiling, H-38 cost-confirm gate + pod-cost tripwire, J-28/J-29
  token ledger, J-49 per-seat quota, anti-spiral + 50-step breaker + max_restarts
  =3 + serial-depth budgets. Model-extraction-as-DoS = local-N/A.

================================================================================
SECTION C — ELAI vs OWASP AGENTIC TOP 10 (2026), design verdict AS-SPECIFIED
================================================================================
(this rubric fits ELAI BETTER than the LLM list — it rewards supervisor/least-
privilege, evidence-bound approval, frozen gates, execution-grounded self-improve.)
ASI01 Agent Goal Hijack — PARTIAL (same as LLM01; binding gap in Section D, WI-1).
ASI02 Tool Misuse — STRONG: E-30 argv allowlist (check_argv Allow/Deny/Require-
  HumanApproval, p99<1ms benched), capability gate, C-35 tool gating, Tier-C
  tool-less by default.
ASI03 Identity & Privilege Abuse — STRONG (multi-seat isolation PLANNED): I-1
  supervisor holds creds + generator process-separated, I-50 auth broker (token
  never journaled), SO_PEERCRED on IPC, J-49/J-59 per-seat.
ASI04 Agentic Supply Chain (runtime MCP/A2A poisoning) — STRONG, ELAI AHEAD:
  E-24 blake3 MCP schema-pin + G-4 LIVE rug-pull re-pin on tools/list_changed with
  description-diff = the EXACT ASI04 control; F-9 dep gate, E-43 skill-audit, C-42
  zero-trust MCP authz. CONFIRMED STRENGTH, not a work item.
ASI05 Unexpected Code Execution — STRONG: F-1 VibeGate, F-2 CWE (CWE-95 eval),
  E-30 allowlist, I-46 bash interceptor, Tier-C sandbox, capability gate.
ASI06 Memory & Context Poisoning — STRONG design (J-21 PLANNED): E-5 MINJA, J-21
  write-quarantine + semantic validation, H-31 firewall + hard-filter superseded,
  H-32 validate-citation-before-use, F-5 staleness. NOTE: memory provenance =
  TrustLevel::Recallable, which by DESIGN does NOT block destructive sinks
  (capability_gate.rs:96; HITL at a higher layer). Confirm that is the intended
  posture under ASI06 (recalled poison could drive a sink). See WI-1 note.
ASI07 Insecure Inter-Agent Communication — STRONG: Tier-C typed 15-msg postcard
  IPC, 10MB frame cap, SO_PEERCRED peer-auth, seccomp; SSB single-writer total
  order. F-8 inter-agent BSP half DEFERRED.
ASI08 Cascading Failures — STRONG: anti-spiral blake3 3-hash stop, Erlang
  max_restarts=3, 50-step circuit breaker, 50-event bubble tripwire, J-16 blast-
  radius gate (PLANNED), cross-domain canary (block if any family regresses >5%).
ASI09 Human-Agent Trust Exploitation — STRONG, GENUINE DIFFERENTIATOR (receipts-
  not-prose): F-4 cant-lie, H-53 citation pre-binding, G-5 reviewer cockpit per-
  hunk proof tokens + DSR, K-3 closed-set evidence renderer, D-13 cross-family
  adversary. Operator approves on evidence, not on the agent's confident narrative.
ASI10 Rogue Agents — STRONG design (much DARK/PLANNED): DGM verification filters
  (H-43 execution-grounded fitness + H-44 behavioral-trap detection replacing the
  keyword-grep placeholder), tripwire stack auto-revert, F-10 security-monotonicity,
  L1 HardVeto never-overridable, R8 frozen gates (a run cannot edit its own
  evaluator), I-1 supervisor.

STANDOUT STRENGTHS (record, do not re-author): ASI04 G-4 live re-pin; ASI09
evidence-bound approval; ASI10 frozen gates + execution-grounded self-improvement;
LLM03 supply-chain stack; LLM09 grounding/cant-lie spine.

================================================================================
SECTION D — LIVE-CODE TRACE: indirect-injection taint->sink SOUNDNESS (grounded,
  codegraph-elai, file:line; CORRECTS the chat's first-pass overstatement)
================================================================================
SUBSTRATE (real, well-designed, fail-closed):
- crates/safety/src/provenance.rs — Provenance{source,trust} + Provenanced<T>.
  Trusted: user_prompt/system. Untrusted: from_web / from_tool_output / from_file /
  from_sandbox. from_file_screened + path_is_trusted_source = FAIL-CLOSED (empty
  project_root => nothing trusted; any `..` component => disqualified).
- crates/safety/src/capability_gate.rs — allow_tool_with_origin rejects any
  DESTRUCTIVE_TOOLS call (bash/shell/fs_write/write_file/rm/delete_file/db_modify/
  sql_execute/curl/web_fetch/http_post/canvas_select) whose argv carries
  TrustLevel::Untrusted; structured Reject{reason,citation,offending_arg_index};
  gate-error => fail-CLOSED. Recallable does NOT block (by design).

WIRING (CORRECTED — all four ingest legs ARE wired in production; the chat's first
pass wrongly said file/sandbox were under-wired by checking bare from_file):
- WEB ingest tagged: from_web <- web_research_orchestrator::run_node_loop
  (web_research_orchestrator.rs:278).
- MCP-tool-output tagged: from_tool_output <- mcp_tool_bridge::call_tool
  (mcp_tool_bridge.rs:248).
- FILE ingest tagged (path-screened): from_file_screened <- translate_tool_request
  (production_agent_rpc.rs:2321). (bare from_file has only a self/test caller — the
  PRODUCTION path uses the screened constructor. file leg IS wired.)
- SANDBOX stdout tagged: from_sandbox <- execute_code_act (codeact_sandbox.rs:206).
- tainted_context() has 16 callers incl all four ingest sites + canvas
  run_select_gate + BOTH production gates + a real contract test
  (crates/contracts/tests/g1_taint_propagation_contract.rs: untrusted_web_ingest_
  blocks_git_commit, untrusted_edit_content_blocks_edit_sink, ten_benign_flows_
  zero_false_blocks FP-guard, not-global isolation). So this is BETTER tested than
  codegraph's "no covering tests" heuristic implied.
- TWO production gate seams (default-ON, kill-switchable, emit SSB on block):
  enforce_taint_sink_gate (production_agent_rpc.rs:224) = per-argv capability gate;
  enforce_trifecta_quarantine_gate (production_agent_rpc.rs:334) = G-7 lethal-
  trifecta SESSION-level gate (g7_confidence_quarantine_contract.rs).

THE ONE GENUINE RESIDUAL DESIGN GAP (the code says it itself, GD-5 honest-scope at
production_agent_rpc.rs ~234): the per-argv binding is a VERBATIM SUBSTRING MATCH
(ctx.lookup_for_argv(part), >= MIN_TAINT_MATCH_LEN bytes, same-process). It does
NOT catch:
  (1) TRANSFORMED/SUMMARIZED/PARAPHRASED untrusted content — i.e. the LLM reads a
      poisoned doc, paraphrases the malicious instruction, and the paraphrase flows
      to a destructive sink. The LLM is a transformation between ingest and sink;
      substring identity cannot survive it. THIS IS THE DOMINANT indirect-injection
      path (LLM01/ASI01) and the per-argv gate misses it.
  (2) CROSS-PROCESS sinks (same-process registry; ELAI's agents are OS processes).
      NOTE taint_flow.rs:472 process_global_slot_is_shared suggests a partial
      shared slot exists — confirm exact cross-process coverage before scoping.
  (3) non-McpToolRegistry MCP transports.
The Provenanced<T> TYPE exists (the strong CaMeL binding the doc-comment aspires
to) but is NOT threaded end-to-end through the LLM-output->argv path; the wired
mechanism is the substring registry MVP. The G-7 trifecta SESSION gate is the
existing backstop for exactly case (1) — lean on it.

NET: G-1/G-7 taint defense is real, default-ON, fail-closed, four ingest legs
wired, contract-tested. Architecture correct (structural taint + capability denial,
not prose). The residual is the BINDING (substring, not transformation-surviving) +
cross-process coverage. Consistent with feature-map E-37 PARTIAL + "B-18
allowed_paths not wired into tool dispatch".

================================================================================
SECTION E — NET-NEW WORK ITEMS (the deliverable; vertical-slice / W-1 / GD shaped)
================================================================================
WI-1 [KEYSTONE] TRANSFORMATION-SURVIVING TAINT BINDING (LLM01/ASI01/ASI06)
  Problem: per-argv substring gate misses paraphrased untrusted->sink flow.
  Approach (recommend hybrid, NOT full CaMeL interpreter which is research-grade /
    HD-IC tension): (a) PRIMARY = strengthen the already-wired G-7 lethal-trifecta
    SESSION gate (enforce_trifecta_quarantine_gate, production_agent_rpc.rs:334) as
    the load-bearing backstop: if untrusted content entered context AND a
    destructive sink is reached AND an egress/exfil capability is present => HITL/
    quarantine, REGARDLESS of per-value match; (b) SECONDARY = a taint-WINDOW
    heuristic: mark argv produced by the model AFTER it consumed untrusted context
    in the same turn as carrying derived-untrusted provenance (span/turn window),
    so transformation does not launder trust; (c) keep the substring gate as the
    cheap exact-match fast path. Full CaMeL control/data-flow interpreter = note as
    the ceiling, defer.
  Seams: production_agent_rpc.rs:224 (per-argv) + :334 (trifecta) + safety/
    taint_flow.rs; provenance.rs Provenanced<T>.
  Composes: G-1, G-7, E-37, F-3 CaMeL, I-1 supervisor. Grounding: arXiv:2503.18813
    CaMeL §3.1; the in-code GD-5 honest-scope comment.
  Done-gate (W-1, MEASURED not asserted): an AgentDojo/SAFEDELTA fixture where a
    PARAPHRASED injection (substring-evading) reaches a destructive sink is BLOCKED
    by the window/trifecta gate; ten-benign-flows FP guard stays green (extend
    g1_taint_propagation_contract.rs with a paraphrase case). SSB: reuse the taint-
    flow / TaintFlow rows (net 0..+1).

WI-2 CROSS-PROCESS TAINT PROPAGATION (ASI07/ASI01)
  Problem: tainted_context() is same-process; Tier-C agents are OS processes.
  Build: propagate Provenance across the Tier-C postcard IPC boundary so cross-
    process destructive sinks see upstream taint (verify the existing
    process_global_slot first, taint_flow.rs:472 — scope to the CONFIRMED gap only,
    do not rebuild a working shared slot).
  Seams: crates/ipc-types postcard + production_agent_rpc::translate_tool_request.
  Composes: Tier-C 4A / G-1. Done-gate: a cross-process untrusted->sink fixture
    blocks; same-process behavior unchanged.

WI-3 E-37 PERIMETER COMPLETION (LLM01/LLM05) — ACTIVATION of planned E-37
  Build the recorded-ABSENT E-37 pieces: output_sanitizer.rs + output_classifier.rs
    + egress allowlist + AgentDojo CI job (feature-map 1.8 PROMPT INJECTION DEFENSE
    [PARTIAL]). Composes G-1 (data-plane) + G-3 (egress). Done-gate: AgentDojo CI
    runs in the suite with a recorded baseline; egress allowlist blocks a seeded
    exfil host.

WI-4 [THE NUMBER] RUN SAFEDELTA / AGENTDOJO INJECTION MEASUREMENT (LLM01/LLM05/
    ASI01) — not new code, the missing RUN
  G-6 SAFEDELTA is BENCH-PENDING (0 safety_delta_measured rows), gated by F-2b
    (RunPod). Without it, ALL of the above is unproven (GD-3: gate-ON-vs-OFF
    ablation, never a leaderboard rank). This is K-plan's whole thesis (A-1..A-4 +
    K-1 dossier). Done-gate: one SAFEDELTA ablation run on SusVibes/BaxBench/
    SeCodePLT/AgentDojo with a recorded gate-ON-vs-OFF delta. Sequence FIRST or in
    parallel — it is the evidence that judges WI-1..WI-3.

WI-5 OWASP CONTROL-MAPPING DOC (positioning / procurement) — E-23/K class
  This analysis -> a docs/strategy/ mapping of ELAI controls to LLM01-10 + ASI01-10,
  feeding K-6 compliance export + K-7 appsec/insurance brief (procurement language:
  the buyer asks "OWASP LLM Top 10 coverage?"). Pure doc, NO SSB, KD-1 claims-policy
  lint applies (every coverage claim tagged measured-vs-design). Composes K-6/K-7/
  G-8/G-9.

PRIORITY for the authoring session: WI-1 (keystone) + WI-4 (the proving number)
first; WI-3 is cheap activation; WI-2 needs the process_global_slot verify; WI-5 is
positioning that rides K. Likely shape: ONE new EXTENSION plan file on E-37/G-1
(WI-1+WI-2+WI-3 are one security-hardening subsystem) + a K-rider for WI-4/WI-5.
Do NOT amend E-37/G-1/F-3 in place (new-plan-files-only rule).

================================================================================
SECTION F — ALREADY-COVERED / DO-NOT-RE-PROPOSE (so the authoring session skips)
================================================================================
- ASI04 runtime MCP poisoning: COVERED by G-4 live re-pin + E-24 pin + C-42 authz.
  Confirmed strength, not work.
- LLM03 supply chain / hallucinated deps: COVERED (F-9 + anti-slopsquat + E-43 +
  I-18 + G-8). Not work.
- LLM06/ASI02/ASI05 excessive-agency / tool-misuse / NL-RCE: COVERED (Tier-C +
  E-30 + capability gate + F-1/F-2 + I-1). Not work.
- LLM09/ASI09 misinformation / trust-exploitation: COVERED (grounding spine + F-4
  cant-lie + H-53 + G-5 + D-13). Not work.
- ASI10 rogue agents: COVERED by DGM verification (H-43/H-44) + frozen gates (R8) +
  L1 HardVeto + tripwire stack + supervisor. Not work (much DARK/PLANNED — that is
  READINESS, not a design gap; do not author "build it" as if absent).
- Model inversion / extraction / theft (LLM02-tail / old LLM10): N/A single-user
  local; do NOT author a defense — it only becomes relevant under multi-seat, where
  J-59 per-seat isolation already owns it. Sequence J-59 BEFORE multi-seat instead.
- Full CaMeL control/data-flow interpreter: DEFER (research-grade; HD/IC tension).
  WI-1 takes the trifecta+window hybrid, not the interpreter.
- DO NOT re-introduce prose-system-prompt guards as a primary defense (OWASP + ELAI
  agree they fail; structural taint is the posture).

================================================================================
SECTION G — OPEN / UNVERIFIED RESIDUE (verify before authoring)
================================================================================
- process_global_slot (taint_flow.rs:472 process_global_slot_is_shared): confirm
  EXACT cross-process taint coverage before scoping WI-2 (may be partially solved).
- ASI06 Recallable-does-not-block: confirm intended posture — recalled poisoned
  memory CAN drive a destructive sink today (capability_gate.rs:96). Decide if WI-1
  window logic should treat Recallable-after-untrusted-write as Untrusted.
- SAFEDELTA/AgentDojo are RunPod-gated (F-2b); WI-4 needs the 2x3090/RunPod box.
- E-37 absent files (output_sanitizer/classifier/egress/AgentDojo-CI) status is
  from the feature map (06-07b static); re-grep at authoring (W-7).
- Status flags throughout are index-derived; confirm DARK-vs-WIRED against live
  wire-dark-allowlist.toml + a fresh elai.map regen before final scoping.
- Did NOT full-read the 10 outdated plan-summary indexes (feature-map consolidates
  them; security grep found nothing OWASP-relevant beyond the map). K-SUMMARY read
  in full (GTM/sequencing, no new security capability).

================================================================================
SECTION H — CONFIG-INGRESS ADMISSION CONTROL (added 2026-06-26; user ask +
  agentlint Leporis14/agentlint grounding; maps OWASP LLM03/LLM04/LLM01 +
  Agentic ASI04/ASI06)
================================================================================
USER ASK (verbatim intent): everything a user can ADD or MODIFY that ends up in
the model's hands — .md skills/rules/agent-defs/CLAUDE.md/AGENTS.md, MCP server
configs — must pass an ELAI safety check at FIRST-TOUCH and again AFTER any
modification; it must NOT be possible to add MCPs/skills/rules out-of-band of ELAI
(everything routes through ELAI's own safety pipes). This is ADMISSION CONTROL on
the agent's own configuration + instruction surface.

AGENTLINT (grounded 2026-06-26, MIT, Python, pip leporis-agentlint): a NARROW
MCP-config-only static scanner (mcp.json / claude_desktop_config.json / .mcp.json).
Checks: hardcoded secrets in env, broad-FS-path args (/home,/etc,/var), missing
auth env vars, absent approval fields (requireApproval/humanInLoop). Risk 1-10, CI
fails >=7. Does NOT cover CLAUDE.md/AGENTS.md/skills/rules; does NOT do injection.
=> STEAL = its MCP check-RULE-SET as DATA only. DO NOT port the pip dep (ELAI is
Rust; reuse F-1 secret-GREP + VibeGate AST; ship checks as rules). Record as
idea/rule-set steal.

ELAI CURRENT STATE (live-code verified):
- MCP: STRONG, ahead of agentlint on identity. crates/plugins/src/mcp/pins.rs
  McpServerPin{exact_version,expected_pubkey_blake3,auth_required} +
  McpPinPolicy{enforce_pin_list,refuse_version_float,refuse_unauthenticated} ->
  enforce_pin Allow/Refused; + E-24 blake3 schema-pin + G-4 live re-pin on
  tools/list_changed + C-42 DENY-ALL authz. GAP = agentlint-style CONTENT checks
  (dangerous FS-path args, hardcoded secret in env block, approval-field presence)
  are NOT in the MCP admission path.
- Skills: SOFT gate. E-43 skill-import-policy + skillspector /skill-audit +
  workflow/skill-imports.log; the policy doc itself says PreToolUse HARD
  enforcement is DEFERRED -> today it's code-review-time, not a runtime block.
- Rules / agents / CLAUDE.md / AGENTS.md: NOTHING. ConfigLoader (core/config.rs:205)
  merges config; J-52 injects path-keyed rules into the context Rules slot; J-62 is
  a pre-WRITE env-card gate. NONE scans a new/modified instruction .md for embedded
  injection/secrets/capability-escalation BEFORE it loads into the model context.
  TOFU-trusted instruction surface = real ASI04/ASI06/LLM01 vector.

ARCHITECTURAL KEY (the "no out-of-band bypass" answer): enforce at LOAD time, not
ADD time. A raw file edit on disk cannot be prevented; but a content-hash-keyed
admission manifest (blake3 TOFU on first-touch, re-scan on hash change) that
FAIL-CLOSED QUARANTINES any artifact lacking a matching ALLOWED row means an
un-admitted/modified artifact NEVER reaches context or the dispatcher. Mediated
`elai mcp add`/`elai skill add` UX is convenience; the load-time hash gate is the
enforcement.

WI-6 [CONFIG-INGRESS ADMISSION GATE] (vertical-slice; the unified capability)
  WI-6a MCP add-scan content checks (agentlint rule-set as DATA): fold dangerous-
    FS-path-arg + hardcoded-secret-in-env (reuse F-1 secret GREP) + approval-field-
    presence + risk-score INTO the EXISTING mcp pins.rs / mcp_pin_cmd admission
    path (which already does version-float/unauth/pubkey). Composes E-24/G-4/C-42/
    F-1. No new dep.
  WI-6b Skills hard-enforce: promote E-43 skill-import-policy from SOFT (code-
    review) to HARD PreToolUse refusal — a skill with no workflow/skill-imports.log
    ALLOWED row matching its CURRENT content-hash is refused at reference/load. The
    policy doc names this as its own deferred piece.
  WI-6c Instruction-surface admission (THE GAP, keystone of WI-6): blake3-keyed
    admission manifest over .claude/rules/*.md + .claude/agents/*.md + CLAUDE.md +
    AGENTS.md + .claude/skills/*/SKILL.md; at LOAD into the context-budget Rules/
    PhaseSpec slot (and at agent dispatch), if the artifact is new OR its hash
    changed since last admission, run the admission scanner and FAIL-CLOSED
    quarantine. Scanner = DETERMINISTIC legs (secrets via F-1 GREP; dangerous-verb/
    capability-escalation directive patterns; known injection phrases "ignore
    previous/forget instructions"; zero-width/homoglyph/encoded-payload detection —
    a concrete deterministic win on obfuscated .md injection; frontmatter/schema
    validation) + ADVISORY semantic leg (cross-family critic "is this directive
    trying to escalate", TD-3, NOT the gate — same LLM-hard limit as WI-1). Seam:
    ConfigLoader (core/config.rs:205) + context_budget Rules-slot fill + agent
    dispatch; NATURAL HOOK = J-52 PathRuleInjector (already intercepts path-keyed
    rule injection). +ConfigAdmissionEvent (Pattern-2: Admitted/Quarantined/
    ReadmissionRequired). Composes E-43/J-52/J-62/F-1/context_budget; OBSERVE-ONLY
    first (GD-4).
  WI-6d No-out-of-band-bypass = the LOAD-TIME hash check of 6a/6b/6c is the
    enforcement (catches raw external edits via changed-hash-without-admission-row).
    Optional mediated `elai mcp add`/`elai skill add` UX rides on top (convenience,
    not the gate).
  Grounding: agentlint MIT MCP-scanner (rule-set only, no dep); ELAI's own E-43
    skill-import-policy precedent; OWASP LLM03/LLM04/LLM01 + ASI04/ASI06.
  Done-gate (W-1, MEASURED): a seeded malicious config set — (i) .claude/rules/x.md
    with an "ignore all gates" directive + a zero-width-obfuscated exfil line, (ii)
    an mcp.json with a /etc FS-path arg + hardcoded key, (iii) an unscanned skill —
    is QUARANTINED at load and never enters context/dispatch; a clean config loads
    unchanged (ten-benign FP guard); modifying an already-admitted file flips it
    back to quarantine until re-admitted (on-modify re-scan proven). SSB: reuse
    skill_system_event / mcp pin rows where possible, else +ConfigAdmissionEvent
    (net 0..+1).
  HONEST RESIDUE: semantic injection-in-instruction detection is LLM-hard (paraphrase
    limit, same as WI-1) -> deterministic legs fail-closed, semantic advisory only.
    Verify whether settings.json hook-registration itself needs admission (a poisoned
    hook command is a sink too). Confirm J-52 PathRuleInjector is the right intercept
    seam at authoring (W-7 re-grep).
  PRIORITY: WI-6c is the keystone (the absent surface); 6a is cheap (extend an
    existing gate); 6b is policy-promotion (soft->hard). Likely shape: ONE new
    EXTENSION plan file on E-43/E-24 (config-ingress admission), NOT an in-place
    amend (new-plan-files-only). Sits next to WI-1..WI-5; both are the "ingress
    governance" theme (WI-1 = untrusted DATA into sinks; WI-6 = untrusted CONFIG
    into the model's instruction surface).

END owasp-llm-agentic-gap-analysis-2026-06-25.md (Section H appended 2026-06-26)
