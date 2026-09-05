STEAL REASONING — Kun Chen (kunchenguid) agentic-engineering toolchain
Date: 2026-06-24
Orchestrator: main session (opus), 1 sonnet source-ingest agent per 3 artifacts, sequential.
Purpose: a later session authors J-plan files from the verdicts below. This file
  is the reasoning artifact, NOT a plan. Every candidate carries a verdict
  (NEW / EXTENSION(plan-id) / ALREADY-SHIPPED / ALREADY-REJECTED / REJECT) checked
  against ELAI-FEATURE-MAP.md Section 4 ledger + live codegraph grounding.

SOURCE CORPUS (all cited paths real, on disk):
  Video transcript: C:\Users\user\Downloads\Hi everyone, welcome to this video.txt
    (Kun, ex-Meta/Atlassian principal eng; the connective narrative for all 8 links)
  Clones (depth-1): F:\.ELAI_workflow\docs\.source\{treehouse,gnhf,firstmate}
    (+ lavish-axi, no-mistakes, vercel-labs-skills, OpenSuperWhisper, axi.md — agents 2/3)
  Interface maps (verbatim extraction, MEDIUM reliability):
    workflow/papers/treehouse-interface.md
    workflow/papers/gnhf-interface.md
    workflow/papers/firstmate-interface.md
  Handoff: workflow/handoff/kunchen-agentic-tools.md
  ELAI map regenerated this session (elai.map / elai_graph.json, 30,334 fns / 252,595 calls).

WHAT THE TOOLCHAIN IS (one line each, from the transcript):
  treehouse  — warm git-worktree POOL manager (fresh on demand, reuse idle, free on close).
  gnhf       — "good night have fun": long-running agent LOOP to a stop condition with caps.
  firstmate  — meta-orchestrator: decompose -> tmux tabs -> treehouse worktrees -> agents -> no-mistakes -> PRs.
  lavish-axi — rich-HTML-artifact planning surface in the project's own design system (agent 2).
  no-mistakes— end-of-task validation pipeline -> evidence -> PR -> babysit-to-merge (agent 2).
  axi.md     — 10 agent-ergonomic tool-design principles; AXI beats MCP on cost+success (agent 2).
  vercel skills — cross-agent `npx/bunx skills` installer (agent 3).
  OpenSuperWhisper — local Whisper voice input + custom-vocab priming (agent 3).

================================================================================
CLUSTER 1 — treehouse + gnhf + firstmate  (crew / loop / supervision)
================================================================================

--------------------------------------------------------------------------------
C1.1  treehouse worktree POOL LIFECYCLE  ->  VERDICT: EXTENSION(I-2) [strong, user-priority-adjacent]
--------------------------------------------------------------------------------
GROUNDING (code-vs-map drift caught): ELAI-FEATURE-MAP lists I-2 COW WORKSPACE
ISOLATION as PLANNED ("overlayfs/reflink Linux, warm git-worktree pool Win/macOS,
merge at deterministic barrier"). LIVE CODE CONTRADICTS the PLANNED flag — the
barrier/isolation HALF is already built and contract-tested:
  crates/orchestrator/src/sandbox/branch_workspace.rs
    - trait BranchWorkspace (spawn / escape_check / barrier_merge), :134
    - struct WorktreePoolImpl (BTreeMap<branch_id,PathBuf> behind tokio::sync::Mutex), :175
    - barrier_merge: GD-2 SSB-seq-ordered, last-seq-wins, ConflictEscalated Tier-1 halt, :432
    - escape_check: delegates containment to elai_safety::blast_radius::check_fs_target, :389
    - prune: Windows handle-race bounded retry (3x) then warn-and-leak (non-fatal), :270
    - WORKTREE_SCRATCH_DIR ".wt-scratch" kept OUTSIDE .elai_cc (git-toplevel trap), :53
  Contract: crates/contracts/tests/i2_cow_workspace_contract.rs ; callers in bsp/engine.rs (3).
  => Update the map: I-2 is PARTIAL, not PLANNED (the BSP barrier-merge leg shipped;
     the overlayfs/reflink + warm-reuse-pool legs are the residue).

WHAT TREEHOUSE HAS THAT ELAI'S WorktreePoolImpl DOES NOT (the genuine steal — these
are the POOL-LIFECYCLE/PERSISTENCE half, orthogonal to ELAI's merge half):
  1. PERSISTENT cross-process state: treehouse-state.json + OS file lock
     (WithStateLock; lock_unix Flock / lock_windows LockFileEx). ELAI's pool is an
     in-memory BTreeMap that dies with the process — no cross-session reuse, no
     resume. (treehouse pool.go acquire @ pool.go:71-165; state.go:10-27.)
  2. PID-RECYCLING-SAFE owner reservation: OwnerPID + OwnerStartedAt, ownerAlive()
     checks process StartedAt == recorded (survives PID reuse). ELAI has nothing
     here — it spawns a fresh worktree per branch_id, never reserves+reuses.
  3. DURABLE LEASE (process-independent, survives zero live processes, explicit
     `treehouse return` to clear) vs EPHEMERAL owner-reservation (self-heals on
     process death) — two orthogonal modes (Leased XOR OwnerPID==0 invariant).
  4. healState() self-heal on every acquire/list (drops dead-owner + vanished-path
     entries, never touches leased). ELAI prunes only at teardown.
  5. WARM REUSE: reset idle worktree to default branch and hand it back (detached
     HEAD, fetch-on-acquire) — a warm pool, not spawn-per-task.
  6. Cross-platform proven on Windows (gopsutil v4 process detection + LockFileEx).
     ELAI's prune already concedes a Windows handle-race KNOWN LIMIT; treehouse's
     dirty/in-use check + healState is the more robust handling of the same race.

WHY THIS MATTERS TO ELAI (two real use-cases the warm-reuse pool unlocks):
  (a) ORCHESTRATE/AUTO-ORCHESTRATE WORKER ISOLATION. The 2026-06-11 `git reset
      --hard x8` data-loss incident (the reason agent-dispatch-preamble.md R-1
      exists) happened because dispatched workers SHARE the repo tree. A warm
      per-worker worktree from a persistent pool structurally removes that class
      (each worker writes its own root; merge at the barrier). ELAI's auto-orchestrate
      decomposes-in-vertical-slices but workers still share CWD. This is arguably
      the single highest-leverage adoption.
  (b) J-60 CANVAS LAYERED OBSERVABILITY explicitly defers "per-agent worktree TAB ...
      worktree diff from the H-49 op-stream UNTIL I-2 COW worktrees ship." A
      persistent, addressable, reusable worktree pool (treehouse model) is the
      substrate J-60 names as its blocker. firstmate is literally this pattern
      (tmux-tab-per-agent over treehouse worktrees).

FRAMING FOR THE J-PLAN: a unit EXTENDING I-2 that adds the POOL-LIFECYCLE layer to
WorktreePoolImpl: persistent state file (single-writer, the SSB is the natural ELAI
store — emit BranchWorkspaceEvent rows are already there; a pool-state projection
re-derivable from the journal per GD-2, never a parallel JSON store like treehouse),
PID-recycle-safe reservation OR an SSB-lease analog, healState self-heal, warm reset-
and-reuse. KEEP ELAI's barrier_merge + escape_check (already stronger than treehouse,
which has no merge). Cross-platform is mandatory (ELAI runs win32; treehouse proves
the LockFileEx/gopsutil path). DO NOT copy treehouse's separate JSON state file —
ELAI's GD-2 rule says re-derive from the SSB journal, never an external store.

--------------------------------------------------------------------------------
C1.2  gnhf long-running LOOP + orthogonal CAPS  ->  VERDICT: mostly CONFIRMATION; small EXTENSION(auto-orchestrate / E-39)
--------------------------------------------------------------------------------
GROUNDING: ELAI's AutonomousGate (crates/orchestrator/src/autonomous_gate.rs:1198)
is an EVIDENCE GATE on FSM transitions, NOT an iteration loop: AutoAdvanceEvaluator
runs 6 evidence checks (knowledge-readiness / zero-ungrounded / tests-fail-correctly
/ test-flipped / zero-blocks / dkg+changelog) per transition (:760-824);
circuit_breaker_limit/count (:1415) = the 50-baby-step count breaker; GateMode +
substitute_model/next_fallback_model (:1435/:1448). The long-running-LOOP concern
lives in the orchestrate/auto-orchestrate SKILLS (prose: T1-T4 tripwires; "30 agent
dispatches without check-in"; stall = BLOCKED-twice / run-state-unchanged-3x).

gnhf's contributions vs ELAI, item by item:
  - THREE ORTHOGONAL CAPS (gnhf orchestrator.ts:74-79 RunLimits): maxIterations
    (pre-gate), maxTokens (pre-gate AND **mid-iteration abort via onUsage ->
    AbortController**), stopWhen (NL). The mid-iteration token-cap abort is the one
    real refinement ELAI lacks at runtime: ELAI's COST ECONOMICS HARD CEILING (E-39,
    DARK, "blocks-not-warns wallet-loop closure") is the analog but has no
    task_runner caller. auto-orchestrate's autonomy envelope counts DISPATCHES, not
    mid-run tokens. => small EXTENSION: a deterministic mid-loop token/iteration/
    wallclock cap that aborts the current step (rides E-39 + auto-orchestrate's
    envelope), not a new subsystem.
  - stopWhen = the AGENT sets should_fully_stop (an LLM judging its own stop).
    VERDICT: DO-NOT-COPY-AS-GATE (TD-3: LLM is advisory, never the load-bearing
    gate; same class as the MiMoCode goal/judge LLM stop-gate already REJECTED in
    Section 4). A DETERMINISTIC stop predicate (observable-outcome / evidence-check)
    is the ELAI-shaped version.
  - notes.md cross-iteration memory (append summary+changes+learnings, inject next
    iteration; orchestrator.ts notes-injection): = SESSION DAG + H-51 handoff digest
    + memory-bank. ALREADY-SHIPPED / STRONGER (ELAI journals to SSB, re-derivable).
  - commit-repair loop (pendingCommitFailure: preserve uncommitted work, inject a
    repair prompt next iteration instead of discarding): a clean failure-recovery
    idea. ELAI's analog is fixer agent + bubble-up. RIDER, not a unit.
  - `git reset --hard` rollback between iterations (orchestrator.ts resetHard on
    failure): DO-NOT-COPY EXHIBIT — this is literally the 2026-06-11 incident
    mechanism. ELAI uses H-49 CAS blake3+zstd snapshots + RV-1 snapshot-before-
    restore (lossless, never clobbers). gnhf's reset-hard is acceptable only because
    it runs in an isolated worktree at the orchestrator (not worker) level; ELAI's
    reversibility doctrine is the stronger substitute.
  - PermanentAgentError (e.g. Claude low-credit) -> abort immediately: maps to I-7
    cascade / cost ceiling. CONFIRMATION.
  - reset consecutiveErrors on agent-reported-fail (no backoff) vs increment on
    hard-throw (exponential backoff): a tidy failure-taxonomy for backoff. RIDER to
    bubble-up.md retry budgets (already has 50-event cap + fixer-3x stall).
NET: no new plan-family-grade unit; one small EXTENSION (mid-loop deterministic
cost/iteration cap on E-39 + auto-orchestrate). The gnhf "run a CLI agent in a
commit-per-iteration loop" shape itself is OFF-WEDGE vs ELAI's FSM-gated execution
(same NO-STEAL class as goose-topology / the /go comparison the transcript itself
makes). pnpm/Node-20 (NOT bun) noted for any reimpl per bun-not-node.md.

--------------------------------------------------------------------------------
C1.3  firstmate event-driven SUPERVISION  ->  VERDICT: CONFIRMATION + 1 EXTENSION(auto-orchestrate) + riders; secondmates OFF-WEDGE
--------------------------------------------------------------------------------
GROUNDING: ELAI orchestrate waits on deps via SLEEP-POLL (tools/dep-wait.ps1, every
60s, mcp_session_list). auto-orchestrate detects stall via run-state-unchanged-3x /
BLOCKED-twice. Neither has process-level wedge detection or a durable wake-queue.

firstmate mechanisms vs ELAI:
  - EVENT-DRIVEN WAKER (fm-watch.sh:130-264): blocks until signal (status-file
    size:mtime signature, NOT mtime-only) / stale (pane-hash identical 2 cycles, no
    busy-signature) / check (per-task poll) / heartbeat (2^streak backoff to 7200s);
    + signal coalescing (FM_SIGNAL_GRACE=30s) + DURABLE wake-queue (survives watcher
    process death) + singleton lock. ELAI's poll is cruder.
    => EXTENSION(auto-orchestrate / orchestrate supervision): the two genuinely-new
       bits are (i) PROCESS-LEVEL WEDGE DETECTION (output unchanged N cycles = stuck;
       ELAI's anti-spiral is reasoning-level blake3-3-hash, orchestrate T4 is
       sweep-fail-3x/fixer-3x — neither catches a silently-hung agent producing no
       output) and (ii) DURABLE WAKE-QUEUE (resume the supervision loop across a
       supervisor crash; composes with SESSION_HANDOFF / PreCompact). Both are
       deterministic, no LLM. Frame as a supervision-hardening rider to
       auto-orchestrate, NOT a new subsystem.
  - TURN-END HOOKS per harness (claude Stop-hook touches a file; codex -c notify;
    opencode session.idle; pi -e ext) as an event-driven completion signal
    (fm-spawn.sh:343-388): ELAI's HOOK SYSTEM already owns Stop/PreCompact. ELAI's
    own Agent-tool dispatch returns on completion (harness-notified), so this only
    matters when ELAI drives EXTERNAL vendor CLIs in a fleet (B-37 AGENT-CLI BRIDGE /
    J-64 ACP). RIDER to B-37/J-64: a turn-end completion signal on cli_common spawn.
  - SECONDMATES (persistent domain supervisors, isolated FM_HOME, hierarchical
    autonomous report-tree; AGENTS.md + fm-home-seed.sh transactional provisioning):
    VERDICT: OFF-WEDGE / likely-REJECT. Autonomous domain supervisors with their own
    homes sit in tension with ELAI's single-FSM-gate-per-session trust spine
    (Section 4: choreography / autonomous-message-bus agents REJECTED — "the central
    FSM gate is the trust spine; choreography dissolves it"). Same verdict class as
    goose multi-agent topology NO-STEAL. Surface the tension; do NOT author. (ELAI's
    HIERARCHICAL PLAN MGMT phase-3G is plan-hierarchy, not agent-supervisor-hierarchy.)
  - AGENTS.md-IS-the-orchestrator (prompt-as-system) + project delivery modes
    (no-mistakes / direct-PR / local-only / +yolo per-project autonomy): ELAI's
    orchestrate/auto-orchestrate skills + GateMode + per-project config are the
    direct analog. CONFIRMATION of ELAI's prompt-as-orchestrator design.
  - `--dangerously-skip-permissions` (claude) / `--dangerously-bypass-approvals-and-
    sandbox` (codex) in the launch templates (fm-spawn.sh:108-130): DO-NOT-COPY
    EXHIBIT. firstmate runs crewmates with ALL guardrails off; ELAI's entire wedge is
    the opposite (Tier-C OS-process isolation, VibeGate fail-closed write gate,
    capability gates, destructive-verb argv allowlist E-30). This is the precise
    anti-pattern ELAI's trust spine exists to refuse.
  - Platform: firstmate is macOS/Linux ONLY (tmux, POSIX). ELAI is win32-primary —
    any supervision-hardening steal must be cross-platform (treehouse, not firstmate,
    is the cross-platform reference in this cluster).

CLUSTER-1 SUMMARY (for the J-plan author):
  NEW/EXTENSION worth authoring:
    - EXTENSION(I-2) [strongest]: warm-reuse + persistent + self-healing worktree
      POOL LIFECYCLE on WorktreePoolImpl, SSB-journal-backed (never an external
      JSON store), cross-platform; unlocks orchestrate-worker isolation (kills the
      reset-hard incident class) + the J-60 per-agent-worktree blocker.
    - EXTENSION(auto-orchestrate): supervision hardening = process-level WEDGE
      DETECTION + DURABLE WAKE-QUEUE (deterministic), optional event-driven waker
      over sleep-poll.
    - small EXTENSION(E-39 + auto-orchestrate): mid-loop deterministic cost/
      iteration/wallclock cap that aborts the current step.
  RIDERS (record, do not author standalone): gnhf commit-repair + backoff taxonomy
    -> bubble-up.md; turn-end completion signal -> B-37/J-64.
  DO-NOT-COPY EXHIBITS: gnhf `git reset --hard` rollback (use H-49/RV-1); firstmate
    `--dangerously-*` flags (violates the trust spine); gnhf LLM-self-judged stopWhen
    (TD-3); treehouse separate JSON state file (GD-2 re-derive from journal).
  OFF-WEDGE: firstmate secondmates (hierarchical autonomous supervisors vs the
    central-FSM-gate trust spine).
  MAP FIX: I-2 status PLANNED -> PARTIAL (barrier-merge/escape leg shipped + contract).

================================================================================
CLUSTER 2 — lavish-axi + no-mistakes + axi  (plan-artifact / validation / tool-ergonomics)
================================================================================
Maps: workflow/papers/{lavish-axi,no-mistakes,axi}-interface.md
Clones: docs/.source/{lavish-axi (JS v0.1.31), no-mistakes (Go v1.30.2), axi (spec+bench)}

--------------------------------------------------------------------------------
C2.1  lavish-axi HTML-ARTIFACT plan/annotation round-trip  ->  VERDICT: mostly CONFIRMATION of canvas H-13/H-14/H-15 + visual-ui-loop; 1 surfaced EXTENSION
--------------------------------------------------------------------------------
MECHANISM (lavish-axi-interface.md): agent writes an HTML artifact -> `npx -y
lavish-axi <file>` spawns an Express server -> artifact served in a sandboxed
iframe with an injected SDK (/sdk.js) that attaches click/hover/text-select
handlers -> user annotates SPECIFIC elements / clicks decisions -> postMessage ->
chrome batches -> POST /api/:key/prompts -> agent long-polls /api/poll and gets
{prompts, layout_warnings} -> agent replies via --agent-reply. Plus a LAYOUT GATE:
the SDK runs auditLayout() (ResizeObserver settle + elementFromPoint overlap probe
+ clipped-text/overflow detection) and the AGENT fixes layout issues BEFORE the
human is shown anything.

ELAI GROUNDING (feature-map + the design system inheritance is the SAME idea):
  - CANVAS WORKBENCH H-13 (model drives canvas, canvas_* ops) / H-14 (APP/WEBSITE
    DESIGN mode: dev-preview proxy + data-oid tagging + two-tier recast writeback) /
    H-15 (spawn_view reply channel + TAINT-GATED DEIXIS, CanvasUiAction::Select
    gated by G-1) is a STRONGER, taint-aware version of lavish's annotate-specific-
    element round-trip. data-oid == lavish's per-element selector; recast writeback
    == lavish's structured-decision return.
  - VISUAL UI LOOP (.claude, WIRED): ui-diff.mjs + ax-scaffold.mjs deterministic
    layout audit BEFORE any vision call == lavish's auditLayout-before-human. ELAI
    already does "deterministic check first, fix before escalating."
  - cook / grill (conditional-routing.md): cook "diverges on real renders" == the
    artifact-as-discussion-surface; grill is the one-question terminal converge.

VERDICT: CONFIRMATION. lavish-axi independently arrives at ELAI's canvas + visual-
ui-loop design, validating both. Two small takeaways:
  (a) CONFIRMATION-RIDER to visual-ui-loop.md: lavish's auditLayout has a specific
      OVERLAPPING-TEXT detector via elementFromPoint probe (does the point at a
      text node's center actually return that node, or is it occluded). ELAI's
      ax-scaffold.mjs crosses AX-tree x box-model but may not do the occlusion
      probe — a cheap deterministic check to add. Rider, not a unit.
  (b) SURFACED EXTENSION (lean: defer / canvas-J-31): render a PLAN / decision-set
      (not just dashboard UI) as an annotatable artifact for plan review — the
      transcript's core "wall-of-text plan -> annotatable visual" pitch. ELAI's
      plan review today is grill (terminal AskUserQuestion) + markdown files. BUT
      the user is explicitly terminal-first (transcript + their own workflow), and
      J-31 system-composition canvas already binds nodes to plan units, so this is
      arguably covered. Surface as a candidate; do NOT prioritize.
DO-NOT-COPY: lavish ships the AUTHOR's brand design system (_ds_bundle.js
  ink/steel/brass), NOT the user-project's — a footgun the SKILL.md itself warns
  against; ELAI's H-14 inherits the real project tokens. Node>=22 + pnpm (not bun).

--------------------------------------------------------------------------------
C2.2  no-mistakes 9-stage VALIDATION PIPELINE  ->  VERDICT: mostly CONFIRMATION (ELAI auditor/RRA/F-4 stronger); 2 real EXTENSIONS + 1 supply-chain CONFIRMATION
--------------------------------------------------------------------------------
PIPELINE (no-mistakes-interface.md): intent -> rebase -> review -> test ->
document -> lint -> push -> pr -> ci. Each stage runs an AI agent with a structured
JSON-schema output in an ISOLATED WORKTREE; findings carry {severity, action:
ask-user|auto-fix|no-op, risk_level}; commands.* + agent are ALWAYS read from the
DEFAULT BRANCH (supply-chain guard). Stages map to ELAI as:

  - intent (infer author intent from local agent transcripts): ELAI HAS intent
    explicitly (the plan/objective) + OBJECTIVE-HASH SEALED RECEIPTS I-45
    (blake3 canonical objective). ELAI's declared-intent > no-mistakes' inferred-
    intent. CONFIRMATION.
  - review (adversarial AI review, fresh context, {severity,action,risk_level}):
    == ELAI auditor + adversarial-auditor + REVIEWER COCKPIT G-5 (per-hunk proof
    tokens, DSR = lines-needing-attention) + FINDING ANCHORING H-52. The
    {action: ask-user (intent-challenging) vs auto-fix (routine)} triage ==
    bubble-up.md; risk_level -> "how much human review" == G-5 DSR. CONFIRMATION
    (ELAI's auditor is off-agent + frozen-gate, structurally stronger).
  - test, document, lint: ELAI has the production_checklist runner suite
    (clippy/cargo-audit/biome/cargo-machete/lighthouse/lsof_port runners,
    crates/orchestrator/src/production_checklist/, codegraph-confirmed) + auditor.
    CONFIRMATION.
  - pr: ELAI has github_pr (crates/orchestrator/src/github_pr/review.rs:
    post_review_comment / build_review_argv / ReviewCommentKind, codegraph-
    confirmed) = E-41 elai pr-review + C-40 (RestApiAdapter deferred). A PR
    review-comment POSTER.
  - ci (BABYSIT the PR to merge: poll 30/60/120s, auto-fix CI failures +
    resolve merge conflicts, up to 3 attempts, until merged/closed): ELAI has
    NO babysit-to-merge loop. github_pr is a reviewer, not a lifecycle manager.

  GENUINE EXTENSIONS (ELAI gaps the pipeline exposes):
  (1) HUMAN-VIEWABLE EVIDENCE ARTIFACTS on the PR  ->  EXTENSION(E-41 / G-5 / G-2).
      no-mistakes' TEST stage records artifacts[{path|url|content}] = SCREENSHOTS /
      VIDEOS / LOGS that prove the change met intent, rendered into the PR body
      (## Testing / Evidence links / collapsible content). ELAI produces MACHINE-
      grade evidence (RRA bundle G-2, SSB rows, F-4 stdout_hash, coverage hits) but
      has NO human-viewable evidence surface attached to a PR/review. The transcript
      makes this the centerpiece ("record evidence that proves the change is working
      that we can later look at ... screenshot / video / log"). This is a real,
      on-wedge gap: ELAI's trust spine HAS the evidence; it lacks the human-facing
      RENDER of it. Frame: an evidence-artifact attachment surface on E-41 PR /
      G-5 reviewer cockpit that renders RRA + test-evidence (incl the visual-ui-loop
      screenshots ELAI already captures) in human-viewable form. STRONG candidate.
  (2) PR-BABYSIT-TO-MERGE loop  ->  EXTENSION(C-40), DOCTRINE-TENSION FLAGGED.
      Poll CI + auto-fix failures + auto-resolve conflicts until merged. ELAI lacks
      it. BUT it tensions with ELAI doctrine: commit-cadence = ONE commit at end,
      worker-git-read-only (agent-dispatch-preamble R-1), and ELAI is build-and-
      verify not a PR-lifecycle product. The auto-fix-CI loop needs agents that
      mutate git on a PR branch. Surface as a candidate with the tension named;
      likely a NARROW deterministic "CI status poll + surface, human merges" (no
      autonomous force-push) is the on-wedge slice, not the full auto-fix loop.
  (3) SUPPLY-CHAIN: commands.* + agent config read from the DEFAULT BRANCH, never
      the pushed SHA (a malicious PR cannot inject pipeline commands). ==
      CONFIRMATION of ELAI's freeze-the-gate (R8) + L3 frozen-surface D-9 + locked-
      keys doctrine, applied to PR-pipeline config. Note as a convergent-validation;
      rider only if ELAI ever sources gate/command config from an untrusted ref.
DESIGN HONESTY: no-mistakes is "soft prompt-steering, NOT an OS sandbox" (its own
  words; "a prompt-injection in a test could escape"). ELAI Tier-C OS-process
  isolation is the strictly-stronger answer. CONFIRMATION of ELAI's wedge.
  Go 1.25.0 go.mod = future/dev version (agent-flagged UNVERIFIED).

--------------------------------------------------------------------------------
C2.3  axi standard + TOON output format  ->  VERDICT: STRONG external CONFIRMATION of H-4/H-34/J-24/mcp_budget; 1 measure-first NEW candidate (token-efficient tool output)
--------------------------------------------------------------------------------
THE 10 AXI PRINCIPLES (axi-interface.md, verbatim in the map): token-efficient
output (TOON), minimal default schemas, content truncation w/ escape hatch, pre-
computed aggregates, definitive empty states, structured errors+exit-codes
(errors->stdout, progress->stderr, idempotent mutations exit 0), ambient context
via SessionStart hook, content-first no-args, contextual disclosure (next-step
hints), consistent help. Benchmark (AUTHOR-SELF-REPORTED, raw jsonl at
docs/.source/axi/bench-browser/published-results/ NOT aggregated by us): AXI tools
~79K tokens/task vs MCP ~185K; AXI 100% success $0.074/task vs MCP 82-99%
$0.10-0.15. TOON ~40% token savings vs JSON (self-reported).

ELAI GROUNDING:
  - The "MCP is token-expensive" thesis is ALREADY ELAI's: mcp_budget.rs
    (crates/plugins/src/mcp_budget.rs, codegraph-read verbatim — TokenBudget +
    enforce(), drops low-priority tools to fit DEFAULT_BUDGET_TOKENS=5000, cites the
    "55,000-token GitHub MCP overhead / 5 servers x 20 tools approx 80k tokens"); H-4
    TOOL SEARCH (77K->8.7K tool-def cut); H-34 MCP DEFER-LOADING; J-24 CONTEXT-FRUGAL
    TOOLS. axi's GitHub-MCP-3x-cost benchmark is DIRECT external CONFIRMATION +
    citable grounding for this whole thread (note: author-self-reported, ELAI
    re-measures per ID-5 / HD-1).
  - ELAI has NO systematic token-efficient OUTPUT FORMAT for its tools. ELAI tool
    returns are serde-JSON / ad-hoc compact strings (e.g. context_budget.format()'s
    hand-rolled CONSTRAINTS block, codegraph-confirmed) — token-compact in places,
    never a uniform format. mcp_budget governs tool-LIST size, not tool-OUTPUT size.

VERDICTS:
  (a) STRONG CONFIRMATION / CITE: the axi benchmark grounds H-4/H-34/J-24/mcp_budget.
      The STRATEGIC FORK worth recording: AXI's answer is "prefer CLI+TOON over MCP";
      ELAI's answer is "make MCP cheap (defer-load H-34 + budget mcp_budget + tool-
      search H-4)." ELAI's deep MCP investment (codegraph / gpt-bridge / local-rag /
      canvas) means it should NOT abandon MCP; the AXI principles instead apply to
      ELAI's OWN CLI surface (elai * subcommands + .claude/tools/*.py harness tools).
  (b) NEW candidate (measure-first): a token-efficient OUTPUT FORMAT for ELAI's
      high-volume tool returns (the elai CLI + IPC tool results + harness tools the
      model consumes). Adopt TOON (real: github.com/toon-format/toon-go pinned in
      no-mistakes; toonformat.dev) OR measure ELAI's own alternatives; done-gate is
      ELAI-MEASURED token delta on real tool outputs, NEVER the 40% claim (ID-5).
      Composes J-24 / H-4 / Tier-C IPC. Scope to the heaviest tool returns first.
      ANTI-SLOPSQUAT: toon-go = pseudo-version v0.0.0-20251202084852 (real); axi-sdk-
      js ^0.1.7 UNVERIFIED (agent-flagged, `npm view` before any JS path); the TOON
      full grammar lives at toonformat.dev/reference/spec.html (NOT fetched — do not
      invent grammar). bun-not-node if any JS reimpl.
  (c) The 10 principles as an ELAI TOOL-AUTHORING ergonomics checklist: mostly
      ALREADY-DOCTRINE (token-discipline, minimal schemas, structured output) or
      CONFIRMATION. Principle 7 (ambient context via SessionStart hook = compact
      dashboard injected at session start) == the ELAI-FEATURE-MAP single-load + the
      lazycodex codegraph session-start auto-sync rider. Principle 8 (content-first
      no-args) + structured-errors-to-stdout are clean cheap wins for the elai CLI.
      Frame as a harness-tool ergonomics rider to J-24, not a standalone unit.

CLUSTER-2 SUMMARY (for the J-plan author):
  EXTENSIONS worth authoring:
    - EXTENSION(E-41 / G-5 / G-2) [strong, on-wedge]: HUMAN-VIEWABLE EVIDENCE
      ARTIFACTS surface — render RRA + test-evidence + visual-ui-loop screenshots
      into the PR / reviewer cockpit so a human SEES the change met intent. ELAI has
      the evidence, lacks the human render.
    - NEW (measure-first) candidate: token-efficient tool-OUTPUT format (TOON or
      ELAI-measured) for heavy tool returns; composes J-24/H-4; ELAI-measured gate.
  RIDERS: lavish overlapping-text elementFromPoint probe -> visual-ui-loop.md;
    no-mistakes commit-repair/backoff already in C1.2; axi 10-principle checklist ->
    J-24 tool-ergonomics; supply-chain commands-from-default-branch -> freeze-gate
    confirmation.
  DOCTRINE-TENSION (surface, do not auto-author): PR-babysit-to-merge auto-fix loop
    vs one-commit-at-end + worker-git-read-only; the on-wedge slice is poll+surface,
    human merges, no autonomous force-push.
  CONFIRMATIONS / CITE: axi MCP-cost benchmark grounds H-4/H-34/J-24/mcp_budget;
    no-mistakes soft-steering admission validates ELAI Tier-C; lavish-axi validates
    canvas H-13/14/15 + visual-ui-loop; commands-from-default-branch validates
    freeze-the-gate.
  DO-NOT-COPY: lavish author-brand design bundle; gnhf/no-mistakes worker git
    mutation patterns (ELAI worker git read-only); the AXI ~40% / cost numbers as
    ELAI numbers (re-measure, ID-5).

================================================================================
CLUSTER 3 — vercel-skills + OpenSuperWhisper + video-narrative  (onboarding / input / philosophy)
================================================================================
Maps: workflow/papers/{vercel-skills,opensuperwhisper,video-narrative}-interface.md
Clones: docs/.source/{vercel-skills (TS v1.5.12), OpenSuperWhisper (Swift macOS),
  video-narrative.txt (transcript copy)}

--------------------------------------------------------------------------------
C3.1  vercel-labs/skills cross-agent INSTALLER  ->  VERDICT: CONFIRMATION of E-43 + skill-import-policy (ELAI AHEAD on security); 1 minor candidate
--------------------------------------------------------------------------------
MECHANISM (vercel-skills-interface.md): `bunx skills add owner/repo` discovers
SKILL.md files (YAML frontmatter name+description, progressive disclosure — only
description loads into the system prompt, body loads on activation) and installs
them to the right per-agent dir across 54 agents (symlink-from-canonical-.agents/
skills/, copy fallback on Windows). Source resolution: github / url / local /
.well-known/agent-skills/index.json (RFC 8615). skills-lock.json with SHA-256
folder hash.

ELAI GROUNDING: E-43 AGENT SKILL SYSTEMS (agentskills.io v1 frontmatter validator
+ `elai skills-mgmt list|eval|publish` + per-skill eval JSONLs) + skill-import-
policy.md (skillspector whole-directory scan BEFORE promotion + workflow/skill-
imports.log row + /skill-audit gate). The SKILL.md format vercel-skills installs IS
the same agentskills.io v1 spec ELAI's E-43 validator gates against — CONFIRMATION.

VERDICT: CONFIRMATION; ELAI is STRUCTURALLY AHEAD on the axis that matters.
  - SECURITY: vercel-skills ships ZERO content trust ("Review skills before use;
    they run with full agent permissions" — verbatim; no sandbox, no signing, no
    hash-verify-at-install, telemetry ON by default, SHA-256 not blake3). ELAI's
    skillspector-scan + import-log + blake3 + import-policy is the hardened answer.
    This is direct external VALIDATION that ELAI's skill-import-policy is the right
    posture, not over-engineering.
  - The video author's OWN warning ("avoid popular-but-unevaluated skills; I
    benchmarked the 177k-star Karpathy-style skills repo with programbench: +5%
    tokens, WORSE results" — AUTHOR claim, UNVERIFIED) is STRONG external grounding
    for E-43 `skills-mgmt eval` + the R5 baseline-RED rider (obra/superpowers TDD-
    for-skills: prove the skill CHANGED behavior vs a no-skill baseline, not just
    pass-WITH-skill). A senior practitioner independently arriving at "skills degrade
    performance unless rigorously evaluated" == E-43's whole eval thesis. CITE.
  - MINOR candidate (low priority): a `.well-known/agent-skills/index.json` publish
    endpoint for `elai skills-mgmt publish` (a standard way to PUBLISH ELAI's trust-
    gated skills — the WEDGE: ELAI publishes skills that PASSED skillspector). Rider
    to E-43, not a unit. ELAI is single-harness so the 54-agent multi-install is N/A.
DO-NOT-COPY: vercel-skills' zero-trust install + on-by-default telemetry. bun-not-
  node (it's pnpm/Node>=18; the video's `npx skills` -> `bunx skills`). SHA-256 ->
  blake3 if any lock format is mirrored.

--------------------------------------------------------------------------------
C3.2  OpenSuperWhisper local voice transcription  ->  VERDICT: ALREADY-REJECTED (voice off-wedge, I-0); 1 portability note only
--------------------------------------------------------------------------------
MECHANISM (opensuperwhisper-interface.md): macOS-only SwiftUI app wrapping
whisper.cpp (ggml .bin) + optional Parakeet/NeMo; PCM Float32 @16kHz -> WhisperFull
Params -> whisper.cpp via C bridge; the `initialPrompt` field biases token
prediction toward domain vocab (prepended as a fake prior transcript).

ELAI GROUNDING: Section-4 ledger — "voice-first input lane: DECLINED (consumer UX,
off-wedge)" (I-0 authoring declines). CONFIRMED REJECT; do NOT re-propose.

VERDICT: ALREADY-REJECTED. Two durable notes only, no unit:
  - OpenSuperWhisper itself is macOS/Swift (AppKit/AVFoundation/Carbon/FluidAudio) —
    NOT portable to ELAI's win32 + remote-Linux-sidecar. If voice were EVER revisited
    (it is off-wedge today), the portable path is faster-whisper (Python/CTranslate2,
    `initial_prompt=` kwarg = the SAME custom-vocab bias) on the existing remote 3090
    sidecar, never a vendored macOS app.
  - The `initialPrompt` domain-vocab-biasing pattern is a general "prime with domain
    terms" trick (whisper-family). Not ELAI-relevant (ELAI's sidecar is a CODE model,
    not ASR). Recorded for completeness; no action.

--------------------------------------------------------------------------------
C3.3  video-narrative (the design philosophy)  ->  VERDICT: overwhelming CONFIRMATION of ELAI's whole shape; the META takeaway
--------------------------------------------------------------------------------
The transcript is an independent senior practitioner (Kun, ex-Meta/Atlassian, "40-50
PRs/day" — AUTHOR claim, UNVERIFIED) converging on ELAI's EXACT architecture. Point-
by-point (all AUTHOR design claims, not measured-by-us):
  - "spend time at START (plan) + END (quality bar), free the MIDDLE (autonomous)"
    == ELAI: heavy planning (idea-intake / planning-workflow / grill / cook) + heavy
    end-gate (auditor / wiring / RRA / SAFEDELTA) + autonomous middle (orchestrate /
    auto-orchestrate / AutonomousGate). CONFIRMATION of ELAI's whole shape.
  - "eng-manager not diff-reviewer; low-risk PRs never hand-reviewed because the
    pipeline caught it; the RISK ASSESSMENT decides how much human attention" ==
    G-5 reviewer cockpit DSR (lines-needing-attention/total) + risk_level + the
    autonomous envelope. The MECHANISM that EARNS "don't review low-risk diffs" is
    exactly ELAI's evidence-grounded trust spine (RRA + F-4 cant-lie + wiring +
    risk). This is the C2.2 EXTENSION (human-viewable evidence + risk-tier-gated
    review) seen from the demand side. CONFIRMATION + reinforces C2.2 priority.
  - "memory: global = personal prefs only (token-minimal) / project = accumulated
    learnings (correct-the-agent-and-store-the-learning) / skills = conditional
    overflow (progressive disclosure)" == CLAUDE.md + memory-bank/gotchas.md +
    .claude/skills + the two-tier USER MEMORY (H-31 tier-B / H-32 tier-A) + E-5 +
    token-discipline.md "every token in CLAUDE.md costs context." CONFIRMATION; ELAI
    is STRONGER (bi-temporal supersession, validate-citation-before-use, auditor-
    objectivity firewall).
  - "e2e testing > unit tests (AI defaults to insufficient unit tests)" ==
    wiring-enforcement R10 (integration smoke through the REAL entrypoint > per-unit
    green) + the "bug fixes start by reproducing end-to-end" rule. STRONG convergent
    validation of R10.
  - "agent-agnostic workflow (the harness/model landscape changes monthly)" == B-37
    MULTI-RUNTIME BACKEND (11 backends) + AGENT-CLI BRIDGE + J-64 ACP. CONFIRMATION.
  - "treat agents as first-class, design tools for agent-ergonomics (AXI)" == C2.3.

  ONE small NEW candidate (UNGROUNDED — flag + measure before acting): the claim
  "frontier models OVER-ESTIMATE development cost (trained on human estimates) and
  therefore bias toward cheap / low-quality / unscalable solutions; corrected by a
  global rule 'do not give too much weight to development cost.'" Plausible, non-
  obvious, actionable as a planner/architect-role prompt bias-correction (I-36
  compiled prompts / I-48 prompt artifacts). But it is an UNGROUNDED author claim —
  ELAI must MEASURE the bias on its own lanes before adopting (grounded-by-default).
  Surface as a candidate prompt-rule, do NOT author blind.

THE META TAKEAWAY (the single most useful thing for ELAI):
  The ENTIRE toolchain is the UNBUNDLED, TRUST-LIGHT version of what ELAI builds
  BUNDLED and TRUST-HEAVY:
    treehouse -> I-2 worktree isolation | gnhf -> AutonomousGate/loop | firstmate ->
    orchestrate/auto-orchestrate | lavish -> canvas H-13/14/15 | no-mistakes ->
    auditor + RRA + wiring + F-4 | axi -> H-4/H-34/J-24/mcp_budget | skills -> E-43 |
    whisper -> (rejected).
  The DECISIVE DIFFERENCE = TRUST/VERIFICATION. The video toolchain runs agents
  UNSANDBOXED (`--dangerously-skip-permissions`), with SOFT prompt-steering ("not an
  OS sandbox; a prompt-injection in a test could escape" — no-mistakes' own words),
  and SELF-judged stops (gnhf should_fully_stop). ELAI's wedge is the EXACT opposite:
  Tier-C OS-process isolation, off-agent FROZEN gates, F-4 cant-lie, deterministic-
  not-LLM-judged (TD-3). So this corpus simultaneously (a) MARKET-VALIDATES ELAI's
  product shape (a 40-50-PR/day practitioner independently built ELAI's architecture
  unbundled) and (b) SHARPENS ELAI's differentiation (it is the trust-light foil —
  ELAI is the credible, evidence-grounded version of the SAME workflow). Ties to
  market-positioning + GTM (the "AI-validated so the human only reviews risky diffs"
  workflow is demonstrably what practitioners WANT; ELAI's trust spine is the
  defensible way to deliver it).

CLUSTER-3 SUMMARY (for the J-plan author):
  EXTENSIONS/CANDIDATES: none new beyond C1/C2 (skills well-known publish endpoint =
    minor E-43 rider; dev-cost-bias prompt-rule = ungrounded candidate, measure first).
  CONFIRMATIONS / CITE: vercel-skills zero-trust + Karpathy-worse claim ground E-43 +
    skill-import-policy + R5 baseline-RED eval; the whole narrative market-validates
    ELAI's plan-bookends + autonomous-middle + trust-gated-quality + token-minimal-
    memory + e2e-over-unit + agent-agnostic shape.
  ALREADY-REJECTED: voice input (I-0); confirmed, do not re-propose (faster-whisper-
    on-sidecar is the only-if-ever-revisited portable note).
  DO-NOT-COPY: vercel-skills zero-trust install + on-by-default telemetry.

================================================================================
SYNTHESIS — RANKED FOR THE J-PLAN AUTHOR (most useful to ELAI first)
================================================================================
Scope note: ZERO new plan files authored this run (per user). This is the steal-
reasoning input for a later session to author J-plan files from. All sources cited
above are on disk (docs/.source/* clones + workflow/papers/*-interface.md +
docs/.source/video-narrative.txt). Cross-checked against ELAI-FEATURE-MAP Section 4;
nothing below re-proposes a ledger reject.

TIER 1 — author these (real, on-wedge gaps, grounded against live code):
  1. EXTENSION(I-2): warm-reuse + persistent + self-healing WORKTREE POOL LIFECYCLE
     on the existing WorktreePoolImpl (branch_workspace.rs) — SSB-journal-backed
     (GD-2, never a treehouse-style external JSON store), PID-recycle-safe / lease
     reservation, healState self-heal, cross-platform (win32 mandatory). Unlocks
     orchestrate-WORKER ISOLATION (structurally kills the 2026-06-11 reset-hard data-
     loss class) + the J-60 per-agent-worktree blocker. Reference: docs/.source/
     treehouse (pool.go acquire / state.go / lease). HIGHEST LEVERAGE.
  2. EXTENSION(E-41 / G-5 / G-2): HUMAN-VIEWABLE EVIDENCE ARTIFACT surface — render
     RRA + test-evidence + the visual-ui-loop screenshots ELAI ALREADY captures into
     a PR / reviewer-cockpit view, so a human SEES the change met intent and reviews
     only by risk tier. ELAI has the (machine) evidence; lacks the human render. The
     no-mistakes ## Testing/Evidence section is the reference; the demand is the
     video's "eng-manager not diff-reviewer" thesis. Reference: docs/.source/
     no-mistakes (pipeline test/pr stages).
  3. EXTENSION(supervision, auto-orchestrate): process-level WEDGE DETECTION (output-
     unchanged-N-cycles = stuck agent; ELAI's anti-spiral is reasoning-level, T4 is
     sweep/fixer-count — neither catches a silently-hung worker) + DURABLE WAKE-QUEUE
     (resume supervision across a supervisor crash). Deterministic. Reference:
     docs/.source/firstmate (fm-watch.sh scan_signals / stale / wake-queue).

TIER 2 — measure-first / smaller (author after a quick ELAI measurement or as riders):
  4. NEW (measure-first): token-efficient tool-OUTPUT FORMAT (TOON or ELAI-measured
     alternative) for ELAI's heaviest tool returns (elai CLI + IPC + harness tools);
     composes J-24/H-4/mcp_budget; done-gate = ELAI-MEASURED token delta, NEVER the
     author's 40% (ID-5). Anti-slopsquat: toon-go pinned-real; axi-sdk-js UNVERIFIED.
  5. small EXTENSION(E-39 + auto-orchestrate): mid-loop DETERMINISTIC cost/iteration/
     wallclock cap that aborts the current step (gnhf's mid-iteration token-abort,
     made deterministic — NOT gnhf's LLM-self-judged stopWhen, which is TD-3-banned).

TIER 3 — riders (record in the relevant plan, do not author standalone):
  - lavish overlapping-text elementFromPoint occlusion probe -> visual-ui-loop.md.
  - axi 10-principle checklist (content-first no-args, structured-errors-to-stdout,
    ambient-context-via-SessionStart) -> J-24 tool-ergonomics + the elai CLI.
  - turn-end completion signal (claude Stop-hook touches a file) -> B-37 / J-64 when
    driving external vendor CLIs in a fleet.
  - gnhf commit-repair + backoff failure-taxonomy -> bubble-up.md.
  - `.well-known/agent-skills` publish endpoint -> E-43 `skills-mgmt publish`.

DOCTRINE-TENSION (surface to user before any J-plan unit; do NOT auto-author):
  - PR-babysit-to-merge auto-fix loop (no-mistakes ci stage) vs ELAI one-commit-at-
    end + worker-git-read-only. On-wedge slice = poll CI + surface, human merges, NO
    autonomous force-push.
  - lavish PLAN-as-annotatable-artifact vs ELAI terminal-first plan review (grill +
    AskUserQuestion); arguably covered by J-31. The user is explicitly terminal-first.

OFF-WEDGE / ALREADY-REJECTED (do NOT re-propose — matches Section-4 verdict class):
  - firstmate secondmates (hierarchical autonomous supervisors) vs central-FSM-gate
    trust spine (same class as goose-topology NO-STEAL / choreography REJECTED).
  - voice input (I-0 decline); whisper macOS app non-portable.

DO-NOT-COPY EXHIBITS (the trust-light anti-patterns ELAI's wedge refuses):
  - gnhf `git reset --hard` iteration rollback -> ELAI H-49 CAS / RV-1.
  - firstmate `--dangerously-skip-permissions` / `--dangerously-bypass-approvals-and-
    sandbox` -> ELAI Tier-C + VibeGate + capability gates.
  - no-mistakes soft-prompt-steering "not an OS sandbox" -> ELAI Tier-C isolation.
  - vercel-skills zero-trust install + on-by-default telemetry -> ELAI skillspector +
    import-log + blake3.
  - gnhf LLM-self-judged stopWhen -> deterministic stop predicate (TD-3).
  - SHA-256 lock hashes (vercel-skills) -> blake3 (CLAUDE.md anti-stale).

EXTERNAL CONFIRMATIONS / CITE-WORTHY GROUNDING (not work — strengthen existing plans):
  - axi GitHub-MCP-3x-cost / 185K-vs-79K benchmark grounds H-4/H-34/J-24/mcp_budget
    (author-self-reported; ELAI re-measures, ID-5).
  - Karpathy-skills-made-it-WORSE + the author's skill-eval discipline ground E-43
    eval + the R5 baseline-RED skill-eval rider.
  - the whole narrative market-validates ELAI's plan-bookends + autonomous-middle +
    trust-gated-quality shape; the trust-light foil sharpens ELAI's differentiation.

MAP FIXES (for the next ELAI-FEATURE-MAP upsert):
  - I-2 status: PLANNED -> PARTIAL (WorktreePoolImpl + barrier_merge + escape_check +
    i2_cow_workspace_contract.rs shipped; overlayfs/reflink + warm-reuse-pool legs
    are the residue).

OPEN ITEMS / VERIFICATION STATUS:
  - All 3 agents hit RAG-primary-down (pre-existing Windows reranker crash, memory
    project_rag_reranker_crash) -> the 8 interface maps are ON DISK at workflow/
    papers/ but NOT RAG-indexed; re-run mcp_ingest_file/mcp_memorize when primary
    restored. Does not block this reasoning artifact.
  - All numbers in this file from the corpus are AUTHOR-SELF-REPORTED / UNVERIFIED
    (40% TOON, 3x MCP, 5% Karpathy, 40-50 PRs/day, Stanford-3x-voice). ELAI's done-
    gates re-measure (HD-1 / ID-5); none of these enters a plan as an ELAI number.
  - axi-sdk-js ^0.1.7 + the TOON full grammar (toonformat.dev/reference/spec.html)
    UNVERIFIED — `npm view` + fetch the spec before any TOON-output J-plan unit.

OUTCOME (2026-06-26 authoring session): plan files created per the user's K-native->K /
technical->J placement ruling — TECHNICAL half (this file's TIER-1/TIER-2 units) authored as
J-80 (worktree pool lifecycle, EXTENDS I-2; MAP FIX I-2 PLANNED->PARTIAL), J-81 (autonomous
supervision hardening = process-wedge + durable wake-queue + mid-loop cost cap wiring DARK
E-39), J-82 (deterministic token-efficient emission = TOON output + dictionary-log lane +
protected-span tokenizer, measure-first). The on-wedge human-evidence steal (C2.2) went to the
K-native half as K-11 (human-viewable evidence render, EXTENDS K-3). J-0/J-SUMMARY + K-0/
K-SUMMARY indexes updated; ELAI-FEATURE-MAP upserted (5 entries + Section-4 verdict block +
MAP FIX). Declines/do-not-copy/riders recorded in J-0 DECLINES + the feature-map ledger.
