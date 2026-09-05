Handoff protocol (HARD RULE)

Loaded by: any time the user says 'write a handoff', 'write a handoff file', 'update the handoff', 'save a handoff', 'handoff this', 'append to the handoff', or otherwise asks for a session/work handoff. Also the recommended close-out for any multi-session effort before context runs low.

WHY: One problem used to spawn a chain of files (SESSION_HANDOFF_2026-05-14.md, CLOUD_E2E_PROGRESS_2026-05-17.md, _R3, _R4, _R5, PHASE_E_REPORT.md, PHASE_E2_REPORT.md, FINAL_REPORT.md, plus per-session SESSION_HANDOFF.md scattered across workflow/sessions/{s-id}/), each a linked-list node pointing back one link, none cumulative. A new session meant a new date-stamp or a new session-id folder meant a new file, so the same problem fractured across many files and the user had to re-read and copy-paste the chain every time. The fix: ONE file per problem, keyed by topic, appended across every session that touches that problem.

THE ONE-FILE RULE (blocking): Handoff files live at a stable, TOPIC-keyed path, never a session-keyed or date-keyed one: workflow/handoff/{topic-slug}.md . topic-slug is a short kebab description of the PROBLEM not the session and not the date. The slug is stable for the life of the problem across however many sessions it takes. On the trigger the main LLM:
  - RESOLVE the topic slug, preferring continuity with prior work that already has a handoff, asking one short question only if genuinely ambiguous else inferring and stating the slug.
  - GLOB workflow/handoff/*.md for an existing handoff for this problem.
  - IF IT EXISTS APPEND a new dated session-stamped entry at the end and update the two LIVING fields in place (CURRENT STATUS line + INDEX), never create a sibling with a date or _R3/_R4 suffix (the sprawl is itself a BLOCK).
  - IF NOT CREATE from the template below.
One handoff per problem for all time; a later session continuing the same problem appends, never spawns a new file. workflow/handoff/ is the existing de-facto handoff directory (singular 'handoff'); reuse it, do not invent a new one.

ANTI-READ-BIG WHEN APPENDING: a long-lived handoff grows past the big-file threshold. Do NOT full-read it to append. Read only the HEAD (first ~40 lines: title + CURRENT STATUS + INDEX) to update the living fields and learn the slug/state, then append the new entry at the END. This is an AUTHORING read on a file you are about to write, which is the explicit carve-out in .claude/rules/token-discipline.md "Anti-read-big (HARD RULE)" EXCEPTION; head-only keeps it cheap. Never load the full body of prior entries just to add one.

FILE TEMPLATE (create form): Title 'Handoff: {topic-slug}'; 'problem:' one-line statement of the whole effort; 'CURRENT STATUS:' one line OVERWRITTEN on every append (single source of truth for where we are now); an INDEX block (newest at bottom, append-only) of '{date} session {id} - {one-line hook}'; then a rule line; then each dated entry with the sections below.

ENTRY SECTIONS (ALL-CAPS label on its own line, then the body):
  SUMMARY — 2-6 lines, what this session did and where it leaves the problem.
  WORK DONE — concrete changes and decisions made this session.
  FILES TOUCHED — each as a real path + one line of what changed.
  USER CHOICES / DECISIONS — explicit choices the user made, verbatim enough they are never re-litigated next session.
  HOW TO PICK UP — exact steps/commands to resume (boot the sidecar, the -p crate to test, the B-N to walk next, etc).
  KNOWN LIMITS / NEXT — open items, registered stubs, gotchas, obvious next move.
  GROUNDING / VERIFICATION — what was built/tested green (pass/fail counts, deterministic check status per commit-cadence house rule), what is unverified.

APPEND FORM: when the file already exists you add ONLY a new INDEX line (bottom of INDEX block), the new dated entry (very end, after a rule line), and you OVERWRITE the CURRENT STATUS line with the latest state. Never rewrite prior entries, they are immutable history. CURRENT STATUS and INDEX are the only living fields.

WHAT EVERY ENTRY MUST CARRY: the point is the user never has to repeat themselves, so an entry is incomplete if it omits any of SUMMARY, FILES TOUCHED (with what changed), USER CHOICES / DECISIONS, HOW TO PICK UP. Keep each tight (token economy), write file paths as real paths, use the repo's house voice for prose.

WHO WRITES IT: the main LLM authors and appends directly. It holds the session context, this is a first-party tooling-doc authoring action, NOT delegated to memory-writer (.claude/agents/memory-writer.md writes RAG-indexed memory-bank files via mcp_memorize/mcp_log_change in MEMORIZING; the handoff is not a memory-bank file and is not RAG-indexed). It works in ANY FSM state (it only reads + appends a tooling doc), so a peer session holding a shared FSM is not a blocker and the FSM is never reset for it.

RELATIONSHIP TO OTHER ARTIFACTS: this is NOT the machine-written compaction-resilience autosave. The PreCompact hook (C:/Users/user/.claude/rag/hooks/pre_compact_check.py, registered in .claude/settings.local.json) writes a machine SESSION_HANDOFF FSM-state snapshot before context truncation; that is a different artifact with a different purpose. The handoff-protocol file is the human-facing, cross-session, one-per-problem narrative. It is also not a replacement for the per-build memory-bank files (workflow/memory-bank/activeContext.md overwritten under 80 lines, changelog.md, progress.md, projectIndex.md) which memory-writer maintains. Keep all three separate: machine FSM snapshot, per-build memory-bank, human topic-keyed handoff.

MIGRATION OF LEGACY CHAINS: workflow/handoff/ currently holds the legacy sprawl (date-stamped SESSION_HANDOFF_2026-05-14.md, CLOUD_E2E_PROGRESS_2026-05-17{,_R3,_R4,_R5}.md, AUTONOMOUS_BUILD_PROOF_*.md, PHASE_E*_REPORT.md, FINAL_REPORT.md, the b28-staging/ subdir, and per-session SESSION_HANDOFF.md under workflow/sessions/{s-id}/). When next handing off a problem that already has a legacy numbered or dated chain, either consolidate the chain into one workflow/handoff/{topic-slug}.md (preferred when cheap) or start the single-file convention going forward with a one-line pointer at the top of the new file naming the old chain so nothing is lost. Do not keep growing the dated / per-session chains.
