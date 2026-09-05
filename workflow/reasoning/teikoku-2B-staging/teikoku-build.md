# Handoff: teikoku-build

problem: orchestrate the TEIKOKU build (merge the ELAI + liteGAME harnesses on a frozen Claude Code local copy, WSL distro `teikoku`, conductor memory runtime, Tauri dashboard)

CURRENT STATUS: P2-B DONE. The memory-writer ROLE is reworked to auto-index handoffs into the TEIKOKU memory; the markdown memory-bank doctrine is retired across TIER-1 (memorizing-and-audit, output-formatting, planning-workflow, orchestrate protocol pair) + TIER-2 (autonomous-orchestrator, main-llm-role, conditional-routing, token-discipline, context-compiler, auditor read-side). New runtime tools conductor/handoff_indexer.py + conductor/memory_bank_migrate.py, wired into memory_server.py (2 MCP tools) + slice_api (CLI + POST route). Wire-check PASS (0 DARK fn). P2-A conductor tests 4/4 green. Next: P2-C dashboard memory surface (the 2.5 surface design pass).

INDEX
- 2026-06-20 session P2-B-exec - reworked memory-writer to auto-index handoffs; retired the markdown memory-bank doctrine; built the auto-indexer + auto-migrate tools

----------------------------------------------------------------------

2026-06-20 P2-B-exec

SUMMARY
Reworked the TEIKOKU runtime memory model (master-plan 2.4, D-08). Handoffs are now the canonical memory unit: an auto-indexer ingests workflow/handoff/*.md as append-only memory events into the P2-A event log so they are grep + semantic + timeline + global retrievable (closing the P2-A gap where grep_handoffs read disk live but handoffs were never rows in the events table). The memory-writer ROLE no longer authors memory-bank markdown; the markdown memory-bank doctrine is retired across the core artifacts. An auto-migrate tool ingests a legacy markdown memory-bank into the new memory once, with a graceful no-op when absent.

WORK DONE
- Built conductor/handoff_indexer.py: parses handoff entries (INDEX + dated blocks split on the rule line, fail-soft whole-file fallback), emits one "handoff" event per dated entry + a hash-gated "state" event for CURRENT STATUS. Idempotent via stable_idem_key(sha256) + SELECT-first under append-only triggers; re-index of unchanged content is a no-op, an edited entry appends a new version event. CLI main + index_handoffs/index_file public fns.
- Threaded an additive optional event_id kwarg through memory_store.append_event_memory (the single write seam) so the indexer gets deterministic ids; the 2 existing callers are kwarg-default-safe; conductor_db.append_event already accepted event_id.
- Built conductor/memory_bank_migrate.py: one-shot changelog->change / activeContext+progress->state / systemPatterns+gotchas->decision; projectIndex.md skipped (codegraph domain) unless --include-index; graceful no-op when the dir is absent/empty. Idempotent (same scheme); reuses the indexer's content-hash + existence helpers (one dedup loop, two front-ends).
- Wired producers to consumers (D-030, same changeset): memory_server.py +memory_index_handoffs +memory_migrate (@mcp.tool, WIRE-DARK[P2-B] rows mirroring P2-A); slice_api.py +memory_index_handoffs_http + memory-index CLI + POST /memory/index-handoffs. The reworked memory-writer ROLE INDEX sub-state shells handoff_indexer.py.
- Rewrote .claude/agents/memory-writer.md: ENSURE_HANDOFF -> INDEX -> VERIFY; dropped mcp_log_change/mcp_update_context/mcp_report_build/mcp_memorize + the 6-file write list; frontmatter (name/description/model: haiku) preserved.
- Retired the markdown-memory-bank doctrine: TIER-1 memorizing-and-audit.md, output-formatting.md, planning-workflow SKILL.md (3 edits), orchestrate/protocol.md (2 edits), orchestrate-protocol.md H-4; TIER-2 autonomous-orchestrator/protocol.md, main-llm-role.md (2), conditional-routing.md, token-discipline.md (2), context-compiler.md (memory-query-first directive), auditor.md (read-side, it reads memory-bank for blast radius but does NOT gate on it). The FSM MEMORIZING state + the "dispatch memory-writer" call sites are preserved and coherent.

FILES TOUCHED
- conductor/handoff_indexer.py NEW the auto-indexer
- conductor/memory_bank_migrate.py NEW the auto-migrate tool
- conductor/memory_store.py append_event_memory +event_id kwarg (additive)
- conductor/memory_server.py +memory_index_handoffs +memory_migrate MCP tools
- conductor/slice_api.py +memory_index_handoffs_http +CLI memory-index +POST /memory/index-handoffs
- .claude/agents/memory-writer.md full rework (handoff auto-index, no memory-bank markdown)
- .claude/rulebook/memorizing-and-audit.md MEMORIZING + verification reworked
- .claude/rules/output-formatting.md memory block reworked
- .claude/skills/planning-workflow/SKILL.md MEMORIZING + change manifest + context-exhaustion reworked
- .claude/skills/orchestrate/protocol.md 8.7 + documentation-drift reworked
- .claude/rules/orchestrate-protocol.md H-4 reworked
- .claude/skills/autonomous-orchestrator/protocol.md TW11 reworked
- .claude/rules/main-llm-role.md write/read memory lines reworked
- .claude/rules/conditional-routing.md Historical Audit trigger reworked
- .claude/rules/token-discipline.md projectIndex/changelog/activeContext refs reworked
- .claude/agents/context-compiler.md memory-query-first directive
- .claude/agents/auditor.md read-side coherence
- workflow/handoff/teikoku-memory-runtime.md +P2-B WIRE-DARK backing rows

USER CHOICES / DECISIONS
- Reuse models.py stable_idem_key (sha256) for the dedup id rather than blake3: it is the established in-repo idempotency primitive and conductor idem_key is sha256; consistency over the blake3 default here (flagged for the auditor, not a blocker).
- Per-ENTRY indexing grain (not per-file): the natural changelog unit, stable ids, clean timeline.
- projectIndex.md SKIPPED by the migrate tool (code topology = codegraph's domain per D-08); doctrine stops mandating its authoring (U1 lean adopted).
- context-compiler reworked as a query-memory-FIRST directive + memory-bank fallback for un-migrated projects (U2 lean: keep both worlds working, full read-side switch deferred).
- auditor.md is TIER-2 not TIER-1 (U3 resolved): it READS memory-bank for blast radius, it does NOT gate "done" on memory-bank being updated.

HOW TO PICK UP
- Boot the memory venv: /home/user/teikoku/conductor/.memory-venv/bin/python
- Re-run the auto-indexer: python conductor/handoff_indexer.py --project teikoku --json (idempotent)
- P2-C is next: the 2.5 dashboard memory surface (read the /memory/timeline,global,search routes + the new POST /memory/index-handoffs).
- The migrate tool fires only when a project WITH a markdown memory-bank is onboarded (the TEIKOKU distro has none).

KNOWN LIMITS / NEXT
- Test-project rows (project_id=test / test2) remain in the events table from the migrate self-test (append-only, harmless, isolated by project_id; not truncated per R-2).
- The python_wire_check crawl chokes (RecursionError) on conductor/.memory-venv + conductor/audit_fixtures; ran it against a clean /tmp/wirecheck mirror of conductor/*.py -> verdict PASS, 0 DARK fn (the spec-sanctioned fallback).
- WIRE-DARK[P2-B] rows for the 2 new @mcp.tool fns live in teikoku-memory-runtime.md (the established backing doc).

GROUNDING / VERIFICATION
- A1 auto-index real handoff: 2 handoff + 1 state events are real ROWS (query_events non-empty); retrievable by GREP (mode=grep, src=event:3505) AND SEMANTIC (mode=semantic+grep, src=event:3498); semantic_available=True.
- A2 idempotent: rows_before=3, re-run indexed=0/skipped=2, rows_after=3.
- A4 migrate: synthetic /tmp memory-bank ingested 4 events (2 change + 1 state + 1 decision), projectIndex skipped, idempotent re-run indexed=0, absent-dir clean no-op, grep-retrievable (event:3504).
- A5 wiring: python_wire_check.py PASS, dark_fn_count=0; new MCP tools REGISTERED-DARK (WIRE-DARK[P2-B] + backing rows); index_handoffs WIRED by slice_api + memory_server; migrate WIRED by memory_server.
- A6 P2-A regression: test_db_schema + test_events_replay = 4 passed.
- A7 R10 smoke: driven through MCP (memory_index_handoffs + memory_search hits=3) + slice_api (CLI index + search mode=semantic+grep count=5) + grep path (hits=2).
- A3 doctrine: TIER-1 grep confirms no surviving mandate to author memory-bank as the memory step; all name the handoff auto-index; FSM MEMORIZING + dispatch call sites intact.
- Harness: 22 agents enumerate at distro-root .claude/agents; memory-writer frontmatter parses (name/model: haiku); no divergent projects/elai_cc copy; all 5 conductor modules import OK.
