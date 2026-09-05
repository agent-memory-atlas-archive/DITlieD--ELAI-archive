E-45 SP2 (SsbActionType golden-file lock) + SP3 partial (cross_lang_diff.py extension)
Session: s-26556
Scope: A. golden lock file + Rust test; B. cross_lang_diff.py --check-action-types flag
Out-of-scope this session: SP1 ts-rs snapshot crate (Task 1), Task 4-6 (schemars / Pydantic / runtime validation in python/elai_inference)

DECISION 1 -- Alphabetical sort over enum-declaration order for the golden body
Why: stable PR diffs. With declaration order, every new variant appended to the end of the enum changes the file by appending a single line at EOF, which is fine -- but renaming an existing variant or inserting a variant mid-enum would reshuffle every subsequent ordinal, making the diff noisy and hiding the actual semantic change. With alphabetical order, an insertion only shifts the ordinals of strictly-later-alphabetical entries; renames produce a clean +line/-line pair adjacent in the diff; removals stand alone. The implementation never relies on the alphabetical order being "the contract" -- the contract is the SET of wire tags -- but the lock-file representation needs a deterministic ordering, and alphabetical wins on diff readability.
Source: chose stable sort over insertion-order after considering both. Same pattern used by `cargo deny`'s deny.lock and most schemars JSON Schema emitters that key-sort object members.

DECISION 2 -- Count_is_locked test alongside golden_matches_runtime
Why: catches a class of bug the byte-equality test alone misses. Specifically: if a future PR renames `Self::FileWrite => "file_write"` to `Self::FileWrite => "file_w"`, the runtime tag set changes by exactly one (file_write removed, file_w added), so the golden_matches_runtime test catches it. Good. But if a future PR renames `Self::FileWrite => "file_write"` AND `Self::FileRead => "file_w"` (swap that is a logic error but preserves the SET), set-based comparison passes. The count_is_locked test does TWO things: asserts ALL_VARIANTS.len() == TOTAL_VARIANTS (catches add/remove mismatch on the const TOTAL_VARIANTS bump), AND asserts no duplicate wire tags from as_sql_str (catches the swap-collision case where two variants both emit the same SQL string).
Source: counterfactual reasoning per CLAUDE.md CSQ rule -- "if as_sql_str() has a typo collision, what happens?"

DECISION 3 -- Hand-locked ALL_VARIANTS array + exhaustive_match_guard instead of strum::EnumIter
Why: ssb.rs is a 7-site lockstep file per its own header comments (lines 39, 119, 132, 145, 261, 287, 454, 485, 515, 577, 1058, 7998). Adding `EnumIter` to the derive list at line 1642 is a touch that future readers must reason about ("does this new derive affect serde representation? affect Default? affect Hash determinism?"). Keeping ssb.rs READ ONLY this session and putting the enumeration burden inside the TEST FILE keeps the lockstep tight. The exhaustive_match_guard fn is a zero-runtime-cost compile-time gate: rustc's exhaustiveness checker fires hard at the test crate compile step if a variant is added to ssb.rs without being added to ALL_VARIANTS, and fires equally hard if a variant is removed from ssb.rs while still listed in ALL_VARIANTS. Both directions of drift surface at the test-crate build level -- before any test executes.
Source: scope rule "SsbActionType enum READ ONLY" (context-package WIRING_CONTRACT stub_ban).

DECISION 4 -- File comment header carries the regen command, append-only contract, and "8th lockstep site" callout
Why: the file IS the lockstep documentation. The header tells the next reader (a) exactly which cargo command to run to refresh it, (b) that arbitrary reordering is forbidden, and (c) that this is the 8th place a variant must be touched. Without the callout, a future PR author who reads only the enum's lockstep comment (which lists 7 sites) will miss the golden file.
Source: ssb.rs:39 lockstep block + similar pattern in C-1 grounding-sweep-cache golden file.

DECISION 5 -- Variant count is 288, not the planner-stated 285
Why: re-counted by parsing as_sql_str() match arms with regex -- 288 unique (variant, wire_tag) pairs, no duplicate variants, no duplicate tags. Planner's 285 figure was approximate; this lock file now nails the exact count.

DECISION 6 -- cross_lang_diff.py --check-action-types is opt-in, not part of default behavior
Why: back-compat. The existing pair-based drift check exits 0/1 on the existing PAIRS list. Callers in pre-commit hooks (deferred Task 8) and CI may already wrap this script with their own retry/skip logic. Bolting a cargo subprocess into the default path would (a) slow down every invocation by ~10s on a warm cache and ~5min on a cold cache, and (b) make the script fail in environments without cargo (the workflow harness itself, when run on a machine without Rust toolchain). Opt-in keeps the existing contract clean and lets the future pre-commit / CI wire-up decide when to flip the flag.

DECISION 7 -- Treat cargo-not-on-PATH and cargo-timeout as drift (returncode != 0)
Why: silent skip on missing cargo would let a broken environment hide real drift. The operator MUST see the message. The CLI prints a structured stderr message ("cargo not on PATH; install via rustup or set CARGO env var.") that points at the fix.

DECISION 8 -- 300s timeout on the cargo subprocess
Why: warm-cache `cargo test -p elai-orchestrator --test action_type_lock` runs in ~12s on this machine. Cold cache (full elai-orchestrator compile from scratch) runs in ~70s. 300s gives 4x headroom for slow CI runners. Higher than that and the operator should see a timeout error rather than wait indefinitely.

DEFERRED -- still owed to E-45
- SP1: ts-rs snapshot crate (crates/ts_bindings + tests/snapshot.rs + workspace member + ts-rs dep). Blocked on: planner's claim "B-3 SHIPPED ts-rs" is FALSE in current main; ts-rs is not in Cargo.lock. Adding it is a workspace-level change with anti-slopsquatting verification required (verify ts-rs v12.x on crates.io via `cargo search ts-rs` BEFORE pinning). Separate effort, not bundled with this SP2+SP3 run per scope.
- Task 4: schemars derive on Rust types that mirror to Python. Out of scope -- requires touching python/elai_inference/ (user-forbidden this run).
- Task 5: Pydantic mirror of schemars output. Same as Task 4.
- Task 6: runtime validation gate. Depends on Tasks 4+5.
- cross_lang_diff.py --strict and --report modes from the planner's SP3 spec. Scope here is the action_type-lock flag only; --strict would shell out to the snapshot test (deferred with SP1) and --report would emit a markdown audit table (deferrable until E-41 wires up CI).
- pre-commit hook integration (Task 8). Deferred per planner scope.
- SCHEMA_VERSION compat window (Task 9). Note: ssb.rs current is 97, planner said 45 (stale memory). Out of scope this run.

EVIDENCE -- this session
1. cargo check -p elai-orchestrator --tests : PASS (1m 09s, 19 warnings -- all pre-existing, none from the new test file).
2. cargo test -p elai-orchestrator --test action_type_lock : 2 passed, 0 failed, 1 ignored. ok.
   - action_type_lock_count_is_locked : ok (TOTAL_VARIANTS = 288, ALL_VARIANTS.len() = 288, no dup tags)
   - action_type_lock_golden_matches_runtime : ok (golden body 288 lines, runtime 288 lines, byte-equal)
   - regen_golden_file : ignored (helper only)
3. python .claude/tools/cross_lang_diff.py --help : shows --check-action-types in the options list.
4. python .claude/tools/cross_lang_diff.py --files "noop.txt" --check-action-types : exit 0 (no pair drift, no action_type drift).
