//! I-46 bash interceptor contract test.
//!
//! Coverage (11 arms; no #[ignore], no stubs, no placeholder asserts):
//!
//! (F1) Redirect arm — grep redirected to grep_search (structural no-spawn proof).
//!      Verifies classify_redirect returns Some with target_tool=="grep_search";
//!      run_bash caller sees Err (never Ok) so execute_bash is structurally unreachable.
//!
//! (F2) Redirect arm — cat/head/tail redirected to read_file.
//!
//! (F3) Redirect arm — find with flag redirected to glob_search.
//!
//! (F4) Fixup arm — command with trailing "| head" is stripped and returns the
//!      rewritten command; result is Err (BashFixedUp structural no-spawn).
//!
//! (F5) Allowed-prefix arm — "cargo build" passes through (Allowed); no interception.
//!
//! (F6) PrefixBlocked arm — "cargo build; rm -rf target" is blocked (PrefixBlocked).
//!
//! (F7-F13) Ported coverage from the 7 routing_rules tests deleted by I-46 OPTION A:
//!   (F7)  cat_file_is_redirected (was cat_file_is_rejected via routing_rules rule 2)
//!   (F8)  head_file_is_redirected (was head_file_is_rejected)
//!   (F9)  tail_file_is_redirected (was tail_file_is_rejected)
//!   (F10) find_with_glob_is_redirected (was find_with_glob_is_rejected via rule 4)
//!   (F11) ls_with_glob_is_redirected (was ls_with_glob_is_rejected via rule 4)
//!   (F12) grep_is_redirected (was grep_is_rejected via rule 5)
//!   (F13) rg_is_redirected (was rg_is_rejected via rule 5)
//!
//! (F14) SSB round-trip — BashRedirectedPayload (all 3 kind arms: Redirected /
//!       FixedUp / PrefixBlocked) round-trips through write_payload + query_by_type
//!       + msgpack decode; catches SQL CHECK / wire-tag mismatch invisible to cargo check.
//!
//! (F15) Schema-lock floor — SCHEMA_VERSION_PUBLIC >= 244 AND
//!       action_type_lock.rs TOTAL_VARIANTS == 553 AND
//!       "bash_redirected" round-trips via ssb_action_type_from_str_public /
//!       as_sql_str.
//!
//! Spec references: docs/thefinalglance/I-plan/I-46_bash_interceptor.md
//! Reads (mandatory): wiring-enforcement.md R1/R3/R4/R6/R7/R8; CLAUDE.md anti-stale-training.

use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_orchestrator::ssb::{
    BashRedirectedKind, BashRedirectedPayload, Ssb, SsbActionType, SsbConfig, SsbPayload,
    SCHEMA_VERSION_PUBLIC, ssb_action_type_from_str_public,
};
use elai_orchestrator::SsbReader;
use elai_tools::bash_interceptor::{
    AllowedPrefixResult, ALLOWED_PREFIXES, apply_fixup, check_allowed_prefix, classify_redirect,
};
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn ssb_config(dir: &TempDir, label: &str) -> SsbConfig {
    SsbConfig {
        db_path: dir.path().join(format!("{label}.db")),
        pool_size: Some(2),
        project_id: "i46-bash-interceptor-contract".to_string(),
        session_id: format!("i46-contract-{label}"),
    }
}

fn clock() -> Arc<dyn elai_core::clock::ClockProvider> {
    Arc::new(OsClock)
}

// ---------------------------------------------------------------------------
// (F1) Redirect arm — grep -> grep_search (structural no-spawn proof)
// ---------------------------------------------------------------------------

/// classify_redirect("grep -r 'fn main' src/") returns Some with grep_search.
///
/// No-spawn structural proof: classify_redirect returns Some, which causes
/// run_bash to return Err(structured_error("bash_redirected", ...)).
/// execute_bash is structurally unreachable for this input.
/// R1: missing evidence = FAIL. A None return would be fail-open.
#[test]
fn f1_grep_redirected_to_grep_search() {
    let decision = classify_redirect("grep -r 'fn main' src/");
    assert!(
        decision.is_some(),
        "classify_redirect must intercept grep (return Some); \
         a None means the interceptor is fail-open for this verb"
    );
    let d = decision.unwrap();
    assert_eq!(
        d.target_tool, "grep_search",
        "grep must redirect to grep_search, not {}",
        d.target_tool
    );
    assert!(
        !d.message.is_empty(),
        "redirect decision message must not be empty"
    );
}

// ---------------------------------------------------------------------------
// (F2) Redirect arm — cat/head/tail -> read_file
// ---------------------------------------------------------------------------

/// cat, head, and tail all redirect to read_file.
#[test]
fn f2_cat_head_tail_redirect_to_read_file() {
    for verb in &["cat README.md", "head -n 10 src/main.rs", "tail -f syslog"] {
        let d = classify_redirect(verb);
        assert!(
            d.is_some(),
            "classify_redirect must intercept '{verb}'; a None is fail-open"
        );
        assert_eq!(
            d.unwrap().target_tool,
            "read_file",
            "'{verb}' must redirect to read_file"
        );
    }
}

// ---------------------------------------------------------------------------
// (F3) Redirect arm — find with -name -> glob_search
// ---------------------------------------------------------------------------

/// find with -name flag is redirected to glob_search.
#[test]
fn f3_find_with_name_flag_redirects_to_glob_search() {
    let d = classify_redirect("find . -name '*.rs'");
    assert!(
        d.is_some(),
        "classify_redirect must intercept 'find . -name'; a None is fail-open"
    );
    assert_eq!(
        d.unwrap().target_tool,
        "glob_search",
        "find with -name must redirect to glob_search"
    );
}

// ---------------------------------------------------------------------------
// (F4) Fixup arm — trailing "| head" stripped, Err returned (no spawn)
// ---------------------------------------------------------------------------

/// apply_fixup("cat src/main.rs | head 20") returns Some with command stripped.
///
/// No-spawn structural proof: apply_fixup returns Some, causing run_bash to
/// return Err(structured_error("bash_fixedup", ...)). execute_bash unreachable.
///
/// NOTE: the fixup table only strips `| head N` where N is a non-negative integer
/// or absent. `| head -20` (with a `-` flag) is NOT stripped (the `-20` does not
/// parse as u64). The contract uses `| head 20` (positive integer, no dash).
#[test]
fn f4_fixup_strips_trailing_head_pipe() {
    let result = apply_fixup("cat src/main.rs | head 20");
    assert!(
        result.is_some(),
        "apply_fixup must strip trailing '| head 20' (positive integer); a None is fail-open. \
         NOTE: '| head -N' (negative flag) is intentionally NOT stripped — the fixup only \
         removes non-negative trailing head/tail pipes."
    );
    let fixed = result.unwrap();
    assert!(
        !fixed.command.contains("| head"),
        "fixedup command must not contain '| head'; got: {}",
        fixed.command
    );
    assert!(
        !fixed.command.is_empty(),
        "fixedup command must not be empty after stripping trailing pipe"
    );
    assert_eq!(fixed.label, "trailing-head-pipe");
}

// ---------------------------------------------------------------------------
// (F5) Allowed-prefix arm — "cargo build" passes through (Allowed)
// ---------------------------------------------------------------------------

/// cargo build is a clean allowed-prefix command and must return Allowed.
///
/// Structural proof: Allowed causes run_bash to SKIP the redirect+fixup layers
/// and fall through to safety_filter and execute_bash (interception does NOT fire).
#[test]
fn f5_cargo_build_passes_through_as_allowed() {
    let result = check_allowed_prefix("cargo build", ALLOWED_PREFIXES);
    assert_eq!(
        result,
        AllowedPrefixResult::Allowed,
        "cargo build must return Allowed; PrefixBlocked or NotMatched means \
         the interceptor is incorrectly blocking legitimate cargo commands"
    );
}

// ---------------------------------------------------------------------------
// (F6) PrefixBlocked arm — "cargo build; rm -rf target" is blocked
// ---------------------------------------------------------------------------

/// An allowed-prefix command containing a shell control character is PrefixBlocked.
///
/// No-spawn structural proof: PrefixBlocked causes run_bash to return
/// Err(structured_error("bash_prefix_blocked", ...)). execute_bash unreachable.
/// R1: a NotMatched or Allowed return here is fail-open.
#[test]
fn f6_cargo_with_semicolon_is_prefix_blocked() {
    let result = check_allowed_prefix("cargo build; rm -rf target", ALLOWED_PREFIXES);
    match result {
        AllowedPrefixResult::PrefixBlocked { reason } => {
            assert!(
                !reason.is_empty(),
                "PrefixBlocked reason must not be empty"
            );
        }
        other => {
            panic!(
                "cargo build; rm -rf target must be PrefixBlocked, got {other:?}; \
                 a semicolon after an allowed-prefix command must be blocked"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// (F7-F13) Ported coverage from deleted routing_rules tests (OPTION A)
//
// These 7 arms replace the deleted routing_rules.rs unit tests:
//   cat_file_is_rejected (rule 2), head_file_is_rejected (rule 2),
//   tail_file_is_rejected (rule 2), find_with_glob_is_rejected (rule 4),
//   ls_with_glob_is_rejected (rule 4), grep_is_rejected (rule 5),
//   rg_is_rejected (rule 5).
//
// I-46 OPTION A removed routing_rules rules 2/4/5 because bash_interceptor
// is now the sole interception path for those verbs. Coverage must live here.
// ---------------------------------------------------------------------------

/// (F7) cat is intercepted by bash_interceptor, not routing_rules.
/// Mirrors deleted routing_rules test `cat_file_is_rejected`.
#[test]
fn f7_cat_is_intercepted_by_bash_interceptor() {
    // bash_interceptor fires first; classify_redirect must return Some.
    assert!(
        classify_redirect("cat src/main.rs").is_some(),
        "cat src/main.rs must be intercepted (Some) — ported from deleted routing_rules::cat_file_is_rejected"
    );
}

/// (F8) head is intercepted by bash_interceptor.
/// Mirrors deleted routing_rules test `head_file_is_rejected`.
#[test]
fn f8_head_is_intercepted_by_bash_interceptor() {
    assert!(
        classify_redirect("head -n 5 src/lib.rs").is_some(),
        "head must be intercepted — ported from deleted routing_rules::head_file_is_rejected"
    );
}

/// (F9) tail is intercepted by bash_interceptor.
/// Mirrors deleted routing_rules test `tail_file_is_rejected`.
#[test]
fn f9_tail_is_intercepted_by_bash_interceptor() {
    assert!(
        classify_redirect("tail -f /var/log/app.log").is_some(),
        "tail must be intercepted — ported from deleted routing_rules::tail_file_is_rejected"
    );
}

/// (F10) find with -name is intercepted by bash_interceptor.
/// Mirrors deleted routing_rules test `find_with_glob_is_rejected`.
#[test]
fn f10_find_with_glob_is_intercepted_by_bash_interceptor() {
    assert!(
        classify_redirect("find . -name '*.rs' -type f").is_some(),
        "find with -name must be intercepted — ported from deleted routing_rules::find_with_glob_is_rejected"
    );
}

/// (F11) ls with glob pattern is intercepted by bash_interceptor.
/// Mirrors deleted routing_rules test `ls_with_glob_is_rejected`.
#[test]
fn f11_ls_with_glob_is_intercepted_by_bash_interceptor() {
    assert!(
        classify_redirect("ls src/*.rs").is_some(),
        "ls with glob must be intercepted — ported from deleted routing_rules::ls_with_glob_is_rejected"
    );
}

/// (F12) grep is intercepted by bash_interceptor.
/// Mirrors deleted routing_rules test `grep_is_rejected`.
#[test]
fn f12_grep_is_intercepted_by_bash_interceptor() {
    assert!(
        classify_redirect("grep -r 'TODO' src/").is_some(),
        "grep must be intercepted — ported from deleted routing_rules::grep_is_rejected"
    );
}

/// (F13) rg is intercepted by bash_interceptor.
/// Mirrors deleted routing_rules test `rg_is_rejected`.
#[test]
fn f13_rg_is_intercepted_by_bash_interceptor() {
    assert!(
        classify_redirect("rg 'fn run' src/").is_some(),
        "rg must be intercepted — ported from deleted routing_rules::rg_is_rejected"
    );
}

// ---------------------------------------------------------------------------
// (F14) SSB round-trip — all 3 BashRedirectedKind arms
// ---------------------------------------------------------------------------

/// write_payload + query_by_type + msgpack decode for BashRedirectedPayload.
///
/// Tests all 3 kind arms: Redirected / FixedUp / PrefixBlocked.
/// Catches the SQL CHECK / wire-tag mismatch that is invisible to cargo check.
/// R1: a missing row is FAIL (not WARN, not SKIP).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn f14_bash_redirected_payload_ssb_round_trip() {
    let dir = TempDir::new().expect("tempdir");
    let ssb = Arc::new(
        Ssb::new(ssb_config(&dir, "round-trip"), clock()).expect("ssb init"),
    );

    let payloads: Vec<SsbPayload> = vec![
        SsbPayload::BashRedirected(BashRedirectedPayload {
            kind: BashRedirectedKind::Redirected,
            command: "grep -r 'fn main' src/".to_string(),
            substitute_call: Some("grep_search".to_string()),
            fixup_applied: None,
            prefix_matched: None,
        }),
        SsbPayload::BashRedirected(BashRedirectedPayload {
            kind: BashRedirectedKind::FixedUp,
            command: "cat src/main.rs | head 20".to_string(),
            substitute_call: None,
            fixup_applied: Some("cat src/main.rs".to_string()),
            prefix_matched: None,
        }),
        SsbPayload::BashRedirected(BashRedirectedPayload {
            kind: BashRedirectedKind::PrefixBlocked,
            command: "cargo build; rm -rf target/".to_string(),
            substitute_call: None,
            fixup_applied: None,
            prefix_matched: Some("cargo".to_string()),
        }),
    ];

    // Write all 3 payloads (async write path).
    for payload in &payloads {
        ssb.write_payload("i46-contract", 0, payload.clone(), Vec::new())
            .await
            .expect("write_payload must succeed");
    }

    // Query back via the sync SsbReader trait (query_by_type is blocking).
    // SsbActionType::BashRedirected(_) returns ALL bash_redirected rows.
    let rows = ssb
        .query_by_type(SsbActionType::BashRedirected(BashRedirectedKind::Redirected))
        .expect("query_by_type must succeed");

    assert!(
        !rows.is_empty(),
        "query_by_type(BashRedirected) must return at least one row; \
         an empty result means the SSB write failed silently (fail-open)"
    );

    // Decode all rows and check kind coverage.
    let mut saw_redirected = false;
    let mut saw_fixedup = false;
    let mut saw_prefix_blocked = false;
    for row in &rows {
        let decoded: SsbPayload =
            rmp_serde::from_slice(&row.payload)
                .expect("msgpack decode must succeed for SsbPayload::BashRedirected");
        let SsbPayload::BashRedirected(p) = decoded else {
            panic!("decoded row must be SsbPayload::BashRedirected");
        };
        match p.kind {
            BashRedirectedKind::Redirected => saw_redirected = true,
            BashRedirectedKind::FixedUp => saw_fixedup = true,
            BashRedirectedKind::PrefixBlocked => saw_prefix_blocked = true,
        }
    }

    assert!(
        saw_redirected,
        "at least one BashRedirectedKind::Redirected row must land (R1)"
    );
    assert!(
        saw_fixedup,
        "at least one BashRedirectedKind::FixedUp row must land (R1)"
    );
    assert!(
        saw_prefix_blocked,
        "at least one BashRedirectedKind::PrefixBlocked row must land (R1)"
    );
}

// ---------------------------------------------------------------------------
// (F15) Schema-lock floor
// ---------------------------------------------------------------------------

/// SCHEMA_VERSION_PUBLIC >= 244 (I-46 floor).
#[test]
fn f15_schema_version_floor_is_244() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= 244,
        "SCHEMA_VERSION_PUBLIC must be >= 244 (I-46 BashRedirected floor); \
         got {SCHEMA_VERSION_PUBLIC} — a value below 244 means the I-46 SSB bump was regressed"
    );
}

/// action_type_lock.rs must declare TOTAL_VARIANTS: usize >= 553 (I-46 +1 TV floor).
/// Uses parse-then-floor pattern (A117) to avoid stale-anchor cascade on future bumps.
#[test]
fn f15_total_variants_is_553() {
    // Floor check: TOTAL_VARIANTS >= 553 (I-46 floor). Tier-B Move 2 derived
    // TOTAL_VARIANTS and relocated ALL_VARIANTS (now SSB_ACTION_TYPES in
    // ssb.rs), so read the runtime registry length instead of parsing
    // action_type_lock.rs.
    const I46_TV_FLOOR: usize = 553;
    let tv = elai_orchestrator::ssb::SSB_ACTION_TYPES.len();
    assert!(
        tv >= I46_TV_FLOOR,
        "TOTAL_VARIANTS must be >= {I46_TV_FLOOR} (I-46 +1 TV floor); got {tv}"
    );
}

/// "bash_redirected" round-trips through ssb_action_type_from_str_public / as_sql_str.
///
/// Catches a mismatch between the wire tag in as_sql_str() and the SQL CHECK
/// constraint (the CHECK is built from the from_str canonical list; if the
/// tags diverge the INSERT silently fails at runtime).
#[test]
fn f15_bash_redirected_wire_tag_round_trips() {
    let parsed = ssb_action_type_from_str_public("bash_redirected")
        .expect("bash_redirected must parse via ssb_action_type_from_str_public");
    let back = parsed.as_sql_str();
    assert_eq!(
        back, "bash_redirected",
        "bash_redirected must round-trip as_sql_str -> from_str -> as_sql_str; got {back}"
    );
}
