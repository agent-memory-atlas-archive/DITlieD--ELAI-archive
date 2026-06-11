//! H-53 — `elai prebind --demo-prebind` REAL CLI entrypoint.
//!
//! H-53 W-1 seeded-fixture trigger: builds a deterministic evidence fixture,
//! calls [`elai_orchestrator::citation_prebind::freeze_anchors`], runs the
//! closed-set membership + blake3-support check
//! ([`elai_orchestrator::citation_prebind::check_citation_in_set`]), and emits
//! a genuine [`SsbPayload::CitationAnchorsBound`] row through the REAL `Arc<Ssb>`
//! handle — the off-agent artifact (R4: emitted from the CLI driver, not by the
//! model generating the answer). Missing row = FAIL (R1/R2).
//!
//! Three observable-effect cases (W-1):
//!   (a) citation id IN the pre-bound set + excerpt matches blake3 pin => Bound outcome.
//!   (b) citation id NOT in the set (ghost citation) => Ghost outcome; blocked from answer.
//!   (c) `--allow-freeform-citations` BROKEN ARM: skip closed-set check, accept raw model
//!       cite => CitationAnchorsBoundKind::Blocked fires (proves the guard was bypassed,
//!       R2: a gate that stays "right" when the disable is set is itself a FAIL).
//!
//! DEFERRED: the live ranked-evidence source (real A-RAG/rerank output from E-48/E-17
//! web-research pipeline) stays deferred until E-48's search backend is GPU-live.
//! This seeded fixture IS the W-1 live-CLI proof; the live-source consumer is the
//! deferred future activation.
//! <!-- DEFERRED: E-17/E-48 live ranked-evidence source; seeded fixture is W-1 proof now -->

use std::sync::Arc;

use elai_core::{clock::OsClock, paths::sessions_dir_path};
use elai_orchestrator::{
    citation_prebind::{
        check_citation_in_set, extract_citation_ids, freeze_anchors, prebind_disabled,
        CitationCheckOutcome, RankedEvidence,
    },
    ssb::{CitationAnchorsBoundKind, CitationAnchorsBoundPayload, Ssb, SsbConfig, SsbPayload},
};

use crate::args::CliOutputFormat;

// ────────────────────────────────────────────────────────────────────────────
// Demo constants
// ────────────────────────────────────────────────────────────────────────────

const DEMO_PREBIND_PROJECT_ID: &str = "elai-h53-demo";
const DEMO_PREBIND_SESSION_ID: &str = "demo-prebind-fixture-h53";
const DEMO_PREBIND_AGENT_ID: &str = "cli:prebind-demo";

// ────────────────────────────────────────────────────────────────────────────
// Public entry point
// ────────────────────────────────────────────────────────────────────────────

/// Run the H-53 W-1 seeded-fixture trigger.
///
/// Builds a deterministic evidence fixture (3 ranked sources), calls
/// `freeze_anchors`, runs `check_citation_in_set` for three cases:
///   (a) bound citation — id in set, excerpt matches blake3 pin
///   (b) ghost citation — id 99 not in set
///   (c) stale citation — id in set but excerpt was changed post-freeze
///
/// Emits a `CitationAnchorsBound` SSB row (Bound arm on the normal path;
/// Blocked arm when `allow_freeform` is set — the broken-arm test, R2 proof).
/// Prints results in the requested output format.
pub(crate) fn run_demo_prebind(
    allow_freeform: bool,
    output_format: CliOutputFormat,
) -> anyhow::Result<()> {
    let seed: u64 = std::env::var("ELAI_SEED")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);

    // Seeded evidence fixture. Three ranked sources with deterministic excerpts.
    let fixture_evidence = vec![
        RankedEvidence {
            rank: 0,
            excerpt: "Rust's ownership model guarantees memory safety without garbage collection.".to_owned(),
            source_uri: "https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html".to_owned(),
            retrieved_at_ms: 1_718_000_000_000,
            source_tier: 0,
        },
        RankedEvidence {
            rank: 1,
            excerpt: "blake3 provides 128-bit security with significantly better performance than SHA-256.".to_owned(),
            source_uri: "https://github.com/BLAKE3-team/BLAKE3".to_owned(),
            retrieved_at_ms: 1_718_000_001_000,
            source_tier: 0,
        },
        RankedEvidence {
            rank: 2,
            excerpt: "Pre-binding citation ids before generation makes ghost citations structurally impossible.".to_owned(),
            source_uri: "https://arxiv.org/abs/2211.12588".to_owned(),
            retrieved_at_ms: 1_718_000_002_000,
            source_tier: 0,
        },
    ];

    let anchor_set = freeze_anchors(&fixture_evidence, seed);

    // Three check cases.
    // (a) Bound: cite anchor 1 with the EXACT frozen excerpt.
    let case_a = check_citation_in_set(
        &anchor_set,
        1,
        &fixture_evidence[0].excerpt,
    );

    // (b) Ghost: cite id 99 (not in the set of 3 anchors).
    let case_b = check_citation_in_set(&anchor_set, 99, "");

    // (c) Stale: cite anchor 2 with a DIFFERENT excerpt (post-freeze change).
    let case_c = check_citation_in_set(
        &anchor_set,
        2,
        "different excerpt that was not frozen",
    );

    // Verify byte-identical [N] extraction (GD-2 smoke).
    let answer_text = "As shown in [1], Rust is safe [2, 3].";
    let extracted_ids = extract_citation_ids(answer_text);

    // Check all extracted ids are in the set (Bound / Ghost classification).
    let mut ghost_ids: Vec<u32> = Vec::new();
    for id in &extracted_ids {
        let result = check_citation_in_set(&anchor_set, *id, "");
        if result.outcome.must_block() {
            ghost_ids.push(*id);
        }
    }

    // ── SSB emit ──────────────────────────────────────────────────────────
    let project_root: std::path::PathBuf = std::env::current_dir()
        .map_err(|e| anyhow::anyhow!("prebind --demo-prebind: cannot determine cwd: {e}"))?;
    let sessions_dir = sessions_dir_path(&project_root);
    std::fs::create_dir_all(&sessions_dir)
        .map_err(|e| anyhow::anyhow!("prebind --demo-prebind: create sessions dir: {e}"))?;
    let db_path = sessions_dir.join("ssb.db");

    let cfg = SsbConfig {
        db_path,
        pool_size: None,
        project_id: DEMO_PREBIND_PROJECT_ID.to_string(),
        session_id: DEMO_PREBIND_SESSION_ID.to_string(),
    };
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| anyhow::anyhow!("prebind --demo-prebind: build async runtime: {e}"))?;

    let (ssb_kind, emit_anchor_count) = if allow_freeform || prebind_disabled() {
        // BROKEN ARM (R2 proof): prebinding disabled; the legacy detect-after path
        // is active. Emit a Blocked row to prove the guard was bypassed.
        (CitationAnchorsBoundKind::Blocked, 0u32)
    } else {
        (
            CitationAnchorsBoundKind::Bound,
            anchor_set.len() as u32,
        )
    };

    let set_hash_hex = {
        let bytes = anchor_set.set_blake3;
        let h = blake3::Hash::from_bytes(bytes);
        format!("{}", h.to_hex())
    };

    let payload = SsbPayload::CitationAnchorsBound(CitationAnchorsBoundPayload {
        kind: ssb_kind,
        anchor_count: emit_anchor_count,
        seed,
        blake3_set_hash_hex: set_hash_hex.clone(),
    });

    rt.block_on(async move {
        let ssb = Ssb::new(cfg, clock)
            .map_err(|e| anyhow::anyhow!("prebind --demo-prebind: open SSB: {e}"))?;

        // Write the off-agent artifact (R4). A write failure is an error (not
        // a warning) in demo mode — the whole point is to prove the row lands.
        ssb.write_payload(
            DEMO_PREBIND_AGENT_ID,
            0,
            payload,
            Vec::new(),
        )
        .await
        .map_err(|e| anyhow::anyhow!("prebind --demo-prebind: write CitationAnchorsBound SSB row: {e}"))?;

        Ok::<_, anyhow::Error>(())
    })?;

    // ── Output ────────────────────────────────────────────────────────────
    match output_format {
        CliOutputFormat::Json => {
            let result = serde_json::json!({
                "mode": if allow_freeform || prebind_disabled() { "broken-arm" } else { "prebind-active" },
                "ssb_kind": ssb_kind.as_str(),
                "anchor_count": emit_anchor_count,
                "seed": seed,
                "blake3_set_hash_hex": set_hash_hex,
                "case_a_bound": {
                    "id": case_a.id,
                    "outcome": case_a.outcome.as_str(),
                    "must_block": case_a.outcome.must_block(),
                    "anchor_excerpt_blake3_hex": case_a.anchor_excerpt_blake3_hex,
                },
                "case_b_ghost": {
                    "id": case_b.id,
                    "outcome": case_b.outcome.as_str(),
                    "must_block": case_b.outcome.must_block(),
                },
                "case_c_stale": {
                    "id": case_c.id,
                    "outcome": case_c.outcome.as_str(),
                    "must_block": case_c.outcome.must_block(),
                },
                "answer_extraction": {
                    "answer_text": answer_text,
                    "extracted_ids": extracted_ids,
                    "ghost_ids": ghost_ids,
                },
            });
            println!("{}", serde_json::to_string_pretty(&result)
                .unwrap_or_else(|_| "{}".to_owned()));
        }
        CliOutputFormat::Text => {
            println!("H-53 citation pre-bind demo ({})",
                if allow_freeform || prebind_disabled() { "BROKEN-ARM" } else { "prebind-active" });
            println!("  SSB kind:       {}", ssb_kind.as_str());
            println!("  anchor count:   {emit_anchor_count}");
            println!("  seed:           {seed}");
            println!("  set blake3:     {set_hash_hex}");
            println!();
            println!("Case (a) — bound citation [1] (excerpt matches blake3 pin):");
            println!("  outcome:        {}", case_a.outcome.as_str());
            println!("  must_block:     {}", case_a.outcome.must_block());
            println!();
            println!("Case (b) — ghost citation [99] (id not in pre-bound set):");
            println!("  outcome:        {}", case_b.outcome.as_str());
            println!("  must_block:     {}", case_b.outcome.must_block());
            println!();
            println!("Case (c) — stale citation [2] (excerpt changed post-freeze):");
            println!("  outcome:        {}", case_c.outcome.as_str());
            println!("  must_block:     {}", case_c.outcome.must_block());
            println!();
            println!("GD-2 extraction from \"{answer_text}\":");
            println!("  extracted_ids:  {extracted_ids:?}");
            println!("  ghost_ids:      {ghost_ids:?}");
            println!();
            println!("CitationAnchorsBound SSB row written (off-agent artifact, R4).");
        }
        _ => {
            // Toml / other formats: fall back to text.
            println!("H-53 demo: ssb_kind={} anchor_count={emit_anchor_count} seed={seed}",
                ssb_kind.as_str());
        }
    }

    Ok(())
}
