//! N-8 SLICE 1 — governance-coverage substrate + fail-closed arms.
//!
//! Joins five governance views over their REAL upstream seams (N-8 seam
//! proposal, BUILD-WITH-SEAM @ base b9efaf5e):
//!
//! * V1 GateLiveness      — consumes shipped B-17 surfaces read-only:
//!                          `.elai_cc/crates/orchestrator/tests/gate_liveness.rs`
//!                          (counts `#[test]` fns minus `#[ignore]` stubs) +
//!                          presence probe `.elai_cc/scripts/check_gate_liveness.sh`.
//! * V2 DoctrineEnforcer  — K-22 upstream is DARK on b9efaf5e (no
//!                          doctrine_coverage.py anywhere) => hard fail-closed
//!                          Unmeasured arm citing probed paths.
//! * V3 DebtRate          — J-46 upstream is DARK on b9efaf5e (no
//!                          doctrine_debt_rate.py / ledger) => fail-closed arm.
//! * V4 Traceability      — join-only parse of `<repo-root>/.claude/
//!                          gate-manifest.txt` DO-333 rows (5 tab-separated
//!                          fields, '#'-prefixed lines skipped). No second
//!                          parser elsewhere; the manifest is read-only here.
//! * V5 MutantKill        — delegates to the shipped H-27 producer
//!                          [`crate::gate_self_validation::validate_gates`];
//!                          no new validator, no re-scoring. `run_coverage`
//!                          reports it UNMEASURED ("not yet scored") unless a
//!                          caller drives the async measurement explicitly —
//!                          never a synthetic number.
//!
//! FAIL-CLOSED CONTRACT (plan NEGATIVE observable effect): inventing coverage
//! must be impossible. Every [`ViewStatus::Measured`] carries concrete values
//! derived from files that were actually read/probed in this process; every
//! gap cites its absence path instead of estimating a number. A decorative
//! rule reported covered is a bug.
//!
//! Slice plan position: slice 1 shipped the substrate TYPES + checker core;
//! N-8 SLICE 2 landed the production emitters — the `GovernanceCoverageMeasured`
//! SSB action (lockstep-registered across enum / as_sql_str / from_str /
//! SSB_ACTION_TYPES / SQL CHECK / SsbPayload+action_type() / test_payload) and
//! the `elai governance-coverage` CLI dispatch beside `CliAction::GateSelfcheck`
//! (`crates/cli/src/governance_coverage_cmd.rs`, wired for BOTH run_coverage and
//! run_coverage_with_mutant_kill). scripts/ and gate files stay untouched here.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

// ---------------------------------------------------------------------------
// Substrate types

/// The five governance views joined by this checker (seam proposal §"The
/// five views"; V1/V4/V5 live legs, V2/V3 mandated fail-closed arms).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum GovernanceView {
    /// V1 — B-17 gate-liveness observation.
    GateLiveness,
    /// V2 — K-22 doctrine-enforcer coverage (dark upstream; fail-closed).
    DoctrineEnforcer,
    /// V3 — J-46 doctrine-debt-rate measurement (dark upstream; fail-closed).
    DebtRate,
    /// V4 — J-70 DO-333 traceability join over the frozen manifest.
    Traceability,
    /// V5 — H-27 mutant-kill over the frozen gate registry (delegated).
    MutantKill,
}

/// Concrete, provenance-backed payload of a genuinely measured view.
///
/// Constructing any variant requires numbers that came from actually reading
/// or probing a real path during this run — there is no way to express a bare
/// percentage without its source artifact. Do NOT add numeric-only variants.
#[derive(Debug, Clone, PartialEq)]
pub enum ViewValue {
    /// V1: active (non-`#[ignore]`) wired tests of the B-17 inventory, plus
    /// the excluded stub names so liveness accounting stays auditable.
    GateLiveness {
        /// Number of `#[test]` fns not marked `#[ignore]`.
        active_wired_tests: usize,
        /// Number of `#[test]` fns marked `#[ignore]` (wiring stubs).
        ignored_stub_tests: usize,
        /// Names of the ignored stub fns (at least the first few kept).
        ignored_stub_names: Vec<String>,
        /// Whether `.elai_cc/scripts/check_gate_liveness.sh` was present.
        liveness_script_present: bool,
        /// Absolute-ish provenance of the parsed test file.
        test_file: String,
    },
    /// V4: one entry per manifest row whose five required fields parse
    /// non-empty; rejected rows are counted, never silently dropped.
    Traceability {
        /// Valid DO-333 rows (all 5 fields non-empty).
        rows_valid: usize,
        /// Rows skipped as malformed/empty-fielded (visible residue, not hidden).
        rows_rejected: usize,
        /// Provenance of the parsed manifest.
        manifest_path: String,
        /// Up to [`SAMPLE_ROWS_CAP`] verbatim rows for audit sampling.
        sample_rows: Vec<TraceabilityRow>,
    },
    /// V5: aggregate kill figures copied VERBATIM from the H-27 outcome
    /// ([`crate::gate_self_validation::GateSelfValidationOutcome`]). The
    /// shipped public API exposes registry aggregates plus the first escapee,
    /// not a per-gate vector; per-gate detail lives in the H-27 SSB rows that
    /// the driver emits as a side effect and is intentionally NOT rebuilt here.
    MutantKill {
        /// Gates in the frozen H-27 registry (the should-set).
        gates_total: u32,
        /// Gates that BLOCKED their planted mutant (killed it).
        gates_killed: u32,
        /// `gates_killed / gates_total * 100` verbatim (0.0 for empty set).
        mutation_score_pct: f64,
        /// First gate that failed the meta-gate (survivor / over-block),
        /// empty on AllGatesValid.
        escaped_gate: String,
        /// Verbatim [`crate::gate_self_validation::GateSelfValidationOutcome::is_pass`].
        is_pass: bool,
        /// Producer identity for provenance.
        source: String,
    },
}

/// One parsed DO-333 traceability row (frozen manifest schema, read-only join).
#[derive(Debug, Clone, PartialEq)]
pub struct TraceabilityRow {
    /// Column 1 — gated file path.
    pub path: String,
    /// Column 2 — covered objective id/text.
    pub objective_covered: String,
    /// Column 3 — verification technique.
    pub technique: String,
    /// Column 4 — evidence artifact.
    pub evidence_artifact: String,
    /// Column 5 — satisfaction verdict string.
    pub satisfaction: String,
}

/// Status of one governance view. Everything not genuinely measured in this
/// run is [`ViewStatus::Unmeasured`] with the absence cited — there is no
/// estimated / partial / assumed state.
#[derive(Debug, Clone, PartialEq)]
pub enum ViewStatus {
    /// Genuinely measured this run; payload guarantees concrete sources.
    Measured(ViewValue),
    /// Not measurable now; `reason` cites the absence path(s) verbatim.
    Unmeasured {
        /// Machine-readable absence citation (paths, not estimates).
        reason: String,
    },
}

/// Result of [`run_coverage`]: one fixed-order view -> status pair per view.
#[derive(Debug, Clone, PartialEq)]
pub struct GovernanceCoverageReport {
    /// Declared order == [`GovernanceView`] declaration order, deterministic.
    views: Vec<(GovernanceView, ViewStatus)>,
}

impl GovernanceCoverageReport {
    /// Borrow the deterministic ordered view table.
    pub fn views(&self) -> &[(GovernanceView, ViewStatus)] {
        &self.views
    }

    /// Lookup one view's status.
    ///
    /// Fully private (N-8 slice 2): the production CLI iterates
    /// [`GovernanceCoverageReport::views`] instead, so a `pub`/`pub(crate)`
    /// accessor here would be a DARK symbol under R-8/D-030. In-crate it backs
    /// [`Self::set_view`] as the fixed-five-view existence probe.
    fn status_of(&self, view: GovernanceView) -> Option<&ViewStatus> {
        self.views
            .iter()
            .find_map(|(v, s)| (*v == view).then_some(s))
    }

    /// Internal mutator for the post-hoc mutant-kill leg of
    /// [`run_coverage_with_mutant_kill`].
    fn set_view(&mut self, view: GovernanceView, status: ViewStatus) {
        // Only REPLACE an existing declared slot: the five-view table is fixed,
        // never appended to. status_of doubles as that existence probe so a
        // typo'd/misdeclared view fails loudly instead of silently dropping the
        // substituted measurement.
        debug_assert!(
            self.status_of(view).is_some(),
            "set_view({view:?}) targeted a view outside the declared five-view table"
        );
        if let Some(slot) = self
            .views
            .iter_mut()
            .find_map(|(v, s)| (*v == view).then_some(s))
        {
            *slot = status;
        }
    }
}

// ---------------------------------------------------------------------------
// Seam paths (single source of truth; no scattered literals)

const LIVENESS_TEST_REL: &str = ".elai_cc/crates/orchestrator/tests/gate_liveness.rs";
const LIVENESS_SCRIPT_REL: &str = ".elai_cc/scripts/check_gate_liveness.sh";
const MANIFEST_REL: &str = ".claude/gate-manifest.txt";

fn liveness_test_path(repo_root: &Path) -> PathBuf {
    repo_root.join(LIVENESS_TEST_REL)
}

fn liveness_script_path(repo_root: &Path) -> PathBuf {
    repo_root.join(LIVENESS_SCRIPT_REL)
}

fn manifest_path(repo_root: &Path) -> PathBuf {
    repo_root.join(MANIFEST_REL)
}

/// Max sample rows embedded into a [`ViewValue::Traceability`] payload.
const SAMPLE_ROWS_CAP: usize = 5;

// ---------------------------------------------------------------------------
// V1 — B-17 gate-liveness (structural count; RESPECT: never edit the file)

#[derive(Debug, Default, PartialEq)]
struct TestFnScan {
    active_names: Vec<String>,
    ignored_names: Vec<String>,
}

/// Name extraction copied verbatim from the original inline chain
/// (behavioral parity with the shipped scanner, quirks included).
fn extract_test_name(t: &str) -> String {
    t.trim_start_matches(['p', 'u', 'b', ' ', 'a', 's', 'y', 'n', 'c'])
        .strip_prefix("fn ")
        .unwrap_or("")
        .split(['(', ' ', '<'])
        .next()
        .unwrap_or("")
        .to_string()
}

/// Flush one terminated attribute group: a TEST-marked group attached to an
/// `fn` definition line contributes exactly one name, routed to its bucket by
/// an `#[ignore]` lookup — the two buckets index-select, no branch ladder.
fn flush_fn_group(out: &mut TestFnScan, attrs: &[String], t: &str) {
    let is_fn_def =
        t.starts_with("fn ") || t.starts_with("pub fn ") || t.starts_with("async fn ");
    let is_wired = is_fn_def && attrs.iter().any(|a| a.starts_with("#[test"));
    let ignored = attrs.iter().any(|a| a.starts_with("#[ignore"));
    let name = extract_test_name(t);
    let mut buckets = [&mut out.active_names, &mut out.ignored_names];
    if is_wired && !name.is_empty() {
        buckets[usize::from(ignored)].push(name);
    }
}

fn scan_test_fns(src: &str) -> TestFnScan {
    let mut out = TestFnScan::default();
    // Pending attribute lines (trimmed) that precede the next item token.
    let mut attrs: Vec<String> = Vec::new();
    for raw in src.lines() {
        let t = raw.trim();
        // Comments/blanks are TRANSPARENT (attribute groups commonly carry
        // interleaved TODO/doc comments, so they neither push nor clear);
        // `#[..]` accumulates; any other token terminates and flushes the
        // pending group.
        match (t.is_empty() || t.starts_with("//"), t.starts_with("#[")) {
            (true, _) => {}
            (false, true) => attrs.push(t.to_string()),
            (false, false) => {
                flush_fn_group(&mut out, &attrs, t);
                attrs.clear();
            }
        }
    }
    out
}

/// V1 measurement: structural count of ACTIVE wired tests in the B-17
/// inventory (read-only), guarded by the presence of the CI liveness script.
pub fn measure_gate_liveness(repo_root: &Path) -> ViewStatus {
    let test_file = liveness_test_path(repo_root);
    let Ok(src) = fs::read_to_string(&test_file) else {
        return ViewStatus::Unmeasured {
            reason: format!(
                "gate_liveness.rs test inventory unreadable at {}; liveness cannot be observed",
                test_file.display()
            ),
        };
    };
    let scan = scan_test_fns(&src);
    if scan.active_names.is_empty() {
        return ViewStatus::Unmeasured {
            reason: format!(
                "parsed 0 active (non-#[ignore]) wired tests at {}; refusing to fabricate a pass count",
                test_file.display()
            ),
        };
    }
    let script = liveness_script_path(repo_root);
    let script_present = script.is_file();
    if !script_present {
        return ViewStatus::Unmeasured {
            reason: format!(
                "{} active wired tests parsed but CI runner script {} is absent; \
                 liveness is not observable without its runner",
                scan.active_names.len(),
                script.display()
            ),
        };
    }
    ViewStatus::Measured(ViewValue::GateLiveness {
        active_wired_tests: scan.active_names.len(),
        ignored_stub_tests: scan.ignored_names.len(),
        // Cap the stub-name payload; the counts above remain the authority.
        ignored_stub_names: scan.ignored_names.into_iter().take(16).collect(),
        liveness_script_present: true,
        test_file: test_file.display().to_string(),
    })
}

// ---------------------------------------------------------------------------
// V4 — J-70 traceability join (parse-only; the manifest is frozen elsewhere)

/// Verdict for one manifest line under the frozen DO-333 join contract.
enum RowVerdict {
    /// Blank or '#'-comment line: transparent — neither valid nor rejected.
    Skip,
    /// A non-comment line failing the 5x-non-empty-field contract.
    Reject,
    Accept(TraceabilityRow),
}

/// Classify one raw manifest line. '#' lines and blanks SKIP; one entry per
/// remaining line whose five TAB-separated fields are all non-empty.
fn classify_manifest_row(line: &str) -> RowVerdict {
    let trimmed = line.trim_start();
    let skippable = trimmed.is_empty() || trimmed.starts_with('#');
    let fields: Vec<&str> = line.split('\t').map(str::trim).collect();
    let row_ok = fields.len() == 5 && fields.iter().all(|f| !f.is_empty());
    // Precedence: accept > reject > skip, resolved without a branch ladder.
    (!skippable && row_ok)
        .then(|| {
            RowVerdict::Accept(TraceabilityRow {
                path: fields[0].to_string(),
                objective_covered: fields[1].to_string(),
                technique: fields[2].to_string(),
                evidence_artifact: fields[3].to_string(),
                satisfaction: fields[4].to_string(),
            })
        })
        .or((!skippable && !row_ok).then_some(RowVerdict::Reject))
        .unwrap_or(RowVerdict::Skip)
}

/// Parse/join the frozen manifest's DO-333 rows: '#' lines and blanks skipped;
/// one entry per line whose five TAB-separated fields are all non-empty. This
/// is a JOIN of rows already validated upstream by `gate_freeze_check.py
/// --trace` — validation semantics beyond the 5x-non-empty field contract
/// belong to that tool and are deliberately not duplicated here.
pub fn measure_traceability(repo_root: &Path) -> ViewStatus {
    let manifest = manifest_path(repo_root);
    let Ok(text) = fs::read_to_string(&manifest) else {
        return ViewStatus::Unmeasured {
            reason: format!(
                "no gate-manifest.txt readable at {}; traceability join has no frozen rows to join",
                manifest.display()
            ),
        };
    };
    let mut rows_valid = 0usize;
    let mut rows_rejected = 0usize;
    let mut sample_rows: Vec<TraceabilityRow> = Vec::new();
    for line in text.lines() {
        match classify_manifest_row(line) {
            RowVerdict::Skip => {}
            RowVerdict::Reject => rows_rejected += 1,
            RowVerdict::Accept(row) => {
                rows_valid += 1;
                if sample_rows.len() < SAMPLE_ROWS_CAP {
                    sample_rows.push(row);
                }
            }
        }
    }
    // Zero valid rows => honest absence citation, never an empty "join".
    let joined = (rows_valid > 0)
        .then(|| {
            ViewStatus::Measured(ViewValue::Traceability {
                rows_valid,
                rows_rejected,
                manifest_path: manifest.display().to_string(),
                sample_rows,
            })
        })
        .unwrap_or_else(|| {
            ViewStatus::Unmeasured {
                reason: format!(
                    "manifest at {} yielded 0 valid 5-field DO-333 rows ({} rejected); \
                     no traceability entry can honestly exist",
                    manifest.display(),
                    rows_rejected
                ),
            }
        });
    joined
}

// ---------------------------------------------------------------------------
// V2/V3 — K-22 doctrine-enforcer + J-46 debt-rate (hard fail-closed arms)

/// Reason tail describing how far a bounded sweep actually got; the two
/// not-found arms differ only by this suffix.
const SWEEP_SUFFIX_CLEAN: &str = " (bounded sweep depth<=3 clean)";

/// See [`SWEEP_SUFFIX_CLEAN`].
const SWEEP_SUFFIX_TRUNCATED: &str = "; bounded sweep truncated at entry cap (depth<=3)";

/// Index-select the reason tail so both dark-view not-found reasons stay
/// textually identical apart from the truncation caveat.
fn sweep_suffix(truncated: bool) -> String {
    // Arrays lack swap_remove; plain index-selection preserves the same
    // per-call string value (the array is rebuilt fresh on every call).
    let suffixes = [SWEEP_SUFFIX_CLEAN.to_string(), SWEEP_SUFFIX_TRUNCATED.to_string()];
    suffixes[usize::from(truncated)].clone()
}

/// Candidate paths probed outright before the bounded walk (these are the
/// conventional landing spots if/when K-22/J-46 ship).
fn doctrine_candidate_paths(repo_root: &Path) -> Vec<PathBuf> {
    vec![
        repo_root.join(".claude/doctrine_coverage.py"),
        repo_root.join(".claude/tools/doctrine_coverage.py"),
        repo_root.join(".elai_cc/scripts/doctrine_coverage.py"),
        repo_root.join(".elai_cc/doctrine_coverage.py"),
    ]
}

/// Ledger-side candidates for J-46 (producer + ledger spots).
fn debt_candidate_paths(repo_root: &Path) -> Vec<PathBuf> {
    vec![
        repo_root.join(".claude/doctrine_debt_rate.py"),
        repo_root.join(".claude/tools/doctrine_debt_rate.py"),
        repo_root.join(".elai_cc/scripts/doctrine_debt_rate.py"),
    ]
}

/// Shared fail-closed runner for BOTH dark views (V2 K-22 / V3 J-46): probe
/// candidate paths outright, then a bounded name sweep, then cite absence —
/// with the sweep-truncation caveat when the entry cap cut the walk short.
/// It NEVER returns Measured: no scoring contract is wired behind either arm,
/// so even an artifact found mid-flight cannot contribute coverage.
///
/// Parameterized by the per-arm reason templates so V2/V3 cannot drift apart.
fn probe_dark_arm(
    repo_root: &Path,
    direct_candidates: &[PathBuf],
    walk_predicate: &dyn Fn(&str) -> bool,
    found_direct: &dyn Fn(&Path) -> String,
    found_sweep: &dyn Fn(&Path) -> String,
    not_found_lead: &str,
) -> ViewStatus {
    let cited = display_list(direct_candidates);
    direct_candidates.iter().find(|p| p.is_file()).map_or_else(
        || {
            let (walk_hits, walk_truncated) = bounded_probe_walk(repo_root, walk_predicate);
            walk_hits.first().map_or_else(
                || {
                    let reason = format!("{not_found_lead}{cited}{}", sweep_suffix(walk_truncated));
                    ViewStatus::Unmeasured { reason }
                },
                |found| {
                    let reason = (found_sweep)(found.as_path());
                    ViewStatus::Unmeasured { reason }
                },
            )
        },
        |found| {
            let reason = (found_direct)(found.as_path());
            ViewStatus::Unmeasured { reason }
        },
    )
}

/// V2 arm. K-22 has NO shipped enforcer on base b9efaf5e; until one exists,
/// a clause/policy string can never contribute coverage. Even if a future
/// `doctrine_coverage.py` appears mid-flight WITHOUT this module's scoring
/// contract being wired, we still refuse to invent a number.
pub fn measure_doctrine_enforcer(repo_root: &Path) -> ViewStatus {
    let candidates = doctrine_candidate_paths(repo_root);
    probe_dark_arm(
        repo_root,
        &candidates,
        &|n: &str| n.contains("doctrine_coverage"),
        &|f: &Path| {
            format!(
                "doctrine_coverage.py FOUND at {} but no scoring contract is wired for its \
                 output yet (K-22 landing slot); coverage stays unmeasured, never estimated",
                f.display()
            )
        },
        &|f: &Path| {
            format!(
                "doctrine_coverage.py FOUND at {} via bounded sweep but no scoring contract \
                 is wired for its output yet (K-22 landing slot); coverage stays unmeasured",
                f.display()
            )
        },
        "no doctrine_coverage.py ledger found at <probed paths>: ",
    )
}

/// V3 arm — same discipline for J-46 (debt-rate producer + ledger absence).
pub fn measure_debt_rate(repo_root: &Path) -> ViewStatus {
    let candidates = debt_candidate_paths(repo_root);
    probe_dark_arm(
        repo_root,
        &candidates,
        &|n: &str| n.contains("doctrine_debt") || n.contains("debt_rate"),
        &|f: &Path| {
            format!(
                "doctrine_debt_rate producer FOUND at {} but no ledger/scoring contract is \
                 wired yet (J-46 landing slot); debt rate stays unmeasured, never estimated",
                f.display()
            )
        },
        &|f: &Path| {
            format!(
                "debt-rate artifact FOUND at {} via bounded sweep but no ledger/scoring \
                 contract is wired yet (J-46 landing slot); debt rate stays unmeasured",
                f.display()
            )
        },
        "no doctrine debt-rate producer/ledger found at <probed paths>: ",
    )
}

/// Bounded, dependency-free filesystem walk under `start`. Deterministic
/// (sorted entries), prunes the usual bulk/generated trees, caps visited
/// entries so the checker stays cheap even at repo root, and collects paths
/// whose FILE NAME satisfies `match_name`. Anything past the cap stops the
/// walk (the reason string says which roots were swept — fail-closed direction:
/// a missed artifact reads as Unmeasured, never as invented coverage).
fn bounded_probe_walk(start: &Path, match_name: &dyn Fn(&str) -> bool) -> (Vec<PathBuf>, bool) {
    const MAX_DEPTH: usize = 3;
    const MAX_ENTRIES: usize = 20_000;
    const PRUNE_DIRS: &[&str] = &[
        ".git",
        "target",
        "node_modules",
        "dist",
        "build",
        "rust_archived",
        ".clawd-agents",
        "browser_e2e",
        "benchmarks",
    ];

    let mut hits = Vec::new();
    let mut truncated = false;
    let mut visited = 0usize;
    let mut stack = vec![(start.to_path_buf(), 0usize)];
    while let Some((dir, depth)) = stack.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        let mut children: Vec<_> = entries.flatten().collect();
        children.sort_by_key(std::fs::DirEntry::file_name);
        for entry in children {
            visited += 1;
            if visited >= MAX_ENTRIES {
                truncated = true;
                return (hits, truncated);
            }
            let path = entry.path();
            let name = entry.file_name();
            let name = name.to_string_lossy();
            let is_dir = path.is_dir();
            // Descend bounded directories; collect matching file hits. The
            // two classifications are disjoint, so no branch ladder needed
            // (the doubled `is_dir` probe is side-effect-free and cheap).
            let descender = (is_dir && depth < MAX_DEPTH && !PRUNE_DIRS.contains(&name.as_ref()))
                .then(|| (path.clone(), depth + 1));
            stack.extend(descender);
            let hit = (!is_dir && match_name(&name)).then_some(path);
            hits.extend(hit);
        }
    }
    (hits, truncated)
}

fn display_list(paths: &[PathBuf]) -> String {
    paths
        .iter()
        .map(|p| p.display().to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

// ---------------------------------------------------------------------------
// V5 — H-27 delegation (REQUIRED reuse; zero new validator code)

/// Honest refusal for an H-27 run over an empty gate registry.
const EMPTY_REGISTRY_REASON: &str =
    "H-27 returned an EMPTY registry (gates_total=0); refusing the vacuous 100% score";

/// Delegate mutant-kill scoring to the shipped H-27 producer
/// [`crate::gate_self_validation::validate_gates`]. Callable whenever a live
/// SSB session exists; errors and empty registries degrade to Unmeasured
/// with the failure cited — NEVER to a synthetic score. The caller decides
/// whether the network/tool surface for the replay is available.
pub async fn measure_mutant_kill(
    ssb: &Arc<crate::ssb::Ssb>,
    db_path: &Path,
    run_id: &str,
    seed: u64,
    inject_weakened_gate: bool,
) -> ViewStatus {
    match crate::gate_self_validation::validate_gates(ssb, db_path, run_id, seed, inject_weakened_gate)
        .await
    {
        Ok(outcome) => (outcome.gates_total != 0)
            .then(|| {
                ViewStatus::Measured(ViewValue::MutantKill {
                    gates_total: outcome.gates_total,
                    gates_killed: outcome.gates_killed,
                    mutation_score_pct: outcome.mutation_score_pct,
                    escaped_gate: outcome.escaped_gate.clone(),
                    is_pass: outcome.is_pass(),
                    source: "elai_orchestrator::gate_self_validation::validate_gates".to_string(),
                })
            })
            .unwrap_or_else(|| ViewStatus::Unmeasured {
                reason: EMPTY_REGISTRY_REASON.to_string(),
            }),
        Err(err) => ViewStatus::Unmeasured {
            reason: format!(
                "H-27 mutant replay did not produce a verdict ({err}); mutant-kill stays \
                 unscored rather than synthetic"
            ),
        },
    }
}

// ---------------------------------------------------------------------------
// Checker core

/// Sync checker core joining all five views on the CURRENT tree. V1/V4 read
/// their real files; V2/V3 run the fail-closed probes; V5 reports the honest
/// "not yet scored" state — driving the real H-27 replay requires a live SSB
/// session, and no synthetic number may ever stand in for it. Call
/// [`run_coverage_with_mutant_kill`] to obtain a possibly-Measured V5 leg.
pub fn run_coverage(repo_root: &Path) -> GovernanceCoverageReport {
    let views = vec![
        (GovernanceView::GateLiveness, measure_gate_liveness(repo_root)),
        (
            GovernanceView::Traceability,
            measure_traceability(repo_root),
        ),
        (
            GovernanceView::MutantKill,
            ViewStatus::Unmeasured {
                reason: "not yet scored: the H-27 mutant corpus replay requires a live SSB \
                         session (run measure_mutant_kill / elai gate-selfcheck); a score \
                         without that run would be synthetic"
                    .to_string(),
            },
        ),
        (
            GovernanceView::DoctrineEnforcer,
            measure_doctrine_enforcer(repo_root),
        ),
        (GovernanceView::DebtRate, measure_debt_rate(repo_root)),
    ];
    GovernanceCoverageReport { views }
}

/// Async checker core: same joins, plus the delegated H-27 mutant-kill replay
/// substituting V5's placeholder state when it genuinely scores.
pub async fn run_coverage_with_mutant_kill(
    repo_root: &Path,
    ssb: &Arc<crate::ssb::Ssb>,
    db_path: &Path,
    run_id: &str,
    seed: u64,
    inject_weakened_gate: bool,
) -> GovernanceCoverageReport {
    let mut report = run_coverage(repo_root);
    let mutant = measure_mutant_kill(ssb, db_path, run_id, seed, inject_weakened_gate).await;
    report.set_view(GovernanceView::MutantKill, mutant);
    report
}

// ---------------------------------------------------------------------------
// In-module unit tests for the fragile pieces (scanner + red-team shapes)

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scanner_counts_active_and_ignore_marked_fns() {
        let src = "\
#[test]
fn wired_one() {}
// TODO(B-x) comment noise above the next attrs
#[test]
#[ignore = \"blocked on B-4\"]
fn stub_pending_b4_wiring() {}
#[derive(Debug)]
struct Noise;
#[test]
fn wired_two() {}
";
        let scan = scan_test_fns(src);
        assert_eq!(scan.active_names, vec!["wired_one".to_string(), "wired_two".to_string()]);
        assert_eq!(scan.ignored_names, vec!["stub_pending_b4_wiring".to_string()]);
    }

    #[test]
    fn redteam_decorative_rule_cannot_become_coverage() {
        // A clause string with no machine enforcer anywhere => Unmeasured,
        // never a score. Feed a scratch dir whose only content is policy text.
        let tmp = std::env::temp_dir().join(format!(
            "gov_cov_redteam_{}",
            std::process::id()
        ));
        fs::create_dir_all(&tmp).expect("scratch dir");
        // The red-team input: a decorative CLAUSE STRING with no machine
        // enforcer anywhere near it. It must contribute zero coverage.
        fs::write(tmp.join("decorative_policy.txt"), "RULE: all outputs grounded").ok();
        let guard = tmp.clone();
        let status = measure_doctrine_enforcer(&tmp);
        let ViewStatus::Unmeasured { reason } = &status else {
            panic!("decorative clause must fail closed, got {status:?}")
        };
        assert!(reason.contains("doctrine_coverage.py"), "reason={reason}");
        fs::remove_dir_all(guard).ok();
    }
}
