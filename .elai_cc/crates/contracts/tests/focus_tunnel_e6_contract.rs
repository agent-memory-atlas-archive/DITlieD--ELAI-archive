//! B-28 E-6 — DependencyResolver library-tier contract.
//!
//! Pins the public surface of [`elai_orchestrator::focus_tunnel::dep_resolver`]:
//!   * Pipeline order: ort-waiver -> deprecated -> allowlist -> air-gapped ->
//!     unsupported-ecosystem -> registry -> autonomous-transitive -> human.
//!   * Anti-slopsquatting: registry 404 -> Blocked::RegistryNotFound.
//!   * Anti-slopsquatting: registry non-200 (429/500/3xx) ->
//!     Blocked::RegistryUnexpectedStatus.
//!   * Anti-stale: `ort` is always Blocked (waiver-pending), even if allowlisted.
//!   * Air-gapped guard via `ELAI_AIR_GAPPED=1`.
//!   * Unsupported ecosystems (Java / C# / Ruby) -> Blocked::UnsupportedEcosystem
//!     BEFORE the HTTP probe (reqwest rejects non-http(s) schemes).
//!   * apply() invokes the install executor ONLY for AutoApprove decisions.
//!   * load_allowlist parses additive [dependencies.allowlist] section.
//!   * dependency_resolution_stats_for_payloads returns zero stats today (the
//!     locked API for the future E-6 SSB variant, mirrors E-5a precedent).
//!
//! Spec: workflow/papers/focus-tunnel-design-interface.md L43-46, L381-440.
//! Locked decisions: workflow/sessions/s-27436/context-package-E6.md.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::OnceLock;

/// Process-global lock for tests that read or mutate `ELAI_AIR_GAPPED`.
/// std::env is process-wide and `cargo test` runs tests in parallel
/// threads by default, so a naive `set_var` / `remove_var` pair leaks
/// state into concurrent tests. EVERY resolver test acquires this lock
/// (read-side via [`env_guard`]) so the air-gapped writer test cannot
/// be observed mid-mutation. Avoids pulling in `serial_test`.
fn air_gapped_env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

/// Reader-side guard: acquire the lock AND ensure the env var is unset
/// for the duration of the test. Use this in every test that does NOT
/// itself set `ELAI_AIR_GAPPED`.
fn env_guard() -> std::sync::MutexGuard<'static, ()> {
    let guard = air_gapped_env_lock().lock().expect("env lock");
    std::env::remove_var("ELAI_AIR_GAPPED");
    guard
}

use elai_orchestrator::focus_tunnel::dep_resolver::{
    dependency_resolution_stats_for_payloads, load_allowlist, Allowlist, AutoApproveReason,
    BlockedReason, DepInstallExecutor, DepProposal, DependencyResolver, HttpProbe,
    HttpProbeError, InstallAttempt, MissingImport, ProposalSource, ResolutionDecision,
    Span, ORT_WAIVER_CRATE, ORT_WAIVER_DOC,
};
use elai_orchestrator::focus_tunnel::pre_assembly::Clock;
use elai_orchestrator::ssb::{DependencyResolvedPayload, SsbActionType, SsbPayload};
use elai_plugins::import_extractor::{DeprecatedImport, ImportLanguage};

// ---------------------------------------------------------------------------
// Test doubles
// ---------------------------------------------------------------------------

#[derive(Debug)]
struct FixedClock(u64);

impl Clock for FixedClock {
    fn now_ms(&self) -> u64 {
        self.0
    }
}

/// Stub HTTP probe — returns a canned status per URL substring (first match
/// wins). Records every call so tests can assert anti-slopsquatting paths
/// were exercised.
#[derive(Debug, Default)]
struct StubHttpProbe {
    rules: Vec<(String, u16)>,
    calls: Mutex<Vec<String>>,
}

impl StubHttpProbe {
    fn new(rules: Vec<(&str, u16)>) -> Self {
        Self {
            rules: rules
                .into_iter()
                .map(|(k, v)| (k.to_owned(), v))
                .collect(),
            calls: Mutex::new(Vec::new()),
        }
    }

    fn call_count(&self) -> usize {
        self.calls
            .lock()
            .expect("stub mutex")
            .len()
    }
}

impl HttpProbe for StubHttpProbe {
    fn probe<'a>(
        &'a self,
        url: String,
    ) -> Pin<Box<dyn Future<Output = Result<u16, HttpProbeError>> + Send + 'a>> {
        Box::pin(async move {
            self.calls
                .lock()
                .expect("stub mutex")
                .push(url.clone());
            for (pat, status) in &self.rules {
                if url.contains(pat.as_str()) {
                    return Ok(*status);
                }
            }
            // Default — pretend the registry returned 200.
            Ok(200)
        })
    }
}

/// Test executor — records every install call. Production code never spawns
/// an install: the executor seam is the integration boundary.
#[derive(Debug, Default)]
struct TestDepInstallExecutor {
    calls: Mutex<Vec<DepProposal>>,
}

impl TestDepInstallExecutor {
    fn calls(&self) -> Vec<DepProposal> {
        self.calls.lock().expect("executor mutex").clone()
    }
}

impl DepInstallExecutor for TestDepInstallExecutor {
    fn install(&self, proposal: &DepProposal) -> InstallAttempt {
        self.calls
            .lock()
            .expect("executor mutex")
            .push(proposal.clone());
        InstallAttempt {
            proposal: proposal.clone(),
            success: true,
            stderr: None,
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn clock() -> Arc<dyn Clock> {
    Arc::new(FixedClock(1_700_000_000_000))
}

fn empty_allowlist() -> Allowlist {
    Allowlist::default()
}

fn allowlist_with(ecosystem: &str, names: &[&str]) -> Allowlist {
    let mut al = Allowlist::default();
    al.insert_many(
        ecosystem,
        names.iter().map(|n| (*n).to_owned()).collect::<Vec<_>>(),
    );
    al
}

fn missing(language: ImportLanguage, name: &str) -> MissingImport {
    MissingImport {
        language,
        name: name.to_owned(),
        source_span: Some(Span { line: 1, col: 0 }),
        transitive_of: None,
    }
}

fn missing_transitive(language: ImportLanguage, name: &str, parent: &str) -> MissingImport {
    MissingImport {
        language,
        name: name.to_owned(),
        source_span: Some(Span { line: 1, col: 0 }),
        transitive_of: Some(parent.to_owned()),
    }
}

fn deprecated_blake2() -> Vec<DeprecatedImport> {
    vec![DeprecatedImport {
        ecosystem: "cargo".to_owned(),
        deprecated_name: "blake2".to_owned(),
        replacement_name: "blake3".to_owned(),
        severity: "BLOCK".to_owned(),
        reason: "blake3 is faster".to_owned(),
    }]
}

fn deprecated_rand08() -> Vec<DeprecatedImport> {
    vec![DeprecatedImport {
        ecosystem: "cargo".to_owned(),
        deprecated_name: "rand 0.8".to_owned(),
        replacement_name: "rand >= 0.9".to_owned(),
        severity: "BLOCK".to_owned(),
        reason: "outdated".to_owned(),
    }]
}

fn build_resolver(
    allowlist: Allowlist,
    autonomous: bool,
    http: Arc<dyn HttpProbe>,
) -> DependencyResolver {
    DependencyResolver::new(allowlist, autonomous, clock(), http)
}

// ---------------------------------------------------------------------------
// Contract 1 — allowlist hit → AutoApprove::Allowlist, no registry call.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn allowlist_hit_yields_auto_approve_allowlist_and_skips_registry() {
    let _g = env_guard();
    let http = Arc::new(StubHttpProbe::default());
    let resolver = build_resolver(
        allowlist_with("cargo", &["tokio"]),
        false,
        http.clone(),
    );

    let plan = resolver
        .resolve(vec![missing(ImportLanguage::Rust, "tokio")], &[])
        .await
        .expect("resolve ok");

    assert_eq!(plan.decisions.len(), 1, "one input -> one decision");
    match &plan.decisions[0] {
        ResolutionDecision::AutoApprove { reason, prop } => {
            assert!(matches!(reason, AutoApproveReason::Allowlist));
            assert_eq!(prop.name, "tokio");
            assert!(
                prop.install_command.starts_with("cargo add"),
                "install_command must dispatch to cargo add, got {:?}",
                prop.install_command
            );
        }
        other => panic!("expected AutoApprove::Allowlist, got {:?}", other),
    }
    assert_eq!(plan.allowlist_hits, vec!["tokio".to_owned()]);
    assert_eq!(
        http.call_count(),
        0,
        "allowlist hit MUST skip registry probe (anti-rate-limit)"
    );
}

// ---------------------------------------------------------------------------
// Contract 2 — deprecated pattern hit → Blocked::DeprecatedPattern; install
// executor MUST NOT be called.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn deprecated_pattern_yields_blocked_and_executor_skipped() {
    let _g = env_guard();
    let http = Arc::new(StubHttpProbe::default());
    let resolver = build_resolver(empty_allowlist(), false, http.clone());

    let plan = resolver
        .resolve_with_deprecated(
            vec![missing(ImportLanguage::Rust, "blake2")],
            &deprecated_blake2(),
        )
        .await
        .expect("resolve ok");

    assert_eq!(plan.decisions.len(), 1);
    match &plan.decisions[0] {
        ResolutionDecision::Blocked { reason, prop } => {
            match reason {
                BlockedReason::DeprecatedPattern {
                    rule_id,
                    suggestion,
                } => {
                    assert!(
                        rule_id.contains("blake2"),
                        "rule_id should reference deprecated name"
                    );
                    assert_eq!(suggestion, "blake3");
                }
                other => panic!("expected DeprecatedPattern, got {:?}", other),
            }
            assert_eq!(prop.name, "blake2");
        }
        other => panic!("expected Blocked, got {:?}", other),
    }

    let executor = TestDepInstallExecutor::default();
    let outcome = resolver.apply(&plan, &executor);
    assert!(
        executor.calls().is_empty(),
        "Blocked deps must NEVER invoke install executor"
    );
    assert!(outcome.install_results.is_empty());
    assert!(outcome.proposals_approved.is_empty());
}

// ---------------------------------------------------------------------------
// Contract 3 — `ort` is always Blocked::AntiStaleWaiverPending, even when
// the user puts it in the allowlist (waiver requires explicit human ack).
// ---------------------------------------------------------------------------

#[tokio::test]
async fn ort_is_blocked_even_when_allowlisted() {
    let _g = env_guard();
    let http = Arc::new(StubHttpProbe::default());
    let resolver = build_resolver(
        allowlist_with("cargo", &["ort"]),
        true, // even in autonomous mode
        http.clone(),
    );

    let plan = resolver
        .resolve(vec![missing(ImportLanguage::Rust, "ort")], &[])
        .await
        .expect("resolve ok");

    assert_eq!(plan.decisions.len(), 1);
    match &plan.decisions[0] {
        ResolutionDecision::Blocked { reason, prop } => {
            match reason {
                BlockedReason::AntiStaleWaiverPending {
                    crate_name,
                    waiver_doc,
                } => {
                    assert_eq!(*crate_name, ORT_WAIVER_CRATE);
                    assert_eq!(*waiver_doc, ORT_WAIVER_DOC);
                }
                other => panic!("expected AntiStaleWaiverPending, got {:?}", other),
            }
            assert_eq!(prop.name, "ort");
        }
        other => panic!("expected Blocked, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// Contract 4 — Registry 404 → Blocked::RegistryNotFound (anti-slopsquatting).
// ---------------------------------------------------------------------------

#[tokio::test]
async fn registry_404_blocks_with_registry_not_found() {
    let _g = env_guard();
    let http = Arc::new(StubHttpProbe::new(vec![("crates.io", 404)]));
    let resolver = build_resolver(empty_allowlist(), false, http.clone());

    let plan = resolver
        .resolve(
            vec![missing(ImportLanguage::Rust, "this-crate-does-not-exist")],
            &[],
        )
        .await
        .expect("resolve ok");

    assert_eq!(plan.decisions.len(), 1);
    match &plan.decisions[0] {
        ResolutionDecision::Blocked { reason, .. } => match reason {
            BlockedReason::RegistryNotFound { queried_url } => {
                assert!(
                    queried_url.contains("crates.io/api/v1/crates"),
                    "queried_url must cite the crates.io endpoint, got {:?}",
                    queried_url
                );
            }
            other => panic!("expected RegistryNotFound, got {:?}", other),
        },
        other => panic!("expected Blocked, got {:?}", other),
    }
    assert!(
        http.call_count() >= 1,
        "registry MUST be probed when no allowlist hit"
    );
}

// ---------------------------------------------------------------------------
// Contract 5 — Registry 200 + autonomous + transitive_of present →
// AutoApprove::TransitiveOfApproved.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn registry_200_autonomous_transitive_yields_auto_approve_transitive() {
    let _g = env_guard();
    let http = Arc::new(StubHttpProbe::new(vec![("crates.io", 200)]));
    let resolver = build_resolver(empty_allowlist(), true, http.clone());

    let plan = resolver
        .resolve(
            vec![missing_transitive(
                ImportLanguage::Rust,
                "serde_derive",
                "serde",
            )],
            &[],
        )
        .await
        .expect("resolve ok");

    assert_eq!(plan.decisions.len(), 1);
    match &plan.decisions[0] {
        ResolutionDecision::AutoApprove { reason, prop } => {
            assert!(matches!(reason, AutoApproveReason::TransitiveOfApproved));
            assert_eq!(prop.name, "serde_derive");
        }
        other => panic!("expected AutoApprove::TransitiveOfApproved, got {:?}", other),
    }
    assert!(plan.autonomous_used);
}

// ---------------------------------------------------------------------------
// Contract 6 — Registry 200 + interactive mode (autonomous=false) + not in
// allowlist → QueueForHuman.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn registry_200_interactive_not_allowlisted_yields_queue_for_human() {
    let _g = env_guard();
    let http = Arc::new(StubHttpProbe::new(vec![("crates.io", 200)]));
    let resolver = build_resolver(empty_allowlist(), false, http.clone());

    let plan = resolver
        .resolve(vec![missing(ImportLanguage::Rust, "ravif")], &[])
        .await
        .expect("resolve ok");

    assert_eq!(plan.decisions.len(), 1);
    match &plan.decisions[0] {
        ResolutionDecision::QueueForHuman { prop } => {
            assert_eq!(prop.name, "ravif");
            assert!(matches!(prop.source, ProposalSource::CratesIo));
        }
        other => panic!("expected QueueForHuman, got {:?}", other),
    }
    assert!(!plan.autonomous_used);
}

// ---------------------------------------------------------------------------
// Contract 7 — Air-gapped mode (ELAI_AIR_GAPPED=1): registry probe skipped;
// anything not allowlisted is Blocked::AirGapped.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn air_gapped_blocks_non_allowlisted_without_registry_probe() {
    // Hold the process-wide air-gapped env lock so a concurrent test
    // doesn't observe a leaked `ELAI_AIR_GAPPED=1`.
    let _guard = air_gapped_env_lock().lock().expect("env lock");
    // Start from a clean state in case a previous test crashed.
    std::env::remove_var("ELAI_AIR_GAPPED");
    std::env::set_var("ELAI_AIR_GAPPED", "1");

    let http = Arc::new(StubHttpProbe::default());
    let resolver = build_resolver(empty_allowlist(), false, http.clone());

    let plan = resolver
        .resolve(vec![missing(ImportLanguage::Rust, "anything")], &[])
        .await
        .expect("resolve ok");

    std::env::remove_var("ELAI_AIR_GAPPED");

    assert_eq!(plan.decisions.len(), 1);
    match &plan.decisions[0] {
        ResolutionDecision::Blocked { reason, .. } => {
            assert!(
                matches!(reason, BlockedReason::AirGapped),
                "expected AirGapped, got {:?}",
                reason
            );
        }
        other => panic!("expected Blocked::AirGapped, got {:?}", other),
    }
    assert_eq!(
        http.call_count(),
        0,
        "air-gapped mode MUST NOT touch the network"
    );
}

// ---------------------------------------------------------------------------
// Contract 8 — apply() executes install_command ONLY for AutoApprove rows.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn apply_only_invokes_executor_for_auto_approved() {
    let _g = env_guard();
    let http = Arc::new(StubHttpProbe::new(vec![("crates.io", 200)]));
    let resolver = build_resolver(
        allowlist_with("cargo", &["tokio"]),
        false,
        http.clone(),
    );

    let plan = resolver
        .resolve(
            vec![
                // Allowlisted -> AutoApprove (executor called)
                missing(ImportLanguage::Rust, "tokio"),
                // Registry-200, interactive, not allowlisted -> QueueForHuman (skip)
                missing(ImportLanguage::Rust, "ravif"),
            ],
            &[],
        )
        .await
        .expect("resolve ok");

    assert_eq!(plan.decisions.len(), 2);

    let executor = TestDepInstallExecutor::default();
    let outcome = resolver.apply(&plan, &executor);

    let calls = executor.calls();
    assert_eq!(
        calls.len(),
        1,
        "exactly ONE install call (the AutoApprove)"
    );
    assert_eq!(calls[0].name, "tokio");
    assert_eq!(outcome.proposals_approved.len(), 1);
    assert_eq!(outcome.proposals_queued.len(), 1);
    assert_eq!(outcome.proposals_queued[0].name, "ravif");
    assert_eq!(outcome.install_results.len(), 1);
    assert!(outcome.install_results[0].success);
}

// ---------------------------------------------------------------------------
// Contract 9 — Multi-language plan: Rust + Python in one resolve() call.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn multi_language_resolve_routes_per_language() {
    let _g = env_guard();
    let http = Arc::new(StubHttpProbe::new(vec![
        ("crates.io", 200),
        ("pypi.org", 200),
    ]));
    let resolver = build_resolver(
        {
            let mut al = Allowlist::default();
            al.insert_many("cargo", vec!["tokio".to_owned()]);
            al.insert_many("pip", vec!["fastapi".to_owned()]);
            al
        },
        false,
        http.clone(),
    );

    let plan = resolver
        .resolve(
            vec![
                missing(ImportLanguage::Rust, "tokio"),
                missing(ImportLanguage::Python, "fastapi"),
            ],
            &[],
        )
        .await
        .expect("resolve ok");

    assert_eq!(plan.decisions.len(), 2);
    assert_eq!(plan.allowlist_hits.len(), 2);

    // Cross-check: each AutoApprove carries the matching ProposalSource.
    let sources: Vec<ProposalSource> = plan
        .decisions
        .iter()
        .filter_map(|d| match d {
            ResolutionDecision::AutoApprove { prop, .. } => Some(prop.source.clone()),
            _ => None,
        })
        .collect();
    assert!(sources.contains(&ProposalSource::CratesIo));
    assert!(sources.contains(&ProposalSource::PyPi));
}

// ---------------------------------------------------------------------------
// Contract 10 — dependency_resolution_stats_for_payloads returns zero stats
// (locked API for the future SSB action_type, mirrors E-5a precedent).
// ---------------------------------------------------------------------------

#[test]
fn aggregator_returns_zero_stats_for_empty_input() {
    let stats = dependency_resolution_stats_for_payloads(&[]);
    assert_eq!(stats.total_auto_approved, 0);
    assert_eq!(stats.total_queued_for_human, 0);
    assert_eq!(stats.total_blocked, 0);
}

// ---------------------------------------------------------------------------
// Contract 11 — load_allowlist parses the additive [dependencies.allowlist]
// section from a TOML fixture.
// ---------------------------------------------------------------------------

#[test]
fn load_allowlist_parses_toml_fixture() {
    let tmp = tempfile::tempdir().expect("tmpdir");
    let path = tmp.path().join("elai_cc.toml");
    std::fs::write(
        &path,
        r#"
[other.section]
unused = true

[dependencies.allowlist]
cargo = ["tokio", "serde"]
pip = ["fastapi"]
npm = []
go = []
"#,
    )
    .expect("write fixture");

    let al = load_allowlist(&path).expect("load_allowlist ok");
    assert!(al.contains("cargo", "tokio"));
    assert!(al.contains("cargo", "serde"));
    assert!(al.contains("pip", "fastapi"));
    assert!(!al.contains("cargo", "ravif"));
    assert!(!al.contains("pip", "tokio"));
}

// ---------------------------------------------------------------------------
// Contract 12 — Anti-Slopsquatting design tradeoff: the allowlist bypasses
// the registry probe by design (operator vouches for the name). The contract
// codifies this so a future "always probe" refactor would deliberately break
// the test rather than silently. See locked decision 6(b).
// ---------------------------------------------------------------------------

#[tokio::test]
async fn allowlist_bypasses_registry_by_design() {
    let _g = env_guard();
    // No matter what the HTTP probe says, allowlist short-circuits FIRST.
    let http = Arc::new(StubHttpProbe::new(vec![("crates.io", 404)]));
    let resolver = build_resolver(
        allowlist_with("cargo", &["pre-vouched-internal"]),
        false,
        http.clone(),
    );

    let plan = resolver
        .resolve(
            vec![missing(ImportLanguage::Rust, "pre-vouched-internal")],
            &[],
        )
        .await
        .expect("resolve ok");

    match &plan.decisions[0] {
        ResolutionDecision::AutoApprove { reason, .. } => {
            assert!(matches!(reason, AutoApproveReason::Allowlist));
        }
        other => panic!("expected AutoApprove::Allowlist, got {:?}", other),
    }
    assert_eq!(
        http.call_count(),
        0,
        "allowlist short-circuit MUST run before any HTTP probe"
    );
}

// ---------------------------------------------------------------------------
// Contract 13 — Deprecated pattern check runs BEFORE allowlist (spec L43-46
// ordering: anti-staleness is a HARD rule that overrides operator allowance).
// ---------------------------------------------------------------------------

#[tokio::test]
async fn deprecated_overrides_allowlist() {
    let _g = env_guard();
    let http = Arc::new(StubHttpProbe::default());
    let resolver = build_resolver(
        allowlist_with("cargo", &["blake2"]),
        false,
        http.clone(),
    );

    let plan = resolver
        .resolve_with_deprecated(
            vec![missing(ImportLanguage::Rust, "blake2")],
            &deprecated_blake2(),
        )
        .await
        .expect("resolve ok");

    match &plan.decisions[0] {
        ResolutionDecision::Blocked { reason, .. } => assert!(matches!(
            reason,
            BlockedReason::DeprecatedPattern { .. }
        )),
        other => panic!("deprecated must override allowlist, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// Contract 14 — `rand 0.8` (deprecated pattern with space suffix) matches
// the bare `rand` import name (parity with import_extractor's matching
// rules, plugins/src/import_extractor.rs L298-311).
// ---------------------------------------------------------------------------

#[tokio::test]
async fn rand_zero_eight_deprecated_matches_bare_rand() {
    let _g = env_guard();
    let http = Arc::new(StubHttpProbe::default());
    let resolver = build_resolver(empty_allowlist(), false, http.clone());

    let plan = resolver
        .resolve_with_deprecated(
            vec![missing(ImportLanguage::Rust, "rand")],
            &deprecated_rand08(),
        )
        .await
        .expect("resolve ok");

    match &plan.decisions[0] {
        ResolutionDecision::Blocked { reason, .. } => assert!(matches!(
            reason,
            BlockedReason::DeprecatedPattern { .. }
        )),
        other => panic!("expected DeprecatedPattern, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// Contract 15 — Java import resolves to Blocked::UnsupportedEcosystem BEFORE
// the HTTP probe. Regression for B-28 E-6 BLOCK 1: reqwest rejects the
// `local://unsupported-ecosystem/...` URL with HttpProbeError::Transport and
// the `?` operator on probe() would abort the entire batch.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn java_import_blocks_with_unsupported_ecosystem_no_http_call() {
    let _g = env_guard();
    let http = Arc::new(StubHttpProbe::default());
    let resolver = build_resolver(empty_allowlist(), false, http.clone());

    let plan = resolver
        .resolve(
            vec![missing(ImportLanguage::Java, "com.example.tool")],
            &[],
        )
        .await
        .expect("resolve ok");

    assert_eq!(plan.decisions.len(), 1);
    match &plan.decisions[0] {
        ResolutionDecision::Blocked { reason, prop } => {
            match reason {
                BlockedReason::UnsupportedEcosystem { language } => {
                    assert_eq!(*language, ImportLanguage::Java);
                }
                other => panic!("expected UnsupportedEcosystem, got {:?}", other),
            }
            assert_eq!(prop.name, "com.example.tool");
            assert!(matches!(prop.source, ProposalSource::Local));
        }
        other => panic!("expected Blocked, got {:?}", other),
    }
    assert_eq!(
        http.call_count(),
        0,
        "unsupported ecosystem MUST short-circuit before HTTP probe"
    );
}

// ---------------------------------------------------------------------------
// Contract 16 — C# import resolves to Blocked::UnsupportedEcosystem.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn csharp_import_blocks_with_unsupported_ecosystem_no_http_call() {
    let _g = env_guard();
    let http = Arc::new(StubHttpProbe::default());
    let resolver = build_resolver(empty_allowlist(), false, http.clone());

    let plan = resolver
        .resolve(
            vec![missing(ImportLanguage::CSharp, "Newtonsoft.Json")],
            &[],
        )
        .await
        .expect("resolve ok");

    assert_eq!(plan.decisions.len(), 1);
    match &plan.decisions[0] {
        ResolutionDecision::Blocked { reason, .. } => match reason {
            BlockedReason::UnsupportedEcosystem { language } => {
                assert_eq!(*language, ImportLanguage::CSharp);
            }
            other => panic!("expected UnsupportedEcosystem, got {:?}", other),
        },
        other => panic!("expected Blocked, got {:?}", other),
    }
    assert_eq!(http.call_count(), 0);
}

// ---------------------------------------------------------------------------
// Contract 17 — Ruby import resolves to Blocked::UnsupportedEcosystem.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn ruby_import_blocks_with_unsupported_ecosystem_no_http_call() {
    let _g = env_guard();
    let http = Arc::new(StubHttpProbe::default());
    let resolver = build_resolver(empty_allowlist(), false, http.clone());

    let plan = resolver
        .resolve(
            vec![missing(ImportLanguage::Ruby, "activerecord")],
            &[],
        )
        .await
        .expect("resolve ok");

    assert_eq!(plan.decisions.len(), 1);
    match &plan.decisions[0] {
        ResolutionDecision::Blocked { reason, .. } => match reason {
            BlockedReason::UnsupportedEcosystem { language } => {
                assert_eq!(*language, ImportLanguage::Ruby);
            }
            other => panic!("expected UnsupportedEcosystem, got {:?}", other),
        },
        other => panic!("expected Blocked, got {:?}", other),
    }
    assert_eq!(http.call_count(), 0);
}

// ---------------------------------------------------------------------------
// Contract 18 — Mixed batch with an unsupported-ecosystem entry: subsequent
// SUPPORTED imports MUST still be classified (regression for the `?`
// short-circuit that previously aborted the entire batch on the Java row).
// ---------------------------------------------------------------------------

#[tokio::test]
async fn unsupported_ecosystem_does_not_abort_batch() {
    let _g = env_guard();
    let http = Arc::new(StubHttpProbe::new(vec![("crates.io", 200)]));
    let resolver = build_resolver(
        allowlist_with("cargo", &["tokio"]),
        false,
        http.clone(),
    );

    let plan = resolver
        .resolve(
            vec![
                // (1) Java -> UnsupportedEcosystem (must NOT abort batch)
                missing(ImportLanguage::Java, "com.example.broken"),
                // (2) Rust allowlisted -> AutoApprove::Allowlist
                missing(ImportLanguage::Rust, "tokio"),
                // (3) Rust not-allowlisted, registry 200 -> QueueForHuman
                missing(ImportLanguage::Rust, "ravif"),
            ],
            &[],
        )
        .await
        .expect("resolve ok");

    assert_eq!(plan.decisions.len(), 3, "all three inputs classified");
    assert!(matches!(
        &plan.decisions[0],
        ResolutionDecision::Blocked {
            reason: BlockedReason::UnsupportedEcosystem { .. },
            ..
        }
    ));
    assert!(matches!(
        &plan.decisions[1],
        ResolutionDecision::AutoApprove {
            reason: AutoApproveReason::Allowlist,
            ..
        }
    ));
    assert!(matches!(
        &plan.decisions[2],
        ResolutionDecision::QueueForHuman { .. }
    ));
}

// ---------------------------------------------------------------------------
// Contract 19 — Registry 429 (rate-limit) in autonomous + transitive mode MUST
// block with RegistryUnexpectedStatus, NOT AutoApprove::TransitiveOfApproved.
// Regression for B-28 E-6 BLOCK 2: only HTTP 200 verifies existence.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn registry_429_autonomous_transitive_blocks_unexpected_status() {
    let _g = env_guard();
    let http = Arc::new(StubHttpProbe::new(vec![("crates.io", 429)]));
    let resolver = build_resolver(empty_allowlist(), true, http.clone());

    let plan = resolver
        .resolve(
            vec![missing_transitive(
                ImportLanguage::Rust,
                "serde_derive",
                "serde",
            )],
            &[],
        )
        .await
        .expect("resolve ok");

    assert_eq!(plan.decisions.len(), 1);
    match &plan.decisions[0] {
        ResolutionDecision::Blocked { reason, .. } => match reason {
            BlockedReason::RegistryUnexpectedStatus {
                status,
                queried_url,
            } => {
                assert_eq!(*status, 429);
                assert!(
                    queried_url.contains("crates.io/api/v1/crates"),
                    "queried_url must cite the crates.io endpoint, got {:?}",
                    queried_url
                );
            }
            other => panic!("expected RegistryUnexpectedStatus, got {:?}", other),
        },
        other => panic!(
            "expected Blocked::RegistryUnexpectedStatus (NOT AutoApprove), got {:?}",
            other
        ),
    }
    assert!(
        !plan.autonomous_used,
        "429 must NOT trip autonomous_used — verification failed"
    );
}

// ---------------------------------------------------------------------------
// Contract 20 — Registry 500 (server error) in interactive mode MUST block
// with RegistryUnexpectedStatus. Anti-slopsquatting: an unknown status is
// never "verified".
// ---------------------------------------------------------------------------

#[tokio::test]
async fn registry_500_blocks_unexpected_status() {
    let _g = env_guard();
    let http = Arc::new(StubHttpProbe::new(vec![("crates.io", 500)]));
    let resolver = build_resolver(empty_allowlist(), false, http.clone());

    let plan = resolver
        .resolve(vec![missing(ImportLanguage::Rust, "anycrate")], &[])
        .await
        .expect("resolve ok");

    assert_eq!(plan.decisions.len(), 1);
    match &plan.decisions[0] {
        ResolutionDecision::Blocked { reason, .. } => match reason {
            BlockedReason::RegistryUnexpectedStatus { status, .. } => {
                assert_eq!(*status, 500);
            }
            other => panic!("expected RegistryUnexpectedStatus, got {:?}", other),
        },
        other => panic!("expected Blocked, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// Contract 21 — Registry 301 (redirect) MUST block with
// RegistryUnexpectedStatus. The four registry endpoints are canonical https
// URLs that should never redirect mid-registry — a 3xx is suspicious.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn registry_301_blocks_unexpected_status() {
    let _g = env_guard();
    let http = Arc::new(StubHttpProbe::new(vec![("crates.io", 301)]));
    let resolver = build_resolver(empty_allowlist(), false, http.clone());

    let plan = resolver
        .resolve(vec![missing(ImportLanguage::Rust, "redirected")], &[])
        .await
        .expect("resolve ok");

    assert_eq!(plan.decisions.len(), 1);
    match &plan.decisions[0] {
        ResolutionDecision::Blocked { reason, .. } => match reason {
            BlockedReason::RegistryUnexpectedStatus { status, .. } => {
                assert_eq!(*status, 301);
            }
            other => panic!("expected RegistryUnexpectedStatus, got {:?}", other),
        },
        other => panic!("expected Blocked, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// B-28 WIRING-1 — DependencyResolved SSB round-trip + aggregator wired
// ---------------------------------------------------------------------------

/// `SsbActionType::DependencyResolved` and its payload must round-trip via
/// msgpack and report the matching action_type.
#[test]
fn dependency_resolved_ssb_round_trip() {
    let payload = SsbPayload::DependencyResolved(DependencyResolvedPayload {
        language: "rust".into(),
        name: "tokio".into(),
        decision: "auto_approve".into(),
        reason: "allowlist_match".into(),
        install_command: Some("cargo add tokio".into()),
    });

    assert_eq!(payload.action_type(), SsbActionType::DependencyResolved);
    assert_eq!(payload.action_type().as_sql_str(), "dependency_resolved");

    let bytes = rmp_serde::to_vec_named(&payload).expect("encode");
    let decoded: SsbPayload = rmp_serde::from_slice(&bytes).expect("decode");
    assert_eq!(decoded, payload, "msgpack round-trip must preserve payload");
}

/// `dependency_resolution_stats_for_payloads` aggregates real
/// `DependencyResolved` rows by decision + per-language buckets.
#[test]
fn dependency_resolution_stats_aggregates_real_rows() {
    let rows = vec![
        SsbPayload::DependencyResolved(DependencyResolvedPayload {
            language: "rust".into(),
            name: "tokio".into(),
            decision: "auto_approve".into(),
            reason: "allowlist_match".into(),
            install_command: Some("cargo add tokio".into()),
        }),
        SsbPayload::DependencyResolved(DependencyResolvedPayload {
            language: "rust".into(),
            name: "obscure".into(),
            decision: "queue_for_human".into(),
            reason: "not_allowlisted".into(),
            install_command: None,
        }),
        SsbPayload::DependencyResolved(DependencyResolvedPayload {
            language: "python".into(),
            name: "ort".into(),
            decision: "blocked".into(),
            reason: "anti_stale_waiver".into(),
            install_command: None,
        }),
    ];
    let stats = dependency_resolution_stats_for_payloads(&rows);
    assert_eq!(stats.total_proposed, 3);
    assert_eq!(stats.total_auto_approved, 1);
    assert_eq!(stats.total_queued, 1);
    assert_eq!(stats.total_blocked, 1);
    assert_eq!(stats.per_language.get("rust").copied().unwrap_or(0), 2);
    assert_eq!(stats.per_language.get("python").copied().unwrap_or(0), 1);
}
