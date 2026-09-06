Integration exploration (HARD RULE — wiring discovery for HEAVY-tier tasks)

Loaded by: HEAVY-tier flows in planning-workflow. Skipped for SIMPLE / FOCUSED / CONFIGURING.

Purpose

Most multi-system feature failures aren't bad code, the new code does NOT WIRE INTO existing systems correctly. Blast-radius analysis (what does this break) is one direction; wiring discovery (what existing APIs/events/contracts should this hook into) is the OTHER direction. This rule defines the parallel-exploration pattern that produces wiring discovery as a first-class artifact.

The pattern

  context-compiler  -> writes context-package.md (memory-bank state + blast radius + interface map)
  scope-planner     -> reads context-package.md, partitions into N integration subsystems, writes exploration-plan.md, decides EXPLORE vs SKIP
  if EXPLORE:
    main LLM dispatches N integration-explorer agents IN PARALLEL (one per subsystem, hard cap: 8)
    each explorer writes workflow/sessions/{id}/exploration/<subsystem>.md (transient, dies with session)
    main LLM waits for all explorers
    synthesizer    -> reads each exploration file, appends WIRING section to context-package.md
  prompt-engineer   -> reads context-package.md (now including WIRING) and produces the implementation prompt

When to EXPLORE

The threshold is 3 distinct INTEGRATION subsystems. Integration means: the new code calls into an existing API, listens to an existing event, persists data through an existing system, or shares state with an existing owner. It is NOT "files in the blast radius."

Examples (calibrated for iOS apps):
  "fix nil check in LoginViewModel" — 0 integration subsystems. SKIP.
  "rename property X to Y" — 0. SKIP.
  "add a debug logger to network layer" — 1. SKIP.
  "add Apple Sign In" — 2 (auth flow, user record). SKIP unless user explicitly requests deeper exploration.
  "add subscription paywall with StoreKit + backend receipt validation + restore + family sharing" — 6+ (StoreKit, backend API, user state, paywall UI, analytics, error states). EXPLORE.
  "rework auth to support biometrics + token refresh + multi-device" — 4-5 (Keychain, auth API, session manager, app delegate, settings UI). EXPLORE.

Hard cap: 8 parallel explorers. If the partition exceeds 8, the partition is wrong — collapse adjacent subsystems or escalate to user.

Why parallel

Each explorer reads its own subsystem deeply and returns ~50k tokens of subagent context the main LLM never sees. The main LLM only reads the synthesizer's WIRING table. Parallel dispatch finishes ~7x faster than sequential for a 7-system task.

Why disjoint partitions

Two explorers reading the same subsystem = duplicated work. scope-planner emits a partition where each subsystem has one owner. Cross-system findings go in each explorer's SEE-ALSO field, flagged, not investigated deep. The synthesizer detects HUBS (3+ explorers flagging the same file) and PARTITION GAPS (subsystem surfaced via see-also but not in the original partition).

Hub escalation

Synthesizer reports a hub: spawn ONE additional integration-explorer scoped to the hub file. Re-spawn synthesizer with the new exploration in the list. Handles join points like AppState.swift or APIClient.swift that legitimately couple multiple subsystems.

Partition gap escalation

Synthesizer reports a missed subsystem: spawn ONE additional integration-explorer for that subsystem. Re-spawn synthesizer.

Hard cap on escalations: 2 per phase. If still unstable, surface to user.

Output paths

  workflow/sessions/{session_id}/exploration-plan.md       — scope-planner output (read by main LLM + integration-explorers)
  workflow/sessions/{session_id}/exploration/<name>.md     — one per subsystem, written by integration-explorer
  workflow/sessions/{session_id}/context-package.md        — context-compiler writes initially; synthesizer appends WIRING section

The WIRING section format (in context-package.md, appended by synthesizer)

  WIRING (synthesized from N parallel explorations)
  | Subsystem | Integration Method |
  | <name>    | <top API + usage in one line> |

  HUB DETECTION: <file> | flagged by [agents] | recommend
  PARTITION GAPS: <missed-subsystem> | source | recommend
  DRIFT: <memory-bank claim> | code reality | flagged by
  GAPS BLOCKING IMPLEMENTATION: <gap> | from <agent> | decision needed

The implementer (executor / main LLM after prompt-engineer) reads this table to know exactly which existing API to call, which sink/event to subscribe to, which keychain key to register with, without re-discovering anything.

MCP awareness across the explorer batch

No iOS-domain MCP is wired currently (no Xcode-bridge MCP exists). integration-explorers fall back to file Reads + Greps over .swift sources. If a future Xcode MCP appears, update integration-explorer.md to prefer it over file walks.

If a task domain has no MCP available, the agent surfaces "MCP-suggested: <domain>" and the main LLM relays to the user.

Cross-reference

  .claude/skills/planning-workflow/SKILL.md — full HEAVY-tier flow with the new exploration steps
  .claude/agents/scope-planner.md — partition + EXPLORE/SKIP decision
  .claude/agents/integration-explorer.md — per-subsystem wiring report
  .claude/agents/synthesizer.md — WIRING table aggregation + hub detection
  .claude/rules/agent-spawning.md — canonical agent list, sequencing rules
