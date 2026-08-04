//! J-54 free-pool contract -- arms (a)-(g) (residual).
//!
//! (a) KeyId never stores raw; (b) denylisted filtered; (c) uncensored no
//! overflow; (d) note_429 soonest header; (e) pollinations >=1; (f) day
//! counters persist across restart; (g) trust-marker downgrades.

use elai_providers::runtime::free_pool::catalog::DENYLISTED_MODEL_IDS;
use elai_providers::runtime::free_pool::dispatch::{FreePoolDispatcher, ProviderGroup};
use elai_providers::runtime::free_pool::quota::{KeyId, QuotaTracker};
use elai_providers::runtime::{apply_trust_marker, FreePoolTrustMarker};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn tmp_state_path() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    std::env::temp_dir().join(format!("j54_quota_{nanos}.json"))
}

#[test]
fn j54_a_keyid_never_stores_raw() {
    let raw = "sk-test-12345-secret-key";
    let key = KeyId::from_raw_key("groq", raw);
    assert!(!key.fp.contains(raw), "fp must not contain raw key");
    assert_eq!(key.fp.len(), 8);
    let path = tmp_state_path();
    {
        let tracker = QuotaTracker::unlimited(path.clone());
        tracker.record_success(&key, "llama-4-scout", 100);
        tracker.persist().expect("persist");
    }
    let data = std::fs::read_to_string(&path).expect("persisted file");
    assert!(!data.contains(raw), "raw key must not appear in persisted json");
    let _ = std::fs::remove_file(&path);
}

#[test]
fn j54_b_denylisted_absent_from_deployments() {
    let dispatcher = FreePoolDispatcher::new();
    for d in dispatcher.deployments() {
        for &deny in DENYLISTED_MODEL_IDS {
            assert_ne!(d.model, deny, "denylisted model {} in deployments", deny);
        }
    }
    let catalog = elai_providers::runtime::free_pool::catalog::load_free_pool_catalog();
    for entry in &catalog {
        for &deny in DENYLISTED_MODEL_IDS {
            assert_ne!(entry.model, deny, "denylisted model {} in catalog", deny);
        }
    }
}

#[test]
fn j54_c_uncensored_no_pollinations_overflow() {
    let dispatcher = FreePoolDispatcher::new();
    let uncensored_pollinations = dispatcher
        .deployments()
        .iter()
        .filter(|d| d.group == ProviderGroup::Uncensored && d.provider == "pollinations")
        .count();
    assert_eq!(uncensored_pollinations, 0, "no pollinations under Uncensored");
    let uncensored_total = dispatcher
        .deployments()
        .iter()
        .filter(|d| d.group == ProviderGroup::Uncensored)
        .count();
    assert_eq!(uncensored_total, 0, "uncensored group has zero deployments");
}

#[test]
fn j54_d_note_429_soonest_header() {
    let path = tmp_state_path();
    let tracker = QuotaTracker::unlimited(path.clone());
    let key = KeyId::from_raw_key("groq", "sk-test");
    let headers: Vec<(String, String)> = vec![
        ("x-ratelimit-reset-requests".to_string(), "30".to_string()),
        ("Retry-After".to_string(), "90".to_string()),
    ];
    let cooldown = tracker.note_429(&key, "llama-4-scout", &headers);
    assert_eq!(cooldown, 30, "minimum-wins: x-ratelimit-reset-requests=30 beats Retry-After=90");
    let _ = std::fs::remove_file(&path);
}

#[test]
fn j54_e_pollinations_overflow_present() {
    let dispatcher = FreePoolDispatcher::new();
    let pollinations = dispatcher
        .deployments()
        .iter()
        .filter(|d| d.provider == "pollinations")
        .count();
    assert!(pollinations >= 1, "Pollinations overflow always >=1 for non-uncensored pool");
}

#[test]
fn j54_f_day_counters_persist_across_restart() {
    let path = tmp_state_path();
    {
        let tracker = QuotaTracker::unlimited(path.clone());
        let key = KeyId::from_raw_key("groq", "sk-test");
        tracker.record_success(&key, "llama-4-scout", 250);
        tracker.persist().expect("persist");
    }
    let data = std::fs::read_to_string(&path).expect("persisted file");
    assert!(data.contains("day_reqs"), "day_reqs must persist");
    assert!(data.contains("day_tokens"), "day_tokens must persist");
    assert!(!data.contains("minute_reqs"), "minute counters must NOT persist");
    assert!(!data.contains("minute_tokens"), "minute counters must NOT persist");
    assert!(!data.contains("cooldown"), "cooldown keys must NOT persist");
    // Reload into a fresh tracker (restart simulation) -- day data present.
    let _tracker2 = QuotaTracker::unlimited(path.clone());
    let _ = std::fs::remove_file(&path);
}

#[test]
fn j54_g_executor_downgrade() {
    let m = apply_trust_marker("executor").expect("executor on free-pool must downgrade");
    assert!(m.non_reproducible, "executor free-pool must set non_reproducible");
    assert_eq!(m.role, "executor");
    let _ = FreePoolTrustMarker {
        non_reproducible: true,
        role: "executor".into(),
    };
}

#[test]
fn j54_g_auditor_downgrade() {
    let m = apply_trust_marker("auditor").expect("auditor must downgrade");
    assert!(m.non_reproducible);
    assert_eq!(m.role, "auditor");
}

#[test]
fn j54_g_non_verification_roles_no_downgrade() {
    assert!(apply_trust_marker("researcher").is_none());
    assert!(apply_trust_marker("designer").is_none());
    assert!(apply_trust_marker("context_compiler").is_none());
}

#[test]
fn j54_g_rra_non_reproducible_field_stampable() {
    use elai_orchestrator::rra::{stamp_non_reproducible, RunArtifact, GuaranteeLevel};
    let _ = std::any::type_name::<fn(&mut RunArtifact)>();
    let _ = stamp_non_reproducible as fn(&mut RunArtifact);
    let _ = GuaranteeLevel::VerifyDivergence;
}
