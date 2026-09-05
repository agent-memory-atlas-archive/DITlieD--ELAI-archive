B-15B — Decision Doc
=====================
Phase: B-15B (HashMap ban replacement + CI lint gate)
Session: s-27276

WHAT
----
Replace `use std::collections::HashMap` (and qualified `std::collections::HashMap::`) in the 46
production .rs files identified by re-baseline. Author check_hashmap_ban.sh + .bat that fails
on any future violation in non-test code. Tighten as a CI gate.

KEY DECISIONS
-------------
D1. **BTREE-everywhere strategy** (deviates from spec's per-file optimal).
    User is asleep; rustc-hash dep needs explicit approval per CLAUDE.md "No new dependencies
    without confirmation" hard rule. To make progress without blocking, ALL 46 files get
    BTreeMap. Trade-off: small perf cost on lookup-heavy paths vs achieving full spec goal of
    ban + lint gate now.
D2. Follow-up: open B-15B-FX as a future micro-phase after user approves rustc-hash. That
    phase swaps BTreeMap → FxHashMap on the 39 lookup-only paths the context-compiler
    identified. Estimated ~30-40 LOC delta, mechanical.
D3. import_extractor.rs:399 is cfg(test)-scoped but lives in src/. The lint script must whitelist
    cfg(test)-gated code, OR the file should be moved to tests/. Choosing the simpler path:
    lint script grep excludes any line wrapped in cfg(test) blocks (best-effort; documented
    edge case).
D4. ChoiceId / Language Ord derives — execution agent must verify these exist before BTREE
    substitution; if not, add #[derive(Ord, PartialOrd, Eq, PartialEq)] minimally.
D5. String literals containing the word "HashMap" in reranker.rs / self_mod_layer1a.rs /
    self_discovery.rs MUST NOT be changed (these are doc/log strings, not type usages).

EXPECTED TEST OUTCOMES
----------------------
- cargo check --workspace PASS after all replacements.
- cargo test --no-run PASS for all touched crates.
- check_hashmap_ban.sh exits 0 (no violations remaining in non-test code).
- HashMap counter on baseline drops from 46 → 0.

UNKNOWNS
--------
- Whether some BTreeMap key types lack Ord (will fail at compile; agent must add derives).
- Whether any public API surface used HashMap in return types — those need a re-export shim
  or a documented breaking change. (Risk-LOW per context-compiler review.)

ROLLBACK
--------
git restore on the 46 files + the new lint scripts. No external state.

ACCEPTANCE
----------
A1: cargo check --workspace PASS.
A2: cargo test --no-run for affected crates PASS.
A3: check_hashmap_ban.sh runs and exits 0.
A4: 46 → 0 violation files in non-test code (verified by the lint script).
A5: B-15B-FX follow-up logged for post-rustc-hash-approval phase.
