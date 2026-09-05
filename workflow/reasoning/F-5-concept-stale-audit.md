F-5 concept_stale read-consumer audit (Slice A2, Phase 1)
session: s-60824
date: 2026-06-02

AUDIT RESULT: concept_stale has ZERO production read consumers.

Evidence (fresh grep of .elai_cc, 2026-06-02):
- crates/knowledge/src/dkg/mod.rs:36  -> schema DDL: `concept_stale INTEGER DEFAULT 1` (column declaration only).
- crates/knowledge/src/dkg/mod.rs:369-370 -> the SOLE production INSERT: column list includes
  concept_stale and the VALUES clause hardcodes the literal `1`
  ("... start_line, end_line, concept_stale, language) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1, ?7)").
  No bind parameter; always 1 on insert.
- crates/orchestrator/src/ssb.rs:2843,4550,8273,25421,25435,25453 -> prose/doc comments only
  (F-5 cascade design notes describing concept_stale=2), not code reads.
- crates/orchestrator/workflow/audit-scratch/test_a48_data_flow_invariants.py:69,145 -> Python
  test fixtures declaring the column in a throwaway schema; not production.

No SELECT ... concept_stale, no WHERE concept_stale, no query_map/get on the column anywhere in
production Rust. Confirmed: nothing reads the value.

MARKER SEMANTICS (locked by F-5 design):
- 0 = explicitly fresh
- 1 = default (DDL default + the value the sole INSERT hardcodes); legacy rows are all 1
- 2 = quarantined by the staleness-quarantine cascade

The cascade only ever WRITES 2; it never disturbs legacy 1s.

LEGACY-ROWS-READ-AS-1 HAZARD (forward-looking):
Because every existing row is 1 by construction (default + hardcoded insert), if a FUTURE read
filter is added it must NOT treat 1 as "fresh". 1 is the ambiguous "unknown/default" state, not a
freshness assertion. A future consumer that wants "fresh only" must filter `concept_stale = 0`, and
one that wants "exclude quarantined" must filter `concept_stale != 2` (NOT `concept_stale = 1`,
which would also exclude any explicitly-fresh 0 rows). Phase 1 adds no read filter, so this is a
hazard note for whoever adds the first reader, not a present-day bug.

PHASE-1 OBSERVABLE:
With zero read consumers there is no read filter to test. The observable is the UPDATE write
round-trip: the new public mutator DkgEngine::set_concept_stale(concept, 2) writes 2, and a
re-query reads back 2. The contract test (crates/contracts/tests/f5_concept_stale_retrieval_contract.rs)
asserts exactly this round-trip plus WHERE-scoping (non-matching concept untouched at 1) and the
zero-rows no-match case.

JOIN CRITERION (documented for the cascade caller):
dkg_nodes has no fact_id column. The fact->concept association is by the `concept` TEXT column.
set_concept_stale therefore matches on `WHERE concept = ?1` (text equality), consistent with the
F-5 design correction that the cascade joins fact->concept via the concept TEXT column.
