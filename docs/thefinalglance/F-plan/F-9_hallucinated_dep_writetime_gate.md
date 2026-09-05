# F-9: Hallucinated-dependency existence+age+provenance gate at WRITE-time

Objective: close the trigger-timing gap on ELAI's anti-slopsquatting defense. D-11 already ships a REAL HTTP existence + publish-date + age checker (`checkers/registry/registry_check.py`, PyPI + npm), but it fires at the PR boundary only (via `verify_pr_deps_no_slopsquat`), and the PR-leg's Rust side is shape-only regex. The instant an agent WRITES an import / `use` / `Cargo.toml` dependency line, nothing checks that the name actually exists in a registry. F-9 moves the real existence+age check to ALSO fire as a PreToolUse hook at write-time, adds cargo (crates.io) coverage to the checker, makes the staleness threshold per-ecosystem configurable, and keeps the network call OFF the hot per-write path via an LRU existence cache (the synthesis section-2 tiered rule). The done-gate is wiring, not cargo-green: the write-time block must fire on a real CLI/dashboard trigger and emit a live `write_time_slopsquat_blocked` SSB row.

Estimated effort: 1-2 sessions (~8-12 h). One baby step for cargo coverage + the cached PreToolUse trigger, per the synthesis REVISED SCOPE. This is a TIER 2 item (after D-11 ships): the network/subprocess axis is split out of the fail-closed hot path per W-10.

Pre-requisites:
  - D-11 (PLANNED, NOT shipped) — owns `checkers/registry/registry_check.py` (the real HTTP checker) and `CheckerKind::RegistryCheck` (the Rust shell-out enum). D-11 is a HARD prerequisite, not a primitive to rewire today. F-9 EXTENDS D-11; it does not re-implement it. If D-11 has not shipped at F-9 start, F-9 BLOCKS at the spec-grounder gate.
  - E-41 (SHIPPED) — owns `verify_pr_deps_no_slopsquat` + `PrPackageVerifyResult` SSB row + `commands_e41.rs::pr_review_status` Tauri command + the dashboard PR-review status surface. F-9 composes with E-41's PR-leg (same dep-name extraction shapes, same SSB-journal discipline).
  - E-11 (SHIPPED) — owns the idea-framing security block + `IdeaFramerSecurityPanel.svelte`. F-9's dashboard pair extends the same security-panel surface family.
  - B-19 (PLANNED) — hook latency budgets T1<1ms / T2<10ms / T3<5ms. F-9's in-process LRU-cache-HIT path must hit T1; the cache-MISS network path is explicitly Tier 2 (not T1, not fail-closed — see W-10 clause).

Parallel-safe with: F-1 (VibeGate compound gate reserves `vibesec_dep_hallucinated_blocked` for its Tier-2 dependency-existence axis; F-9's `write_time_slopsquat_blocked` is the standalone write-time variant — coordinate the SSB action_type golden-file edit window so the two land under one SCHEMA bump if co-sequenced). F-10 (security-monotonicity, separate SSB action_type, no overlap). NOT parallel-safe with any other edit to `crates/orchestrator/src/github_pr/package_verify.rs` or `checkers/registry/registry_check.py` (lock the file window).

Source: workflow/reasoning/VIBESEC-SHARD-STEAL-AND-BUILD-2026-06-01.md cluster C9 (the full grounded mechanism + the section-1.3 correction that D-11's checker is REAL, not shape-only) + section-2 tiered rule + section-5 sequencing (C9 = TIER 2, after D-11). F-0_INDEX.md doctrine W-1..W-10.

## Context

Per synthesis C9: the slopsquat threat is real and quantified — Spracklen USENIX 2025 Distinguished Paper measured 19.7% hallucinated package references across LLM codegen, 205,474 unique fake names, 43% of which recur on all 10 reruns (a stable, repeatable attack surface, not noise). Socket Firewall (free; cargo + pip/uv + npm transitive) explicitly CANNOT close the freshly-registered-slop gap — a brand-new fake name has no CVE — and that gap is exactly the codegen-time existence+age slot the VibeSec tools-matrix (J #2) calls uncovered.

The section-1.3 correction is load-bearing and must NOT be re-introduced as a draft error: ELAI's registry check is NOT shape-only. `checkers/registry/registry_check.py` is a real HTTP existence + publish-date + age checker for PyPI and npm (confirmed in elai.map: `_check_pypi` line 90, `_check_npm` line 120, `_http_get_json` line 58, `_isoparse` line 43, `main` line 145). The Rust enum `CheckerKind::RegistryCheck` (checker_registry.rs:62-94) shells out to it. The TRUE gap is narrower and twofold:
  1. INVOCATION TIMING. The checker fires at the PR boundary (via E-41's `verify_pr_deps_no_slopsquat`, package_verify.rs:51) and via the checker-registry subprocess path — never at the instant an import line is written. A hallucinated `import reqquests` reaches disk uninspected and only gets caught (if ever) at PR time.
  2. ECOSYSTEM COVERAGE. The checker covers PyPI + npm. It does NOT cover crates.io. ELAI's own dominant language is Rust; a hallucinated cargo dep is the highest-likelihood self-inflicted slop and is currently uncovered.

A third, smaller gap (the synthesis flags it): the staleness threshold is hard-coded to 730 days (`registry_check.py:175`, `max_age_days = int(candidate.get("max_age_days", 730))`). Stable Rust crates routinely go years without a publish; a hard 730d age-floor fails real, healthy deps. The threshold must be per-ecosystem configurable.

The section-2 tiered design rule governs the build: the existence lookup is a NETWORK call. It MUST NOT sit on the fail-closed hot per-write path advertising a fail-closed promise it cannot keep (a 500ms registry timeout that fails-open silently passes a slopsquatted import exactly when it is most dangerous — air-gapped build, flaky registry). F-9 SPLITS the axis per W-10: an in-process LRU existence cache answers the common case (already-seen, known-good names) at T1 latency; only a cache MISS triggers the network subprocess, and that path is explicitly labelled Tier 2 (best-effort, may fail-open under registry unavailability, NEVER advertised as fail-closed). The cache is what keeps the network call off the hot path.

Socket Firewall at the install boundary is defense-in-depth but is an OPS RUNBOOK, not in-plan code (synthesis REVISED SCOPE explicitly moves it out of the plan).

## Existing state (B/C/D/E coverage)

- D-11 (PLANNED): `checkers/registry/registry_check.py` — real HTTP checker. Confirmed in elai.map: `DEF_FN _check_pypi checkers/registry/registry_check.py:90`, `DEF_FN _check_npm :120`, `DEF_FN _http_get_json :58`, `DEF_FN _isoparse :43`, `DEF_FN main :145`. Reads a JSON claim on stdin (`max_age_days` default 730, line 175), returns `{"result": ..., "reason": ...}` JSON. `CheckerKind::RegistryCheck` (checker_registry.rs:62-94) is the Rust shell-out enum, frozen per D-9 locked keys. NO `_check_cargo` exists today (confirmed: grep `_check_cargo` returns zero DEF rows).
- E-41 (SHIPPED): `verify_pr_deps_no_slopsquat(ssb, pr_number)` (package_verify.rs:51) shells out to `gh pr diff`, runs `extract_dep_candidates` (line 110) over the diff, applies `flag_candidate` (line 152, deterministic shape/allowlist heuristic — explicitly NOT the registry checker; the module header lines 9-17 say the verdicts "will be promoted to call that [RegistryCheck] surface" in a follow-up — F-9 IS that follow-up for the write-time path). Emits `PrPackageVerifyResult` SSB row. CLI caller: `pr_review_cmd.rs:171`. Dep-name parsers: `parse_json_dep_name` (233), `parse_pep508_dep_name` (257), `is_dep_name_shape` (191). NOTE: `extract_dep_candidates` parses Cargo.toml only via the loose `name = "..."` shape; it has NO `[dependencies]`-section-aware cargo parse.
- E-41 dashboard (SHIPPED): `commands_e41.rs::pr_review_status` Tauri command + `PrReviewStatusDto` (with `slopsquat_count: lookup_u32(..., "pr_package_verify_result", ...)` at commands_e41.rs:134). `pr_review_status` stub at `src/lib/api/stub.ts:3326`. F-9 extends this DTO surface with a write-time block count.
- E-11 (SHIPPED): `IdeaFramerSecurityPanel.svelte` (windows/IdeaFramerSecurityPanel.svelte) — the idea-framing security checklist panel. F-9's deps/security panel is the same surface family.
- PreToolUse backend (SHIPPED): `dispatch_pre_tool_use<'a>` (hook_dispatcher.rs:1477), the `analyze(&self, invocation: &HookInvocation<'_>)` backend trait (hook_dispatcher.rs:622), `HookInvocation` struct (line 271), `HookInvocation::for_pre_tool_use(action_type, tool, step)` constructor (line 324), `HookVerdictKind::{Allow,...}` (line 139). This is the real production hook the new write-time analyzer attaches to. The baby-step PRE_GATE already flows through this path (`for_pre_tool_use("baby_step_pre_gate", ...)`).
- SSB (SHIPPED): `Ssb::write_payload(agent_id, superstep, payload, vec![])`, `ssb_action_type_from_str_public`, the `PrPackageVerifyResult` Pattern as the nearest analog, `GhCliAdapter::hash_prefix`, `clock::now_ms`. SCHEMA_VERSION baseline 191 per CLAUDE.md.
- anti-slopsquatting (HARD RULE, CLAUDE.md + .claude/rules/token-discipline.md): "NEVER assume a dependency exists. Verify name + repo + version + last-published date BEFORE adding." The write-time gate is the in-code enforcement of this human-process rule — the first time the rule becomes a fail-closed gate instead of a documented expectation.

## Reads (mandatory before coding)

- `F:\.ELAI_workflow\workflow\reasoning\VIBESEC-SHARD-STEAL-AND-BUILD-2026-06-01.md` — section 4 item C9 (mechanism + REVISED SCOPE), section 1.3 (the "NOT shape-only" correction — do NOT re-introduce), section 2 (the tiered-gate rule), section 5 (C9 = TIER 2 after D-11), section 7 (do-NOT list: never advertise fail-closed on a fail-open network axis).
- `F:\.ELAI_workflow\docs\thefinalglance\F-plan\F-0_INDEX.md` — W-1..W-10 doctrine; the F-9 SCHEMA bump site line (`+write_time_slopsquat_blocked`).
- `F:\.ELAI_workflow\.elai_cc\checkers\registry\registry_check.py:35-195` — the real checker (`_check_pypi`, `_check_npm`, `_http_get_json`, `_isoparse`, `main`; the `max_age_days` default 730 at line 175 is the per-ecosystem-threshold edit site).
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\checker_registry.rs:55-95` — `CheckerKind` enum (the `RegistryCheck` shell-out variant; the closed-taxonomy / D-9 locked-keys note that governs adding a cargo claim shape).
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\github_pr\package_verify.rs:1-265` — E-41 PR-leg (`verify_pr_deps_no_slopsquat`, `extract_dep_candidates`, `flag_candidate`, `is_dep_name_shape`, `parse_json_dep_name`, `parse_pep508_dep_name`). The cargo `[dependencies]` parse extension lands here AND in the checker.
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\hook_dispatcher.rs:271-340, 615-720, 1470-1490` — `HookInvocation`, `for_pre_tool_use`, the `analyze` backend trait, `dispatch_pre_tool_use`.
- `F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src-tauri\src\commands_e41.rs:90-215` — the `pr_review_status` Tauri command + `lookup_u32`/`lookup_u64` DTO-extraction helpers (the pattern the new write-time-blocks command mirrors).
- `F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src\lib\windows\IdeaFramerSecurityPanel.svelte` — the security-panel surface F-9's panel extends.
- arXiv (Spracklen, USENIX Security 2025 Distinguished Paper) — 19.7% hallucinated, 205,474 unique fake names, 43% recur-on-all-10. Cite via the synthesis doc; do NOT restate the number as a target without the doc's grounding tag.
- Socket Firewall docs (socket.dev) — the install-boundary wrap; OPS-ONLY (runbook), not in-plan.

## Tasks

1. **Add `_check_cargo()` to `checkers/registry/registry_check.py` (~40 LOC).** New `def _check_cargo(pkg: str, max_age_days: int) -> tuple[str, str]:` mirroring `_check_pypi` (line 90). Hits `https://crates.io/api/v1/crates/{pkg}` via the existing `_http_get_json` (line 58); reads `crate.updated_at` / the newest version's `created_at` and runs it through `_isoparse` (line 43) for the age check; returns the same `(result, reason)` tuple shape. Wire it into `main` (line 145) alongside the existing pypi/npm dispatch (`main` at line 183-185 already branches `_check_pypi` / `_check_npm`): add an `ecosystem` field to the stdin JSON claim (`"ecosystem": "cargo"` selects `_check_cargo`; default to existing pypi/npm sniffing for back-compat). Per CLAUDE.md: stdlib-only (`_http_get_json` uses `requests` already vendored for the checker; do NOT add a dep). NO `.unwrap`-equivalent silent swallow — a network failure returns `("inapplicable", reason)` (the existing fail-soft shape for the checker subprocess), and the Rust caller treats `inapplicable` as Tier-2-fail-open (see Task 4).

2. **Per-ecosystem stale threshold (~15 LOC in `registry_check.py` + a config table).** Replace the hard-coded `max_age_days = int(candidate.get("max_age_days", 730))` (line 175) with a per-ecosystem default table: `_DEFAULT_MAX_AGE_DAYS = {"pypi": 730, "npm": 730, "cargo": 1825}` (cargo gets the longer floor per synthesis: stable crates routinely go years without a publish; a hard 730d fails real deps). An explicit `max_age_days` in the stdin claim still overrides. UNVERIFIED-design-choice: the 1825d cargo floor is a reasoned default, not a measured optimum — label it as a tunable in the decision doc; the measurable acceptance (0 FP on the 20 real ELAI deps) is what validates it.

3. **Add a cargo-aware dep extractor to `package_verify.rs` (~50 LOC).** Extend `extract_dep_candidates` (line 110) with a `[dependencies]` / `[dev-dependencies]` / `[build-dependencies]` TOML-section-aware cargo parse so a `+serde = { version = "1" }` or `+tokio = "1.40"` added line yields `serde` / `tokio` as candidates with an `ecosystem=cargo` tag (today only the loose `name = "..."` shape is caught, with no ecosystem tag). Reuse `is_dep_name_shape` (line 191). Return a richer `DepCandidate { name: String, ecosystem: Ecosystem }` (new small enum `Ecosystem { Pypi, Npm, Cargo }`) so the write-time hook (Task 4) can route to the right `_check_*`. Keep `verify_pr_deps_no_slopsquat` (line 51) compiling against the new shape (its `flag_candidate` shape-check stays as the cheap PR-leg pre-filter; F-9 does not remove it).

4. **Add the write-time PreToolUse analyzer `crates/orchestrator/src/vibesec/dep_writetime_gate.rs` (~220 LOC).** New `pub struct DepWriteTimeGate` implementing the hook backend trait `analyze(&self, invocation: &HookInvocation<'_>) -> Result<Vec<HookFinding>, HookError>` (the trait at hook_dispatcher.rs:622). Logic:
   - FIRE ONLY when the PreToolUse `invocation.tool` is an Edit/Write whose target path or candidate content matches an import / `use` / `Cargo.toml` / `package.json` / `pyproject.toml` dependency pattern (reuse `extract_dep_candidates` from Task 3 against the candidate write content). On a non-dep write the analyzer returns `Ok(vec![])` immediately (zero latency).
   - For each `DepCandidate`: consult an in-process `LruExistenceCache` (new, `~80 LOC`, `pub struct LruExistenceCache` backed by a bounded `std::collections::HashMap` + a `VecDeque` recency list — NO new dep; do NOT pull an `lru` crate without confirmation. TTL ~1h via `clock::now_ms`). Cache HIT (name known-good) -> Allow at T1 latency, no network. Cache MISS -> spawn the `CheckerKind::RegistryCheck` subprocess (Task 1's extended `registry_check.py`) with the `ecosystem` tag; on a `not_found` / `too_new` verdict, return a `HookFinding` with `HookVerdictKind::Block` and emit `write_time_slopsquat_blocked`; on `ok`, insert into the cache and Allow.
   - TIER SPLIT (W-10, MANDATORY): the cache-HIT path is Tier 1 (in-process, T1<1ms). The cache-MISS network subprocess path is Tier 2 — it is explicitly best-effort and MAY fail-open if the registry is unreachable (`inapplicable` verdict -> Allow + emit a `write_time_slopsquat_blocked` row with `kind=registry_unreachable` advisory, NEVER a silent pass). The struct doc-comment MUST state which guarantee each path delivers and MUST NOT advertise fail-closed for the network path (synthesis section 7 do-NOT clause). NO `.unwrap()` on the subprocess result; propagate via `Result`/`?`.
   - blake3 (NOT blake2/sha2) for any candidate-name hash written to the SSB payload (CLAUDE.md HARD RULE). Reuse `GhCliAdapter::hash_prefix` if it is already blake3; else add a blake3 helper in the new module.

5. **Wire `DepWriteTimeGate` into `dispatch_pre_tool_use` (~30 LOC delta in `hook_dispatcher.rs`).** Register the new analyzer as a backend in the PreToolUse dispatch chain (the same chain the baby-step PRE_GATE flows through — `dispatch_pre_tool_use` at line 1477). It must run as a real production hook backend, reachable_from the live FSM EXECUTING write path, NOT test-only (W-3). Default ON (W-2): the gate is wired into the production dispatch with NO `ELAI_*_ENABLE` env gate as its terminal state. A kill-switch env var `ELAI_DEP_WRITETIME_GATE_DISABLE=1` MAY exist to turn it OFF for an air-gapped build, but the DEFAULT path has it ON and the observable_effect runs with NO env override (W-2 clause). The LLM cannot disable it via prompt (no bypass branch — stub_ban).

6. **Add the `write_time_slopsquat_blocked` SSB action_type (~60 LOC in `crates/orchestrator/src/ssb.rs`).** Per W-9 + CLAUDE.md Pattern-2: add ONE new variant `WriteTimeSlopsquatBlocked` carrying a `WriteTimeSlopsquatBlockedPayload { dep_name_hash: String (blake3), ecosystem: String, verdict_kind: WriteTimeSlopVerdictKind, registry_age_days: Option<u32>, ts_ms: i64 }`, where `WriteTimeSlopVerdictKind` is the 3-arm discriminator `{ NotFound, TooNew, RegistryUnreachable }` (the Pattern-2 single-variant-with-Kind-discriminator collapse — do NOT add three separate action_types). Add the `sql_str` value, the parse arm in `ssb_action_type_from_str_public`, the SQL CHECK constraint entry, the payload variant, and the round-trip golden fixture. Flow the variant through the `action_type_lock.golden` file (E-45 pattern). SCHEMA_VERSION: bump to baseline+1 (baseline 191 per CLAUDE.md — name it "baseline + 1" in the WIRING_CONTRACT, never hard-code a final number). If F-9 co-sequences with F-1, coordinate so both land under one bump.

7. **Frontend pair: deps/security write-time-blocks panel (~180 LOC svelte + ~120 LOC tauri) (W-4).** New `crates/dashboard-v2/src-tauri/src/commands_f9.rs::write_time_slop_status(state) -> WriteTimeSlopStatusDto` Tauri command, mirroring `commands_e41.rs::pr_review_status` (line ~90) and its `lookup_u32`/`lookup_u64` payload-extraction helpers (line 205); it aggregates recent `write_time_slopsquat_blocked` rows (count by ecosystem + by verdict_kind). New `crates/dashboard-v2/src/lib/windows/DepsSecurityPanel.svelte` (same surface family as `IdeaFramerSecurityPanel.svelte`) rendering "N write-time slop blocks today (cargo: x, npm: y, pypi: z); M registry-unreachable advisories". Register the panel in the workspace drawer/window registry (the `DrawerKey` taxonomy at `src/lib/state/workspace.svelte.ts:374`). `python .claude/tools/cross_lang_diff.py` MUST pass (no Rust<->TS DTO drift). Add the `write_time_slop_status` stub to `src/lib/api/stub.ts` mirroring the `pr_review_status` stub (line 3326).

8. **Decision doc `workflow/reasoning/F-9-decisions.md` (~90 LOC).** Records: (a) why the network axis is Tier-2 fail-open-with-advisory, not fail-closed (synthesis section 2 + 7); (b) the LRU cache is the mechanism that keeps the network off the hot path (TTL ~1h, bounded size — name the cap); (c) the 1825d cargo stale floor is a reasoned default, UNVERIFIED as an optimum, tunable per-ecosystem; (d) Socket Firewall is OPS-RUNBOOK, deliberately out-of-plan; (e) the section-1.3 correction is honored — the checker is REAL, F-9 only adds cargo + the write-time trigger + the cache; (f) F-9 composes with E-41's PR-leg (the PR-leg stays as the cheap shape pre-filter; F-9 is the registry-grounded write-time leg).

Tier split (W-10, explicit):
  - TIER 1 (in-process, genuinely fail-closed, IS the done-gate): the cache-HIT path — known-good names Allow at T1; known-bad / already-seen-not-found names Block at T1 with no network. This path is deterministic and replayable under ELAI_SEED.
  - TIER 2 (network/subprocess, best-effort, may fail-open): the cache-MISS `registry_check.py` subprocess. Documented as fail-open-with-advisory; NEVER advertises fail-closed. This is the separately-gated sub-phase per the doctrine.

## WIRING_CONTRACT

produces_symbols:
  - crates/orchestrator/src/vibesec/dep_writetime_gate.rs::DepWriteTimeGate
  - crates/orchestrator/src/vibesec/dep_writetime_gate.rs::LruExistenceCache
  - crates/orchestrator/src/vibesec/dep_writetime_gate.rs::RegistryExistenceChecker
  - crates/orchestrator/src/vibesec/dep_writetime_gate.rs::StubRegistryChecker
  - crates/orchestrator/src/vibesec/dep_writetime_gate.rs::SubprocessRegistryChecker
  - crates/orchestrator/src/vibesec/dep_writetime_gate.rs::RegistryVerdict
  - crates/orchestrator/src/vibesec/dep_writetime_gate.rs::extract_dep_candidates_agnostic
  - crates/orchestrator/src/vibesec/dep_writetime_gate.rs::dep_name_blake3_u64
  - checkers/registry/registry_check.py::_check_cargo
  - SsbActionType::WriteTimeSlopsquatBlocked
  - crates/orchestrator/src/ssb.rs::WriteTimeSlopsquatBlockedPayload
  - crates/orchestrator/src/ssb.rs::WriteTimeSlopVerdictKind
  - crates/cli/src/dep_check_cmd.rs::run
  - crates/dashboard-v2/src-tauri/src/commands_f9.rs::write_time_slop_status
  - crates/dashboard-v2/src/lib/drawers/DepsSecurityPanel.svelte
consumes_symbols:
  - crates/orchestrator/src/runtime_setup.rs::build_production_hook_dispatcher
  - crates/orchestrator/src/hook_dispatcher.rs::InProcessHookDispatcher::with_dep_writetime_gate
  - crates/orchestrator/src/hook_dispatcher.rs::dispatch_pre_tool_use
  - crates/cli/src/args.rs::CliAction::DepCheck
  - crates/orchestrator/src/ssb.rs::Ssb (write_payload)
  - crates/dashboard-v2/src-tauri/src/commands_f9.rs::write_time_slop_status
reachable_from:
  - FSM EXECUTING write path -> build_production_hook_dispatcher (triple-bootstrap: run_subcommand.rs:358, task_cmd.rs:428, dashboard lib.rs:1434) -> InProcessHookDispatcher::with_dep_writetime_gate -> dispatch_pre_tool_use -> DepWriteTimeGate.analyze -> Block on hallucinated dep
  - CLI: `elai dep-check <path> [--offline]` (CliAction::DepCheck, args.rs:985) -> dep_check_cmd::run (dep_check_cmd.rs:51) -> DepWriteTimeGate.analyze_collect over extracted dep candidates -> Ssb.write_payload(WriteTimeSlopsquatBlocked) row (this is the observable_effect trigger; a real production CLI entrypoint, not a test)
  - Tauri command write_time_slop_status (commands_f9.rs:143) -> DepsSecurityPanel.svelte (dashboard read path)
stub_ban:
  - "TODO: wire cargo check" literal absent
  - "// registry later" / "// slop check later" literal absent
  - the gate's flag default-OFF terminal state is FORBIDDEN: ELAI_DEP_WRITETIME_GATE_DISABLE defaults UNSET (gate ON); a default-OFF ELAI_*_ENABLE pattern as the terminal wiring state is a W-2 violation and a BLOCK
  - NO LLM-bypass branch: the analyzer cannot be disabled by any prompt-supplied flag, agent argument, or in-content directive; the only disable path is the explicit operator env kill-switch
  - NO silent fail-open without an advisory SSB row: a registry-unreachable cache-miss MUST emit write_time_slopsquat_blocked{kind=RegistryUnreachable}, never a silent Allow
  - NO .unwrap()/.expect() on the subprocess or cache result in the production path
frontend_pair:
  svelte: crates/dashboard-v2/src/lib/drawers/DepsSecurityPanel.svelte
  tauri: crates/dashboard-v2/src-tauri/src/commands_f9.rs::write_time_slop_status
observable_effect:
  trigger: "cd .elai_cc && cargo run -p elai-cli -- dep-check tests/fixtures/f9_hallucinated_import.rs   (the fixture's added line is `+use reqquests::get;` — a 404-confirmed slop name)"
  expect_ssb: "a write_time_slopsquat_blocked row appears in the live ssb.db within 2s of the trigger, with verdict_kind=NotFound, ecosystem=cargo, dep_name_hash a blake3 prefix; queryable via the SSB reader"
  expect_artifact: "stdout contains a BLOCK line matching /write-time slop block: reqquests \\(cargo\\) not found in registry/ AND the file write is refused (the hallucinated import does NOT reach disk); regex on the CLI stdout + an inspectable ssb.db row"

## DETERMINISTIC_CHECK

```bash
# 0. THE W-1 WIRING DONE-GATE (necessary AND sufficient gate; cargo-green alone is NOT enough)
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/F-plan/F-9_hallucinated_dep_writetime_gate.md --run-trigger

# 1. produced symbols exist
rg "pub struct DepWriteTimeGate" .elai_cc/crates/orchestrator/src/vibesec/dep_writetime_gate.rs
rg "pub struct LruExistenceCache" .elai_cc/crates/orchestrator/src/vibesec/dep_writetime_gate.rs
rg "def _check_cargo" .elai_cc/checkers/registry/registry_check.py
rg "pub enum Ecosystem" .elai_cc/crates/orchestrator/src/github_pr/package_verify.rs

# 2. the SSB action_type + Pattern-2 verdict discriminator exist + flow through the golden lock
rg "WriteTimeSlopsquatBlocked|write_time_slopsquat_blocked" .elai_cc/crates/orchestrator/src/ssb.rs
rg "WriteTimeSlopVerdictKind" .elai_cc/crates/orchestrator/src/ssb.rs
rg "write_time_slopsquat_blocked" .elai_cc/crates/orchestrator/src/*.golden .elai_cc/crates/contracts/**/action_type_lock.golden

# 3. the analyzer is wired into the REAL PreToolUse dispatch (not test-only)
rg "DepWriteTimeGate" .elai_cc/crates/orchestrator/src/hook_dispatcher.rs

# 4. default-ON: the kill-switch is a DISABLE var, never a default-off ENABLE terminal state
rg "ELAI_DEP_WRITETIME_GATE_DISABLE" .elai_cc/crates/orchestrator/src/vibesec/dep_writetime_gate.rs
rg -q "ELAI_DEP_WRITETIME_GATE_ENABLE" .elai_cc/crates/orchestrator/src/ && echo "W-2 VIOLATION: default-off enable gate present" && exit 1

# 5. frontend pair + cross-language drift
test -f .elai_cc/crates/dashboard-v2/src/lib/windows/DepsSecurityPanel.svelte
rg "write_time_slop_status" .elai_cc/crates/dashboard-v2/src-tauri/src/commands_f9.rs
python .claude/tools/cross_lang_diff.py

# 6. blake3 (not blake2/sha2) on the dep-name hash
rg "blake3" .elai_cc/crates/orchestrator/src/vibesec/dep_writetime_gate.rs
rg -q "blake2|sha2" .elai_cc/crates/orchestrator/src/vibesec/dep_writetime_gate.rs && echo "HARD RULE VIOLATION: blake2/sha2 in new gate" && exit 1

# 7. decision doc
test -f workflow/reasoning/F-9-decisions.md
```

## ACCEPTANCE

- [ ] W-1 WIRING DONE-GATE: `python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/F-plan/F-9_hallucinated_dep_writetime_gate.md --run-trigger` returns PASS (exit 0) — every produces_symbol resolves to a DEF/SCIP_DEF, every consumes_symbol has a real CALL/REF, every reachable_from anchor is reached by BFS over CALL_RESOLVED within depth 8, stub_ban finds zero forbidden literals, AND the observable_effect trigger RAN and emitted the SSB row + artifact. (cargo check --workspace GREEN is necessary but NOT sufficient.)
- [ ] W-2 DEFAULT-ON: the observable_effect trigger runs and BLOCKS the `reqquests` import with NO env override set (no `ELAI_*` flag in the environment). The kill-switch `ELAI_DEP_WRITETIME_GATE_DISABLE=1` is a DISABLE-only escape hatch; the terminal wired state is ON.
- [ ] W-6 DASHBOARD E2E: through the dashboard (dashboard-smoker / a tauri:dev launch), the DepsSecurityPanel renders the live `write_time_slopsquat_blocked` count after the observable_effect trigger fires — verified THROUGH the dashboard, not via a unit test. `cross_lang_diff.py` PASS.
- [ ] W-10 TIER SPLIT verified two ways: (1) cache-HIT path Allows/Blocks at T1 with NO network call (deterministic, replayable under ELAI_SEED); (2) DETERMINISTIC-FIXTURE-HARNESS checkbox — a fixture that stubs `registry_check.py` to a fixed `not_found` verdict exercises the REAL cache-miss -> subprocess -> Block code path (not a mock of the gate), proving the wired path works without a live registry.
- [ ] LIVE-RUN checkbox (separate, RunPod/network-gated): with real network, `elai dep-check` on the 50-sample 404-confirmed slop suite blocks 50/50; on the 20 most-used real ELAI deps, 0 false-positives. (Gated separately from the fixture-harness checkbox per W-5.)
- [ ] cargo coverage: a hallucinated cargo dep (`+reqquests = "0.1"` in a Cargo.toml added line) is extracted by the new cargo-aware `extract_dep_candidates`, routed `ecosystem=cargo`, and blocked by `_check_cargo`.
- [ ] SSB round-trip: `WriteTimeSlopsquatBlocked` + its 3-arm `WriteTimeSlopVerdictKind` round-trips via the SSB roundtrip contract; the variant is present in `action_type_lock.golden`; SCHEMA_VERSION = baseline + 1 (named, not hard-coded).
- [ ] No new dependency added outside the anti-slopsquatting verify (no `lru` crate, no new HTTP crate — confirmed via cargo + the registry verify protocol).
- [ ] E-41 PR-leg still compiles and passes its contract (`verify_pr_deps_no_slopsquat` unchanged in behavior; `extract_dep_candidates` extended back-compatibly).

## WIRING DONE-GATE

This section states clause-by-clause how F-9 satisfies the doctrine so the implementer cannot ship it wire-dark.

- W-1 (no-wire-dark done-gate): the DONE definition is `wiring_contract_check.py --run-trigger` PASS, NOT cargo-green. The observable_effect (`elai dep-check` on the `reqquests` fixture -> live `write_time_slopsquat_blocked` row + refused write) is the runnable trigger the validator executes. cargo-green is necessary but explicitly not sufficient.
- W-2 (no default-off env gate as terminal state): the gate is wired ON in `dispatch_pre_tool_use` with no `ELAI_*_ENABLE` flag. The only env var is `ELAI_DEP_WRITETIME_GATE_DISABLE` (a kill-switch; default UNSET = ON). The DETERMINISTIC_CHECK explicitly greps-and-fails on any `ELAI_DEP_WRITETIME_GATE_ENABLE` pattern. The observable_effect ACCEPTANCE runs with no env override.
- W-3 (production entrypoint): reachable_from the live FSM EXECUTING write path through `dispatch_pre_tool_use` (a real PreToolUse hook, the same chain the baby-step PRE_GATE uses) AND from a real `elai dep-check` CLI subcommand. No #[cfg(test)]-only caller.
- W-4 (frontend pair mandatory): `DepsSecurityPanel.svelte` + `commands_f9.rs::write_time_slop_status` Tauri command; `cross_lang_diff.py` must pass. The SSB signal is dashboard-surfaceable, so a UI consumer is mandatory and named.
- W-5 (observable_effect is a live end-to-end trigger): the trigger is a real `cargo run -p elai-cli -- dep-check` command -> live ssb.db row + inspectable stdout/refused-write, NOT a unit test. Because the registry network is RunPod/network-gated, F-9 ships a DETERMINISTIC FIXTURE HARNESS (stubbed `registry_check.py` verdict) exercising the REAL cache-miss->subprocess->Block path, and labels the live 50/50 + 0-FP run as a SEPARATE gated checkbox.
- W-6 (integration checkpoint through the dashboard): the W-6 ACCEPTANCE item verifies the DepsSecurityPanel renders the live block count through a tauri:dev / dashboard-smoker pass.
- W-7 (no hallucinated integration points): every consumes_symbol + reachable_from anchor + MODIFY target was confirmed in elai.map at authoring time — `dispatch_pre_tool_use` (hook_dispatcher.rs:1477), `HookInvocation::for_pre_tool_use` (324), the `analyze` trait (622), `CheckerKind::RegistryCheck` (checker_registry.rs:62-94), `extract_dep_candidates` (package_verify.rs:110), `is_dep_name_shape` (191), `registry_check.py` `_check_pypi`/`_check_npm`/`_isoparse`/`main`, `commands_e41.rs::pr_review_status` + `lookup_u32`/`lookup_u64`, `IdeaFramerSecurityPanel.svelte`, `DrawerKey` (workspace.svelte.ts:374). The validator re-greps.
- W-8 (produces/consumes symmetry): every produced symbol has an in-plan caller — `DepWriteTimeGate` is called by `dispatch_pre_tool_use` (Task 5) and `dep-check` CLI; `LruExistenceCache` is called by `DepWriteTimeGate`; `_check_cargo` is invoked via the `CheckerKind::RegistryCheck` subprocess from `DepWriteTimeGate`; `Ecosystem` is produced+consumed by the cargo extractor + the gate; `WriteTimeSlopsquatBlocked` is emitted by the gate and consumed by `write_time_slop_status`; the Svelte panel consumes the Tauri command. No produced symbol is left caller-less.
- W-9 (SSB variant through golden lock + named SCHEMA bump): ONE Pattern-2 variant `WriteTimeSlopsquatBlocked` with the 3-arm `WriteTimeSlopVerdictKind` discriminator (NotFound / TooNew / RegistryUnreachable) — not three action_types. Flows through `action_type_lock.golden`; SCHEMA_VERSION = baseline + 1 (baseline 191 per CLAUDE.md).
- W-10 (tiered-gate): the network/subprocess axis is split OUT of the fail-closed hot path. Tier 1 = in-process LRU-cache-HIT (genuinely fail-closed, T1, the done-gate). Tier 2 = cache-MISS network subprocess (best-effort, fail-open-with-advisory, NEVER advertises fail-closed). The struct doc-comment states which guarantee each path delivers; a fail-open never masquerades as fail-closed (synthesis section 7).

## Tests to add

- `crates/contracts/tests/f9_dep_writetime_gate_contract.rs` (~160 LOC): (a) a non-dep Edit returns `Ok(vec![])` (zero false-fire); (b) cache-HIT on a known-good name Allows at T1 with no subprocess spawn (assert via a subprocess-spy); (c) the DETERMINISTIC FIXTURE HARNESS — a stubbed `registry_check.py` returning `not_found` drives the REAL cache-miss -> Block path and emits `write_time_slopsquat_blocked{NotFound}`; (d) a registry-unreachable stub yields `RegistryUnreachable` advisory + Allow (fail-open-with-row), proving the Tier-2 guarantee; (e) the LLM-bypass attempt (a prompt-supplied disable flag) does NOT disable the gate.
- `crates/contracts/tests/f9_ssb_roundtrip_contract.rs` (~60 LOC): `WriteTimeSlopsquatBlockedPayload` + all 3 `WriteTimeSlopVerdictKind` arms round-trip; the variant is in `action_type_lock.golden`.
- `crates/orchestrator/src/github_pr/package_verify.rs` inline `#[cfg(test)]`: the cargo `[dependencies]` extractor yields `serde`/`tokio` with `Ecosystem::Cargo`; `verify_pr_deps_no_slopsquat` still compiles + passes.
- `checkers/registry/` fixture: `_check_cargo` against a 404 name returns `not_found`; against a known crate returns `ok`; the 1825d cargo floor applies (mock-clock).
- `crates/dashboard-v2` svelte-check + the `write_time_slop_status` DTO cross_lang_diff.

## Files touched

- CREATE: `crates/orchestrator/src/vibesec/dep_writetime_gate.rs` (~220 LOC: DepWriteTimeGate + LruExistenceCache + the analyze impl)
- CREATE: `crates/orchestrator/src/vibesec/mod.rs` (~5 LOC: module declaration, if the vibesec module dir does not already exist — verify via Glob; F-1 may already create it, coordinate)
- CREATE: `crates/dashboard-v2/src-tauri/src/commands_f9.rs` (~120 LOC: write_time_slop_status Tauri command + DTO + lookup helpers mirroring commands_e41.rs)
- CREATE: `crates/dashboard-v2/src/lib/windows/DepsSecurityPanel.svelte` (~180 LOC)
- CREATE: `crates/contracts/tests/f9_dep_writetime_gate_contract.rs` (~160 LOC)
- CREATE: `crates/contracts/tests/f9_ssb_roundtrip_contract.rs` (~60 LOC)
- CREATE: `tests/fixtures/f9_hallucinated_import.rs` + a `f9_hallucinated_cargo.toml` fixture (the slop-name fixtures the observable_effect + harness use)
- CREATE: `workflow/reasoning/F-9-decisions.md` (~90 LOC)
- MODIFY: `checkers/registry/registry_check.py` (~55 LOC: `_check_cargo` + per-ecosystem threshold table + `ecosystem` claim field + `main` dispatch arm)
- MODIFY: `crates/orchestrator/src/github_pr/package_verify.rs:110-145` (~50 LOC: cargo `[dependencies]`-aware `extract_dep_candidates` + the `Ecosystem`/`DepCandidate` shape; keep `verify_pr_deps_no_slopsquat` compiling)
- MODIFY: `crates/orchestrator/src/hook_dispatcher.rs:1470-1490` (~30 LOC: register DepWriteTimeGate in the PreToolUse dispatch chain, default ON)
- MODIFY: `crates/orchestrator/src/ssb.rs` (~60 LOC: WriteTimeSlopsquatBlocked variant + WriteTimeSlopVerdictKind + payload + sql_str + parse arm + CHECK + golden round-trip)
- MODIFY: `crates/orchestrator/src/lib.rs` (~3 LOC: re-export the vibesec gate types if needed by the CLI)
- MODIFY: `crates/cli/src/main.rs` (~40 LOC: `dep-check <path>` subcommand — the observable_effect CLI entrypoint)
- MODIFY: `crates/dashboard-v2/src/lib/state/workspace.svelte.ts` (~5 LOC: add the DepsSecurity DrawerKey/window registration)
- MODIFY: `crates/dashboard-v2/src/lib/api/stub.ts` (~8 LOC: write_time_slop_status stub mirroring pr_review_status)
- MODIFY: the `action_type_lock.golden` (E-45 pattern) (~2 LOC: the locked variant)

## Citations / Grounding

- workflow/reasoning/VIBESEC-SHARD-STEAL-AND-BUILD-2026-06-01.md section 4 C9 — the mechanism + REVISED SCOPE (cargo + cached PreToolUse trigger as a D-11/E-41 extension; Socket Firewall as ops-only). [synthesis-grounded; survived adversarial run wf_b959963a-db6]
- workflow/reasoning/VIBESEC-SHARD-STEAL-AND-BUILD-2026-06-01.md section 1.3 — CORRECTION: D-11's registry_check.py is REAL HTTP existence+age, NOT shape-only; the true gap is trigger-timing + cargo. [correction, do NOT re-introduce the "shape-only" draft claim]
- workflow/reasoning/VIBESEC-SHARD-STEAL-AND-BUILD-2026-06-01.md section 2 + 7 — the tiered-gate rule + the do-NOT-advertise-fail-closed-on-a-fail-open-network-axis clause. [architectural insight, the most reusable finding in the doc]
- Spracklen, USENIX Security 2025 (Distinguished Paper) — 19.7% hallucinated, 205,474 unique fake names, 43% recur on all 10 reruns. [VERIFIED per synthesis; cite via the synthesis doc, not as an F-9-internal target number]
- Socket Firewall (socket.dev) — install-boundary cargo+pip/uv+npm transitive wrap; explicitly cannot close the freshly-registered-slop gap (a new fake name has no CVE). [VERIFIED per synthesis; OPS-RUNBOOK, deliberately out-of-plan]
- elai.map (F:\.ELAI_workflow\.claude\tools\elai.map) — every consumes_symbol + reachable_from anchor confirmed at authoring time (W-7): registry_check.py `_check_pypi`:90 / `_check_npm`:120 / `_isoparse`:43 / `main`:145; CheckerKind:62-94; package_verify.rs verify_pr_deps_no_slopsquat:51 / extract_dep_candidates:110 / flag_candidate:152 / is_dep_name_shape:191; hook_dispatcher.rs dispatch_pre_tool_use:1477 / for_pre_tool_use:324 / analyze:622; commands_e41.rs pr_review_status / lookup_u32:205; workspace.svelte.ts DrawerKey:374. [code-read, HIGH confidence]
- CLAUDE.md HARD RULES — blake3 not blake2/sha2; no .unwrap/.expect in prod; anti-slopsquatting (verify before add, no new dep without confirmation); Pattern-2 SSB single-variant-with-Kind collapse; SCHEMA_VERSION baseline 191.

## Open questions / risks

- UNVERIFIED design choice: the 1825d cargo stale floor is a reasoned default (stable crates publish infrequently), not a measured optimum. The 0-FP-on-20-real-deps acceptance validates it empirically; surface the threshold as a per-ecosystem tunable in the decision doc and flag it for the user.
- RISK (top failure mode 1): a too-eager dep-line detector false-fires on a non-dep `name = "..."` line (e.g. a `[package]` `name = "elai"` row in Cargo.toml) and blocks a legitimate write. MITIGATION: reuse `is_dep_name_shape` + restrict the cargo path to `[dependencies]`/`[dev-dependencies]`/`[build-dependencies]` sections only; the contract test (a) asserts non-dep writes return `Ok(vec![])`.
- RISK (top failure mode 2): the cache-miss subprocess stalls the FSM mid-EXECUTING (a 10s registry hang on every novel dep). MITIGATION: the Tier-2 path runs with the checker's existing `_http_get_json` 10s timeout AND the cache absorbs repeats; on timeout the path fail-opens with a `RegistryUnreachable` advisory row (never blocks the write on a network hang) — this is the documented Tier-2 guarantee, NOT a fail-closed promise.
- RISK (top failure mode 3): a fail-open under registry-unreachable silently passes a slopsquatted import (the exact section-2 hazard). MITIGATION: fail-open ALWAYS emits a `write_time_slopsquat_blocked{RegistryUnreachable}` advisory row + a dashboard advisory count; the operator sees that the gate degraded. The struct doc-comment and the decision doc both state this is best-effort, not fail-closed.
- DEPENDENCY RISK: D-11 has not shipped. If D-11's `registry_check.py` / `CheckerKind::RegistryCheck` are absent at F-9 start, the spec-grounder BLOCKS F-9 (W-7: cannot hook a non-existent symbol). F-9 MUST NOT re-implement the checker; it waits on D-11.
- NO new dep: the LRU cache is hand-rolled (bounded HashMap + VecDeque), NOT an `lru` crate, to honor "no new dependency without confirmation". If a vetted `lru` crate is later approved, it is a follow-up, not in F-9.
- COORDINATION: F-1's Tier-2 reserves `vibesec_dep_hallucinated_blocked` for the SAME axis inside the unified VibeGate. F-9's `write_time_slopsquat_blocked` is the standalone write-time variant; if F-1 and F-9 co-sequence, fold F-9's gate into VibeGate's dependency-axis evaluator and emit BOTH action_types or unify under one — decide at sequencing time, document in the decision doc, land under one SCHEMA bump.

## Next

F-1 (VibeGate) is the natural composition: F-9's `DepWriteTimeGate` becomes VibeGate's Tier-2 dependency-existence axis-evaluator, fanned alongside the code-vuln (F-2) and secret axes on the same PreToolUse event. The LRU existence cache F-9 builds is the substrate that lets VibeGate's dependency axis stay off the hot path. E-41's PR-leg stays as the cheap shape pre-filter; F-9 is the registry-grounded write-time leg; together they bracket the dep lifecycle (write-time + PR-time). The Socket Firewall install-boundary wrap (ops runbook, out-of-plan) is the third bracket at install-time — defense-in-depth across the three moments a slopsquatted dep can enter.
