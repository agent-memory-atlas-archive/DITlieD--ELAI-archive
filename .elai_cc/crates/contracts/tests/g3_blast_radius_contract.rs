//! G-3 -- deterministic runtime blast-radius gate contract tests.
//!
//! Two deterministic, fail-closed gates emitting a `blast_radius_blocked` SSB
//! row carrying the FULL pre- AND post-expansion command:
//!   (a) FS post-EXPANSION write/delete gate -- `rm -rf $X` where $X expands to
//!       ~/ OR a symlink/.. escaping the worktree -> resolved path OUTSIDE root
//!       -> BLOCKED before execution; the SSB row carries BOTH the original
//!       argv AND the canonicalized path.
//!   (b) FP GUARD: a write INSIDE the worktree -> ALLOWED, no block.
//!   (c) egress: a host with a NUL byte / control char -> BLOCKED before connect
//!       (and BEFORE resolution); carries the rejected host.
//!   (d) FP GUARD: an allowed (pinned) host -> resolved+pinned, ALLOWED.
//!
//! The verdict functions are PURE (no LLM classifier, no substring heuristic --
//! an LLM permission classifier collapses to ~81% FNR under adversarial
//! scope-escalation, arXiv:2604.04978). These cases assert the cross-platform
//! Tier-1 fail-closed gate directly (the W-5 deterministic in-process fixture);
//! the live `elai run` driving a real rm-rf/egress fixture is Tier-2, tracked
//! separately. The SSB-row-lands-from-a-real-production-path assertion is the
//! G-1 sibling pattern (translate_tool_request); here we assert the Pattern-2
//! round-trip + SQL CHECK bijection for both Kind arms (the off-agent artifact
//! shape), and the pure verdict's fail-closed property.
//!
//! No #[ignore] / stubs / placeholder assertions. No #[traced_test] (we assert
//! on verdicts + SSB round-trip, not on logs).
//!
//! Grounding: workflow/reasoning/G-3-decisions.md (DD1-DD6); rm-rf #10077 /
//! #12637 (shell-expansion-after-validation); SOCKS5 null-byte egress bypass.

use std::net::{IpAddr, Ipv4Addr};

use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, BlastRadiusBlockedPayload, BlastRadiusKind, Ssb, SsbActionType,
    SsbConfig, SsbPayload, SCHEMA_VERSION_PUBLIC,
};
use elai_orchestrator::SsbReader;
use elai_safety::blast_radius::{
    check_egress_host, check_fs_target, expand_fs_token, BlastRadiusVerdict, ExpandedToken,
};
use elai_core::clock::OsClock;
use std::sync::Arc;
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// (a) FS escape: ~ / .. resolving OUTSIDE the worktree root -> BLOCK FsEscape,
//     carrying BOTH the original argv AND the canonicalized path.
// ---------------------------------------------------------------------------

#[test]
fn fs_escape_tilde_outside_root_is_blocked_with_pre_and_post() {
    // Worktree root is a real temp dir (so canonicalize succeeds); the target
    // expands via ~ to a HOME outside that root.
    let root = TempDir::new().expect("tempdir");
    let home = TempDir::new().expect("home tempdir");
    let token = "~/important";
    let expanded = match expand_fs_token(token, Some(home.path())) {
        ExpandedToken::Path(p) => p,
        other => panic!("~ must expand to a concrete path, got {other:?}"),
    };
    assert!(
        expanded.starts_with(home.path()),
        "~ must expand to HOME, not the literal tilde dir"
    );

    let pre = "rm -rf ~/important";
    let verdict = check_fs_target(&expanded, root.path(), pre);
    match verdict {
        BlastRadiusVerdict::Block {
            kind,
            pre_expansion,
            post_expansion,
        } => {
            // safety-crate Kind; the wire-enum bijection is asserted in the
            // round-trip test. Here assert the forensics fields carry BOTH the
            // original argv AND the resolved path.
            let _ = kind;
            assert_eq!(pre_expansion, pre, "pre-expansion = original argv (#10077)");
            assert!(
                !post_expansion.is_empty() && post_expansion != pre,
                "post-expansion = canonicalized target, distinct from argv: {post_expansion}"
            );
            assert!(
                post_expansion.contains("important"),
                "post-expansion must name the resolved ~/ target: {post_expansion}"
            );
        }
        BlastRadiusVerdict::Allow => panic!("~/ escape MUST be blocked (fail-closed)"),
    }
}

#[test]
fn fs_escape_parent_dir_outside_root_is_blocked() {
    let root = TempDir::new().expect("tempdir");
    let subroot = root.path().join("sub");
    std::fs::create_dir_all(&subroot).expect("mkdir sub");
    // ../escape leaves `sub` back into root (outside the subroot containment).
    let target = subroot.join("..").join("escape.txt");
    let verdict = check_fs_target(&target, &subroot, "rm -rf ../escape.txt");
    assert!(
        !verdict.is_allow(),
        ".. escaping the worktree root MUST be blocked (fail-closed): {verdict:?}"
    );
}

// ---------------------------------------------------------------------------
// (b) FP GUARD: a write INSIDE the worktree -> ALLOWED, no block.
// ---------------------------------------------------------------------------

#[test]
fn in_worktree_write_is_allowed_no_block() {
    let root = TempDir::new().expect("tempdir");
    let target = root.path().join("src").join("new_file.rs");
    let verdict = check_fs_target(&target, root.path(), "edit src/new_file.rs");
    assert!(
        verdict.is_allow(),
        "an in-worktree write must be ALLOWED (FP guard): {verdict:?}"
    );
}

// ---------------------------------------------------------------------------
// (c) egress: a host with a NUL byte / control char -> BLOCKED BEFORE connect
//     AND BEFORE resolution; carries the rejected host.
// ---------------------------------------------------------------------------

#[test]
fn egress_null_byte_host_is_blocked_before_resolve() {
    use std::sync::atomic::{AtomicBool, Ordering};
    let pinned = vec![IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4))];
    let resolver_called = AtomicBool::new(false);
    let host = "evil\u{0}.example";
    let verdict = check_egress_host(host, &pinned, |_| {
        resolver_called.store(true, Ordering::SeqCst);
        vec![IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4))]
    });
    match verdict {
        BlastRadiusVerdict::Block { pre_expansion, .. } => {
            assert_eq!(pre_expansion, host, "row carries the rejected host verbatim");
        }
        BlastRadiusVerdict::Allow => panic!("NUL-byte host MUST be blocked (fail-closed)"),
    }
    assert!(
        !resolver_called.load(Ordering::SeqCst),
        "the gate MUST reject BEFORE resolution (fail-closed before connect)"
    );
}

#[test]
fn egress_control_char_host_is_blocked() {
    let pinned = vec![IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4))];
    let verdict = check_egress_host("ho\u{1b}st.example", &pinned, |_| {
        vec![IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4))]
    });
    assert!(!verdict.is_allow(), "control-char host MUST be blocked");
}

// ---------------------------------------------------------------------------
// (d) FP GUARD: an allowed (pinned) host -> resolved+pinned, ALLOWED.
// ---------------------------------------------------------------------------

#[test]
fn egress_pinned_host_is_allowed_no_block() {
    let pinned = vec![IpAddr::V4(Ipv4Addr::new(140, 82, 121, 4))];
    let verdict = check_egress_host("github.example", &pinned, |_| {
        vec![IpAddr::V4(Ipv4Addr::new(140, 82, 121, 4))]
    });
    assert!(
        verdict.is_allow(),
        "a host resolving entirely into the pinned allow-set must be ALLOWED (FP guard): {verdict:?}"
    );
}

#[test]
fn egress_unpinned_resolved_ip_is_blocked() {
    let pinned = vec![IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4))];
    let verdict = check_egress_host("attacker.example", &pinned, |_| {
        vec![IpAddr::V4(Ipv4Addr::new(9, 9, 9, 9))]
    });
    assert!(
        !verdict.is_allow(),
        "a host resolving OUTSIDE the allow-set MUST be blocked (fail-closed)"
    );
}

#[test]
fn egress_unresolvable_host_is_blocked_fail_closed() {
    let pinned = vec![IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4))];
    let verdict = check_egress_host("nowhere.invalid", &pinned, |_| Vec::new());
    assert!(
        !verdict.is_allow(),
        "an unresolvable host MUST be blocked (fail-closed -- 'did not resolve' != 'allowed')"
    );
}

// ---------------------------------------------------------------------------
// Pattern-2 SSB round-trip: BlastRadiusBlocked carries both Kind arms through
// msgpack + the SQL CHECK value list + the action_type bijection. This is the
// off-agent artifact shape (the row a real production path writes through the
// live Ssb -- driven end-to-end in the G-1 sibling pattern).
// ---------------------------------------------------------------------------

fn ssb_config(dir: &TempDir) -> SsbConfig {
    SsbConfig {
        db_path: dir.path().join("g3_contract_ssb.db"),
        pool_size: Some(2),
        project_id: "g3-contract".to_string(),
        session_id: "g3-session".to_string(),
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn blast_radius_blocked_roundtrips_both_kind_arms() {
    let dir = TempDir::new().expect("tempdir");
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    let ssb = Ssb::new(ssb_config(&dir), clock).expect("ssb init");

    let payloads = vec![
        SsbPayload::BlastRadiusBlocked(BlastRadiusBlockedPayload {
            kind: BlastRadiusKind::FsEscape,
            pre_expansion_command: "rm -rf ~/important".into(),
            post_expansion_command: "/home/user/important".into(),
            target_or_host: "/home/user/important".into(),
            worktree_root_blake3: "abc1234567890def".into(),
            trust_label: String::new(),
            ts_ms: 1_700_000_000_000,
        }),
        SsbPayload::BlastRadiusBlocked(BlastRadiusBlockedPayload {
            kind: BlastRadiusKind::EgressDenied,
            pre_expansion_command: "fetch https://attacker.example".into(),
            post_expansion_command: "host=attacker.example resolved [\"9.9.9.9\"] not all in allow-set".into(),
            target_or_host: "attacker.example".into(),
            worktree_root_blake3: String::new(),
            trust_label: String::new(),
            ts_ms: 1_700_000_000_001,
        }),
    ];

    for (i, payload) in payloads.iter().enumerate() {
        let seq = ssb
            .write_payload(&format!("agent-{i}"), i as i64, payload.clone(), Vec::new())
            .await
            .unwrap_or_else(|e| panic!("write_payload variant {i} failed (SQL CHECK?): {e}"));
        assert!(seq > 0, "seq must be positive for variant {i}");
    }

    // Both arms map to the single action_type, and the bijection holds.
    assert_eq!(
        SsbActionType::BlastRadiusBlocked.as_sql_str(),
        "blast_radius_blocked"
    );
    assert_eq!(
        ssb_action_type_from_str_public("blast_radius_blocked")
            .expect("from_str must accept the wire tag"),
        SsbActionType::BlastRadiusBlocked
    );

    // Read back: both rows are present under the single action type.
    let rows = ssb
        .query_by_type(SsbActionType::BlastRadiusBlocked)
        .expect("query_by_type");
    assert_eq!(rows.len(), 2, "both Kind arms must round-trip under one type");
}

// ---------------------------------------------------------------------------
// SCHEMA floor pin (>=, never ==, per the TRANCHE_D gotcha -- exact pins break
// on every SSB addition).
// ---------------------------------------------------------------------------

#[test]
fn schema_version_floor_includes_blast_radius() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= 211,
        "G-3 lands at SCHEMA_VERSION 211; floor pin (>=) must hold: got {SCHEMA_VERSION_PUBLIC}"
    );
}

// ---------------------------------------------------------------------------
// Glob fail-closed-on-ambiguity (DD1): a glob whose literal base is outside the
// root is refused; a relative bare-glob (base inside the root) is allowed.
// ---------------------------------------------------------------------------

#[test]
fn glob_with_base_outside_root_is_blocked() {
    let root = TempDir::new().expect("tempdir");
    // Absolute glob base OUTSIDE the root.
    let outside = TempDir::new().expect("outside tempdir");
    let token = format!("{}/*.key", outside.path().display());
    let base = match expand_fs_token(&token, None) {
        ExpandedToken::GlobBase(b) => b,
        other => panic!("token with * must yield GlobBase, got {other:?}"),
    };
    assert!(!base.as_os_str().is_empty(), "glob base must be the literal dir");
    let verdict = check_fs_target(&base, root.path(), &token);
    assert!(
        !verdict.is_allow(),
        "a glob whose literal base is OUTSIDE the root MUST be blocked (fail-closed on ambiguity)"
    );
}

#[test]
fn glob_base_inside_root_is_allowed() {
    let root = TempDir::new().expect("tempdir");
    let token = format!("{}/*.rs", root.path().display());
    let base = match expand_fs_token(&token, None) {
        ExpandedToken::GlobBase(b) => b,
        other => panic!("token with * must yield GlobBase, got {other:?}"),
    };
    let verdict = check_fs_target(&base, root.path(), &token);
    assert!(
        verdict.is_allow(),
        "an in-root glob base must be ALLOWED (FP guard): {verdict:?}"
    );
}
