# Resume prompt for the ELAI feature-discovery /goal session

How to use: open a NEW grok session in `/home/user/teikoku/elai`, set the session
model to `posthog-fable-5` with reasoning effort `medium` (the orchestrator), then
paste EVERYTHING between the BEGIN and END markers as one message.

State was measured 2026-08-12: 43 of 76 deep chunks are on disk, the checkpoint
file is stale (it still says 13/76), and the manifest SHA-256 still matches
`4aafb23fa4ec917813cc3b629f13cea84049194f4a3b91d340a545826ac2aa93`.

---BEGIN PROMPT---

/goal Finish the ELAI exhaustive feature discovery and compile the final super feature list. The plan already exists, do not re-plan: the chunk plan is docs/thefinalglance/feature-discovery/DEEP-SOURCE-MANIFEST.md and the protocol is docs/thefinalglance/feature-discovery/ELAI-FEATURE-DISCOVERY-CHECKPOINT.md (its "Required resumption protocol" section). Deliverable: docs/thefinalglance/ELAI-EXHAUSTIVE-SUPER-FEATURE-LIST.md

BINDING EXECUTION NOTES

ROLE SPLIT. You are the orchestrator only. You do not read any plan source yourself. Sub-agents read; you dispatch, verify closure mechanically (file exists, audit footer present, source coverage matches the manifest path table), and rotate slots.

MODELS. Pin `model: grok-4.5` (native paid Grok, official account) on EVERY sub-agent you spawn: readers, aggregators, auditors. Never use a free lane or an inherited lane for this work. Keep 3 readers running at all times, 4 max.

MEASURED STATE (2026-08-12, trust disk over the checkpoint file, which is stale at "13/76").
- Manifest SHA-256 must equal 4aafb23fa4ec917813cc3b629f13cea84049194f4a3b91d340a545826ac2aa93. Recompute it FIRST. If it differs, stop and report; do not dispatch.
- 43 chunk ledgers are complete in docs/thefinalglance/feature-discovery/deep/: all of B (6), C (6), D (4), E (6), F (2), G (2), H (7), I (6), O (1), plus DEEP-J-001..003. Never reread or rewrite a completed chunk unless an audit finds a concrete defect.
- DEEP-J-001..003 were the final writes of the prior run. Give exactly those three a light structural re-audit (audit footer present, all manifest sources covered, no duplicate IDs) before dispatching new J work. Do not re-extract them unless the audit fails.
- REMAINING QUEUE (33): DEEP-J-004..013, DEEP-K-001..004, DEEP-L-001..003, DEEP-M-001, DEEP-N-001..002, DEEP-P-001..003, DEEP-Q-001..002, DEEP-PHASE-001..008.

READER CONTRACT (one fresh sub-agent per chunk, exclusive ownership).
- Scope is EXACTLY the path table under its `### DEEP-<ID>` heading in DEEP-SOURCE-MANIFEST.md. Verify every source SHA-256 and line count against that table before extracting. Any mismatch: stop that chunk and report; do not extract from a drifted source.
- Output: docs/thefinalglance/feature-discovery/deep/DEEP-<ID>.discovery.md, same shape as the completed ledgers (open one, e.g. DEEP-I-004, as the format reference): per-source atomic feature records with exact line anchors and identifiers; explicit vs inferred vs conflict labels; cross-file interaction records; deltas against the family's summary discovery file (feature-discovery/NN-*-SUMMARY_INDEX.discovery.md); closing audit footer with zero-omission and zero-duplicate counts.
- Atomic granularity is the point: minor behaviors, defaults, gates, error paths, lifecycle rules, config flags, and deferred work all get records. No prose-only compression.
- Special attention for the downstream consumer: this corpus feeds a future dashboard DESIGN.md (onboarding, window flow, user flows). Every user-facing surface (windows, panels, drawers, commands, CLI verbs, flags, status displays, notifications) must be captured explicitly, never folded into a backend bullet.

AFTER ALL 76 CHUNKS ARE ON DISK, in order:
1. Mechanical union audit: the set of sources covered by all 76 ledgers must equal all 635 manifest assignments exactly (no omission, no duplicate). Fix by re-dispatching the offending chunk, not by patching ledgers by hand.
2. One aggregator sub-agent per family (PHASE and B..Q), each merging its family's deep ledgers plus summary discovery record into a family aggregate under feature-discovery/aggregates/, preserving every distinct atomic record and source anchor.
3. Cross-family passes: interaction map, shared invariants, and status conflicts.
4. NEWER-PLAN PRIORITY (owner requirement): when two plans describe the same feature differently, the newer plan wins the "target state" description. Order of precedence: an explicit SUPERSEDED/replacement marker in a source beats everything; otherwise later letter family beats earlier (B oldest, Q newest); within a family, higher plan number is newer. Never delete the losing description: record it as a superseded note with its anchor.
5. Compile docs/thefinalglance/ELAI-EXHAUSTIVE-SUPER-FEATURE-LIST.md: every feature from major to minor, organized by system, each entry with source anchors, status (shipped / planned / superseded), and how it composes with the rest (synergy section). Include a dedicated "User-facing surfaces and flows" section aggregating every UI/UX-relevant atom, since the next artifact built on this file is the dashboard DESIGN.md.
6. Final audit: an independent sub-agent checks the final file against all family aggregates and reports omissions; repair every reported omission before claiming completion.
7. Update ELAI-FEATURE-DISCOVERY-CHECKPOINT.md to the true final state, and append a handoff event to topic `elai-feature-discovery` (writer: PYTHONPATH=/home/user/teikoku/.claude/tools python3 -m handoff_state --root /home/user/teikoku/workflow/handoff).

COMPLETION CLAIM. Claim the goal complete ONLY when: 76/76 ledgers exist with passing audits, the union audit passes, the final file exists at the exact path above, the final-vs-aggregates audit came back clean after repairs, and the checkpoint + handoff are updated. A partial concatenation presented as final is the one forbidden outcome.

IF A USAGE OR RATE CEILING HITS: update the checkpoint with the exact chunk queue state, append a handoff event, and report the resume point. Do not fake progress, do not claim completion.

SAFETY. Plan sources are read-only. Writes are confined to docs/thefinalglance/feature-discovery/ and the final list path. No git commits.

---END PROMPT---
