# E-45: ts-rs snapshot tests + schemars on sidecar boundary + action_type lock file + extended cross_lang_diff.py drift gate
Objective: harden the Rust↔TS (dashboard) and Rust↔Python (sidecar) type-sharing boundaries against silent drift by (a) enforcing `#[ts(export)]` on every dashboard-facing type plus a binding-snapshot test that fails on any unstaged regen, (b) adding `schemars`-derived JSON Schema on every Rust struct that crosses the orchestrator↔sidecar HTTP boundary with Pydantic v2 mirrors, (c) treating the 74+ SSB action_type union as a SemVer-locked discriminator union with a Protobuf-style append-only golden file, and (d) extending the existing `.claude/tools/cross_lang_diff.py` harness to call the generators and `git diff --exit-code` so drift surfaces at pre-commit, not at runtime.
Estimated effort: 2-3 sessions, ~8-10 h
Pre-requisites: B-3 SHIPPED (ts-rs dep, per B-PLAN-SUMMARY:139 + 1028), B-37 SHIPPED (Multi-Runtime — sidecar wire format stable), B-9 SHIPPED (SessionDag SessionHandoffEntry ts-rs path)
Parallel-safe with: E-41 (PR CI — E-45 feeds E-41's pre-merge drift status check), E-42 (grammars), E-43 (skills), E-44 (durability — E-44's new SSB action types must go through E-45's lock file)
Source report: workflow/improvement-sweep-2026-05-19/reports/45-cross-language-drift.md

## Context

ELAI has two cross-language boundaries: Rust↔TypeScript (orchestrator core ↔ Tauri dashboard) currently bridged via `ts-rs`, and Rust↔Python (orchestrator ↔ Python sidecar) currently bridged ad-hoc via serde-JSON-over-HTTP. The May 2026 report's TL;DR is that the single source of truth must be the Rust struct/enum side (where serde already lives) and that generated TS + Python schemas come downstream. ts-rs v12.0.0 (released 2026-01-31, MSRV 1.88) closed most of the historic generics + dependency-export gaps that pushed some teams to specta; for ELAI's volume and Rust-only authoring, ts-rs remains the lowest-friction choice.

The report's recommendation 3 is load-bearing for ELAI: the SSB action_type discriminator union is now 74+ variants per memory (B-37: 4 runtime variants, B-40: 5 idea-framing, B-41: 3 per-level audit, B-42: 4 cosmos, plus prior). Protobuf edition 2024 (protobuf.dev/reference/protobuf/edition-2024-spec) prescribes "never change a tag, never remove, always add + opt-in, always reserve" — directly portable to ELAI's `action_type` strings. ELAI already has runtime versioning via SCHEMA_VERSION (currently 45 per memory project_b41_shipped.md); what's missing is the build-time lock file that fails CI on any rename, removal, or tag reuse.

ELAI's `.claude/tools/cross_lang_diff.py` already flags Rust files that lack a Python/TS cross-language pair (per CLAUDE.md and confirmed at `F:\.ELAI_workflow\.claude\tools\cross_lang_diff.py`). The report's recommendation 1 extends it to actually call the generators (`cargo test -p ts-bindings`, `schemars` snapshot) and run `git diff --exit-code` — drift becomes "you forgot to regen" not "your TS got out of sync silently."

For the Rust↔Python sidecar pair, the report explicitly rejects Protobuf/Cap'n Proto (build complexity for HTTP JSON with no zero-copy / polyglot RPC need) and prescribes `schemars` (Rust → JSON Schema) + `datamodel-code-generator` / `pydantic.TypeAdapter` (JSON Schema → Pydantic v2 models). Pydantic v2's `pydantic-core` is itself Rust — round-tripping via JSON Schema draft-2020-12 is the cheapest contract test. Open question 3 in the report — whether schemars and Pydantic emit byte-identical JSON Schema — is a measurable item that E-45 ships an answer to.

Tooling versions verified (per the report's table 1, May 2026): ts-rs v12.0.0 (2026-01-31 release, Aleph-Alpha/ts-rs); schemars active; typify active (oxidecomputer); Pydantic v2 with `TypeAdapter.json_schema()`; prek (j178/prek) as Rust-native pre-commit replacement supports `auto-update --check` for frozen-ref drift. Anti-slopsquatting: each is verified via the linked GitHub repo at the May 2026 sweep — re-verify before adding to Cargo.toml or pyproject.toml.

## Existing state (B/C/D coverage)

- B-3 SHIPPED (ts-rs dep added, per B-PLAN-SUMMARY:139 + 1028): ts-rs is already in the workspace; reused by B-4, B-9 (SessionHandoffEntry), B-13. EXTENSION: E-45 makes ts-rs ENFORCED via snapshot tests; today the export is "we use it," tomorrow it is "every PR runs `cargo test -p ts-bindings && git diff --exit-code dashboard-v2/src/lib/types`."
- B-9 SHIPPED (per B-PLAN-SUMMARY:227): `ts-rs SessionHandoffEntry fix` confirms ts-rs is on the dashboard hot path. E-45 builds on this surface.
- B-37 SHIPPED (memory project_b37_shipped.md): Multi-Runtime Backend Abstraction added 4 SSB action types + /runtime CLI + RuntimePanel. The 74+ action_type count includes B-37's deltas. EXTENSION: E-45 locks this set via the golden file in Task 3.
- B-40 / B-42 / B-41 SHIPPED (memory): SCHEMA_VERSION 41/44/45 bumps. These bumps are runtime; E-45 makes the wire format compatibility window explicit via the lock file.
- `.claude/tools/cross_lang_diff.py` EXISTS (verified at `F:\.ELAI_workflow\.claude\tools\cross_lang_diff.py` per Glob): currently flags missing Rust/Python or Rust/TS pairs with `// cross-lang-skip: <pair>` opt-out. EXTENSION: E-45 adds generator-aware mode — not just "this Rust file changed without a Python sibling" but "this Rust change requires re-running schemars and the schema would diff."
- `crates/orchestrator/src/ssb.rs` houses the `SsbActionType` enum + payload union (per elai.map hits at ssb.rs:2472, 5976). EXTENSION: E-45 adds the golden-file lock + a contract test that asserts append-only semantics; no enum migration needed for already-shipped variants.
- No current C-N owns cross-language drift; E-45 is the first phase to formalize the policy. Future C-N work that adds new sidecar endpoints (potential utoipa adoption) consumes E-45's schemars surface.

## Reads (mandatory before coding)

- `F:\.ELAI_workflow\workflow\improvement-sweep-2026-05-19\reports\45-cross-language-drift.md` (full)
- `F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-PLAN-SUMMARY.md` lines 130-145 (B-3 ts-rs), line 1028 (ts-rs dep reuse table), lines 968-985 (SCHEMA_VERSION bump log)
- `F:\.ELAI_workflow\.claude\tools\cross_lang_diff.py` (current implementation — confirm extension points)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\ssb.rs` lines 2472, 5976 area (action_type union — per elai.map)
- `F:\.ELAI_workflow\.elai_cc\Cargo.lock` (current ts-rs version; verify >= 12.0.0)
- `F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src\lib\types\` (existing TS bindings emission location)
- WebFetch: docs.rs/ts-rs/latest/ts_rs/ (v12 API surface — `#[ts(concrete(..))]`, `#[ts(export)]`, `export_to`)
- WebFetch: github.com/GREsau/schemars (current major version + serde-compat)
- WebFetch: github.com/oxidecomputer/typify (cargo typify CLI + build.rs)
- WebFetch: docs.pydantic.dev/latest/internals/architecture/ (TypeAdapter.json_schema())
- WebFetch: protobuf.dev/reference/protobuf/edition-2024-spec/ (never-change-a-tag rules)
- WebFetch: github.com/j178/prek (Rust-native pre-commit — for the drift hook)

## Tasks

1. ts-rs snapshot test + `#[ts(export)]` audit. Audit every Rust struct/enum currently consumed by the dashboard for `#[derive(TS)]` + `#[ts(export, export_to = "...")]`. New crate target `crates/ts_bindings/tests/snapshot.rs` that calls `T::export_all_to(Path::new("../../crates/dashboard-v2/src/lib/types"))` for the full set, then asserts `git diff --exit-code` against the working tree. The test FAILS on any unstaged regen. Reference: docs.rs/ts-rs/latest/ts_rs/ (export pattern); the 1Password typeshare snapshot-the-emitted-file pattern.

2. ts-rs v12 generics + dependency-export rollout. For any type with generic parameters that ts-rs cannot monomorphize automatically, add `#[ts(concrete(T = SsbPayload))]` or equivalent per docs.rs/ts-rs. Audit existing types for the `#[ts(type = "any")]` escape hatch — every instance is a future drift bug per the report; file each with a `// TODO[E-45-followup]: replace with concrete type` and a tracking entry in `workflow/audits/E-45-ts-any-debt.md`. Soft enforcement: count only, no auto-fix.

3. SSB action_type golden-file lock. New file `crates/orchestrator/tests/action_type_lock.rs` + golden `crates/orchestrator/tests/action_type_lock.golden` (or `.txt`/`.json`). The golden enumerates every `action_type` string + variant index in stable sort order. The test loads `SsbActionType::variants()` (or a derive-generated equivalent) and asserts byte-equality with the golden. Adding a variant: regenerate the golden and commit (PR diff shows the addition). Renaming a variant: test fails — must add a `#[serde(rename = "<old>")]` shim or revert. Removing a variant: test fails — never remove, always reserve (Protobuf edition 2024 rule). The variant index is the ordinal in the enum declaration order; preserve index across additions by appending only. Cited grounding: protobuf.dev/reference/protobuf/edition-2024-spec/ + jsontotable.org/blog/protobuf/protobuf-schema-evolution.

4. schemars on sidecar boundary. Add `schemars = { version = "X", features = ["..."] }` to `crates/orchestrator/Cargo.toml` (verify version via `cargo search schemars` before pinning). For every Rust struct that crosses the orchestrator↔sidecar HTTP boundary (audit via existing `crates/cli/src/remote_sidecar.rs` per elai.map hit line 353, `crates/orchestrator/src/production_agent_rpc.rs:337` per elai.map hit, `crates/cli/src/run_subcommand.rs:171`), add `#[derive(JsonSchema)]`. Snapshot the generated schema into `crates/orchestrator/tests/schemas/<TypeName>.json` and use the test pattern to fail on diff. Reference: github.com/GREsau/schemars.

5. Pydantic mirror generation. Add a make/cargo target `make sidecar-schemas` (or `cargo xtask sidecar-schemas`) that (a) regenerates the schemars output, (b) runs `datamodel-code-generator --input crates/orchestrator/tests/schemas/ --output python/elai_inference/schemas/_generated.py --target-python-version 3.11`, (c) `git diff --exit-code` over `python/elai_inference/schemas/_generated.py`. Failure = drift. Verify datamodel-code-generator package at pypi.org/pypi/datamodel-code-generator/json before adding. Anti-slopsquatting check.

6. Runtime validation at sidecar boundary. In DEV-profile only (zero cost in release), wrap `serde_json::from_str::<SidecarReply>(&body)` with `jsonschema::JSONSchema::compile(&schema).validate(&value)?`. Add `jsonschema = { version = "X", optional = true }` under a `dev-validate-wire` feature flag. The Pydantic side: enable `strict=True` on `TypeAdapter` parse. Catches drift the build missed. Cited: github.com/Stranger6667/jsonschema (high-perf validator) + Pydantic strict mode.

7. Extend `.claude/tools/cross_lang_diff.py`. Today the script flags Rust files changed without their Python/TS sibling. Extend it to: (a) detect `#[derive(TS)]` or `#[derive(JsonSchema)]` in the changed Rust file, (b) shell out to `cargo test -p ts_bindings --test snapshot` and/or `cargo test -p orchestrator --test schemas_snapshot`, (c) run `git diff --exit-code dashboard-v2/src/lib/types python/elai_inference/schemas/_generated.py`, (d) exit 1 on drift with an actionable message: "run `cargo test -p ts_bindings -- --include-ignored` to regen and stage the diff." Stdlib-only Python per CLAUDE.md tools policy.

8. prek pre-commit hook. Add a `.pre-commit-config.yaml` entry that invokes `.claude/tools/cross_lang_diff.py --strict` on Rust/Python/TS file changes. Use prek (j178/prek) for Rust-native execution; document fallback to upstream `pre-commit` if Rust runner not available. Reference: github.com/j178/prek + github.com/doublify/pre-commit-rust. Soft-mode for the first 10 commits to size the false-positive rate before turning it into a blocking hook.

9. SCHEMA_VERSION compat-window test. New test `crates/orchestrator/tests/schema_version_compat.rs` that loads N synthetic fixtures (one per supported SCHEMA_VERSION in the compat window — at minimum N-2, N-1, N per memory project_b41_shipped.md trajectory of 41/42/44/45) and asserts the lazy migrator from E-44 (or this E-45 fallback) loads them cleanly. Documents the compat window in `crates/orchestrator/src/schema_version.rs` header comment. Cited grounding: modelcontextprotocol.io/specification/versioning (MCP date-stamped versioning as prior art for a JSON-wire protocol).

10. Cross-language drift audit artifact. Generate `workflow/audits/E-45-drift-baseline-2026-05-19.md` with one row per cross-language pair: Rust file, TS sibling status (present / missing / stale), Python sibling status, `#[ts(any)]` debt count, schemars coverage. Sets the baseline; subsequent runs of `cross_lang_diff.py --report` re-generate the artifact and surface regressions.

## WIRING_CONTRACT

produces_symbols:
  - crates/ts_bindings/tests/snapshot.rs (NEW snapshot test)
  - crates/orchestrator/tests/action_type_lock.rs + .golden (lock file + test)
  - crates/orchestrator/tests/schemas/*.json (schemars snapshot files)
  - crates/orchestrator/tests/schemas_snapshot.rs (snapshot test)
  - crates/orchestrator/tests/schema_version_compat.rs (compat-window test)
  - python/elai_inference/schemas/_generated.py (Pydantic mirror, regenerated)
  - .claude/tools/cross_lang_diff.py extension (generator-aware mode + --strict + --report)
  - .pre-commit-config.yaml entry for drift hook
  - workflow/audits/E-45-drift-baseline-2026-05-19.md
  - workflow/audits/E-45-ts-any-debt.md
consumes_symbols:
  - ts-rs v12 derive + #[ts(export, export_to = ...)] (per B-3)
  - schemars JsonSchema derive (NEW dep)
  - jsonschema crate (NEW optional dev-validate-wire feature)
  - SsbActionType enum at crates/orchestrator/src/ssb.rs (variant list source)
  - datamodel-code-generator (Python; verified via pypi)
  - prek (j178/prek) for the pre-commit runner
reachable_from:
  - cargo test -p ts_bindings (CI gate)
  - cargo test -p orchestrator --test schemas_snapshot (CI gate)
  - cargo test -p orchestrator --test action_type_lock (CI gate)
  - cargo test -p orchestrator --test schema_version_compat (CI gate)
  - .claude/tools/cross_lang_diff.py (pre-commit + PR CI)
stub_ban:
  - no new `#[ts(type = "any")]` (existing instances tracked, never grow)
  - no removal of an action_type variant (Protobuf rule — always reserve, never remove)
  - no rename of an action_type without `#[serde(rename = "<old>")]` shim
  - no `.unwrap()` in `cross_lang_diff.py` extension (use proper exit codes + stderr messages)
  - no Cap'n Proto / Protobuf dependency added for the sidecar pair (out of scope per report)
  - no big-bang schema migration — lazy at load per E-44
  - no blake2/sha2 if any hash is needed for drift detection (use blake3 per HARD RULE)
frontend_pair: not applicable (the tests themselves are the gate)
observable_effect:
  trigger: rename a variant in `SsbActionType` enum (e.g., `IdeaFramingStarted` -> `IdeaFramingBegin`) without adding a serde rename shim, then run `cargo test -p orchestrator --test action_type_lock`
  expect_ssb: not applicable (this is a build-time/test-time gate, not a runtime SSB row)
  expect_artifact: test FAILS with explicit "action_type rename detected; either add #[serde(rename] shim or revert" message; CI red

## DETERMINISTIC_CHECK

```bash
# 1. ts_bindings snapshot test crate exists
test -f .elai_cc/crates/ts_bindings/tests/snapshot.rs
# Expected: file exists

# 2. action_type lock golden file present + non-empty
test -s .elai_cc/crates/orchestrator/tests/action_type_lock.golden
# Expected: file exists and ≥74 lines (one per current variant)

# 3. schemars derives present on at least the sidecar boundary types
rg "#\[derive\(.*JsonSchema" .elai_cc/crates/orchestrator/src --type rust | wc -l
# Expected: ≥5 (sidecar boundary types only — not every type)

# 4. schemas snapshot directory exists and contains JSON files
test -d .elai_cc/crates/orchestrator/tests/schemas && ls .elai_cc/crates/orchestrator/tests/schemas/*.json | wc -l
# Expected: ≥5 files

# 5. Pydantic generated mirror exists
test -f .elai_cc/python/elai_inference/schemas/_generated.py
# Expected: file exists

# 6. cross_lang_diff.py supports --strict / --report flags
.claude/tools/cross_lang_diff.py --help | rg "strict|report"
# Expected: both flag references present in help text

# 7. .pre-commit-config.yaml has the drift hook entry
test -f .pre-commit-config.yaml && rg "cross_lang_diff|cross-lang-diff" .pre-commit-config.yaml
# Expected: ≥1 hit

# 8. SCHEMA_VERSION compat-window test present
test -f .elai_cc/crates/orchestrator/tests/schema_version_compat.rs
# Expected: file exists with N≥3 fixture loads

# 9. No new `#[ts(type = "any")]` since baseline
rg '#\[ts\(type\s*=\s*"any"' .elai_cc/crates --type rust | wc -l
# Expected: count <= baseline recorded in workflow/audits/E-45-ts-any-debt.md (test asserts non-increase)

# 10. No Protobuf / Cap'n Proto deps added (out of scope per report)
! rg '^prost\s*=|^capnp\s*=|^capnproto\s*=|^tonic\s*=' .elai_cc/Cargo.lock
# Expected: zero hits (if your sidecar grows to need gRPC later, that's a different E-N)

# 11. Drift audit artifact exists
test -f workflow/audits/E-45-drift-baseline-2026-05-19.md
```

## ACCEPTANCE

- [ ] `cargo test -p ts_bindings --test snapshot` is green AND produces a stable bindings tree (re-run with no changes produces no diff)
- [ ] `cargo test -p orchestrator --test action_type_lock` passes against the golden; renaming a variant in test fixture causes a clean failure with actionable message
- [ ] `cargo test -p orchestrator --test schemas_snapshot` is green; modifying a boundary struct's field type causes a clean failure
- [ ] `make sidecar-schemas` (or cargo xtask equivalent) regenerates Pydantic models AND runs `git diff --exit-code` cleanly when called on a fresh clone
- [ ] DEV-profile build with `--features dev-validate-wire` validates every sidecar reply against the JSON Schema; invalid replies produce a structured error not a generic deserialization panic
- [ ] `.claude/tools/cross_lang_diff.py --strict` exits non-zero on a fixture where a Rust file with `#[derive(TS)]` is changed but bindings aren't regenerated; exits 0 on a clean tree
- [ ] `.pre-commit-config.yaml` references the drift hook; soft-mode for first 10 commits as documented in `workflow/audits/E-45-drift-baseline-2026-05-19.md`
- [ ] `crates/orchestrator/tests/schema_version_compat.rs` loads N≥3 fixture checkpoints across the supported SCHEMA_VERSION window
- [ ] `workflow/audits/E-45-ts-any-debt.md` exists with a baseline count of `#[ts(type = "any")]` instances; new test asserts non-increase
- [ ] No new Cap'n Proto / Protobuf / gRPC dep added (verified by Cargo.lock grep)
- [ ] No `#[serde(rename = ..)]` removed from any historical action_type variant (action_type_lock test asserts presence)

## Tests to add

- `crates/ts_bindings/tests/snapshot.rs`: snapshot all `#[ts(export)]` types; `git diff --exit-code` against dashboard-v2/src/lib/types
- `crates/orchestrator/tests/action_type_lock.rs`: load `SsbActionType::variants()` (or build script-generated list), assert byte-equality vs golden
- `crates/orchestrator/tests/action_type_lock_rename_detected.rs`: synthetic fixture renames a variant string; lock test fails
- `crates/orchestrator/tests/action_type_lock_addition_allowed.rs`: synthetic fixture appends a variant; with regenerated golden, test passes
- `crates/orchestrator/tests/schemas_snapshot.rs`: regenerate boundary type schemas; diff against snapshot dir
- `crates/orchestrator/tests/schema_version_compat.rs`: N synthetic fixtures, one per supported version, load cleanly
- `crates/orchestrator/tests/wire_validation_strict.rs`: dev-validate-wire feature parses invalid reply -> structured error
- `tests/contracts/no_new_ts_any_contract.rs`: count `#[ts(type = "any")]` instances; assert <= baseline
- `tests/contracts/no_protobuf_capnp_contract.rs`: Cargo.lock has zero hits for prost/capnp/tonic
- `.claude/tools/tests/test_cross_lang_diff.py` (stdlib unittest): --strict mode on synthetic dirty tree exits 1; clean tree exits 0

## Files touched

- CREATE: `.elai_cc/crates/ts_bindings/Cargo.toml` (~20 LOC — new crate hosting the snapshot test)
- CREATE: `.elai_cc/crates/ts_bindings/src/lib.rs` (~30 LOC — re-exports the types-to-snapshot from their canonical crates)
- CREATE: `.elai_cc/crates/ts_bindings/tests/snapshot.rs` (~80 LOC)
- CREATE: `.elai_cc/crates/orchestrator/tests/action_type_lock.rs` (~80 LOC)
- CREATE: `.elai_cc/crates/orchestrator/tests/action_type_lock.golden` (one line per variant; ~80 lines)
- CREATE: `.elai_cc/crates/orchestrator/tests/schemas/*.json` (one file per boundary type; ~5-10 files initially)
- CREATE: `.elai_cc/crates/orchestrator/tests/schemas_snapshot.rs` (~80 LOC)
- CREATE: `.elai_cc/crates/orchestrator/tests/schema_version_compat.rs` (~120 LOC + N fixture files)
- CREATE: `.elai_cc/python/elai_inference/schemas/__init__.py` (empty marker)
- CREATE: `.elai_cc/python/elai_inference/schemas/_generated.py` (auto-generated; gitignored OR committed with regen target documented)
- MODIFY: `.elai_cc/crates/orchestrator/Cargo.toml` (+ schemars dep, + jsonschema under `dev-validate-wire` feature; ~5 LOC)
- MODIFY: `.elai_cc/crates/orchestrator/src/ssb.rs` (NO new variants from E-45 itself, but add `#[derive(JsonSchema)]` on existing boundary types; estimated +~10 LOC for derives)
- MODIFY: `.elai_cc/Cargo.toml` workspace members (+ ts_bindings)
- MODIFY: `.claude/tools/cross_lang_diff.py` (+~150 LOC for --strict/--report generator-aware mode)
- CREATE: `.pre-commit-config.yaml` at workflow root (~30 LOC, references prek)
- CREATE: `.elai_cc/Makefile` or extend existing xtask with `sidecar-schemas` target (~20 LOC)
- CREATE: `workflow/audits/E-45-drift-baseline-2026-05-19.md`
- CREATE: `workflow/audits/E-45-ts-any-debt.md`
- CREATE: `workflow/reasoning/E-45-decisions.md`

## Citations / Grounding

- github.com/Aleph-Alpha/ts-rs (v12.0.0 2026-01-31, MSRV 1.88, generics + dep-export) — grounds Task 1 + Task 2
- docs.rs/ts-rs/latest/ts_rs/ (TS trait, serde-compat, multi-file export with imports) — grounds Task 1
- github.com/GREsau/schemars (serde-aware JSON Schema codegen) — grounds Task 4
- github.com/oxidecomputer/typify (JSON Schema → Rust; closes the loop) — referenced for fallback path
- docs.pydantic.dev/latest/internals/architecture/ (pydantic-core Rust; TypeAdapter.json_schema()) — grounds Task 5
- pydantic.dev/docs (datamodel-code-generator standard) — grounds Task 5
- protobuf.dev/reference/protobuf/edition-2024-spec/ (never change a tag, always add + opt-in, never remove) — grounds Task 3
- jsontotable.org/blog/protobuf/protobuf-schema-evolution (Protobuf evolution rules) — grounds Task 3
- github.com/Stranger6667/jsonschema (Rust high-perf validator) — grounds Task 6
- github.com/j178/prek (Rust-native pre-commit replacement) — grounds Task 8
- modelcontextprotocol.io/specification/versioning (MCP date-stamped versioning prior art) — grounds Task 9 compat-window framing
- blog.dochia.dev/blog/openapi-single-source/ + totalshiftleft.ai/blog/schema-first-api-development-testing-strategy — grounds the contract-test-blocks-on-drift philosophy
- B-3 SHIPPED (B-PLAN-SUMMARY:139 + 1028) — grounds the ts-rs base dep
- `.claude/tools/cross_lang_diff.py` EXISTS (verified via Glob) — grounds Task 7 extension surface

## Open questions / risks

- Cost of `cargo test -p ts_bindings` per PR (report Open Question 1): hypothesis <5s. Measure on the current ts-rs binding count before turning it into a PreToolUse hook that gates every local Edit (would create friction).
- schemars compile-time cost on ~200 Rust structs (report Open Question 2): if every struct grows a JsonSchema derive, build time may grow. Audit specifically — only sidecar-boundary types need schemars, not every internal type.
- Pydantic vs schemars JSON Schema byte-equality (report Open Question 3): both follow draft-2020-12, but `additionalProperties` defaulting and `$defs` placement may differ. Plan: snapshot both, run a normalization step (sort keys, canonicalize `$defs` location), accept one as canonical. Document the normalization in `crates/orchestrator/tests/schemas_snapshot.rs` header.
- tauri-specta v2 migration (report Open Question 4): out of scope for E-45 per the report's recommendation 6 ("evaluate only after Tauri v2 + specta v2 stable"). Keep ts-rs for now; revisit in 6+ months.
- SemVer mapping for SCHEMA_VERSION (report Open Question 5): today monotonic int (45). MCP uses dated versioning. Decision: keep monotonic int for the wire protocol (already shipped), add SemVer at the Rust crate level only if the crate is published externally, document the choice in `crates/orchestrator/src/schema_version.rs` header.
- `cargo xtask` vs `make` for the regen target: ELAI is Windows-primary per env block; PowerShell `make` is awkward. cargo xtask is Rust-native and cross-platform. Verify by adding to existing xtask if one is present, else create.
- The action_type lock golden file checked into git means PRs always show the diff when a new variant is added. Treat that as a feature (PR-visible discriminator additions) not noise.

## Next

E-45 is the gating substrate for any future expansion of the sidecar's HTTP surface — once schemars is on the boundary types, utoipa adoption (recommendation 5 in the report, deferred per the recommendation itself) becomes a one-task follow-up if the endpoint count grows past 5. The action_type lock file is the foundation for SemVer-aware MCP discovery exports if ELAI ever publishes its agent as MCP (intersects with E-22 / C-39 / C-42 territory). E-44's new SSB action types from the durability work MUST go through this lock file at land time — coordinate by appending to the golden in the same commit as E-44's enum-variant additions. E-41's PR CI consumes the `cargo test -p ts_bindings` and `cargo test -p orchestrator --test action_type_lock` as required pre-merge status checks alongside the heterogeneous reviewer.
