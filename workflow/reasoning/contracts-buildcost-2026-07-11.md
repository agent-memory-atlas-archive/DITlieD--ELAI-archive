contracts build-cost driver — Phase 3 findings (ram-build-efficiency plan, unit G)

session: ram-P3 executor, 2026-07-11
scope: crates/contracts (elai_cc_contracts) — 526 integration-test files, each compiled as its own [[test]] binary. That per-file binary explosion is the build-cost / link-cost driver this phase investigates and pilots a fix for.

========================================================================
D1 — stray-dirs root cause (RECON_NEEDED R2, INVESTIGATE ONLY, NO DELETION)
========================================================================

THE THREE STRAY DIRS (under crates/contracts/, all git-UNTRACKED — confirmed via `git ls-files workflow .elai_cc C` returns empty):
  1. crates/contracts/workflow/sessions/f13-drop-on/archive/ssb-v205.sqlite (+ f13-enabled/…)
  2. crates/contracts/.elai_cc/sessions/ssb.db  +  .elai_cc/sessions/default/agents/<uuid>/…
  3. crates/contracts/C/Users/user/AppData/Local/Temp/elai-e25-edit-checkpoints-contract-<pid>-…/beta/file.txt

ROOT CAUSE (single class): production code resolves default session/archive/snapshot paths from CWD-RELATIVE string literals (or relativizes an absolute drive path). Under `cargo test`, the test-process CWD is the crate manifest dir = crates/contracts, so any test that does not fully override the root leaks its writes there. The tests are NOT individually buggy path-builders; they exercise production defaults that assume the process CWD is the project root.

PER-DIR ATTRIBUTION (offending file:line):

(1) workflow/sessions/f13-* — session-archive default path is CWD-relative.
    Production: crates/orchestrator/src/session_archive.rs:45  `const SESSIONS_ROOT: &str = "workflow/sessions";`
               crates/orchestrator/src/session_archive.rs:115 `PathBuf::from(SESSIONS_ROOT)` (…join(session_id).join("archive")…) — a relative base, resolved against process CWD.
    Exercised by:
      crates/contracts/tests/session_archive_drop_preserved_contract.rs  (session ids "f13-drop-on", "f13-drop-off")
      crates/contracts/tests/session_archive_ssb_roundtrip_contract.rs    (session id "f13-enabled")
    Mechanism: those tests wrap the migration in an EnvScope (session_archive_drop_preserved_contract.rs:28-63) that `std::env::set_current_dir(tmp)` under a process-global ENV_LOCK mutex and restores prev_cwd on Drop. CWD is PROCESS-GLOBAL, but cargo runs a binary's #[test]s on multiple threads by default — so a concurrent thread's archive-on-bump (or the archive write landing in the window after Drop restored prev_cwd == crates/contracts) resolves `workflow/sessions/…` against crates/contracts and leaks the v205 archive there. The in-tmp assertions (…:141-149) pass because they check the tmp copy; the leaked crates/contracts copy is never cleaned.

(2) .elai_cc/sessions/default/agents/<uuid> (+ .elai_cc/sessions/ssb.db) — agent-pool + SSB defaults are CWD-relative.
    Production: crates/orchestrator/src/agent_pool.rs:207 `scratch_base: PathBuf::from(".elai_cc/sessions/default/agents")` — a relative default; when an AgentPool is built without overriding scratch_base it mkdir's `.elai_cc/sessions/default/agents/<uuid>` under CWD.
    The sibling `.elai_cc/sessions/ssb.db` is the SSB default (subdir const SSB_SESSIONS_SUBDIR = ".elai_cc/sessions", re-exported at orchestrator/src/lib.rs:1757; path_unification.rs:174 pins it) resolved the same CWD-relative way when a test constructs an Ssb/agent-pool on the default project layout rather than a tempdir db_path.
    NOTE: the two in-crate direct callers agent_pool_executor_wiring_contract.rs:211 and c9_autonomous_infra_contract.rs:403 both root at `tmp.path()` (tempdir) — they do NOT leak. The leak comes from a test that drives the DEFAULT agent-pool/SSB layout (session "default") without a tempdir override; the leaked `default/agents/<uuid>` tree is the agent_pool.rs:207 default firing against CWD.

(3) C/Users/… — the edit-checkpoint snapshot encoder relativizes an ABSOLUTE drive path into a CWD-relative path.
    Exercised by: crates/contracts/tests/e25_edit_checkpoints_contract.rs
      fresh_test_root() (:21-35) roots each case at std::env::temp_dir() → an ABSOLUTE Windows path `C:\Users\user\AppData\Local\Temp\elai-e25-edit-checkpoints-contract-<pid>-…`.
      Tests then call snapshot_pre_edit(&root, "s-c4", &src, …) with src = root.join("beta").join("file.txt") (:57) — an absolute path.
    Mechanism: the production edit-checkpoint snapshot path builder ENCODES the source path into the snapshot tree by stripping the drive colon and converting `\`→`/` (`C:\Users\…` → `C/Users/…`), producing a RELATIVE path that resolves under CWD (crates/contracts). The leaked file `C/Users/user/AppData/Local/Temp/elai-e25-…/beta/file.txt` is that encoded source mirror. The test cleans its tempdir root (fs::remove_dir_all(&root)) but never the CWD-relative `C/Users/…` encoded mirror.

WHY NOT DELETED: plan Scope boundary — "No deletion of the 3 stray dirs under crates/contracts (investigate only)." The dirs are git-untracked local test residue; harmless to git, and the real fix (make the three production defaults resolve against an explicit project-root arg / ELAI_* env rather than process CWD, and make e25 pass a relative src or an in-tmp checkpoint root) is a separate change out of this phase's scope. Recorded here for the follow-up.

REMEDY SKETCH (for a future unit, NOT done here):
  - session_archive.rs / agent_pool.rs / SSB: thread an explicit project_root PathBuf (or read a resolved ELAI project-root env) instead of `PathBuf::from("<relative literal>")`; tests already pass a tempdir root to the direct callers, so the leak is only via the default-layout path.
  - e25 snapshot encoder: either root the encoded snapshot tree under the provided checkpoint root (absolute), or have the e25 contract pass an in-tmp src so no absolute drive path is encoded CWD-relative.

========================================================================
D2 + D3 — grouped [[test]] pilot (LANDED + VERIFIED)  session ram-P3, 2026-07-11
========================================================================

PLAN BUG (surfaced for the auditor's off-agent VERIFY): the plan's Phase-3 commands all say `-p elai-contracts`, but the real cargo package name is `elai_cc_contracts` (crates/contracts/Cargo.toml:2). `-p elai-contracts` exits 101 "package ID specification did not match any packages". EVERY off-agent re-run MUST use `-p elai_cc_contracts`.

HOST DISK FINDING (the headline — build-cost driver made concrete):
  `cargo nextest list -p elai_cc_contracts` (= `cargo test --no-run` = full codegen + LINK of every [[test]] target) FAILED at exit 101 during linking:
    - `fatal error LNK1180: insufficient disk space to complete link`
    - `LINK : fatal error LNK1318: Unexpected PDB error; LIMIT (12)`
  F: drive was at 0 bytes free (Get-PSDrive F: Free=0, 446.52 GB used). The 526 per-file test binaries each emit a debug .exe+.pdb(+.lib+.exp); that link set overran the volume.
  `cargo clean -p elai_cc_contracts` (plan M3-authorized) freed 22.3 GiB across 13751 files — i.e. the contracts test-binary artifacts ALONE were ~22GB and were what filled the disk. This is the strongest possible evidence for the consolidation: the fix is not "faster build", it is "makes the link set fit on the volume at all".

PRE-EXISTING RED (not this phase's change — surfaced for the auditor):
  `cargo check -p elai_cc_contracts --tests` fails to compile ONE target: crates/contracts/tests/agent_role_dispatcher_contract.rs:91 — `error[E0004]` non-exhaustive `match` missing `ProviderKind::Gpt55Proxy` (the variant added by the uncommitted crates/providers/src/auth_broker.rs work in `git status`; the test's arm list was not updated). One error, one target, NOT in the pilot set, no shared-dep impact. Baseline (pre-pilot) `cargo check --tests` = RED at 105.7s (time-to-first-error, aborts early). This must be fixed separately (add the `ProviderKind::Gpt55Proxy` arm to agent_role_dispatcher_contract.rs) before the full contracts suite checks clean; it is orthogonal to this pilot.

D2 PILOT (LANDED): 13 smallest contract test files with NO shared-state markers (static/once_cell/serial_test) AND no file-relative macros were moved (via `mv` = content-preserving rename) from crates/contracts/tests/<name> into crates/contracts/tests/grouped_pilot/<name>, then #[path]-declared as isolated modules in tests/grouped_pilot/main.rs, with an explicit `[[test]] name="grouped_pilot"` added to Cargo.toml.
  Files (13): mcp_transport_streamable_http_contract, llm_guard_decision_contract, config_basic, tool_basic, llama_cpp_runtime_adapter_contract, seed_builder_contract, runtime_chain_no_default_mlx_llamacpp, gateway_rejection_startup_warning, provider_trait, mlx_runtime_adapter_contract, a46_context_telescope_contract, h6_masking_readback_contract, h4_tool_search_beta_contract.
  Excluded (and why): catrag_contract.rs + arag_contract.rs carry inner `#![…]` crate-attributes (illegal inside a `mod`); vllm_runtime_contract.rs uses `include_str!("../../providers/…")` (a file-relative path that breaks when the file is relocated into the subdir). h4/h6 were KEPT despite reading source because they anchor on `env!("CARGO_MANIFEST_DIR")` (crate-root, move-invariant), not a file-relative path.
  Collisions: NONE. Each file is wrapped in its own module, so the two `_assert_impl_runtime_adapter` fns (llama_cpp + mlx) and the two `read_src`/`crates_dir` helpers (h4 + h6) coexist. Zero M2 swaps were needed.

D3 MEASURED ARTIFACTS:
  BINARY-COUNT (deterministic, PRIMARY per M3 ON_FAIL — no build needed):
    top-level tests/*.rs link targets: 526 BEFORE  ->  513 AFTER  (verified `ls crates/contracts/tests/*.rs | wc -l`).
    total [[test]] link targets: 526 BEFORE  ->  514 AFTER (513 top-level + 1 grouped_pilot).
    NET REDUCTION = 12 link targets (= pilot_file_count - 1 = 13 - 1), exactly as M3 EXPECTED_OBSERVATION predicts. Extrapolated: consolidating the full 526-file suite into ~G grouped binaries collapses the link set from 526 toward G, which is the lever that makes it fit on disk.
  TEST-COUNT PARITY (by construction): the 13 files were relocated byte-identical (rename, not rewrite); all 36 `#[test]`/`#[tokio::test]` annotations (1 `#[ignore]`, several cfg-gated) are preserved as modules; ZERO tests dropped, ZERO `#[ignore]` added. The compiled/listed test count is identical before and after because the same cfg resolution applies in both the standalone and grouped binaries. Numeric `cargo nextest list` parity is DEFERRED to the auditor (it needs the link that the disk cannot currently complete) but holds trivially by construction.
  WALL-CLOCK: INCONCLUSIVE (M3 ON_FAIL, honest — no fabricated win). A clean full-suite before/after `cargo check --tests` wall-clock is not obtainable here: the run aborts early at the unrelated pre-existing agent_role_dispatcher error (nondeterministic stop point), and the 12 removed frontend compiles are <2.5% of 526 = within noise. The link-based measurement cannot run at all (disk). What IS verified: the grouped binary compiles clean in isolation — `cargo check -p elai_cc_contracts --test grouped_pilot` exit 0 in 18.3s (one pre-existing unused-import warning in the moved mlx file, not introduced here).

VERIFICATION (this session, self-verify; auditor re-runs off-agent with `-p elai_cc_contracts`):
  - `cargo check -p elai_cc_contracts --test grouped_pilot` -> exit 0 (grouped binary + all 13 modules compile; the reorg is sound).
  - top-level target count 526 -> 513 (`ls tests/*.rs | wc -l`).
  - The full-suite `cargo check --tests` / `nextest list` count-parity is the AUDITOR's off-agent leg once disk headroom exists AND the pre-existing agent_role_dispatcher error is fixed; both are documented above so the auditor does not mistake them for pilot regressions.

CONSOLIDATION PROPOSAL (D2, for a future letter-plan — NOT executed here):
  The pilot proves the mechanism is safe and mechanical for the "pure logic/type contract" class (no shared static, no file-relative macros, no inner crate-attrs). To roll it out suite-wide: bucket the 526 files by concern (e.g. ssb-action-type contracts, runtime-adapter contracts, source-grep contracts) into a handful of grouped [[test]] binaries, EXCLUDING three residual classes that need per-file handling — (1) files with process-global side effects (CWD chdir / env mutation under a shared lock, e.g. the session_archive + e25 writers in D1 above), which must stay isolated or be made hermetic first; (2) files with inner `#![…]` crate-attributes; (3) files with file-relative `include_str!/include!` paths (rewrite those to `env!("CARGO_MANIFEST_DIR")`-anchored reads first, as h4/h6 already are). That collapses the 526-binary link explosion toward a few dozen, which is what lets the suite link on this host.
