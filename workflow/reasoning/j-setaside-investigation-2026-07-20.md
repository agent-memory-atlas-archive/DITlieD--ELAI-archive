J-SET-ASIDE INVESTIGATION — 9 plans, four workstreams
task_id j-setaside-investigation, read-only, grounded against live .elai_cc HEAD
authored 2026-07-21 (NOW stamp); base evidence from conductor/opus_walk/evidence + walk-state escalation_queue

ORIENTATION
The grok-orchestrated ELAI backlog walk lands only .elai_cc product code via opus_land. Nine J-plans
sit in walk-state.json escalation_queue as SET_ASIDE. This report investigates each, grounds every
seam claim against real code (file:line), and gives a per-plan disposition. Every claim below was
either grep/read-verified against the live tree or is explicitly marked UNVERIFIED.

One material correction to the task framing up front (grounded): the prompt says J-23's machinery
"arch_gate::run_catalog / sarif_ingest::parse_sarif exists but no AUDITING-path hook consumes it."
That is INACCURATE. Grep for arch_gate | sarif_ingest | run_catalog | parse_sarif across .elai_cc =
ZERO real hits (the two hits are false positives: an "l" install-step token and a doctor step). The
S3 precheck agrees: "All J-23 symbols are DARK (greenfield, 0 hits on HEAD)." J-23 is greenfield in
BOTH the machinery AND the seam, not "built-but-unhooked". Corrected below in workstream C.

================================================================================
WORKSTREAM A — MIS-SCOPING AUDIT (the owner's top concern)
================================================================================

OWNER HYPOTHESIS: the model that authored J-3/J-4/J-5/J-7 (and maybe J-1) got confused and scoped
root-teikoku/.claude harness features into the ELAI (.elai_cc) product corpus.

FINDING (grounded): the hypothesis is REFUTED for the four harness plans. The authoring was NOT
confused about which root owns which feature. Each of J-3/J-4/J-5/J-7 explicitly labels itself
"harness-side only" in its STATUS line, and each names a SEPARATE, already-shipped ELAI-side twin.
The real defect is corpus PLACEMENT, not feature SCOPING: harness-lane plans were filed under the
J letter, which is the same corpus the ELAI codex/opus_land walk drains — so the walk keeps tripping
over plans that were never meant for the .elai_cc lane. The fix is corpus hygiene (segregate or tag
harness-lane plans out of the ELAI walk), NOT re-pointing them into .elai_cc (that would DUPLICATE
features that already shipped on the ELAI side).

Per-plan:

J-3 (harness undo + per-session file backups)
- Deliverables (plan lines 78-96, 131-135): rag/hooks/pre_edit_backup.py, .claude/tools/undo.py,
  .claude/settings.local.json (one hook line), workflow/reasoning/J-3-decisions.md. ZERO .elai_cc
  target. STATUS line 3: "harness-side only".
- The plan's OWN framing (lines 5, 43): "H-49/I-52/E-25 give ELAI undo ... J-3 ships the
  claude-code-workflow HALF of U3." So the ELAI-product undo was deliberately built elsewhere.
- GROUNDED that the ELAI twin already shipped: crates/orchestrator/src/session_dag.rs is the H-49
  undo-tree — session_dag.rs:291 "head is a session-wide pointer: the snap_id of the current
  undo-tree", :403/:477 file_manifest+head columns, :1740-1782 undo-tree HEAD API + children;
  ssb.rs:1259 "by elai undo/redo/jump CLI ops. Default-ON kill-switch ELAI_TIMETRAVEL_DISABLE";
  task_runner.rs:8221-8223 sets HEAD so `elai undo` finds it. So re-scoping J-3 into .elai_cc would
  DUPLICATE the shipped H-49 undo-tree.
- VERDICT: KEEP-AS-HARNESS-REMOVE from the ELAI walk corpus. If the owner still wants it, it is a
  teikoku-root harness task (the .claude/ backup hook), not .elai_cc product work.

J-4 (handoff file ledger)
- Deliverables (lines 60-67, 106-109): .claude/tools/handoff_ledger.py, one line in
  .claude/rules/handoff-protocol.md, workflow/reasoning/J-4-decisions.md. ZERO .elai_cc target.
  STATUS line 3: "harness-side only".
- ELAI twin named in-plan (line 22): "H-51 enforces the elai-side analog (digest DECISIONS/FILES
  must match journal FileWrite rows — off-agent, not recollection)." H-51 journal-match machinery is
  present in the tree (journal/FileWrite references across crates/orchestrator + dashboard-v2;
  grep-confirmed, not enumerated here). J-4 is deliberately the harness twin.
- VERDICT: KEEP-AS-HARNESS-REMOVE.

J-5 (litegame harness port)
- Deliverables (lines 52-56): copies of pre_edit_backup.py / undo.py / handoff_ledger.py into
  D:\.liteGAME_workflow\.claude\ + that root's settings + local-rag MCP registration. Targets a
  DIFFERENT project root entirely (liteGAME), not even teikoku. Depends on J-3+J-4+J-7. STATUS
  line 1-2: "PORT ... harness". A cross-root harness port has no place in the .elai_cc walk.
- VERDICT: KEEP-AS-HARNESS-REMOVE.

J-7 (ext-steal intake closeout)
- Deliverables (lines 112-117): .claude/rules/skill-import-policy.md, workflow/skill-imports.log,
  the dashboard-handoff skill checklist, 9 plan files (one pointer line each),
  workflow/reasoning/J-7-decisions.md. ZERO .elai_cc target. OWNER-GATED ("the whole unit is
  pending the user's yes", STATUS line 3-4).
- VERDICT: KEEP-AS-HARNESS-REMOVE. It is a harness policy/pointer chore gated on the owner.

J-1 (perception gateway + /see skill) — the ONE genuinely-mixed plan
- Deliverables (lines 84-90, 147-153): crates/orchestrator/src/perception/ (REAL .elai_cc core),
  ssb.rs +PerceptionScaffoldEvent, the image-dispatch seam, python sidecar endpoints (legs 2/4),
  AND .claude/skills/see/ (harness leg).
- GROUNDED that the ELAI core is ALREADY BUILT AND WIRED (this makes the escalation reason stale):
  * crates/orchestrator/src/perception/{mod,preprocess,router,scaffold}.rs all exist;
    mod.rs:55 perception_gateway_enabled(), mod.rs:79 struct PerceptionGateway.
  * ssb.rs:1502-1512 PerceptionScaffoldEvent LANDED at SCHEMA_VERSION 269 (J-1.T4, 2026-06-14),
    Kind arms Preprocessed/ScaffoldEmitted/RoutedToSpecialist/DegradedPassthrough.
  * production_agent_rpc.rs:1876 already calls perception::prompt_carries_image(&prompt), :1909
    MediaType::Image, :1976/:2007 "scaffold fires ONLY on image-bearing dispatches" — the image
    dispatch choke-point IS production_agent_rpc.rs and it is already carved as .elai_cc-only.
  * the harness /see leg ALSO already exists at .claude/skills/see/
    (SKILL.md, eval, scripts) — it is out of the .elai_cc lane and is already shipped.
- So J-1's escalation ("UNDESIGNED production seam ... pin the single image-dispatch choke-point")
  is answered by the existing code: the choke-point is production_agent_rpc.rs:1876, already wired.
  What actually remains for J-1 are legs 2/4 (the sidecar/VLM hybrid composer + probe-small-escalate
  router), which are GPU/sidecar-gated (re-triage under workstream B).
- VERDICT: RE-SCOPE-TO-ELAI is already effectively DONE for the core — the .elai_cc portion is in
  the right place and largely landed; the /see harness leg should be DROPPED from the ELAI-walk
  scope (it is harness and already shipped). The remaining GPU-gated legs 2/4 are the only open work.

MIS-SCOPING SWEEP (broader J corpus + whether other letters show it)
Method: for each J-*.md, count CREATE/MODIFY/produces lines targeting .claude//rag/hooks//skills/
vs .elai_cc/crates. Results (pure-harness = zero .elai_cc target):
- PURE HARNESS sitting in the ELAI J corpus (should not be in the ELAI walk): J-3, J-4, J-7 (all
  self-labeled), J-5 (liteGAME paths), PLUS two NOT in the escalation set — J-6
  (incremental_eval_skill, self-labeled harness, 0 .elai_cc target) and J-27
  (harness_protected_blocks_webcache_dispatch, 0 .elai_cc target). So the placement problem is at
  least SIX J-plans, not four.
- MIXED (legit .elai_cc plans that also drop a harness pointer/skill-card, NOT mis-scoped): J-1
  (perception core + /see), J-102, J-62, J-70, J-81, J-98. These are majority-.elai_cc; the harness
  line is an adjunct, not a mis-scope. No action beyond J-1's /see drop.
- Other letters: not exhaustively swept, but the mechanism (a "harness-side" WAVE inside a
  product-letter index) is a J-corpus artifact of the 2026-06-10 redesign that spawned J-1..J-7 as a
  mixed product+harness batch. RECON_NEEDED to confirm E/I letters are clean: run the same sweep
  (grep CREATE/MODIFY .claude/ vs .elai_cc) over E-*.md and I-*.md before declaring the corpus clean.

ANSWER TO THE OWNER'S REAL QUESTION: plan-authoring did NOT confuse the two roots. The harness plans
are correctly authored AS harness plans, each with a distinct already-shipped ELAI twin. The mistake
is that these harness-lane plans live in the same J corpus the ELAI codex/opus_land walk drains. The
remedy is corpus segregation (a harness-lane tag or a separate index the ELAI walk excludes), which
is a zero-code-risk hygiene fix, not a re-scope.

================================================================================
WORKSTREAM B — GPU + FREE-API RE-TRIAGE
================================================================================

WHERE THE "ZERO LOCAL GPU / API-ONLY" DIRECTIVE LIVES (grounded)
- File: /home/user/teikoku/conductor/OWNER_DIRECTIVE.md (top block, "ZERO LOCAL GPU / API-ONLY
  MODE", owner-issued 2026-07-19 JST, HIGHEST PRIORITY).
- Injection: conductor/opus_walk/opus_supervisor.py:121 OWNER_DIRECTIVE_PATH, :339 _owner_directive(),
  :455-459 directive_block — the file's verbatim content is appended to EVERY headless Opus boot
  prompt (capped OWNER_DIRECTIVE_MAX_CHARS=8000). Toggle shape: the owner edits or removes the FILE;
  NO restart needed (opus_supervisor.py:116-122).
- To UPDATE the directive after the owner's GPU-GO: edit conductor/OWNER_DIRECTIVE.md in place
  (prepend a new "LOCAL GPU RE-ENABLED / free-API real-inference permitted" block; the file's own
  convention is newest-block-wins with older text kept as provenance). No code change, no restart.
- Note the directive already sanctions the free-API substitute: "model-dependent work through
  verified FREE API workers only: glm-5-2 first, then swe-1-7, then swe-1-6" and "If a unit is
  hard-wired to a forbidden local endpoint and cannot safely use the existing API provider path,
  record GPU-PAUSED-BY-OWNER and move on."

ELAI'S REAL-INFERENCE PROVIDER SEAM (grounded — name the adapters)
- crates/providers/src/runtime/ is the provider abstraction. RuntimeBackend enum (22 backends as of
  I-49), resolve_backend() at mod.rs:148, create_runtime() factory at mod.rs:211.
- Real local-inference adapters that return live runtimes:
  * SidecarRuntime (runtime/sidecar.rs) -> the python sidecar server.py. THIS is the only path with
    the J-13 token-determinism substrate: server.py:1087 SeededGumbelProcessor (SGLang det-mode
    multinomial_with_seed) + server.py:1134 per-token logprob/entropy collector.
  * LlamaCppAdapter (runtime/llama_cpp.rs) -> llama.cpp Vulkan :8181 4B lane (Qwen3.5-4B Q4_K_M,
    RX 5700 XT). Routed at mod.rs:236. This lane serves generation but does NOT carry the
    SeededGumbelProcessor determinism (that lives in the python sidecar server.py, not llama-server).
  * AnthropicApiRuntime (runtime/anthropic_api.rs) -> hosted Anthropic-protocol endpoint (the
    ANTHROPIC_BASE_URL path). Plus ollama.rs, mlx.rs, sglang.rs, vllm.rs, gemini_cli.rs, codex_cli.rs
    and 15 others.
- HOW TO WIRE THE TEIKOKU FREE-API ROTATION AS REAL ELAI INFERENCE: the teikoku free-API rotation is
  the litellm proxy at 127.0.0.1:4000 (teikoku/free-proxy/, OpenAI-compat; providers groq/
  pollinations/cerebras/gemini/mistral/openrouter per workers.toml; adapter conductor/
  free_api_worker.py). To make ELAI's in-product model leg use it as REAL inference, the cheapest
  path is to point AnthropicApiRuntime (or add a thin OpenAI-compat adapter beside genai_adapter.rs)
  at the rotation endpoint via ANTHROPIC_BASE_URL/OPENAI_BASE_URL. This is genuine hosted inference,
  suitable for the e2e functional program and general generation. It is NOT a stub/fake.
  CAVEAT (load-bearing): the free-API rotation is UNSUITABLE for token-determinism verification
  (J-14 T1/T2). Hosted OpenAI-compat providers do not expose a reproducible token stream or sealed
  logprobs; the SeededGumbelProcessor path only exists in the python sidecar (SidecarRuntime ->
  server.py). Route determinism work to SidecarRuntime, generation work to the rotation.

J-14 RE-TRIAGE (grounded against rra.rs / rra_tiers.rs / the S1 assessment)
- The block is NOT purely "GPU paused". It is a SUBSTRATE gap plus a live-run need:
  * rra.rs:217-288 RunArtifact has NO token_stream_blake3, NO score_blake3, NO logprob_blake3, NO
    tokens/scores vectors (grep = 0). rra_tiers.rs:314-347 grade_tier returns Downgraded{T1/T2 ->
    ActionFaithful, reason "...not yet wired in verify leg"}. verify_artifact_with_objective
    (rra.rs:802-857) has no surface to blake3-compare a field no producer writes.
  * The smallest honest T1 slice is a PRODUCER change: add token_stream_blake3 to RunArtifact +
    thread it from the live seeded local-lane stream + extend grade_tier to compare it. T2 needs
    score_blake3/logprob_blake3 from provider_signals.GGSignals (only the local lane exposes
    logprobs; cloud mean_logprob = None).
- With the GPU pause lifted, the live-run requirement CAN now be met — but ONLY via a real-inference
  path that exposes the seeded token stream: SidecarRuntime -> python sidecar server.py
  (SeededGumbelProcessor), on a GPU (remote 3090 per REMOTE-SIDECAR.md preferred, or local). The
  llama.cpp :8181 4B lane does not have SeededGumbel; the free-API rotation cannot produce a token
  stream at all. So J-14 is real-inference-executable but NOT free-API-executable.
- The producer-field authoring (add the fields to RunArtifact + the verify-leg blake3 compare) is
  offline/API-only-authorable NOW behind a flag; only the live seeding run is GPU-gated.
- DISPOSITION: GPU-NOW-EXECUTABLE via SidecarRuntime seeded lane (NOT free-API). Sequence: (1) author
  a NEW plan in the current OPEN letter (not a J-14 amend, per new-plan-files-only) that adds
  token_stream_blake3 + score_blake3 to RunArtifact and extends grade_tier; (2) seed with one live
  `elai run --local --emit-rra` against the seeded python sidecar; (3) gate. UNVERIFIED: whether the
  remote 3090 sidecar is currently reachable (no probe performed, per directive) — the sidecar-
  protocol 3-step health check must pass at execute time.

SHOULD THE WALK DIRECTIVE BE UPDATED? Yes, if the owner has issued the GPU-GO. Edit
conductor/OWNER_DIRECTIVE.md (newest-block-wins) to (a) re-enable GPU-backed local lanes, and
(b) explicitly permit ELAI's in-product leg to use the teikoku free-API rotation for real hosted
inference (generation), while routing token-determinism work (J-14) to the seeded python sidecar.
No code or restart needed; the supervisor re-reads the file every boot.

================================================================================
WORKSTREAM C — SEAM DESIGNS (executable-blind)
================================================================================

J-23 (catalog-based deterministic audit) — SEAM DESIGNED
CORRECTION FIRST: the machinery does NOT pre-exist. arch_gate::run_catalog / sarif_ingest::parse_sarif
are 0-hit greenfield (verified + S3 precheck). J-23 must build both the catalog runner AND wire it.
GROUNDED anchors:
- crates/orchestrator/src/deterministic_audit_gate.rs:72 trait DeterministicAuditGate (pluggable),
  :110 struct DefaultDeterministicAuditGate, :151-167 impl ... evaluate() -> calls
  auditor.run_all(&changed).
- crates/orchestrator/src/deterministic_auditor.rs:76 enum CheckKind — nine variants: Build,
  VersionGate, FileSizeCensus, ImportCrossRef, LazyCode, Completeness, ModuleCoherence, Mutation
  (C-28, "Step 6 in run_all()"), Metamorphic (C-29, "Step 7"). The "frozen 7-step" is the 5 core +
  Mutation(6) + Metamorphic(7). CRUCIALLY: Mutation and Metamorphic were ADDED as later config-gated
  post-passes (they run only after the 5 core pass, gated by [testing.mutation]/[testing.metamorphic]
  in elai_cc.toml). So the enum has an established additive-post-pass extension pattern.
- Install site: crates/cli/src/live_cli.rs:3500 DefaultDeterministicAuditGate (gated by
  ELAI_DETERMINISTIC_AUDIT_GATE_ENABLE=1). Config plumbing:
  DeterministicAuditorConfig::from_profile_with_mutation reads [testing.mutation].
RECOMMENDED SEAM (canonical layer, does NOT break the frozen sequence):
- Add a CheckKind::CatalogReview variant as Step 8 in DeterministicAuditor::run_all(), config-gated
  by a NEW [review.catalog] block in elai_cc.toml (enabled=false default; tools=[...]; lang), exactly
  mirroring how Mutation(step6)/Metamorphic(step7) were added. It runs ONLY after the prior passes,
  emits advisory findings (a CatalogReviewEvent SSB row per finding), and is a no-op when disabled.
- Because DefaultDeterministicAuditGate::evaluate already calls run_all(), NO change to the gate
  struct or the install site is needed — the new step flows through the existing evaluate path. This
  is additive + default-off, so it CANNOT break the frozen 7-step gate (the 7 steps run unchanged;
  step 8 only fires when [review.catalog].enabled=true).
- The catalog runner (arch_gate::run_catalog) + SARIF parser (sarif_ingest::parse_sarif) are new
  modules the step calls; author them fresh (greenfield). The SSB add is the full 9-tuple lockstep
  from a re-grepped LIVE baseline (ssb.rs SCHEMA currently 281/590 per J-18 evidence; re-grep at
  edit, W-9). Wire-dark-allowlist row for the new default-off [review.catalog] flag (R9).
- REJECTED alternative (record it so it is not re-litigated): a new
  ArchGateDeterministicAuditGate impl of the trait, installed alongside the default — the S3 warns
  this runs external shell-outs on every AUDITING transition and invents a second AUDITING seam.
  The config-gated Step-8 is strictly less risky.
- FORK (executor-facing): if the owner prefers on-demand over in-gate, the alternative is a new
  `elai review --catalog {lang,target,tools}` subcommand (ReviewAction::Catalog at
  crates/cli/src/args.rs:2302, which today has only Cockpit). This matches the J-9 dogfood shape and
  never touches the FSM gate, but it is NOT the "extends AUDITING" read the plan names. PICK: the
  Step-8 CheckKind is the recommended canonical answer; the subcommand is the fallback if the owner
  wants zero FSM-path change.

J-1 (perception gateway / image-bearing dispatch) — SEAM ALREADY PINNED
- The single image-dispatch choke-point is production_agent_rpc.rs:1876 (perception::
  prompt_carries_image gate) through :2007 (scaffold fires only on image-bearing dispatches). It is
  already .elai_cc-only and already wired; the perception/ core module is built and the SSB event
  landed (SCHEMA 269). No "one seam vs per-agent" fork remains — it is one choke-point.
- Drop the harness /see leg from the ELAI-walk scope (.claude/skills/see already exists and is
  harness). The residual real work is legs 2/4 (sidecar VLM hybrid composer + probe-small-escalate
  router) — GPU/sidecar-gated, re-triage under workstream B alongside J-14.
- DISPOSITION: SEAM-DESIGNED:production_agent_rpc.rs:1876 (core landed; /see out-of-lane drop;
  legs2/4 GPU-gated).

J-80 (worktree pool lifecycle) — SEAM DESIGNED (ephemeral + SSB-lease), needs wargame refinement
GROUNDED anchors:
- crates/orchestrator/src/sandbox/branch_workspace.rs: trait BranchWorkspace (:134),
  WorktreePoolImpl (BTreeMap<branch_id,PathBuf> behind tokio::sync::Mutex), barrier_merge,
  escape_check, prune, WORKTREE_SCRATCH_DIR — all BUILT (I-2). It emits BranchWorkspaceEvent rows:
  branch_workspace.rs:48 uses BranchWorkspaceEventKind/Payload, :367 SsbPayload::BranchWorkspaceEvent
  { kind: Spawned }, :404 another emit.
- ssb.rs:1327-1330 BranchWorkspaceEvent SSB action (SCHEMA 246, I-2), Pattern-2 single-variant 4-arm
  BranchWorkspaceEventKind (Spawned representative).
- CONFIRMED: SSB CAN express a lease. The BranchWorkspaceEvent row already carries a Kind + payload;
  adding lease semantics = add Leased/Reused/Healed arms to BranchWorkspaceEventKind and
  owner={pid, started_at} + lease-expiry fields to BranchWorkspaceEventPayload, then re-derive
  PoolState as a projection over those rows (no external JSON file, GD-2).
- MIRROR PRECEDENT: the conductor already manages leases with TTL + owner-liveness (conductor uses
  ln_ttl_seconds leases and creates worktrees under conductor/elai-worktrees/; the lns lease pattern
  is the cross-platform substrate). This confirms the owner's steer: ephemeral + SSB-lease, not
  LockFileEx-primary. LockFileEx is Windows-only; the SSB journal seq is the cross-platform source of
  truth (use an OS lock only for the scratch-dir mutate, never as the lease authority).
- RECOMMENDED SEAM: a lifecycle layer ON WorktreePoolImpl in branch_workspace.rs — (1) PoolState
  projection re-derived from BranchWorkspaceEvent rows; (2) ephemeral owner {pid, started_at} lease
  (ownerAlive() = recorded started_at matches the live process, survives PID reuse); durable-lease
  only for a user-pinned worktree; (3) healState on acquire/list (drop dead-owner/vanished-path,
  never touch leased); (4) warm reset-and-reuse. CONSUMER: orchestrate/auto-orchestrate worker
  dispatch acquires from the pool; merge at the existing barrier_merge (KEEP unchanged, GD-1). This
  structurally removes the 2026-06-11 `git reset --hard x8` data-loss class and unblocks J-60.
- BLOCKER: the plan is a legacy 6-task file that FAILS plan_gate_check 7/10 (no wargame markers:
  P1/P3/P4/P5/P6/P7/P10). It MUST get a wargame-refinement pass (EXPECTED_OBSERVATION/ON_FAIL per
  move, FORK triggers, RECON_NEEDED, ABORT, REDTEAM) before an executor can run it blind.
- DISPOSITION: SEAM-DESIGNED:branch_workspace.rs-lifecycle-layer (ephemeral + SSB-lease); refine
  before execute.

================================================================================
WORKSTREAM D — J-18-LEG2 LSP CHANNEL
================================================================================

WHAT LSP IS (plain): the Language Server Protocol is a JSON-RPC 2.0 protocol (usually over a child
process's stdin/stdout) that lets a tool ask a running "language server" (rust-analyzer, pyright,
tsc, svelte-check) for semantic facts about code — errors/warnings (diagnostics), hover types,
go-to-definition, references. You spawn the server as a subprocess, send an `initialize` request,
then exchange JSON-RPC messages framed with a `Content-Length:` header; the server streams
`textDocument/publishDiagnostics` notifications (or answers `textDocument/diagnostic` pulls) with the
compiler's real errors. In ELAI the "lsp_channel" leg's purpose is to add a real compiler-diagnostics
signal to the HYBRID retrieval / evidence path: instead of guessing whether code is broken, ELAI asks
the actual language server and uses the diagnostics as grounded evidence (a retrieval channel and/or
an EvidenceGate input).

IS E-52 GENUINELY JUST TYPES? YES (grounded).
- crates/knowledge/src/diagnostic.rs is types-only. It ships DiagnosticSeverity, ProbeKind (5
  variants incl GenericLsp), Diagnostic, DiagnosticProbeOutcome, DiagnosticProbeRunStatus — envelope
  structs/enums plus a lsp_severity() scale helper (diagnostic.rs:54) and unit tests
  (diagnostic.rs:226+). There is NO DiagnosticProbe TRAIT, NO Command::new/spawn, NO stdio, NO
  JSON-RPC client. The file header itself (diagnostic.rs:11-20) states the trait + 3 concrete probes
  + 7 SSB variants + EvidenceGate wiring are "All deferred per E-52-decisions.md."
- So the J-18 plan's claim "compose, do not re-author the subprocess harness" is FALSE against live
  code — there is no harness to compose with. LEG2 is correctly SET_ASIDE as undesigned substrate.
- Nearest reusable prior-art (NOT a drop-in): crates/plugins/src/mcp_bridge.rs already builds
  JSON-RPC 2.0 envelopes (ln_envelope/post_ln, initialize + tools/list) — but it is an HTTP MCP
  client, not a stdio LSP client. The envelope-construction shape is reusable; the transport
  (Content-Length-framed stdio over a child process) is not present anywhere in .elai_cc (grep for
  lsp-server/lsp-types/publishDiagnostics = 0).

IMPLEMENTATION OUTLINE (enough for a follow-up executor)
Prerequisite plan (this is E-52.B, NOT J-18 — J-18 LEG2 should DEPEND on it, not inline it):
1. Add a `DiagnosticProbe` trait in crates/knowledge/src/diagnostic.rs:
   `fn run(&self, root: &Path, files: &[PathBuf]) -> DiagnosticProbeOutcome;` (the outcome type
   already exists).
2. Author a real stdio LSP client (new module, e.g. crates/knowledge/src/lsp_client.rs):
   - Spawn the server: `tokio::process::Command::new("rust-analyzer").stdin(Stdio::piped())
     .stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()`. Anti-slopsquat/verify the binary is on
     PATH; ToolMissing -> DiagnosticProbeRunStatus::ToolMissing (the type already models it).
   - Frame messages with the LSP `Content-Length: <n>\r\n\r\n<json>` header over stdin; read framed
     responses off stdout with an async length-prefixed reader.
   - Handshake: send `initialize` (rootUri, capabilities) -> await result -> send `initialized`
     notification. For diagnostics, either push mode (open documents via `textDocument/didOpen` and
     collect `textDocument/publishDiagnostics` notifications) or pull mode
     (`textDocument/diagnostic`, LSP 3.17+). rust-analyzer publishes; pyright publishes; prefer push
     with a bounded timeout -> TimedOut.
   - Map each LSP Diagnostic (range.start.line/character, severity 1..4, message, code) into the
     existing crates/knowledge Diagnostic struct (diagnostic.rs). Reuse the mcp_bridge.rs JSON-RPC
     envelope idiom for request construction.
   - Enforce a hard timeout + kill-on-drop (never leak a child), and a bounded output cap.
3. Concrete probes: RustAnalyzerProbe (+ PyrightProbe, TscProbe) implementing DiagnosticProbe over
   the stdio client, keyed by ProbeKind.
J-18 LEG2 proper (the retrieval/evidence consumer, once E-52.B lands):
4. Wire the DiagnosticProbeOutcome into the HYBRID path as a channel/evidence input. NOTE the RRF
   geometry correction from the S2 evidence: the shipped pipeline (pipeline.rs:581 query_with_tier)
   already has 5 ranker lists (bm25/code_vec/text_vec/catrag/symbol_graph_vec), the 5th
   (symbol_graph_vec, E-40) gated by ELAI_SYMBOL_GRAPH_5TH_RRF_ENABLE. J-18's LSP channel would be a
   6th list, not the "4th" the stale plan claims. Alternatively feed diagnostics to the EvidenceGate
   rather than the RRF cascade (cleaner: diagnostics are a boolean/typed evidence signal, not a
   ranked list).
5. SSB: the 7 deferred E-52 diagnostic action variants + any lsp_channel_queried variant, full
   9-tuple lockstep from the LIVE baseline (re-grep; ~281/590 per J-18 evidence). Default-off flag
   (ELAI_LSP_DIAGNOSTIC_PROBE_ENABLE already exists for E-52; do NOT conflate it with a retrieval-
   channel flag) + wire-dark-allowlist row.
- DISPOSITION: IMPLEMENT-OUTLINE-READY. The honest sequence is E-52.B (trait + stdio client +
  rust-analyzer probe) FIRST as its own plan, THEN J-18 LEG2 consumes it. Shipping LEG2 alone would
  require inlining ~200+ LOC of subprocess/LSP-JSON-RPC that the plan claims already exists (it does
  not).

================================================================================
NINE-PLAN DISPOSITION TABLE
================================================================================
J-1   SEAM-DESIGNED:production_agent_rpc.rs:1876 (core landed SCHEMA269; drop /see; legs2/4 GPU-gated)
J-3   KEEP-AS-HARNESS-REMOVE (twin H-49 undo-tree already in session_dag.rs; re-scope = duplicate)
J-4   KEEP-AS-HARNESS-REMOVE (twin H-51 journal-match already shipped)
J-5   KEEP-AS-HARNESS-REMOVE (liteGAME cross-root port; not even teikoku)
J-7   KEEP-AS-HARNESS-REMOVE (skill-import policy + 9 pointer lines; owner-gated)
J-14  GPU-NOW-EXECUTABLE:sidecar-seeded-lane (SidecarRuntime->server.py SeededGumbel; NOT free-API;
      producer field + live seeded run; new plan not a J-14 amend)
J-18-LEG2  IMPLEMENT-OUTLINE-READY (E-52 is genuinely types-only; needs E-52.B stdio LSP client first)
J-23  SEAM-DESIGNED:deterministic_auditor.rs::run_all Step-8 CatalogReview CheckKind, config-gated
      [review.catalog] default-off (does not break frozen 7-step); machinery is GREENFIELD not
      built-but-unhooked (task framing corrected)
J-80  SEAM-DESIGNED:branch_workspace.rs-lifecycle-layer (ephemeral + SSB-lease confirmed; refine
      before execute — plan_gate FAIL 7/10)

MIS-SCOPING SWEEP RESULT: 6 J-plans are pure-harness (0 .elai_cc target) sitting in the ELAI walk
corpus — J-3, J-4, J-5, J-7 (escalated) + J-6, J-27 (not escalated). Plus J-1's /see adjunct. The
authoring was NOT confused about roots; the plans are correctly harness-authored but mis-PLACED in
the product-lane corpus. Remedy = corpus segregation, not re-scope. RECON_NEEDED: sweep E/I letters
for the same placement artifact before declaring the whole corpus clean.

TOP-3 DO-FIRST
1. Corpus hygiene (cheapest, zero code risk, directly answers the owner): remove/tag J-3/J-4/J-5/J-7
   (and J-6/J-27) out of the ELAI codex/opus_land walk so it stops re-tripping on harness plans.
   Route them to a harness-lane if the owner still wants them.
2. J-23 Step-8 CatalogReview seam: canonical-layer, additive, default-off, mirrors the proven
   Mutation/Metamorphic post-pass pattern, cannot break the frozen 7-step. Clean executable-blind
   slice once the greenfield catalog runner + SARIF parser are authored.
3. J-80 ephemeral + SSB-lease: owner-approved direction, substrate confirmed (BranchWorkspaceEvent
   SSB + conductor lease precedent), kills the reset-hard data-loss class, unblocks J-60. Needs a
   wargame-refinement pass first (it fails plan_gate 7/10).

UNVERIFIED / CAVEATS
- Remote 3090 sidecar reachability for J-14 was NOT probed (directive forbids GPU probes); the
  sidecar-protocol health check must pass at execute time.
- E-52.B effort (LSP stdio client) estimated from the outline, not measured.
- The E/I-letter mis-placement sweep was not run (only the J corpus); flagged as RECON_NEEDED.
- H-51 journal-match twin confirmed by grep presence across crates, not by reading the full impl.
