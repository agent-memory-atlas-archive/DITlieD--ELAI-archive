//! D-7 held-out canary corpus contract — on-disk seed corpus is schema-valid
//! and meets the per-family + total floor the canary draws against.
//!
//! This is the off-agent artifact for the D-7-heldout seeding unit. The
//! load-bearing proof is NOT the agent's own stdout: the test reads the real
//! JSONL files on disk under `.elai_cc/held-out/2026-05/` and deserializes
//! every row's `source` and `expected_outcome` sub-objects into the REAL
//! orchestrator [`TaskSource`] / [`ExpectedOutcome`] enums. If any authored
//! fixture row drifts from the actual Rust serde shape (wrong `source_kind`
//! tag, wrong `expected_outcome.kind`, missing required variant field), the
//! deserialize fails and the test fails. The corpus-size + per-family floors
//! are asserted against the parsed rows, so a regression that empties or
//! shrinks the corpus is a hard FAIL (R1 missing-evidence-is-FAIL).
//!
//! Grounding:
//!   - crates/orchestrator/src/held_out.rs (TaskSource 5-variant closed enum;
//!     ExpectedOutcome code/math/prose; HeldOutTask JSONL envelope. The
//!     fixture files intentionally omit task_id/content_blake3/ngram_signature
//!     — those are computed at ingest by `held_out::build_task`, per
//!     held-out/2026-05/README.md).
//!   - workflow/reasoning/D-7-decisions.md DECISION-4 (ToolUse, NOT KnowledgeQa).
//!   - docs/thefinalglance/D-plan/D-7_cross_domain_canary.md ACCEPTANCE
//!     "50 fixture tasks across 5 families seeded in D-6 store".

use std::fs;
use std::path::{Path, PathBuf};

use elai_orchestrator::held_out::{ExpectedOutcome, TaskSource};
use serde::Deserialize;

/// The on-disk fixture envelope. Mirrors the serialized [`HeldOutTask`] form
/// MINUS the three derived fields (`task_id`, `content_blake3`,
/// `ngram_signature`) which `held_out::build_task` computes at ingest. The
/// `source` and `expected_outcome` fields are typed against the REAL
/// orchestrator enums, so a parse here is a structural validation of the row
/// against the production schema, not a loose JSON sniff.
#[derive(Debug, Deserialize)]
struct SeedRow {
    source: TaskSource,
    body: String,
    expected_outcome: ExpectedOutcome,
    ingested_ts_ms: i64,
    eligibility_window_ms: i64,
}

/// The 5 canonical seed family files. The directory layout is one file per
/// [`TaskSource`] variant; the file list is the source-of-truth for the 5
/// families this corpus carries.
const FAMILY_FILES: [&str; 5] = [
    "seed_synthetic.jsonl",
    "seed_benchmark.jsonl",
    "seed_github_issue.jsonl",
    "seed_arxiv_repro.jsonl",
    "seed_human_curated.jsonl",
];

/// Minimum rows per family file (plan: ">=10 tasks per family").
const MIN_PER_FAMILY: usize = 10;

/// Minimum total corpus size across all families (plan ACCEPTANCE:
/// "50 fixture tasks across 5 families").
const MIN_TOTAL: usize = 50;

fn corpus_dir() -> PathBuf {
    // CARGO_MANIFEST_DIR == .elai_cc/crates/contracts ; up two levels to
    // the .elai_cc root, then into held-out/2026-05.
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates dir")
        .parent()
        .expect(".elai_cc root")
        .join("held-out")
        .join("2026-05")
}

/// Parse one family file into typed rows. Every non-blank line MUST
/// deserialize into a [`SeedRow`] (and therefore into the real
/// [`TaskSource`] + [`ExpectedOutcome`] enums) or the test fails with the
/// offending file + line.
fn parse_family_file(path: &Path) -> Vec<SeedRow> {
    let text = fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("seed corpus file not found: {} ({e})", path.display()));
    let mut rows = Vec::new();
    for (i, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let row: SeedRow = serde_json::from_str(line).unwrap_or_else(|e| {
            panic!(
                "schema-invalid held-out row at {}:{}: {e}\n  line: {line}",
                path.display(),
                i + 1
            )
        });
        rows.push(row);
    }
    rows
}

// --- (a) Every family file parses against the real schema + meets the floor. ---

#[test]
fn each_family_file_parses_and_meets_min_per_family() {
    let dir = corpus_dir();
    for fname in FAMILY_FILES {
        let path = dir.join(fname);
        let rows = parse_family_file(&path);
        assert!(
            rows.len() >= MIN_PER_FAMILY,
            "family file {} has {} rows, below the per-family floor of {}",
            fname,
            rows.len(),
            MIN_PER_FAMILY
        );
        // Each row's invariants: non-empty body + non-negative timestamps.
        for (i, r) in rows.iter().enumerate() {
            assert!(
                !r.body.trim().is_empty(),
                "{}:{} has an empty body",
                fname,
                i + 1
            );
            assert!(
                r.ingested_ts_ms > 0,
                "{}:{} ingested_ts_ms must be a positive UNIX-ms",
                fname,
                i + 1
            );
            assert!(
                r.eligibility_window_ms >= 0,
                "{}:{} eligibility_window_ms must be non-negative",
                fname,
                i + 1
            );
        }
    }
}

// --- (b) The corpus carries all 5 distinct TaskSource families. ---

#[test]
fn corpus_covers_five_distinct_task_source_families() {
    let dir = corpus_dir();
    let mut seen_labels = std::collections::BTreeSet::new();
    for fname in FAMILY_FILES {
        let rows = parse_family_file(&dir.join(fname));
        for r in &rows {
            seen_labels.insert(r.source.label());
        }
    }
    // The 5 canonical TaskSource labels (held_out.rs::TaskSource::label).
    let expected: std::collections::BTreeSet<&str> = [
        "synthetic",
        "benchmark_fixture",
        "github_issue",
        "arxiv_repro",
        "human_curated",
    ]
    .into_iter()
    .collect();
    assert_eq!(
        seen_labels, expected,
        "corpus must span exactly the 5 canonical TaskSource families"
    );
}

// --- (c) The total corpus meets the >=50 ACCEPTANCE floor. ---

#[test]
fn corpus_total_meets_fifty_task_floor() {
    let dir = corpus_dir();
    let total: usize = FAMILY_FILES
        .iter()
        .map(|f| parse_family_file(&dir.join(f)).len())
        .sum();
    assert!(
        total >= MIN_TOTAL,
        "held-out canary corpus has {total} tasks, below the D-7 ACCEPTANCE floor of {MIN_TOTAL}"
    );
}

// --- (d) Outcomes span all three ExpectedOutcome kinds (cross-domain). ---
//
// D-7's point is a CROSS-domain corpus: a corpus that is all `code` would not
// exercise the math-proof / prose-reason / planning / tool-use canary axes.
// The CanaryFamily axis (canary.rs) maps onto Domain {Code, Math, Prose,
// Planning, ToolUse}; the held-out envelope's ExpectedOutcome carries the
// code/math/prose discriminant. Assert all three are present so the corpus is
// genuinely cross-domain, not single-axis.

#[test]
fn corpus_spans_all_expected_outcome_kinds() {
    let dir = corpus_dir();
    let mut has_code = false;
    let mut has_math = false;
    let mut has_prose = false;
    for fname in FAMILY_FILES {
        for r in parse_family_file(&dir.join(fname)) {
            match r.expected_outcome {
                ExpectedOutcome::Code(_) => has_code = true,
                ExpectedOutcome::Math(_) => has_math = true,
                ExpectedOutcome::Prose(_) => has_prose = true,
            }
        }
    }
    assert!(
        has_code && has_math && has_prose,
        "cross-domain corpus must carry code AND math AND prose outcomes \
         (code={has_code}, math={has_math}, prose={has_prose})"
    );
}
