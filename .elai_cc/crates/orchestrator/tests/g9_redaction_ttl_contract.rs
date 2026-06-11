//! G-9 — write-time secret+PII redaction gate + file-granular retention TTL.
//! THE deterministic Tier-1 fail-closed contract for the write-path redactor.
//!
//! These tests drive the REAL `Ssb::write_payload -> redact_payload ->
//! to_vec_named -> execute_write` path over a LIVE `Ssb` (the same writer +
//! reader production `elai run` uses) and the REAL `build_rra` export path, NOT
//! a hand-mocked redactor. The off-agent artifacts asserted are: the committed
//! row bytes (raw secret ABSENT, token PRESENT), the `secret_redacted` advisory
//! row, and the G-2 RunArtifact bundle bytes (DD8 — the load-bearing R4 proof
//! that the RRA can no longer export a raw secret).
//!
//! Coverage (mirrors G-9-decisions.md EXPECTED TEST OUTCOME):
//!   (1) write_payload of an IdeaFramingStarted payload carrying an AWS key +
//!       an email -> the committed row bytes contain [REDACTED:...] tokens, NOT
//!       the raw secret; a secret_redacted advisory row lands (token + field +
//!       count, NO raw secret).
//!   (2) build_rra over that session -> the raw secret bytes are ABSENT from the
//!       bundle, the token is PRESENT (DD8, the off-agent proof).
//!   (3) the GD-4 benign corpus (version / UUID / hash / path) -> ZERO
//!       redactions (no over-redaction); the secrets corpus -> full recall.
//!   (4) the TTL purge over a TEMP sessions dir -> expired transcript / archive
//!       files deleted, the ACTIVE session's files KEPT, grace-window files
//!       KEPT, NO real ssb.db deletion.
//!   (5) same secret -> same token across rows (stability).
//!
//! No #[ignore] / stubs / placeholder assertions. No #[traced_test] (we assert
//! on the row bytes + the advisory row + the RRA bundle + the purge report, not
//! on logs).

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use elai_core::clock::{ClockProvider, OsClock};
use elai_orchestrator::determinism::{DeterminismConfig, GpuSku, HardwareClass};
use elai_orchestrator::rra::build_rra;
use elai_orchestrator::redaction::{
    purge_expired, redact, redact_payload, RetentionPolicy, RETENTION_PURGE_ENV,
};
use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, ArtifactSearchedPayload, ChatResponsePayload,
    GitCommitPayload, IdeaFramingStartedPayload, SsbActionType, SsbPayload, Ssb, SsbConfig,
    SecretRedactedPayload,
};
use tempfile::TempDir;

const FIXTURE_SESSION: &str = "g9-redaction-session";
const AWS_KEY: &str = "AKIAIOSFODNN7EXAMPLE";
const EMAIL: &str = "alice@example.com";

fn make_ssb(tmp: &TempDir) -> Arc<Ssb> {
    let cfg = SsbConfig {
        db_path: tmp.path().join("ssb_g9.db"),
        pool_size: Some(2),
        project_id: "default".to_string(),
        session_id: FIXTURE_SESSION.to_string(),
    };
    let clock: Arc<dyn ClockProvider> = Arc::new(OsClock);
    Arc::new(Ssb::new(cfg, clock).expect("ssb init"))
}

fn local_pins(seed: u64) -> DeterminismConfig {
    DeterminismConfig {
        hardware_class: HardwareClass::Gpu(GpuSku::new("rtx3090")),
        seed,
        blake3_cache_prefix: [0u8; 8],
        system_fingerprint_pin: Some("local-fp-pin".to_string()),
    }
}

// (1) write_payload redacts the typed payload before serialization; the
//     committed row bytes carry the token, NOT the raw secret.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn write_payload_redacts_committed_bytes_and_emits_advisory() {
    let tmp = TempDir::new().expect("tempdir");
    let ssb = make_ssb(&tmp);

    let prompt = format!("deploy with {AWS_KEY} and notify {EMAIL} now");
    ssb.write_payload(
        "framing",
        0,
        SsbPayload::IdeaFramingStarted(IdeaFramingStartedPayload {
            session_id: FIXTURE_SESSION.into(),
            prompt,
            started_at_ms: 1_700_000_000_000,
        }),
        Vec::new(),
    )
    .await
    .expect("write idea-framing row");

    let stream = ssb
        .query_session_stream_asc(FIXTURE_SESSION)
        .expect("read stream");

    // The committed payload bytes must NOT contain the raw secret, and MUST
    // contain the stable redaction token.
    let framing = stream
        .iter()
        .find(|e| e.action_type == SsbActionType::IdeaFramingStarted)
        .expect("framing row present");
    let bytes = framing.payload.as_ref();
    assert!(
        !contains_subslice(bytes, AWS_KEY.as_bytes()),
        "raw AWS key must be ABSENT from committed bytes"
    );
    assert!(
        !contains_subslice(bytes, EMAIL.as_bytes()),
        "raw email must be ABSENT from committed bytes"
    );
    assert!(
        contains_subslice(bytes, b"[REDACTED:aws_access_key_id:"),
        "AWS-key redaction token must be PRESENT in committed bytes"
    );
    assert!(
        contains_subslice(bytes, b"[REDACTED:email:"),
        "email redaction token must be PRESENT in committed bytes"
    );

    // A secret_redacted advisory row must land carrying the token + field +
    // count, and NO raw secret.
    let advisory = stream
        .iter()
        .find(|e| e.action_type == SsbActionType::SecretRedacted)
        .expect("secret_redacted advisory row present");
    let decoded: SsbPayload =
        rmp_serde::from_slice(advisory.payload.as_ref()).expect("decode SsbPayload");
    let SsbPayload::SecretRedacted(adv) = decoded else {
        panic!("expected SecretRedacted variant");
    };
    assert_eq!(adv.field, "idea_framing_started.prompt");
    assert_eq!(adv.match_count, 2, "AWS key + email = 2 redactions");
    assert!(adv.tokens.iter().any(|t| t.starts_with("[REDACTED:aws_access_key_id:")));
    assert!(adv.tokens.iter().any(|t| t.starts_with("[REDACTED:email:")));
    let adv_bytes = advisory.payload.as_ref();
    assert!(
        !contains_subslice(adv_bytes, AWS_KEY.as_bytes()),
        "advisory row must NOT carry the raw secret"
    );
    assert!(
        !contains_subslice(adv_bytes, EMAIL.as_bytes()),
        "advisory row must NOT carry the raw email"
    );

    // Round-trips through the action_type bijection.
    assert_eq!(
        ssb_action_type_from_str_public("secret_redacted").expect("known type"),
        SsbActionType::SecretRedacted
    );
}

// (1b) G-9 LEAK FIX — the three HIGH-RISK free-text variants that previously
//      bypassed redaction via the `_ => no-op` arm (ChatResponse.body,
//      ArtifactSearched.query, GitCommit.message) now redact. Drives the REAL
//      write_payload path and asserts the raw secret is ABSENT from the
//      committed row bytes for EACH, the token is PRESENT, and the advisory
//      lands with the correct field identity.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn write_payload_redacts_free_text_leak_variants() {
    let tmp = TempDir::new().expect("tempdir");
    let ssb = make_ssb(&tmp);

    let payloads = [
        SsbPayload::ChatResponse(ChatResponsePayload {
            session_id: FIXTURE_SESSION.into(),
            card_kind: "error".into(),
            body: format!("rendered card leaked {AWS_KEY} to the user"),
            project_id: "default".into(),
            emitted_at_ms: 1_700_000_000_000,
        }),
        SsbPayload::ArtifactSearched(ArtifactSearchedPayload {
            project_id: "default".into(),
            artifact_name: None,
            query: format!("find docs mentioning {AWS_KEY} please"),
            result_count: 0,
            latency_ms: 1,
        }),
        SsbPayload::GitCommit(GitCommitPayload {
            commit_sha: "deadbeef".into(),
            branch: "main".into(),
            files_count: 1,
            step_id: "S1".into(),
            message: format!("wip: pasted {AWS_KEY} into the commit body by mistake"),
        }),
    ];

    for (i, payload) in payloads.into_iter().enumerate() {
        ssb.write_payload(&format!("leak-{i}"), i as i64, payload, Vec::new())
            .await
            .expect("write leak-variant row");
    }

    let stream = ssb
        .query_session_stream_asc(FIXTURE_SESSION)
        .expect("read stream");

    let expectations = [
        (SsbActionType::ChatResponse, "chat_response.body"),
        (SsbActionType::ArtifactSearched, "artifact_searched.query"),
        (SsbActionType::GitCommit, "git_commit.message"),
    ];

    for (action, _advisory_field) in expectations {
        let row = stream
            .iter()
            .find(|e| e.action_type == action)
            .unwrap_or_else(|| panic!("{action:?} row present"));
        let bytes = row.payload.as_ref();
        assert!(
            !contains_subslice(bytes, AWS_KEY.as_bytes()),
            "raw AWS key must be ABSENT from committed {action:?} bytes (leak fix)"
        );
        assert!(
            contains_subslice(bytes, b"[REDACTED:aws_access_key_id:"),
            "redaction token must be PRESENT in committed {action:?} bytes"
        );
    }

    // One secret_redacted advisory per leak variant, each carrying the correct
    // field identity and NO raw secret.
    let advisories: Vec<&_> = stream
        .iter()
        .filter(|e| e.action_type == SsbActionType::SecretRedacted)
        .collect();
    assert_eq!(
        advisories.len(),
        3,
        "exactly three secret_redacted advisories (one per leak variant)"
    );
    let mut seen_fields: Vec<String> = Vec::new();
    for adv_row in &advisories {
        let decoded: SsbPayload =
            rmp_serde::from_slice(adv_row.payload.as_ref()).expect("decode SsbPayload");
        let SsbPayload::SecretRedacted(adv) = decoded else {
            panic!("expected SecretRedacted variant");
        };
        assert!(
            !contains_subslice(adv_row.payload.as_ref(), AWS_KEY.as_bytes()),
            "advisory row must NOT carry the raw secret"
        );
        assert!(
            adv.tokens
                .iter()
                .any(|t| t.starts_with("[REDACTED:aws_access_key_id:")),
            "advisory carries the redaction token"
        );
        seen_fields.push(adv.field);
    }
    for (_, advisory_field) in expectations {
        assert!(
            seen_fields.iter().any(|f| f == advisory_field),
            "advisory for field {advisory_field} must be present"
        );
    }
}

// (2) DD8 — build_rra over the session: the raw secret is ABSENT from the RRA
//     bundle (the load-bearing off-agent R4 proof that G-2 can't export it).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn build_rra_over_redacted_session_has_no_raw_secret() {
    let tmp = TempDir::new().expect("tempdir");
    let ssb = make_ssb(&tmp);

    ssb.write_payload(
        "framing",
        0,
        SsbPayload::IdeaFramingStarted(IdeaFramingStartedPayload {
            session_id: FIXTURE_SESSION.into(),
            prompt: format!("plant {AWS_KEY} into the artifact"),
            started_at_ms: 1_700_000_000_000,
        }),
        Vec::new(),
    )
    .await
    .expect("write planted-secret row");

    let stream = ssb
        .query_session_stream_asc(FIXTURE_SESSION)
        .expect("read stream");
    let rra = build_rra(FIXTURE_SESSION, &stream, &local_pins(42));

    // The RRA re-bundles each row's raw msgpack payload bytes verbatim. After
    // write-time redaction, NONE of those bytes can be the raw secret.
    let mut token_seen = false;
    for event in &rra.events {
        assert!(
            !contains_subslice(&event.payload, AWS_KEY.as_bytes()),
            "RRA event payload must NOT contain the raw AWS key (DD8)"
        );
        if contains_subslice(&event.payload, b"[REDACTED:aws_access_key_id:") {
            token_seen = true;
        }
    }
    assert!(token_seen, "the redaction token must be PRESENT in the RRA bundle");
}

// (3) GD-4 — the benign corpus produces ZERO redactions; the secrets corpus is
//     fully recalled (every line redacted at least once).
#[test]
fn benign_corpus_zero_over_redaction_and_secrets_full_recall() {
    let benign = std::fs::read_to_string(fixture("benign.txt")).expect("benign corpus");
    let mut over_redactions = 0usize;
    for line in benign.lines().filter(|l| !l.trim().is_empty()) {
        let (out, hits) = redact(line);
        if !hits.is_empty() {
            over_redactions += hits.len();
            eprintln!("OVER-REDACTION on benign line: {line:?} -> {out:?}");
        }
    }
    assert_eq!(
        over_redactions, 0,
        "GD-4 bound: benign corpus must have ZERO over-redactions"
    );

    let secrets = std::fs::read_to_string(fixture("secrets.txt")).expect("secrets corpus");
    for line in secrets.lines().filter(|l| !l.trim().is_empty()) {
        let (_out, hits) = redact(line);
        assert!(
            !hits.is_empty(),
            "secrets corpus full-recall: line must be redacted: {line:?}"
        );
    }
}

// (5) same secret -> same token (stability / correlatability).
#[test]
fn same_secret_yields_same_token() {
    let (a, hits_a) = redact(&format!("row1 has {AWS_KEY}"));
    let (b, hits_b) = redact(&format!("row2 also has {AWS_KEY} here"));
    assert_eq!(hits_a.len(), 1);
    assert_eq!(hits_b.len(), 1);
    assert_eq!(
        hits_a[0].token, hits_b[0].token,
        "identical secret must map to identical token"
    );
    assert!(a.contains(&hits_a[0].token));
    assert!(b.contains(&hits_b[0].token));
}

// redact_payload short-circuits on the SecretRedacted variant (anti-recursion).
#[test]
fn redact_payload_short_circuits_advisory_variant() {
    let mut adv = SsbPayload::SecretRedacted(SecretRedactedPayload {
        session_id: "s".into(),
        // A token-shaped string that itself contains an AWS-key-shaped value:
        // proves the short-circuit (would otherwise re-redact).
        field: "idea_framing_started.prompt".into(),
        tokens: vec!["[REDACTED:aws_access_key_id:0123456789abcdef]".into()],
        match_count: 1,
        kind_counts: vec![("aws_access_key_id".into(), 1)],
    });
    let outcome = redact_payload(&mut adv);
    assert!(!outcome.fired(), "advisory variant must be a no-op (anti-recursion)");
}

// (4) TTL purge over a TEMP sessions dir: expired files deleted, active kept,
//     grace-window kept, ssb.db never touched.
#[test]
fn purge_deletes_expired_keeps_active_and_grace() {
    let tmp = TempDir::new().expect("tempdir");
    let sessions = tmp.path().join("sessions");
    std::fs::create_dir_all(&sessions).expect("mk sessions");

    // A real ssb.db that must NEVER be deleted.
    let ssb_db = sessions.join("ssb.db");
    std::fs::write(&ssb_db, b"sqlite-bytes").expect("write ssb.db");

    // An EXPIRED transcript, an ACTIVE-session transcript, a FRESH (grace)
    // transcript.
    let expired = sessions.join("old-session.jsonl");
    let active = sessions.join("live-session.jsonl");
    let fresh = sessions.join("fresh-session.jsonl");
    for p in [&expired, &active, &fresh] {
        std::fs::write(p, b"{}").expect("write transcript");
    }

    // Backdate the expired file's mtime well past the TTL.
    let now = std::time::SystemTime::now();
    let old = now - Duration::from_secs(40 * 86_400); // 40 days old
    filetime_set(&expired, old);
    // active + fresh keep their just-now mtime (inside grace window).

    let now_ms = now
        .duration_since(std::time::UNIX_EPOCH)
        .expect("now")
        .as_millis();

    // Enable the destructive purge for THIS test only.
    std::env::set_var(RETENTION_PURGE_ENV, "1");
    let report = purge_expired(
        &sessions,
        &tmp.path().join("archive-root"), // empty/absent archive root is fine
        RetentionPolicy::default(),
        now_ms,
        "live-session",        // active session id (self-exclude)
        tmp.path(),            // project root (no index lock held)
    );
    std::env::remove_var(RETENTION_PURGE_ENV);

    assert!(report.enabled, "purge must be enabled when the env flag is set");
    assert!(
        report.transcripts_deleted.iter().any(|p| p == &expired),
        "expired transcript must be deleted"
    );
    assert!(!expired.exists(), "expired file gone from disk");
    assert!(active.exists(), "ACTIVE session transcript must be KEPT");
    assert!(fresh.exists(), "grace-window transcript must be KEPT");
    assert!(ssb_db.exists(), "shared ssb.db must NEVER be deleted");
    // ssb.db is not a managed-session file, so it must not even be considered.
    assert!(
        !report.transcripts_deleted.iter().any(|p| p == &ssb_db),
        "ssb.db must never appear in the delete set"
    );
}

#[test]
fn purge_disabled_by_default_is_no_op() {
    let tmp = TempDir::new().expect("tempdir");
    let sessions = tmp.path().join("sessions");
    std::fs::create_dir_all(&sessions).expect("mk sessions");
    let old = sessions.join("ancient.jsonl");
    std::fs::write(&old, b"{}").expect("write");
    let backdate = std::time::SystemTime::now() - Duration::from_secs(99 * 86_400);
    filetime_set(&old, backdate);

    // Env NOT set -> disabled -> zero mutation.
    std::env::remove_var(RETENTION_PURGE_ENV);
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("now")
        .as_millis();
    let report = purge_expired(
        &sessions,
        &tmp.path().join("archive-root"),
        RetentionPolicy::default(),
        now_ms,
        "any",
        tmp.path(),
    );
    assert!(!report.enabled, "default-off: purge must be disabled");
    assert!(report.transcripts_deleted.is_empty(), "no deletions when disabled");
    assert!(old.exists(), "expired file untouched when purge disabled");
}

// --------------------------------------------------------------------------
// helpers
// --------------------------------------------------------------------------

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("redaction")
        .join(name)
}

fn contains_subslice(haystack: &[u8], needle: &[u8]) -> bool {
    if needle.is_empty() || needle.len() > haystack.len() {
        return false;
    }
    haystack
        .windows(needle.len())
        .any(|window| window == needle)
}

/// Set a file's mtime via `std::fs::File::set_modified` (stable since Rust
/// 1.75) — NO new dependency (the `filetime` crate is not needed).
fn filetime_set(path: &std::path::Path, when: std::time::SystemTime) {
    let file = std::fs::OpenOptions::new()
        .write(true)
        .open(path)
        .expect("open for mtime set");
    file.set_modified(when).expect("set mtime");
}
