Phase 4A Grounding Results
Date: 2026-04-06
Session: s-1449012

SPEC VERIFICATION

  orchestrator.txt Section 12 | EXISTS | Line 343. Full Tier C spec with all audit fixes applied.
  tools.txt "Section 1 (tool registry)" | INCORRECT | Section 1 is "RESPONSIBILITIES". Fixed to Section 8 (visibility filter) + Section 9 (bash sandbox).
  audit-results-phase4.txt "BLOCKs 1-3" | INCOMPLETE | File has 9 BLOCKs + 9 WARNs. Fixed to reference all.
  architecture-overview.txt Section 9 | EXISTS | Line 269. Agent roles.
  reasoning.txt Section 4 | ADDED | GRC verify methods + GroundingRequest IPC. Was missing from READ directives.

CLAIM GROUNDING

  "IPC round-trip <1ms p99 for <64KB" | UNGROUNDED | No prior benchmark. Unix domain sockets typically <100us. Reasonable target.
  "Agent spawn time <100ms" | UNGROUNDED | orchestrator.txt estimates ~50ms (L555). Verify in 4A.6.
  "Memory per agent <30MB" | UNGROUNDED | orchestrator.txt estimates ~20MB (L556). Verify in 4A.6.
  "Overhead vs in-process <5%" | UNGROUNDED | IPC negligible vs LLM inference. Verify in 4A.6.
  "seccomp-bpf with BPF argument check on socket domain" | GROUNDED | audit BLOCK-1, orchestrator.txt L499-511.
  "postcard wire format" | GROUNDED | postcard crate on crates.io, in project deps (architecture-overview.txt L291).
  "SO_PEERCRED for peer PID verification" | GROUNDED | Linux socket(7) man page, orchestrator.txt L438-439.
  "0700 socket directory" | GROUNDED | orchestrator.txt L435, audit BLOCK-5 fix.
  "AppContainer on Windows" | VERIFIED | Windows API CreateAppContainerProfile documented. orchestrator.txt L530-538.
  "Zero capabilities retained" | GROUNDED | orchestrator.txt L495-497.
  "veth + iptables for researcher" | GROUNDED | orchestrator.txt L497, audit BLOCK-2 fix.
  "LLM calls proxied via IPC" | GROUNDED | orchestrator.txt L423-425, audit BLOCK-3 fix.
  "Orchestrator-enforced GRC budget" | GROUNDED | orchestrator.txt L478-491, audit BLOCK-4 fix.
  "SSB write synchronous with ToolResult" | GROUNDED | orchestrator.txt L468-469, audit WARN-6 fix.
  "GroundingRequest IPC message" | GROUNDED | reasoning.txt L137-152, orchestrator.txt L452-453.
  "ReplanRequest max 1 per agent per phase" | GROUNDED | orchestrator.txt L458-464.
  "Self-mod: realpath + RO mount + git filter" | GROUNDED | orchestrator.txt L559-565, audit BLOCK-8 fix.
  "Auditor self-discovery: PROPOSED + second auditor" | GROUNDED | orchestrator.txt L567-573, audit BLOCK-6 fix.

STALE CONTENT FIXED

  Phase-4A SKELETON was stale vs orchestrator.txt S12. Missing:
    - "Does NOT contain" list for agent binary (BLOCK-3 fix)
    - Complete message type enum (14->15 types, added GroundingRequest/Result)
    - Connection sequence (listen-before-spawn)
    - Version handshake
    - agent_id = UUID v4 format
    - MAX_FRAME_SIZE + per-type limits
    - ReplanRequest in skeleton (was only in tests)
  Phase-4A RISKS only had BLOCKs 1-3. Now has all 9.
  Phase-4A had no structured improvement blocks. Now has PRE_GATE, DETERMINISTIC_CHECK, ADVERSARIAL, ACCEPTANCE per step.

IMPROVEMENTS ADDED (each provably beneficial)

  1. Synced SKELETON with orchestrator.txt S12 | Removes contradictions | BLOCK-3 proves need
  2. Expanded RISKS to all 9 BLOCKs | Audit proves all are real | BLOCK-4 through BLOCK-9 were missing
  3. Fixed tools.txt READ ref to S8+S9 | S1 is "responsibilities", not isolation-relevant
  4. Added reasoning.txt S4 to READ | WARN-2 proves agents need GroundingRequest proxy
  5. Added GroundingRequest tests | reasoning.txt L137-152 defines the IPC message
  6. Added crash recovery tests | WARN-6 proves gap in original spec
  7. Added Windows-specific tests | BLOCK-7 proves Windows path underspecified
  8. Added auditor self-discovery tests | BLOCK-6 proves circular trust risk
  9. Added self-mod lock tests with git/bash paths | BLOCK-8 proves IPC-only is insufficient
  10. Split steps into 6 numbered phases (was flat) | Enables sub-phase resume per implementation-guide rules
  11. Added SEPARATION COSTS section | orchestrator.txt L553-557 provides data for realistic planning
  12. Added GROUNDING section | Makes all assumptions visible and auditable

POST-IMPROVEMENT AUDIT (adversarial-auditor, 2026-04-06)

  Audit file: docs/specs/decisions/audit-results-phase4-improvement.txt
  Result: 3 BLOCKs, 7 WARNs. All fixed in second pass.

  BLOCK-1 (FIXED): RLIMIT_FSIZE=100MB and RLIMIT_AS=512MB were missing from IMPLEMENT.
    Added to Linux process isolation section. Tests added for rlimit enforcement.
  BLOCK-2 (FIXED): /proc masking and /sys unmounting missing from IMPLEMENT.
    Added: "/proc: masked (hidepid=2) or not mounted. /sys: not mounted."
    Tests added for /proc and /sys inaccessibility.
  BLOCK-3 (FIXED): ShortCoder deferral to 4B not documented.
    Added DEFERRED TO 4B section in IMPLEMENT + EXIT CRITERIA forward-reference.
  WARN-1 (FIXED): GroundingRequest role-method matrix not enumerated.
    Added explicit matrix from reasoning.txt S9 to GroundingRequest test section.
  WARN-2 (FIXED): Orphaned-topic GroundingRequest budget bypass.
    Added test: GroundingRequest for topic with no active ThinkStep budget -> rejected.
  WARN-3 (FIXED): PID namespace verification via agent self-report.
    Changed to host-side /proc/{child_pid}/status NSpid check.
  WARN-4 (FIXED): Ambiguous grep+count for enum variants.
    Changed to compile-time const_assert! or enum variant count.
  WARN-5 (FIXED): u32::MAX frame test moved from ADVERSARIAL to TASK.
    Added as explicit test case in adversarial IPC TASK section.
  WARN-6 (FIXED): GRC enforcement transition test missing.
    Added test for agent local count != orchestrator count edge case.
  WARN-7 (ACKNOWLEDGED): Tier A chroot CAP_SYS_CHROOT issue out of scope for 4A.
  CRITICAL GAP (FIXED): Inter-agent AF_UNIX socket creation.
    Session socket dir not mounted into agent namespace. Per-agent /tmp isolated.
    Test added: two concurrent agents cannot create sockets to each other.
