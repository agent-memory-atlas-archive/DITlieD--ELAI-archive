Phase 1A.3 decisions — TDD test writing for knowledge/DKG engine

What: Created 27 integration tests in crates/knowledge/tests/dkg_tests.rs. Added build_graph() todo!() stub and 3 edge-type constants (EDGE_IMPORTS, EDGE_CONTAINS, EDGE_IMPLEMENTS) to dkg/mod.rs. Added tempfile, rusqlite, blake2 to dev-dependencies.

Key decision: dkg_readiness before build_graph asserts Unavailable (per spec), not Empty (per current impl). This creates an intentional TDD red that 1A.4 must fix by adding a build-invocation sentinel (struct field or dkg_meta row). The auditor confirmed this is correct.

Key decision: test helpers open a second rusqlite::Connection to the same WAL-mode DB for insert setup, avoiding unsafe access to DkgEngine's private conn field.

Expected test outcome: 27 tests, 11 fail (TDD red). 10 fail on build_graph todo!() panic. 1 fails on readiness assertion mismatch (Empty != Unavailable). 16 pass via direct SQL setup + public query API.

Unknowns for 1A.4: build_graph needs to (a) walk project directory for .rs files, (b) parse with tree-sitter via RustGrammar, (c) populate dkg_nodes/dkg_edges/dkg_files, (d) compute blake2s file hashes, (e) set a "build_initialized" sentinel so dkg_readiness returns Empty (not Unavailable) for 0-node post-build state.
