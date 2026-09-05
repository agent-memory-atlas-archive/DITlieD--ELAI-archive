K1-svelte-sort-investigation.md

Session: s-22600 (Tranche K, ITEM K.1)
Date: 2026-06-01
Decision: NO Svelte source edits required. Tranche E Rust-side BTreeMap migration is sufficient.

PANELS REVIEWED (4)

K.1.a HeldOutPanel.svelte L106 -- sourceList: Object.entries(stats.by_source).sort((a, b) => b[1] - a[1])
  Classification: VALUE-SORT (count DESC). Preserves "top sources first" display affordance.
  Decision: KEEP AS-IS. BTreeMap migration yields KEY-sorted iteration (alphabetical source ID); a value-sort on count cannot be replaced by a key-sort upstream. Stripping it would silently demote the ranking UX to alphabetical, a regression for the user.

K.1.b CanaryPanel.svelte L107 -- familyRows: breakdown.by_report[reportId]
  Classification: NO SORT, per-key dictionary lookup. Grep verified zero matches for .sort / Object.keys / Object.entries across the file.
  Decision: NO EDIT. Panel never iterates by_report; BTreeMap key-order guarantee is unused but harmless.

K.1.c LockedKeysPanel.svelte L114 -- violationsFor: recent.violations_by_audit[auditId]
  Classification: NO SORT, per-key dictionary lookup. Grep verified zero matches.
  Decision: NO EDIT. Same pattern as K.1.b.

K.1.d PlanningLedgerPanel.svelte -- firings_by_canary declared @L60, seeded @L69, NOT iterated visually. Visible firings table iterates drift.firings.slice(0, 20) @L254 (a sibling array, not the map). The only .sort() in the file is reasonHistogram @L129-137 which sorts shortcuts: AgentlessShortcutRow[] (unrelated array DTO).
  Decision: NO EDIT. firings_by_canary is never rendered; reasonHistogram sort is on a different field and is value-sort (preserve).

NET OUTCOME: ZERO Svelte files modified in Tranche K.
  - 1 panel (HeldOutPanel) has a value-sort that MUST be preserved.
  - 3 panels (CanaryPanel / LockedKeysPanel / PlanningLedgerPanel) have no sort to drop.
  - The Tranche E migration is silently absorbed: wire-side determinism gained, client behavior unchanged.

DASHBOARD-OWNER REVIEW NOTE (carry into handoff)
If the dashboard owner WANTS alphabetical (key-sorted) source IDs in HeldOutPanel rather than count-ranked, the edit is a 1-line swap at HeldOutPanel.svelte:106 from `.sort((a, b) => b[1] - a[1])` to nothing (rely on BTreeMap iteration order). This is a UX decision, not a tech-debt cleanup; defer to owner. Posted to workflow/audits/ governance forum.

RATIONALE (anti-laziness)
The original phrasing "drop client-sort since BTreeMap guarantees deterministic alphabetical order" assumed each panel sorted by KEY. After reading the 4 panels, the only existing .sort() on a migrated DTO is HeldOutPanel.sourceList -- and it sorts by VALUE (count desc), not key. The grounded "non-lazy" call is to NOT strip a value-sort because a key-sort exists upstream.

STUB CLOSURE
Updates B15-dashboard-BTreeMap row in s-22300 stubs.md: SVELTE-PENDING -> SVELTE-CLIENT-SORT-INVESTIGATED-NO-EDIT.
