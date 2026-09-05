Phase 1D Step 1 decisions

What: add pagerank module skeleton to knowledge crate (crates/knowledge/src/pagerank/mod.rs).

Key decisions:
- PageRankNodeInfo (not DkgNodeInfo) for the pagerank-local node struct. lib.rs already exports DkgNodeInfo with different fields. Avoids glob import collision.
- token_count_estimate does not exist in crates/providers/src/ (0 grep hits). compute_repo_map takes &dyn Fn(&str)->usize closure instead, decoupling from providers. This gap is a future-phase deliverable, not a blocker.
- proptest = "1" already in crates/knowledge/Cargo.toml dev-dependencies (line 27). No re-add needed.
- todo!() bodies permitted for skeleton step in exactly two places: compute_repo_map and DkgSnapshot::from_engine. All other code is fully implemented.
- 21 named test stubs plus proptest block. All tests compile but panic at runtime via todo!().

Expected test outcome: cargo check passes with 0 errors. cargo test panics on all 21+ tests (expected: "not yet implemented").

Unknowns:
- token_count_estimate provider function (future phase).
- Actual PageRank algorithm parameters may need tuning during P1D.2+.

Audit WARNs for P1D.2:
- DkgSnapshot.edges uses usize positional indices but node_id is i64 DB row ID. from_engine() MUST build HashMap<i64, usize> for non-contiguous ID mapping.
- Contract test file needed in crates/contracts/tests/pagerank_contract.rs before P1D.2.
- Remove #[should_panic] from proptest block when compute_repo_map is implemented.

Blast radius: new file pagerank/mod.rs + one-line addition to lib.rs. No existing function bodies modified.
