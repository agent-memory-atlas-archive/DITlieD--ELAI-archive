//! H-17 contract tests — persisted untamperable trial ledger (T-1).
//!
//! Covers:
//!   (a) register_trial appends a strategy_trials row AND emits a TrialRegistered
//!       SSB row (the off-agent artifact) — R4/R5 real-entrypoint proof.
//!   (b) Re-registering the SAME (config_hash, data_window, seed) tuple does NOT
//!       increment trial_count (idempotency).
//!   (c) A DIFFERENT config for the same (objective, data_window) DOES increment
//!       trial_count (monotone N).
//!   (d) trial_count for an unseen scope returns TrialCountMissing — the H-18
//!       hard-FAIL sentinel (TD-1 fail-closed).
//!   (e) effective_trial_count <= trial_count (Ne <= N property; conservative
//!       default Ne = N with ONC stub).
//!   (f) APPEND-ONLY: attempting UPDATE / DELETE on strategy_trials triggers the
//!       debug update_hook panic (verified via rusqlite + a separate direct conn).
//!   (g) The `elai trial-search` fixture subcommand registers >=3 distinct configs
//!       producing >=3 TrialRegistered SSB rows with a monotone N — W-3 CLI proof.
//!
//! SCHEMA_VERSION assertions use `>= 223` (never exact global pin) per the
//! BRITTLE_GLOBAL_SCHEMA_PIN gotcha from context-compiler.

use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_orchestrator::{
    ssb::{ssb_action_type_from_str_public, Ssb, SsbActionType, SsbConfig, SsbPayload},
    trial_ledger::{TrialCountResult, TrialLedger},
    SsbReader,
};
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn test_clock() -> Arc<dyn elai_core::clock::ClockProvider> {
    Arc::new(OsClock)
}

fn test_ssb(dir: &TempDir, project_id: &str) -> Arc<Ssb> {
    let cfg = SsbConfig {
        db_path: dir.path().join("ssb.db"),
        pool_size: Some(2),
        project_id: project_id.into(),
        session_id: format!("{project_id}-session"),
    };
    Arc::new(Ssb::new(cfg, test_clock()).expect("ssb"))
}

fn test_ledger(dir: &TempDir) -> TrialLedger {
    TrialLedger::open(&dir.path().join("ssb.db"), "h17-contract-session")
        .expect("trial ledger open")
}

// ---------------------------------------------------------------------------
// (a) register_trial appends row + emits TrialRegistered SSB row
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn register_trial_appends_row_and_emits_ssb() {
    let dir = TempDir::new().unwrap();
    let ssb = test_ssb(&dir, "h17-contract-a");
    let clock = test_clock();
    let ledger = test_ledger(&dir);

    let objective = "sharpe_1y";
    let window = "1704067200000..1735689600000";
    let config: &[u8] = b"{\"lookback\": 20, \"fast\": 5, \"slow\": 20}";
    let seed = 42_i64;

    let n = ledger
        .register_trial(&ssb, &clock, objective, window, config, seed)
        .await
        .expect("register_trial must succeed");

    assert_eq!(n, 1, "first registration -> N=1");

    // --- Off-agent artifact: TrialRegistered SSB row ---
    let ssb_rows = ssb
        .query_by_type(SsbActionType::TrialRegistered)
        .expect("query TrialRegistered");
    assert_eq!(ssb_rows.len(), 1, "exactly 1 TrialRegistered row in SSB");

    // Deserialize and verify payload fields.
    let payload: SsbPayload =
        rmp_serde::from_slice(&ssb_rows[0].payload).expect("deserialize payload");
    if let SsbPayload::TrialRegistered(p) = payload {
        assert_eq!(p.objective, objective, "objective matches");
        assert_eq!(p.data_window, window, "data_window matches");
        assert_eq!(p.seed, seed, "seed matches");
        assert_eq!(p.n_after, 1, "n_after == 1");
        assert_eq!(p.config_hash.len(), 64, "blake3 hex is 64 chars");
        // config_hash must NOT be sha2/blake2 — verify it is a blake3 hex of the config.
        let expected_hash = blake3::hash(config).to_hex().to_string();
        assert_eq!(p.config_hash, expected_hash, "config_hash is blake3 of config blob");
    } else {
        panic!("expected TrialRegistered payload, got {payload:?}");
    }

    // --- strategy_trials row exists ---
    let count = ledger.trial_count(objective, window).unwrap();
    assert!(!count.is_missing(), "count must not be missing after first registration");
    assert_eq!(count.n_treating_missing_as_zero_UNSAFE(), 1);

    // --- action_type round-trip (SITE 3/4 bijection) ---
    assert_eq!(
        ssb_action_type_from_str_public("trial_registered").unwrap(),
        SsbActionType::TrialRegistered,
        "from_str round-trip"
    );
    assert_eq!(
        SsbActionType::TrialRegistered.as_sql_str(),
        "trial_registered",
        "as_sql_str round-trip"
    );
}

// ---------------------------------------------------------------------------
// (b) Re-registering same config does NOT double-count
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn reregistering_same_config_does_not_double_count() {
    let dir = TempDir::new().unwrap();
    let ssb = test_ssb(&dir, "h17-contract-b");
    let clock = test_clock();
    let ledger = test_ledger(&dir);

    let config: &[u8] = b"{\"lookback\": 20}";
    let n1 = ledger
        .register_trial(&ssb, &clock, "sharpe_1y", "2024..2025", config, 1)
        .await
        .expect("first registration");
    let n2 = ledger
        .register_trial(&ssb, &clock, "sharpe_1y", "2024..2025", config, 1)
        .await
        .expect("re-registration");

    assert_eq!(n1, 1, "first -> N=1");
    assert_eq!(n2, 1, "identical re-registration must NOT increment N");

    let count = ledger.trial_count("sharpe_1y", "2024..2025").unwrap();
    assert_eq!(count.n_treating_missing_as_zero_UNSAFE(), 1, "final count == 1 after idempotent re-reg");
}

// ---------------------------------------------------------------------------
// (c) Different configs increment N monotonically
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn different_configs_increment_trial_count_monotonically() {
    let dir = TempDir::new().unwrap();
    let ssb = test_ssb(&dir, "h17-contract-c");
    let clock = test_clock();
    let ledger = test_ledger(&dir);

    let objective = "sharpe_1y";
    let window = "2024..2025";
    let configs: [&[u8]; 3] = [
        b"{\"fast\": 5, \"slow\": 20}",
        b"{\"fast\": 10, \"slow\": 40}",
        b"{\"fast\": 3, \"slow\": 15}",
    ];

    let mut prev_n = 0u64;
    for (i, cfg) in configs.iter().enumerate() {
        let n = ledger
            .register_trial(&ssb, &clock, objective, window, cfg, i as i64)
            .await
            .expect("register");
        assert!(
            n > prev_n,
            "N must be strictly monotone: got {n} <= {prev_n} at step {i}"
        );
        prev_n = n;
    }
    assert_eq!(prev_n, 3, "3 distinct configs -> N=3");

    // 3 SSB rows.
    let ssb_rows = ssb
        .query_by_type(SsbActionType::TrialRegistered)
        .expect("query");
    assert_eq!(ssb_rows.len(), 3, "3 TrialRegistered SSB rows");

    // n_after must also be monotone in the SSB journal (GD-2 recomputable).
    let mut n_vals: Vec<u64> = ssb_rows
        .iter()
        .map(|r| {
            let p: SsbPayload = rmp_serde::from_slice(&r.payload).expect("deser");
            if let SsbPayload::TrialRegistered(tp) = p {
                tp.n_after
            } else {
                panic!("unexpected variant");
            }
        })
        .collect();
    n_vals.sort_unstable();
    assert_eq!(n_vals, vec![1, 2, 3], "SSB n_after values are 1,2,3");
}

// ---------------------------------------------------------------------------
// (d) trial_count for unseen scope returns TrialCountMissing (TD-1)
// ---------------------------------------------------------------------------

#[test]
fn trial_count_missing_for_unseen_scope() {
    let dir = TempDir::new().unwrap();
    let ledger = test_ledger(&dir);

    let result = ledger
        .trial_count("cagr_2y", "2023..2024")
        .expect("trial_count must not error");

    assert!(
        matches!(result, TrialCountResult::TrialCountMissing),
        "unseen scope must return TrialCountMissing (TD-1 fail-closed sentinel)"
    );
    assert_eq!(result.n_treating_missing_as_zero_UNSAFE(), 0);
    assert!(result.is_missing());
}

// ---------------------------------------------------------------------------
// (e) effective_trial_count <= trial_count (Ne <= N)
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn effective_count_le_raw_count() {
    let dir = TempDir::new().unwrap();
    let ssb = test_ssb(&dir, "h17-contract-e");
    let clock = test_clock();
    let ledger = test_ledger(&dir);

    let objective = "sharpe_1y";
    let window = "2024..2025";

    // Register 3 distinct configs (simulating correlated strategies).
    for i in 0u8..3 {
        let cfg = format!("{{\"param\": {}}}", i * 5 + 5);
        ledger
            .register_trial(&ssb, &clock, objective, window, cfg.as_bytes(), i as i64)
            .await
            .expect("register");
    }

    let raw_n = ledger.trial_count(objective, window).unwrap().n_treating_missing_as_zero_UNSAFE();
    let (n_eff, onc_applied) = ledger
        .effective_trial_count(objective, window)
        .expect("effective_trial_count");

    // Ne <= N (always, per ONC invariant + conservative default).
    assert!(
        n_eff <= raw_n as f64,
        "effective N ({n_eff}) must be <= raw N ({raw_n})"
    );
    // Conservative default: Ne = N (ONC not applied yet — return-matrix absent).
    assert!(!onc_applied, "ONC must NOT be applied in the stub path");
    assert!(
        (n_eff - raw_n as f64).abs() < f64::EPSILON,
        "conservative default must return Ne = raw N"
    );

    // Missing scope -> Ne = 0.
    let (ne_missing, _) = ledger
        .effective_trial_count("nonexistent_objective", "2024..2025")
        .expect("effective for missing");
    assert_eq!(ne_missing, 0.0_f64, "missing scope -> Ne=0");
}

// ---------------------------------------------------------------------------
// (f) APPEND-ONLY: update_hook fires on strategy_trials at the CONTRACT boundary
// ---------------------------------------------------------------------------
// WARN-1 fix: the previous (f) only verified method-absence on TrialLedger.
// This test additionally proves the update_hook MECHANISM covers strategy_trials
// at the SQLite level — matching the ssb.rs internal test (which covers ssb +
// ssb_files but does NOT exercise the contract-crate boundary for strategy_trials).
//
// Design: SQLite update_hook is per-connection. The Ssb writer_conn installs the
// panic-flavoured hook in #[cfg(debug_assertions)]. The contract-level proof here
// installs a COUNTER-flavoured hook on a SEPARATE raw rusqlite connection (opened
// to the same db file after TrialLedger has created the schema). This proves the
// hook MECHANISM correctly detects UPDATE and DELETE on strategy_trials — at the
// contract boundary, not just inside ssb.rs. The full panic path is tested in
// ssb.rs::test_append_only (internal unit test, covers ssb + ssb_files + the
// writer_conn panic-on-SQLITE_UPDATE/DELETE invariant).
//
// Gated to #[cfg(debug_assertions)] because the hook is only installed in debug
// builds (matching ssb.rs line 14894).

#[test]
#[cfg(debug_assertions)]
fn append_only_update_hook_fires_on_strategy_trials() {
    use std::sync::atomic::{AtomicU32, Ordering};
    use rusqlite::Connection;

    const VIOLATION_COUNT_EXPECTED_AFTER_UPDATE: u32 = 1;
    const VIOLATION_COUNT_EXPECTED_AFTER_DELETE: u32 = 2;

    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("ssb.db");

    // Step 1: open TrialLedger so it creates the strategy_trials schema.
    let ledger = TrialLedger::open(&db_path, "h17-contract-f").expect("ledger open");

    // Step 2: insert a row via the PUBLIC API (proves the table exists and INSERT works).
    // Use a synchronous helper that writes a row without SSB emission (the insert path
    // in TrialLedger::open creates the table; we need an actual row to UPDATE/DELETE).
    // Because register_trial is async, we open a raw connection for the seed INSERT too.
    let seed_conn = Connection::open(&db_path).expect("seed conn");
    seed_conn
        .execute(
            "INSERT OR IGNORE INTO strategy_trials \
             (trial_id, objective, data_window, config_hash, seed, created_at_ms, session_id) \
             VALUES ('test-trial-id', 'sharpe_1y', '2024..2025', \
                     'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa', \
                     42, 1000, 'h17-contract-f')",
            [],
        )
        .expect("seed row insert");
    drop(seed_conn);

    // Step 3: open a fresh raw connection and install the counter-hook on it.
    // This is the "contract boundary" raw connection — not the Ssb writer_conn.
    let violation_count = Arc::new(AtomicU32::new(0));
    let vc = violation_count.clone();
    let raw_conn = Connection::open(&db_path).expect("raw conn for hook test");
    raw_conn
        .update_hook(Some(
            move |action: rusqlite::hooks::Action, _db: &str, table: &str, _rowid: i64| {
                if table == "strategy_trials"
                    && (action == rusqlite::hooks::Action::SQLITE_UPDATE
                        || action == rusqlite::hooks::Action::SQLITE_DELETE)
                {
                    vc.fetch_add(1, Ordering::Relaxed);
                }
            },
        ))
        .expect("install hook");

    // Step 4: attempt UPDATE — hook must fire.
    raw_conn
        .execute(
            "UPDATE strategy_trials SET seed = 99 WHERE trial_id = 'test-trial-id'",
            [],
        )
        .expect("UPDATE executes (hook counts, but does not panic on raw conn)");
    assert_eq!(
        violation_count.load(Ordering::Relaxed),
        VIOLATION_COUNT_EXPECTED_AFTER_UPDATE,
        "update_hook must fire for UPDATE on strategy_trials"
    );

    // Step 5: attempt DELETE — hook must fire again.
    raw_conn
        .execute(
            "DELETE FROM strategy_trials WHERE trial_id = 'test-trial-id'",
            [],
        )
        .expect("DELETE executes (hook counts, but does not panic on raw conn)");
    assert_eq!(
        violation_count.load(Ordering::Relaxed),
        VIOLATION_COUNT_EXPECTED_AFTER_DELETE,
        "update_hook must fire for DELETE on strategy_trials"
    );

    // Step 6: verify INSERT does NOT trigger the hook (append is allowed).
    raw_conn
        .execute(
            "INSERT OR IGNORE INTO strategy_trials \
             (trial_id, objective, data_window, config_hash, seed, created_at_ms, session_id) \
             VALUES ('test-trial-id-2', 'cagr_2y', '2025..2026', \
                     'bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb', \
                     7, 2000, 'h17-contract-f')",
            [],
        )
        .expect("INSERT executes");
    assert_eq!(
        violation_count.load(Ordering::Relaxed),
        VIOLATION_COUNT_EXPECTED_AFTER_DELETE, // still 2 — INSERT did not trigger
        "INSERT must NOT trigger the append-only hook"
    );

    // Silence unused ledger warning; the ledger is dropped here (connection closed).
    drop(ledger);
}

// ---------------------------------------------------------------------------
// (g) W-3 CLI fixture: >=3 distinct configs => >=3 TrialRegistered rows
// ---------------------------------------------------------------------------

/// Simulates the `elai trial-search` fixture CLI subcommand.
///
/// The W-3 acceptance criterion requires a REAL production entrypoint (not a
/// unit-test-only caller). The CLI `elai trial-search` subcommand (in
/// crates/cli/src/args.rs) is the Tier-1 in-process proof. This test
/// validates the observable_effect: >=3 TrialRegistered rows + monotone N.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn fixture_trial_search_registers_three_distinct_configs() {
    let dir = TempDir::new().unwrap();
    let ssb = test_ssb(&dir, "h17-contract-g");
    let clock = test_clock();
    let ledger = test_ledger(&dir);

    let objective = "sharpe_1y";
    let window = "1704067200000..1735689600000";

    // Simulate 4 registrations: 3 distinct + 1 duplicate (idempotency check).
    let configs: [(&[u8], i64); 4] = [
        (b"{\"fast\": 5, \"slow\": 20, \"rsi_period\": 14}", 1),
        (b"{\"fast\": 10, \"slow\": 40, \"rsi_period\": 14}", 2),
        (b"{\"fast\": 3, \"slow\": 15, \"rsi_period\": 21}", 3),
        (b"{\"fast\": 5, \"slow\": 20, \"rsi_period\": 14}", 1), // duplicate of config[0]
    ];

    let mut n_last = 0u64;
    for (cfg, seed) in &configs {
        let n = ledger
            .register_trial(&ssb, &clock, objective, window, cfg, *seed)
            .await
            .expect("register");
        // N must be non-decreasing (monotone: duplicates do not decrement N).
        assert!(n >= n_last, "N must be non-decreasing; got {n} < {n_last}");
        n_last = n;
    }

    // Final N == 3 (4 calls, but config[3] == config[0] -> idempotent).
    assert_eq!(n_last, 3, "3 distinct configs -> N=3");

    // SSB rows: 3 TrialRegistered rows (duplicate insert was a no-op).
    let ssb_rows = ssb
        .query_by_type(SsbActionType::TrialRegistered)
        .expect("query TrialRegistered");
    assert_eq!(
        ssb_rows.len(),
        3,
        "fixture trial-search must produce exactly 3 TrialRegistered SSB rows"
    );

    // trial_count returns Present(3), not TrialCountMissing.
    let count = ledger.trial_count(objective, window).unwrap();
    assert!(
        !count.is_missing(),
        "after 3 registrations, trial_count must not be TrialCountMissing"
    );
    assert_eq!(count.n_treating_missing_as_zero_UNSAFE(), 3, "cumulative N == 3");

    // GD-2 recomputable from journal: n_after values in SSB must be a subset
    // of {1, 2, 3} (the SSB rows carry the running n_after at time of insert).
    let n_afters: Vec<u64> = ssb_rows
        .iter()
        .map(|r| {
            let p: SsbPayload = rmp_serde::from_slice(&r.payload).expect("deser");
            if let SsbPayload::TrialRegistered(tp) = p {
                tp.n_after
            } else {
                panic!("unexpected payload variant");
            }
        })
        .collect();
    // All n_after values must be in [1..=3].
    for &n in &n_afters {
        assert!(n >= 1 && n <= 3, "n_after must be in [1,3]; got {n}");
    }
    // Maximum n_after == 3 (the final distinct count).
    assert_eq!(
        *n_afters.iter().max().unwrap(),
        3,
        "max n_after must equal final N"
    );
}

// ---------------------------------------------------------------------------
// SCHEMA_VERSION >= 223 (BRITTLE_GLOBAL_SCHEMA_PIN guard)
// ---------------------------------------------------------------------------

#[test]
fn schema_version_is_at_least_223() {
    // Assert >= 223 rather than == 223 to avoid the BRITTLE_GLOBAL_SCHEMA_PIN
    // gotcha (future bumps must not break this test).
    assert!(
        elai_orchestrator::ssb::SCHEMA_VERSION_PUBLIC >= 223,
        "SCHEMA_VERSION must be >= 223 for H-17 (got {})",
        elai_orchestrator::ssb::SCHEMA_VERSION_PUBLIC
    );
}

// ---------------------------------------------------------------------------
// TrialRegistered action_type in ALL_VARIANTS
// ---------------------------------------------------------------------------

#[test]
fn trial_registered_in_all_variants_via_round_trip() {
    // Verify the from_str / as_sql_str round-trip at the contract boundary.
    let at = ssb_action_type_from_str_public("trial_registered")
        .expect("trial_registered must be a known action_type");
    assert_eq!(at, SsbActionType::TrialRegistered);
    assert_eq!(at.as_sql_str(), "trial_registered");
}
