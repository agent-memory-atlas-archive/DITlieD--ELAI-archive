F-9 hallucinated-dependency write-time gate decisions (session s-6344, Unit D)

What I will implement
Move D-11's real HTTP existence+publish-date check from the PR boundary to ALSO fire as a PreToolUse
hook the instant an import/use/Cargo.toml dep line is written; add cargo (crates.io) coverage; an LRU
existence cache keeps the network call OFF the hot per-write path; a new SSB write_time_slopsquat_blocked
variant; the `elai dep-check <path>` CLI entrypoint (the observable_effect); a DepsSecurityPanel dashboard
pair. (D-11 IS shipped - confirmed: checkers/registry/registry_check.py + CheckerKind::RegistryCheck.)

Post-Unit-A/B/C live baseline (RE-GREP): SCHEMA_VERSION 198, TOTAL_VARIANTS 475. F-9 bumps 198->199, 475->476.

Key decisions (grounded)
- NETWORK SPLIT (W-10): DepWriteTimeGate is constructed with an INJECTABLE registry-existence checker
  (a trait RegistryExistenceChecker or an Fn). The PRODUCTION impl shells out to registry_check.py
  (crates.io / pypi / npm - real HTTP, the D-11 path). The CONTRACT TEST + FIXTURE + the offline CLI mode
  use a DETERMINISTIC STUB (e.g. "reqquests" -> NotFound, "serde"/"requests" -> Found) so everything runs
  OFFLINE and deterministically. The LIVE network registry run is a SEPARATE gated checkbox (W-5). The
  in-process Tier-1 (cache hit) is genuinely fail-closed; the network Tier-2 (cache miss) is explicitly
  the network axis.
- SYNC HOT PATH: InProcessHookDispatcher::dispatch_pre_tool_use's analyze path is SYNC. So
  LruExistenceCache uses std::sync::Mutex (NOT tokio::sync::Mutex). Cache HIT = fast in-process verdict.
  Cache MISS on the sync hook path = a BOUNDED blocking std::process::Command call to the checker (with a
  short timeout), then cache (TTL ~1h). The async `elai dep-check` CLI path may use the full async
  subprocess. This keeps the network call off the per-write hot path (W-10).
- VERDICT semantics: WriteTimeSlopVerdictKind { NotFound, TooNew, RegistryUnreachable } (kind stored as
  String, the PrPackageVerifyResult analog). NotFound + TooNew -> HookVerdictKind::Block (fail-closed).
  RegistryUnreachable -> WARN (fail-OPEN on registry-down): per W-10 we do NOT advertise fail-closed for
  the network axis; a registry outage must not brick every write. Documented explicitly.
- DUAL BOOTSTRAP (W-3 - the recurring wire-dark trap; MANDATE both sites): InProcessHookDispatcher gains a
  dep_writetime_gate: Option<Arc<DepWriteTimeGate>> field + with_dep_writetime_gate builder; the gate is
  called in dispatch_pre_tool_use (hook_dispatcher.rs:862) after the E-30 bash check, before tiered
  analysis. build_production_hook_dispatcher gains an ssb param + .with_dep_writetime_gate(...) and MUST
  be updated at BOTH call sites: runtime_setup.rs:658 (CLI path) AND dashboard src-tauri lib.rs:1405
  (dashboard path) - or the gate fires in only one binary. The CLI `dep-check` path also constructs the
  gate directly.
- registry_check.py: add _check_cargo(pkg, max_age_days) hitting https://crates.io/api/v1/crates/{pkg}
  (crate.updated_at via _isoparse); add _DEFAULT_MAX_AGE_DAYS = {pypi:730, npm:730, cargo:1825}
  (stable Rust crates go years without a publish) replacing the hard-coded 730; add a 'cargo' dispatch arm.
- package_verify.rs: add extract_dep_candidates_rich(diff) -> Vec<DepCandidate{name, ecosystem}> with
  [dependencies]/[dev-dependencies]/[build-dependencies] TOML section tracking for cargo; KEEP
  extract_dep_candidates unchanged (PR-leg back-compat). NOTE: an Ecosystem enum already exists in
  crates/cli/src/dep_audit.rs:70 - use a distinct path (orchestrator::github_pr::package_verify::Ecosystem)
  or reuse; avoid an ambiguous duplicate import.
- SSB: SsbActionType::WriteTimeSlopsquatBlocked + WriteTimeSlopsquatBlockedPayload { dep_name_hash: u64
  (blake3 of the name, LOCAL fn - GhCliAdapter::hash_prefix is pub(super), inaccessible), ecosystem: String,
  verdict_kind: String, registry_age_days: Option<u32>, ts_ms: i64 }. SCHEMA 198->199, TOTAL_VARIANTS
  475->476; golden alphabetical insert; exhaustive_match_guard; all_test_payloads sample. blake3 only.
- CLI (W-5 entrypoint): CliAction::DepCheck { path: String } + crates/cli/src/dep_check_cmd.rs::run -
  reads added dep/import lines from the file, runs the gate, prints a BLOCK line + emits the SSB row.
- Frontend pair (W-4): commands_f9.rs::write_time_slop_status (mirror commands_e41 lookup helpers, do NOT
  import the private ones) + DepsSecurityPanel.svelte (DrawerKey 'deps-security'); update types.ts DrawerKey
  + Rail DrawerKeyUpper + Drawer.svelte DRAWER_LABELS + stub.ts + workspace.svelte.ts + src-tauri lib.rs
  mod + generate_handler.
- DEFAULT-ON (W-2): the gate is active by default, no default-off env gate.

Expected test outcome
cargo check --workspace GREEN. f9_dep_writetime_gate_contract (stub checker: hallucinated name -> Block
NotFound; real name -> Allow; too-new -> Block TooNew; registry-unreachable -> WARN). f9_ssb_roundtrip.
observable_effect (offline-stub mode): `elai dep-check tests/fixtures/f9_hallucinated_import.rs` (containing
`+use reqquests::get;`) -> stdout BLOCK line /reqquests .* not found/ + a write_time_slopsquat_blocked SSB
row (verdict_kind=NotFound). LIVE crates.io/pypi run = gated checkbox.

Unknowns / risks
- The sync-analyze blocking-subprocess timeout value (keep short, e.g. 1.5s) + fail-open on timeout (WARN).
- Both build_production_hook_dispatcher call sites MUST be updated (the W-3 trap - verify both compile + wire).
- Ecosystem naming collision with dep_audit.rs - keep paths distinct.
- registry_check.py is Python (sidecar/checker tooling under .elai_cc/checkers) - ruff-clean, no network in tests.
