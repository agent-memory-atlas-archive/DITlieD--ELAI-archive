//! B-28 E-6 — DependencyResolver library-tier (proposer-only).
//!
//! Spec: `workflow/papers/focus-tunnel-design-interface.md`
//!   * L43-46  DependencyResolver design (detect → check-deprecated →
//!             registry → conflict → propose → approval → install).
//!   * L381-440 Approval gate + autonomous-mode transitive auto-approve.
//!   * L229-230 ASSERTION: new_dep_requires_user_confirmation_unless_in
//!             _allowlist_or_transitive.
//!   * L329-330 DO NOT skip the deprecated_patterns check before the
//!             registry query; DO NOT auto-approve deps not in allowlist
//!             nor already transitive; DO NOT run cargo add / pip install
//!             from model output.
//!
//! Library-tier shape — async only where genuinely required (HTTP probe).
//! Mirrors the E-5a [`crate::focus_tunnel::pre_assembly`] posture: the
//! reader-side aggregator [`dependency_resolution_stats_for_payloads`]
//! ships in E-6 but returns zero stats today because the
//! `SsbPayload::DependencyResolved` variant does NOT yet exist (ssb.rs
//! LOCKED by session s-28564). The locked API surface means the future
//! E-6-SSB sub-phase only needs to plug in one match arm.
//!
//! The resolver NEVER spawns an install command. Install execution is
//! delegated through the [`DepInstallExecutor`] trait, which the WIRING
//! phase (post-E-7) supplies with a sandboxed implementation. Spec
//! pitfall L330 — "DO NOT run cargo add / pip install from model output".
//!
//! Registry endpoints are DUPLICATED verbatim from
//! `crates/cli/src/dep_audit.rs:26-38` with citation comments. dep_audit
//! is in the `cli` crate which `orchestrator` cannot depend on (downstream
//! direction); reuse via path-dep would introduce a cycle. The four
//! constants below are intentionally a local copy — anti-DRY in the
//! small, anti-cycle in the large.
//!
//! HARD INVARIANTS (locked):
//!   * `ort` is ALWAYS blocked with [`BlockedReason::AntiStaleWaiverPending`]
//!     even if operator-allowlisted (waiver lives in
//!     `workflow/reasoning/B-16A-decisions.md`).
//!   * Air-gapped mode (`ELAI_AIR_GAPPED=1`) skips the registry probe; any
//!     non-allowlisted, non-deprecated input becomes
//!     [`BlockedReason::AirGapped`].
//!   * The deprecated_patterns check runs BEFORE the allowlist short-circuit
//!     (spec L329-330).
//!   * Allowlist hits SKIP the registry probe by design — the operator has
//!     pre-vouched for the name (see contract test
//!     `allowlist_bypasses_registry_by_design`).

use std::collections::BTreeSet;
use std::env;
use std::fmt;
use std::future::Future;
use std::io;
use std::path::Path;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use indexmap::IndexMap;

use elai_plugins::import_extractor::{DeprecatedImport, ImportLanguage};

use crate::focus_tunnel::pre_assembly::Clock;
use crate::ssb::SsbPayload;

// ---------------------------------------------------------------------------
// Registry endpoint constants — duplicated verbatim from
// crates/cli/src/dep_audit.rs:26-38. dep_audit.rs is the source of truth;
// keep these in sync if dep_audit's endpoints change. cli/ is downstream
// of orchestrator/ (orchestrator cannot depend on cli), so reuse via a
// path-dep would introduce a cycle.
// ---------------------------------------------------------------------------

/// Citation: cli/src/dep_audit.rs:26.
const CRATES_IO_BASE: &str = "https://crates.io/api/v1/crates";

/// Citation: cli/src/dep_audit.rs:27.
const PYPI_BASE: &str = "https://pypi.org/pypi";

/// Citation: cli/src/dep_audit.rs:28.
const NPM_BASE: &str = "https://registry.npmjs.org";

/// Citation: cli/src/dep_audit.rs:29.
const GO_PROXY_BASE: &str = "https://proxy.golang.org";

/// Citation: cli/src/dep_audit.rs:38. Mandated by crates.io ToS.
const USER_AGENT: &str = "elai-dep-audit/0.1 (https://github.com/elai)";

/// HTTP probe timeout — short enough that the resolver fails fast when the
/// caller is offline or behind a captive portal.
const HTTP_TIMEOUT: Duration = Duration::from_secs(15);

/// Environment variable that disables all registry probes. When set to
/// `"1"` the resolver routes any non-allowlisted, non-deprecated input to
/// [`BlockedReason::AirGapped`].
const AIR_GAPPED_ENV: &str = "ELAI_AIR_GAPPED";

/// Anti-stale waiver subject. `ort` ships at `=2.0.0-rc.12` per
/// `workflow/reasoning/B-16A-decisions.md`; the resolver always blocks it
/// to force a manual waiver acknowledgement.
pub const ORT_WAIVER_CRATE: &str = "ort";

/// Anti-stale waiver document path.
pub const ORT_WAIVER_DOC: &str = "workflow/reasoning/B-16A-decisions.md";

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Position of an import in source code. Optional in [`MissingImport`] so
/// callers that synthesise resolver inputs without source spans (e.g. when
/// running over a manifest diff) don't have to invent a line number.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub line: u32,
    pub col: u32,
}

/// A single missing dependency the resolver must classify. Populated by
/// callers from [`elai_plugins::import_extractor::extract_imports`]
/// output that did not match an existing manifest entry.
#[derive(Debug, Clone)]
pub struct MissingImport {
    pub language: ImportLanguage,
    pub name: String,
    pub source_span: Option<Span>,
    /// Optional parent crate name when the missing import is a transitive
    /// of a declared dep (e.g. `serde_derive` of `serde`). Resolver uses
    /// this to escalate to [`AutoApproveReason::TransitiveOfApproved`]
    /// when running in autonomous mode.
    pub transitive_of: Option<String>,
}

/// Where the dependency would be sourced from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProposalSource {
    CratesIo,
    PyPi,
    Npm,
    GoProxy,
    /// Reserved for ecosystems with no first-class registry probe in E-6
    /// (maven / nuget / bundler) and for path-deps inside the workspace.
    Local,
}

/// A concrete install proposal — the resolver produces these for every
/// decision (including Blocked) so the dashboard can surface what was
/// rejected and why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DepProposal {
    pub language: ImportLanguage,
    pub name: String,
    pub version: Option<String>,
    pub install_command: String,
    pub source: ProposalSource,
}

/// Why a proposal was auto-approved. The variant is what gets surfaced in
/// the SSB audit row so reviewers can tell allowlist-hits from
/// autonomous-transitive escalations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutoApproveReason {
    /// Name appears in the operator-curated allowlist.
    Allowlist,
    /// Autonomous mode + the missing import is transitive of a previously
    /// approved dep. Locked decision 6(e).
    TransitiveOfApproved,
}

/// Why a proposal was Blocked. Every variant is a HARD refusal — the
/// resolver does NOT escalate Blocked to QueueForHuman; it expects an
/// out-of-band waiver (or a manifest change that removes the import).
///
/// Serde is intentionally NOT derived: the `AntiStaleWaiverPending`
/// variant carries `&'static str` fields, which cannot satisfy
/// `Deserialize<'de>` without an interior owned representation. The
/// future SSB sub-phase serializes via a flat string mirror
/// (same pattern as `ExecutionModeChangedPayload`), so the enum itself
/// does not need to be (de)serializable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockedReason {
    /// Deprecated import matched the rules.db patterns (or the locally
    /// supplied list). `suggestion` mirrors the
    /// [`DeprecatedImport::replacement_name`] field.
    DeprecatedPattern {
        rule_id: String,
        suggestion: String,
    },
    /// Registry returned a 404 — anti-slopsquatting (CLAUDE.md HARD RULE).
    RegistryNotFound { queried_url: String },
    /// Registry returned a non-200 status that is not 404 (e.g. 429
    /// rate-limit, 500 server error, 301 redirect). Anti-slopsquatting:
    /// only HTTP 200 verifies a package exists — every other status is
    /// treated as "unknown" so a 429 in autonomous mode cannot
    /// auto-approve a transitive dep. Locked decision (B-28 E-6 fix
    /// cycle): redirects (3xx) are also suspicious because the four
    /// registry endpoints in this file are canonical https URLs that
    /// should never redirect mid-registry.
    RegistryUnexpectedStatus {
        status: u16,
        queried_url: String,
    },
    /// `ELAI_AIR_GAPPED=1` is set; no network probes allowed.
    AirGapped,
    /// Crate requires a manual waiver (today only `ort` — see
    /// [`ORT_WAIVER_CRATE`]).
    AntiStaleWaiverPending {
        crate_name: &'static str,
        waiver_doc: &'static str,
    },
    /// Ecosystem has no first-class registry probe in E-6 (maven /
    /// nuget / bundler). The resolver SHORT-CIRCUITS before the HTTP
    /// probe so `local://unsupported-ecosystem/...` URLs never reach
    /// `reqwest`, which would otherwise reject the non-http(s) scheme
    /// with `HttpProbeError::Transport` and abort the entire batch.
    /// Surfaced in the Blocked row so the dashboard can prompt the
    /// operator for a manual install path.
    UnsupportedEcosystem {
        language: ImportLanguage,
    },
}

/// The resolver's per-input verdict. Serde NOT derived — see
/// [`BlockedReason`] doc-comment; the future SSB variant serialises a
/// flat-string mirror.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolutionDecision {
    AutoApprove {
        prop: DepProposal,
        reason: AutoApproveReason,
    },
    QueueForHuman {
        prop: DepProposal,
    },
    Blocked {
        prop: DepProposal,
        reason: BlockedReason,
    },
}

/// Output of [`DependencyResolver::resolve`]. The locked invariants:
///   * `decisions.len() == input_missing_imports.len()` (closure).
///   * `allowlist_hits` contains the name of every input that was matched
///     against the allowlist (a superset of the AutoApprove::Allowlist set
///     because deprecated overrides allowlist — see contract test
///     `deprecated_overrides_allowlist`).
///   * `autonomous_used` is `true` iff at least one decision was
///     [`AutoApproveReason::TransitiveOfApproved`].
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ResolutionPlan {
    pub decisions: Vec<ResolutionDecision>,
    pub allowlist_hits: Vec<String>,
    pub autonomous_used: bool,
}

/// Record of one install attempt — the resolver does NOT spawn installs
/// itself, so this is populated by the supplied [`DepInstallExecutor`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallAttempt {
    pub proposal: DepProposal,
    pub success: bool,
    pub stderr: Option<String>,
}

/// Output of [`DependencyResolver::apply`]. Tests assert that
/// `install_results.len() == proposals_approved.len()` — the executor is
/// invoked exactly once per AutoApprove decision and NEVER for
/// QueueForHuman or Blocked.
#[derive(Debug, Clone, Default)]
pub struct ApplyOutcome {
    pub proposals_approved: Vec<DepProposal>,
    pub proposals_queued: Vec<DepProposal>,
    pub install_results: Vec<InstallAttempt>,
}

// ---------------------------------------------------------------------------
// Allowlist
// ---------------------------------------------------------------------------

/// Per-ecosystem name set. Backed by [`IndexMap`] to preserve insertion
/// order from the underlying TOML file (deterministic test output even
/// when the user reorders ecosystems).
#[derive(Debug, Clone, Default)]
pub struct Allowlist {
    inner: IndexMap<String, BTreeSet<String>>,
}

impl Allowlist {
    /// Bulk-insert a slice of names into one ecosystem bucket.
    pub fn insert_many<I>(&mut self, ecosystem: &str, names: I)
    where
        I: IntoIterator<Item = String>,
    {
        let entry = self
            .inner
            .entry(ecosystem.to_owned())
            .or_insert_with(BTreeSet::new);
        for n in names {
            entry.insert(n);
        }
    }

    /// `true` iff `name` is in `ecosystem`'s bucket.
    #[must_use]
    pub fn contains(&self, ecosystem: &str, name: &str) -> bool {
        match self.inner.get(ecosystem) {
            Some(set) => set.contains(name),
            None => false,
        }
    }

    /// Ecosystem keys in insertion order.
    pub fn ecosystems(&self) -> impl Iterator<Item = &str> {
        self.inner.keys().map(String::as_str)
    }
}

/// Parse the additive `[dependencies.allowlist]` section from an elai_cc
/// TOML file. Returns an empty [`Allowlist`] when the section is absent —
/// the caller must NOT treat absence as an error (existing elai_cc.toml
/// files predate B-28 E-6 and have no allowlist section).
pub fn load_allowlist(path: &Path) -> io::Result<Allowlist> {
    let bytes = std::fs::read_to_string(path)?;
    let doc: toml::Value = match toml::from_str(&bytes) {
        Ok(v) => v,
        Err(e) => {
            return Err(io::Error::new(io::ErrorKind::InvalidData, e.to_string()));
        }
    };

    let mut allowlist = Allowlist::default();
    let table = match doc
        .get("dependencies")
        .and_then(|v| v.get("allowlist"))
        .and_then(|v| v.as_table())
    {
        Some(t) => t,
        None => return Ok(allowlist),
    };

    for (ecosystem, value) in table {
        let arr = match value.as_array() {
            Some(a) => a,
            None => continue,
        };
        let names: Vec<String> = arr
            .iter()
            .filter_map(|v| v.as_str().map(str::to_owned))
            .collect();
        allowlist.insert_many(ecosystem, names);
    }
    Ok(allowlist)
}

// ---------------------------------------------------------------------------
// HttpProbe trait + Reqwest impl
// ---------------------------------------------------------------------------

/// Errors surfaced by [`HttpProbe`] implementations. Kept small —
/// production code only needs to distinguish "the network is down" from
/// "the registry said 404", which is encoded in the `Ok(u16)` path.
#[derive(Debug)]
pub enum HttpProbeError {
    /// Underlying HTTP client failed to send the request (DNS, TLS,
    /// connection refused, timeout).
    Transport(String),
}

impl fmt::Display for HttpProbeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Transport(msg) => write!(f, "http transport error: {msg}"),
        }
    }
}

impl std::error::Error for HttpProbeError {}

/// Minimal async surface used by the resolver. Returns the HTTP status
/// code (200 → present, 404 → anti-slopsquatting block).
///
/// The signature uses an explicit `Pin<Box<dyn Future ...>>` return type
/// because edition-2021 traits do not support `async fn` with `dyn` (the
/// resolver injects this trait via `Arc<dyn HttpProbe>`). Once we move
/// to edition 2024 we can replace this with native AFIT.
pub trait HttpProbe: Send + Sync + fmt::Debug {
    fn probe<'a>(
        &'a self,
        url: String,
    ) -> Pin<Box<dyn Future<Output = Result<u16, HttpProbeError>> + Send + 'a>>;
}

/// Production [`HttpProbe`] backed by `reqwest::Client`. Reuses the same
/// `default_features=false, json, rustls-tls` build configuration the rest
/// of the orchestrator uses (`crates/orchestrator/Cargo.toml` L28).
#[derive(Debug)]
pub struct ReqwestHttpProbe {
    client: reqwest::Client,
}

impl ReqwestHttpProbe {
    /// Build a probe with the locked timeout + User-Agent. Returns
    /// `Err(HttpProbeError::Transport)` only when the client builder
    /// itself rejects the configuration (extremely rare — would require
    /// e.g. an unreachable TLS root store).
    pub fn new() -> Result<Self, HttpProbeError> {
        reqwest::Client::builder()
            .timeout(HTTP_TIMEOUT)
            .user_agent(USER_AGENT)
            .build()
            .map(|client| Self { client })
            .map_err(|e| HttpProbeError::Transport(e.to_string()))
    }
}

impl HttpProbe for ReqwestHttpProbe {
    fn probe<'a>(
        &'a self,
        url: String,
    ) -> Pin<Box<dyn Future<Output = Result<u16, HttpProbeError>> + Send + 'a>> {
        Box::pin(async move {
            match self.client.get(&url).send().await {
                Ok(resp) => Ok(resp.status().as_u16()),
                Err(e) => Err(HttpProbeError::Transport(e.to_string())),
            }
        })
    }
}

// ---------------------------------------------------------------------------
// DepInstallExecutor trait
// ---------------------------------------------------------------------------

/// Seam for the actual `cargo add` / `pip install` invocation. Production
/// implementations live behind the WIRING phase's sandbox + permission
/// gate; the test crate supplies a recording stub.
///
/// HARD RULE (spec L330): the LLM must NEVER spawn this; the resolver
/// invokes the executor only AFTER explicit AutoApprove (allowlist or
/// autonomous-transitive) — never directly off model output.
pub trait DepInstallExecutor: Send + Sync {
    fn install(&self, proposal: &DepProposal) -> InstallAttempt;
}

// ---------------------------------------------------------------------------
// DependencyResolver
// ---------------------------------------------------------------------------

/// Proposer-only resolver. Construct via [`DependencyResolver::new`] +
/// inject an [`HttpProbe`] (production: [`ReqwestHttpProbe`]; tests: a
/// stub). The async [`resolve`](Self::resolve) method classifies every
/// input into a [`ResolutionDecision`]; [`apply`](Self::apply) is a
/// synchronous dispatcher that only invokes the supplied executor for
/// [`ResolutionDecision::AutoApprove`] rows.
#[derive(Debug)]
pub struct DependencyResolver {
    allowlist: Allowlist,
    autonomous: bool,
    #[allow(dead_code)] // Clock is reserved for the future SSB row that
    // records `resolved_at_ms`; the field is plumbed today so the WIRING
    // phase's SsbPayload::DependencyResolved emit path doesn't need a
    // constructor change.
    clock: Arc<dyn Clock>,
    http: Arc<dyn HttpProbe>,
    last_probe_ms: Mutex<Option<u64>>,
}

impl DependencyResolver {
    /// Construct the resolver with explicit dependency injection. Tests
    /// supply a [`crate::focus_tunnel::pre_assembly::Clock`] and an
    /// [`HttpProbe`] stub; production uses [`crate::focus_tunnel::pre_assembly::SystemTimeClock`]
    /// and [`ReqwestHttpProbe`].
    #[must_use]
    pub fn new(
        allowlist: Allowlist,
        autonomous: bool,
        clock: Arc<dyn Clock>,
        http: Arc<dyn HttpProbe>,
    ) -> Self {
        Self {
            allowlist,
            autonomous,
            clock,
            http,
            last_probe_ms: Mutex::new(None),
        }
    }

    /// Resolve a batch of missing imports without supplying a deprecated
    /// pattern list. Thin wrapper around
    /// [`resolve_with_deprecated`](Self::resolve_with_deprecated) for the
    /// common case where callers have already filtered deprecated
    /// imports upstream (the streaming validator does this via
    /// `plugins::check_deprecated_imports`).
    pub async fn resolve(
        &self,
        missing_imports: Vec<MissingImport>,
        deprecated: &[DeprecatedImport],
    ) -> Result<ResolutionPlan, HttpProbeError> {
        self.resolve_with_deprecated(missing_imports, deprecated)
            .await
    }

    /// Core resolution entrypoint. Pipeline order (spec L43-46):
    ///   1. Anti-stale waiver (e.g. `ort`) → Blocked.
    ///   2. Deprecated pattern hit → Blocked (overrides allowlist).
    ///   3. Allowlist hit → AutoApprove::Allowlist (skip registry).
    ///   4. Air-gapped → Blocked::AirGapped (skip registry).
    ///   5a. Unsupported ecosystem (Java/C#/Ruby) → Blocked::UnsupportedEcosystem
    ///       (skip registry — `reqwest` rejects `local://` URLs).
    ///   5. Registry probe — 404 → Blocked::RegistryNotFound.
    ///   5b. Registry probe — non-200 (429/500/3xx) → Blocked::RegistryUnexpectedStatus.
    ///   6. Autonomous + transitive_of → AutoApprove::TransitiveOfApproved.
    ///   7. Else → QueueForHuman.
    pub async fn resolve_with_deprecated(
        &self,
        missing_imports: Vec<MissingImport>,
        deprecated: &[DeprecatedImport],
    ) -> Result<ResolutionPlan, HttpProbeError> {
        let air_gapped = env::var(AIR_GAPPED_ENV)
            .map(|v| v.trim() == "1")
            .unwrap_or(false);

        let mut plan = ResolutionPlan::default();

        for missing in missing_imports {
            let proposal = self.build_proposal(&missing);

            // (1) Anti-stale ort waiver — overrides EVERYTHING (locked
            // decision 6c). Even an autonomous-mode operator with `ort` in
            // the allowlist must take the manual waiver path.
            if missing.name == ORT_WAIVER_CRATE {
                plan.decisions.push(ResolutionDecision::Blocked {
                    prop: proposal,
                    reason: BlockedReason::AntiStaleWaiverPending {
                        crate_name: ORT_WAIVER_CRATE,
                        waiver_doc: ORT_WAIVER_DOC,
                    },
                });
                continue;
            }

            // (2) Deprecated pattern — spec L329-330. Runs BEFORE the
            // allowlist short-circuit so an outdated dep can't sneak past
            // an operator-curated list.
            if let Some(dep) = match_deprecated(&missing, deprecated) {
                plan.decisions.push(ResolutionDecision::Blocked {
                    prop: proposal,
                    reason: BlockedReason::DeprecatedPattern {
                        rule_id: format!("deprecated:{}", dep.deprecated_name),
                        suggestion: dep.replacement_name.clone(),
                    },
                });
                continue;
            }

            // (3) Allowlist hit — operator pre-vouched, skip registry by
            // design (locked decision 6b + contract test
            // `allowlist_bypasses_registry_by_design`).
            let ecosystem = missing.language.ecosystem();
            if self.allowlist.contains(ecosystem, &missing.name) {
                plan.allowlist_hits.push(missing.name.clone());
                plan.decisions.push(ResolutionDecision::AutoApprove {
                    prop: proposal,
                    reason: AutoApproveReason::Allowlist,
                });
                continue;
            }

            // (4) Air-gapped guard — no network allowed.
            if air_gapped {
                plan.decisions.push(ResolutionDecision::Blocked {
                    prop: proposal,
                    reason: BlockedReason::AirGapped,
                });
                continue;
            }

            // (5a) Unsupported-ecosystem short-circuit — Java / C# / Ruby
            // resolve to `local://unsupported-ecosystem/...` via
            // `registry_url_for`. `reqwest` rejects non-http(s) schemes
            // with `HttpProbeError::Transport`, and `?` on the probe
            // result aborts the WHOLE batch. Route to Blocked BEFORE the
            // probe so the remaining inputs are still classified
            // (B-28 E-6 fix cycle: BLOCK 1).
            if matches!(proposal.source, ProposalSource::Local) {
                tracing::debug!(
                    target: "elai_orchestrator::focus_tunnel::dep_resolver",
                    name = %missing.name,
                    language = ?missing.language,
                    "unsupported ecosystem — skipping HTTP probe"
                );
                plan.decisions.push(ResolutionDecision::Blocked {
                    prop: proposal,
                    reason: BlockedReason::UnsupportedEcosystem {
                        language: missing.language,
                    },
                });
                continue;
            }

            // (5) Registry probe — anti-slopsquatting (CLAUDE.md HARD RULE).
            let queried_url = registry_url_for(missing.language, &missing.name);
            let status = self.http.probe(queried_url.clone()).await?;
            // Record an observable_effect timestamp for the future SSB row
            // (see Clock field doc).
            if let Ok(mut guard) = self.last_probe_ms.lock() {
                *guard = Some(self.clock.now_ms());
            }
            if status == 404 {
                tracing::warn!(
                    target: "elai_orchestrator::focus_tunnel::dep_resolver",
                    name = %missing.name,
                    url = %queried_url,
                    "registry returned 404 — blocking (anti-slopsquatting)"
                );
                plan.decisions.push(ResolutionDecision::Blocked {
                    prop: proposal,
                    reason: BlockedReason::RegistryNotFound {
                        queried_url: queried_url.clone(),
                    },
                });
                continue;
            }
            // (5b) Anti-slopsquatting tightening — only HTTP 200 verifies
            // a package exists. Treat 429 (rate-limit), 5xx (server
            // error), and 3xx (redirect) as "unknown" so an autonomous
            // batch cannot promote a transitive dep that we failed to
            // verify. B-28 E-6 fix cycle: BLOCK 2.
            if status != 200 {
                tracing::warn!(
                    target: "elai_orchestrator::focus_tunnel::dep_resolver",
                    name = %missing.name,
                    url = %queried_url,
                    status = status,
                    "registry returned non-200 — blocking (anti-slopsquatting)"
                );
                plan.decisions.push(ResolutionDecision::Blocked {
                    prop: proposal,
                    reason: BlockedReason::RegistryUnexpectedStatus {
                        status,
                        queried_url: queried_url.clone(),
                    },
                });
                continue;
            }

            // (6) Autonomous + transitive_of → AutoApprove::TransitiveOfApproved.
            if self.autonomous && missing.transitive_of.is_some() {
                plan.autonomous_used = true;
                plan.decisions.push(ResolutionDecision::AutoApprove {
                    prop: proposal,
                    reason: AutoApproveReason::TransitiveOfApproved,
                });
                continue;
            }

            // (7) Default — human gate. Spec L229-230 ASSERTION.
            plan.decisions
                .push(ResolutionDecision::QueueForHuman { prop: proposal });
        }
        Ok(plan)
    }

    /// Dispatch the install command for every [`ResolutionDecision::AutoApprove`]
    /// decision. QueueForHuman and Blocked decisions are NOT routed to
    /// the executor — the contract test
    /// `apply_only_invokes_executor_for_auto_approved` pins this.
    pub fn apply(&self, plan: &ResolutionPlan, executor: &dyn DepInstallExecutor) -> ApplyOutcome {
        let mut outcome = ApplyOutcome::default();
        for decision in &plan.decisions {
            match decision {
                ResolutionDecision::AutoApprove { prop, .. } => {
                    outcome.proposals_approved.push(prop.clone());
                    outcome.install_results.push(executor.install(prop));
                }
                ResolutionDecision::QueueForHuman { prop } => {
                    outcome.proposals_queued.push(prop.clone());
                }
                ResolutionDecision::Blocked { .. } => {
                    // Intentionally drop — caller can re-walk plan.decisions
                    // for blocked-row reporting; the executor is never
                    // invoked.
                }
            }
        }
        outcome
    }

    /// Build a [`DepProposal`] for the given missing import. Version is
    /// left `None` in E-6 (a follow-up phase parses the registry JSON
    /// response for a concrete latest version); install_command is built
    /// from the locked ecosystem mapping.
    fn build_proposal(&self, missing: &MissingImport) -> DepProposal {
        let source = registry_source_for(missing.language);
        let install_command = install_command_for(missing.language, &missing.name);
        DepProposal {
            language: missing.language,
            name: missing.name.clone(),
            version: None,
            install_command,
            source,
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers — private
// ---------------------------------------------------------------------------

fn match_deprecated<'a>(
    missing: &MissingImport,
    deprecated: &'a [DeprecatedImport],
) -> Option<&'a DeprecatedImport> {
    let ecosystem = missing.language.ecosystem();
    for dep in deprecated {
        if dep.ecosystem != ecosystem {
            continue;
        }
        if name_matches_deprecated(&missing.name, &dep.deprecated_name) {
            return Some(dep);
        }
    }
    None
}

/// Mirrors `plugins::import_extractor::matches_deprecated` (private there).
/// Local copy keeps E-6 free of plugins-internal surface changes.
fn name_matches_deprecated(extracted: &str, deprecated_name: &str) -> bool {
    if extracted == deprecated_name {
        return true;
    }
    let first_token = deprecated_name
        .split_whitespace()
        .next()
        .unwrap_or(deprecated_name);
    let first_segment = first_token.split('.').next().unwrap_or(first_token);
    extracted == first_token || extracted == first_segment
}

fn registry_url_for(language: ImportLanguage, name: &str) -> String {
    match language {
        ImportLanguage::Rust => format!("{CRATES_IO_BASE}/{name}"),
        ImportLanguage::Python => format!("{PYPI_BASE}/{name}/json"),
        ImportLanguage::TypeScript => format!("{NPM_BASE}/{name}/latest"),
        ImportLanguage::Go => format!("{GO_PROXY_BASE}/{name}/@latest"),
        // Maven / NuGet / Bundler — no first-class probe in E-6. This URL
        // is RESERVED for diagnostic/observability output only; the
        // pipeline short-circuits these languages to
        // BlockedReason::UnsupportedEcosystem BEFORE calling
        // `self.http.probe(...)`. Reqwest would otherwise reject the
        // non-http(s) scheme with HttpProbeError::Transport, aborting
        // the entire batch.
        ImportLanguage::Java | ImportLanguage::CSharp | ImportLanguage::Ruby => {
            format!("local://unsupported-ecosystem/{name}")
        }
    }
}

fn registry_source_for(language: ImportLanguage) -> ProposalSource {
    match language {
        ImportLanguage::Rust => ProposalSource::CratesIo,
        ImportLanguage::Python => ProposalSource::PyPi,
        ImportLanguage::TypeScript => ProposalSource::Npm,
        ImportLanguage::Go => ProposalSource::GoProxy,
        ImportLanguage::Java | ImportLanguage::CSharp | ImportLanguage::Ruby => {
            ProposalSource::Local
        }
    }
}

fn install_command_for(language: ImportLanguage, name: &str) -> String {
    match language {
        ImportLanguage::Rust => format!("cargo add {name}"),
        ImportLanguage::Python => format!("pip install {name}"),
        ImportLanguage::TypeScript => format!("npm install {name}"),
        ImportLanguage::Go => format!("go get {name}"),
        ImportLanguage::Java => format!("mvn install:install-file -Dfile={name}"),
        ImportLanguage::CSharp => format!("dotnet add package {name}"),
        ImportLanguage::Ruby => format!("bundle add {name}"),
    }
}

// ---------------------------------------------------------------------------
// Reader-side aggregator — parallels mode_distribution_for_payloads +
// pre_assembly_stats_for_payloads.
// ---------------------------------------------------------------------------

/// Aggregated counters derived from a slice of SSB payloads. E-6 contract:
/// the function exists and accepts `&[SsbPayload]`; the
/// `SsbPayload::DependencyResolved` variant does NOT exist yet (ssb.rs
/// LOCKED), so the aggregator returns
/// [`DependencyResolutionStats::default`] for any input. The locked API
/// surface means the SSB-bump sub-phase only needs to plug in a single
/// match arm.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DependencyResolutionStats {
    /// B-28 WIRING-1: total number of decisions observed.
    pub total_proposed: u64,
    pub total_auto_approved: u64,
    /// Mirror of `total_queued_for_human` under the WIRING-1 wire-form name
    /// `"queue_for_human"`.
    pub total_queued: u64,
    /// Legacy alias retained for E-6 callers; kept in lockstep with
    /// `total_queued` (auditor invariant).
    pub total_queued_for_human: u64,
    pub total_blocked: u64,
    /// B-28 WIRING-1: per-language counts (`ImportLanguage::wire_str` keys).
    /// BTreeMap for deterministic order (auditor invariant + dashboard JSON
    /// stability).
    pub per_language: std::collections::BTreeMap<String, u32>,
}

/// Aggregate a slice of [`SsbPayload`] into [`DependencyResolutionStats`].
///
/// B-28 WIRING-1: scans `SsbPayload::DependencyResolved` rows and produces
/// per-decision counters + per-language buckets. Any other payload
/// variants are skipped, so callers may pass an unfiltered SSB stream.
#[must_use]
pub fn dependency_resolution_stats_for_payloads(
    payloads: &[SsbPayload],
) -> DependencyResolutionStats {
    let mut stats = DependencyResolutionStats::default();
    for payload in payloads {
        if let SsbPayload::DependencyResolved(p) = payload {
            stats.total_proposed = stats.total_proposed.saturating_add(1);
            match p.decision.as_str() {
                "auto_approve" => {
                    stats.total_auto_approved = stats.total_auto_approved.saturating_add(1);
                }
                "queue_for_human" => {
                    stats.total_queued = stats.total_queued.saturating_add(1);
                    stats.total_queued_for_human =
                        stats.total_queued_for_human.saturating_add(1);
                }
                "blocked" => {
                    stats.total_blocked = stats.total_blocked.saturating_add(1);
                }
                _ => {}
            }
            let lang_entry = stats.per_language.entry(p.language.clone()).or_insert(0);
            *lang_entry = lang_entry.saturating_add(1);
        }
    }
    stats
}

// ---------------------------------------------------------------------------
// Inline unit tests — keep small; the wide contract suite lives in
// crates/contracts/tests/focus_tunnel_e6_contract.rs.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::focus_tunnel::pre_assembly::SystemTimeClock;
    use std::sync::OnceLock;

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    /// Acquire the env lock and clear `ELAI_AIR_GAPPED`. The contract
    /// suite has its own copy of this lock — inline unit tests run in a
    /// SEPARATE process (the orchestrator's lib-test binary) so the two
    /// lock instances do not interact, but the inline tests still need
    /// to serialise among themselves.
    fn env_guard() -> std::sync::MutexGuard<'static, ()> {
        let g = env_lock().lock().expect("env lock");
        std::env::remove_var(AIR_GAPPED_ENV);
        g
    }

    #[derive(Debug)]
    struct ZeroProbe;
    impl HttpProbe for ZeroProbe {
        fn probe<'a>(
            &'a self,
            _url: String,
        ) -> Pin<Box<dyn Future<Output = Result<u16, HttpProbeError>> + Send + 'a>> {
            Box::pin(async { Ok(200) })
        }
    }

    fn build_resolver(allowlist: Allowlist, autonomous: bool) -> DependencyResolver {
        DependencyResolver::new(
            allowlist,
            autonomous,
            Arc::new(SystemTimeClock),
            Arc::new(ZeroProbe),
        )
    }

    #[test]
    fn allowlist_round_trips_per_ecosystem() {
        let mut al = Allowlist::default();
        al.insert_many("cargo", vec!["tokio".to_owned(), "serde".to_owned()]);
        assert!(al.contains("cargo", "tokio"));
        assert!(al.contains("cargo", "serde"));
        assert!(!al.contains("pip", "tokio"));
    }

    #[test]
    fn install_command_per_language() {
        assert_eq!(
            install_command_for(ImportLanguage::Rust, "tokio"),
            "cargo add tokio"
        );
        assert_eq!(
            install_command_for(ImportLanguage::Python, "fastapi"),
            "pip install fastapi"
        );
        assert_eq!(
            install_command_for(ImportLanguage::TypeScript, "svelte"),
            "npm install svelte"
        );
        assert_eq!(
            install_command_for(ImportLanguage::Go, "rsc.io/quote"),
            "go get rsc.io/quote",
        );
    }

    #[test]
    fn registry_url_per_language() {
        assert!(registry_url_for(ImportLanguage::Rust, "tokio")
            .starts_with(CRATES_IO_BASE));
        assert!(registry_url_for(ImportLanguage::Python, "fastapi")
            .starts_with(PYPI_BASE));
        assert!(registry_url_for(ImportLanguage::TypeScript, "svelte")
            .starts_with(NPM_BASE));
        assert!(registry_url_for(ImportLanguage::Go, "rsc.io/quote")
            .starts_with(GO_PROXY_BASE));
    }

    #[test]
    fn name_matches_deprecated_handles_space_suffix() {
        assert!(name_matches_deprecated("rand", "rand 0.8"));
        assert!(!name_matches_deprecated("random", "rand 0.8"));
    }

    #[test]
    fn stats_for_empty_returns_default() {
        let stats = dependency_resolution_stats_for_payloads(&[]);
        assert_eq!(stats, DependencyResolutionStats::default());
    }

    #[tokio::test]
    async fn ort_blocks_even_in_autonomous() {
        let _g = env_guard();
        let resolver = build_resolver(Allowlist::default(), true);
        let plan = resolver
            .resolve(
                vec![MissingImport {
                    language: ImportLanguage::Rust,
                    name: "ort".to_owned(),
                    source_span: None,
                    transitive_of: None,
                }],
                &[],
            )
            .await
            .expect("resolve ok");
        assert!(matches!(
            plan.decisions[0],
            ResolutionDecision::Blocked {
                reason: BlockedReason::AntiStaleWaiverPending { .. },
                ..
            }
        ));
    }

    #[tokio::test]
    async fn empty_input_returns_empty_plan() {
        let _g = env_guard();
        let resolver = build_resolver(Allowlist::default(), false);
        let plan = resolver.resolve(vec![], &[]).await.expect("resolve ok");
        assert!(plan.decisions.is_empty());
        assert!(plan.allowlist_hits.is_empty());
        assert!(!plan.autonomous_used);
    }
}
