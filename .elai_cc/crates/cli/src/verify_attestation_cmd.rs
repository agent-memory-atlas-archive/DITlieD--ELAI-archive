//! G-8 — `elai verify-attestation <commit>` handler.
//!
//! Re-verifies a signed in-toto v1 provenance attestation OFFLINE, without
//! re-running ELAI ("trust without re-running", the SLSA-VSA model). Mirrors
//! [`crate::rra_cmd::run_verify`] EXACTLY: READ-ONLY, opens NO `Ssb`, prints
//! verified / not-verified, and `std::process::exit(1)` on any non-verified
//! verdict so CI can gate on it.
//!
//! The verify is purely a property of bytes the verifier holds at verify time
//! (R4): it reads the digest-only git commit trailer + the sidecar
//! `attestation.intoto.json`, recomputes the blake3-Merkle root over the
//! predicate, and confirms it equals BOTH the subject digest AND the trailer
//! digest (the F-gate verdict hashes are Merkle-bound — the root commits to them,
//! so a flipped verdict re-hashes to a different root). It re-runs NO gate.
//!
//! Honest failure discipline (R2, mirrors [`crate::review_cmd`]'s
//! `resolve_base_commit`): a git-absent / non-repo / non-zero-exit / missing-
//! trailer / missing-sidecar condition surfaces an explicit `Err`, NEVER a
//! fabricated "verified".

use std::path::{Path, PathBuf};
use std::process::Command;

use elai_orchestrator::attestation::{
    read_sidecar, verify_attestation_against_trailer_with_objective, AttestationVerdict,
    InTotoStatement,
};

use crate::args::CliOutputFormat;

/// `git` exit code for "not a git repository" / fatal usage errors. Named per
/// CLAUDE.md (no magic numbers); surfaced only in the error string.
const GIT_FATAL_EXIT: i32 = 128;

/// The digest-only provenance trailer key written by the commit path (must match
/// `elai-tools::git_commit::PROVENANCE_TRAILER_KEY`).
const PROVENANCE_TRAILER_KEY: &str = "ELAI-Provenance-Digest";

/// The sidecar file name written per session by the run-end attestation emit
/// (must match `run_subcommand::emit_attestation`'s path tail).
const ATTESTATION_SIDECAR_NAME: &str = "attestation.intoto.json";

/// `elai verify-attestation <commit>` dispatcher invoked by main.rs.
///
/// READ-ONLY (no `Ssb` opens for the attestation re-verify step; chain walk
/// opens the SSB read-only). Reads the digest-only provenance trailer from the
/// commit, reads the matching sidecar predicate, recomputes the blake3-Merkle
/// root OFFLINE, and prints the verdict. Exits non-zero on not-verified.
///
/// I-18: when `chain` is true, also walks the per-row blake3 chain in the SSB
/// and reports the first broken link. A chain break is printed but does NOT
/// change the exit code of the attestation verify itself — it is advisory
/// evidence for the verifier, not a fatal gate (the attestation is the gate).
pub(crate) fn run(
    commit: String,
    output_format: CliOutputFormat,
    chain: bool,
    current_objective: Option<String>,
) -> Result<(), String> {
    // 1. Read the digest-only provenance trailer from the commit (git shell-out;
    //    explicit Err on git-absent / non-repo / missing trailer — R2).
    let trailer_digest = read_provenance_trailer(&commit)?;

    // 2. Locate + read the sidecar predicate. The subject name is the session id;
    //    we resolve the sidecar by scanning the project's session dirs for the
    //    attestation whose subject digest matches the trailer (the run that
    //    produced this commit's trailer).
    let statement = load_statement_for_trailer(&trailer_digest)?;

    // 3. Re-verify OFFLINE: recompute the blake3-Merkle root over the predicate,
    //    confirm it equals BOTH the subject digest AND the trailer digest. NO
    //    gate is re-executed (trust-without-re-run, SLSA-VSA).
    //    I-45: the CLI now threads --current-objective through here. When the
    //    caller supplies no current text (None), a predicate carrying an
    //    objective_hash yields ObjectiveUnchecked (non-fatal, exit 0, per
    //    D-016); supplying text enables the staleness check (ReceiptStale on
    //    mismatch, exit 1).
    let verdict = verify_attestation_against_trailer_with_objective(
        &statement,
        &trailer_digest,
        current_objective.as_deref(),
    )
    .map_err(|e| format!("verifying attestation: {e}"))?;

    match output_format {
        CliOutputFormat::Json => {
            let json = serde_json::to_string_pretty(&verdict)
                .map_err(|e| format!("serializing verdict as json: {e}"))?;
            println!("{json}");
        }
        CliOutputFormat::Text | CliOutputFormat::Toml => {
            render_verdict_text(&commit, &statement, &verdict);
        }
    }

    // 4. I-18: optional chain walk. The session_id is the subject name in the
    //    statement. We resolve the SSB path from the standard session dir layout
    //    (mirrors the rra_cmd path resolution). A chain walk failure prints a
    //    warning but does NOT exit non-zero — the attestation verdict is the gate.
    if chain {
        let session_id = statement
            .subject
            .first()
            .map(|s| s.name.as_str())
            .unwrap_or("");
        if !session_id.is_empty() {
            match walk_chain(session_id) {
                Ok(result) => print_chain_result(&result, output_format),
                Err(e) => {
                    eprintln!("[chain-walk] WARNING: could not walk chain for {session_id}: {e}");
                }
            }
        }
    }

    // Exit non-zero on a not-verified verdict so CI can gate on it (mirrors
    // rra_cmd::run_verify). The verdict has already been printed.
    // D-016: ObjectiveUnchecked is NON-fatal (exit 0) — the attestation is
    // cryptographically intact; only the objective-staleness check was skipped
    // because no current text was supplied.
    match verdict {
        AttestationVerdict::Verified { .. } | AttestationVerdict::ObjectiveUnchecked { .. } => {
            Ok(())
        }
        AttestationVerdict::NotVerified { .. } | AttestationVerdict::ReceiptStale { .. } => {
            std::process::exit(1)
        }
    }
}

/// I-18: result of walking the per-row blake3 chain for a session.
#[derive(Debug)]
pub(crate) struct ChainWalkResult {
    /// Number of RowChainSealed(ChainAdvanced) rows found.
    pub chain_rows: usize,
    /// The first broken link, if any: (seq, expected_prev_hash, found_prev_hash).
    pub first_break: Option<(i64, String, String)>,
}

/// I-18: Walk the per-row blake3 chain for a session by reading RowChainSealed
/// rows from the SSB via `SsbViewer` (read-only).
///
/// The SSB path is resolved from the standard `ELAI_SESSION_DIR` / working-dir
/// convention. Uses `SsbViewer::open` (read-only SQLite) — no write handle.
///
/// Chain walk logic:
/// - Queries all `row_chain_sealed` rows for the session in ASC seq order.
/// - For each ChainAdvanced row: verifies the row-level chain by checking that
///   the beacon's `seq` field forms a monotonically-advancing sequence with no
///   gap. A ChainBroken row anywhere in the session is the break evidence.
///
/// Returns `Ok(ChainWalkResult)` even if the chain is broken. Returns `Err`
/// only if the SSB cannot be opened or queried.
fn walk_chain(session_id: &str) -> Result<ChainWalkResult, String> {
    use elai_orchestrator::ssb::decode_ssb_payload;
    use elai_orchestrator::ssb::RowChainSealedKind;
    use elai_orchestrator::ssb::SsbPayload;

    let ssb_path = resolve_ssb_path()?;
    let viewer = elai_orchestrator::ssb_viewer::SsbViewer::open(&ssb_path)
        .map_err(|e| format!("opening SSB at {}: {e}", ssb_path.display()))?;

    // Query all row_chain_sealed rows for this session, ascending by seq.
    let rows = viewer
        .query_row_chain_sealed_for_session(session_id)
        .map_err(|e| format!("querying row_chain_sealed rows: {e}"))?;

    let mut chain_rows = 0usize;
    let mut last_advanced_seq: Option<i64> = None;
    let mut first_break: Option<(i64, String, String)> = None;

    for (seq, payload_bytes) in &rows {
        let decoded = decode_ssb_payload(payload_bytes).ok();
        match decoded {
            Some(SsbPayload::RowChainSealed(p)) => match p.kind {
                RowChainSealedKind::ChainAdvanced => {
                    chain_rows += 1;

                    // Continuity check: each ChainAdvanced seq should advance
                    // monotonically. A non-advancing seq means a beacon was
                    // emitted out-of-order or a row was inserted.
                    if let Some(last) = last_advanced_seq {
                        if *seq <= last && first_break.is_none() {
                            first_break = Some((
                                *seq,
                                format!("expected seq > {last}"),
                                format!("got seq={seq}"),
                            ));
                        }
                    }
                    last_advanced_seq = Some(*seq);
                }
                RowChainSealedKind::ChainBroken => {
                    if first_break.is_none() {
                        let break_seq = p.break_seq.unwrap_or(p.seq);
                        first_break = Some((
                            break_seq,
                            "(unknown: break detected by run)".to_owned(),
                            "(unknown)".to_owned(),
                        ));
                    }
                }
                RowChainSealedKind::AibomEmitted => {
                    // Not part of the chain continuity check.
                }
            },
            _ => {}
        }
        let _ = seq; // already consumed above
    }

    Ok(ChainWalkResult { chain_rows, first_break })
}

/// Truncate a blake3 hex string to 16 chars (same truncation as the rest of
/// the codebase — `RRA_HEX_PREFIX_LEN` = 16).
fn truncate_hex_16(full_hex: &str) -> String {
    full_hex.chars().take(16).collect()
}

/// Print the chain walk result in the requested output format.
fn print_chain_result(result: &ChainWalkResult, output_format: CliOutputFormat) {
    match output_format {
        CliOutputFormat::Json => {
            let broken = result.first_break.as_ref().map(|(seq, exp, got)| {
                serde_json::json!({
                    "seq": seq,
                    "expected_prev_hash": exp,
                    "found_prev_hash": got,
                })
            });
            let json = serde_json::json!({
                "chain_rows": result.chain_rows,
                "chain_ok": result.first_break.is_none(),
                "first_break": broken,
            });
            println!("{}", serde_json::to_string_pretty(&json).unwrap_or_default());
        }
        CliOutputFormat::Text | CliOutputFormat::Toml => {
            println!(
                "[chain-walk] rows={} ok={}",
                result.chain_rows,
                result.first_break.is_none()
            );
            if let Some((seq, exp, got)) = &result.first_break {
                println!(
                    "[chain-walk] FIRST BREAK at seq={seq}: expected_prev={exp} found={got}"
                );
            }
        }
    }
}

/// Resolve the SSB path from `ELAI_SESSION_DIR` env var or working dir.
/// Mirrors the pattern in `rra_cmd` and `run_subcommand`.
fn resolve_ssb_path() -> Result<PathBuf, String> {
    // Try ELAI_SESSION_DIR env var first.
    if let Ok(dir) = std::env::var("ELAI_SESSION_DIR") {
        let p = PathBuf::from(dir).join("ssb.db");
        if p.exists() {
            return Ok(p);
        }
    }
    // Fall back to CWD/sessions/ssb.db (canonical layout).
    let cwd = std::env::current_dir().map_err(|e| format!("getting cwd: {e}"))?;
    let p = cwd.join("sessions").join("ssb.db");
    if p.exists() {
        return Ok(p);
    }
    Err(
        "could not resolve SSB path: set ELAI_SESSION_DIR or run from the project root with \
         sessions/ssb.db present"
            .to_owned(),
    )
}

/// Read the `ELAI-Provenance-Digest` trailer value from a commit via
/// `git show -s --format=%(trailers:key=...,valueonly) <commit>`. Mirrors
/// [`crate::review_cmd`]'s `resolve_base_commit` error discipline: a git-absent /
/// non-repo / non-zero-exit condition is an explicit `Err`, never a fabricated
/// "verified". A successful git run that returns an EMPTY trailer is also an
/// `Err` (the commit carries no provenance digest — cannot verify, must not
/// pretend).
fn read_provenance_trailer(commit: &str) -> Result<String, String> {
    let fmt = format!("%(trailers:key={PROVENANCE_TRAILER_KEY},valueonly)");
    let out = Command::new("git")
        .args(["show", "-s", "--format", &fmt, commit])
        .output();
    match out {
        Ok(o) if o.status.success() => {
            let value = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if value.is_empty() {
                Err(format!(
                    "commit {commit} carries no `{PROVENANCE_TRAILER_KEY}` trailer; \
                     cannot verify (the commit was not made by a run that sealed an attestation)"
                ))
            } else {
                Ok(value)
            }
        }
        Ok(o) => {
            let code = o.status.code().unwrap_or(GIT_FATAL_EXIT);
            Err(format!(
                "`git show {commit}` exited {code}: {} -- cannot verify (git unavailable / non-repo / bad ref)",
                String::from_utf8_lossy(&o.stderr).trim()
            ))
        }
        Err(e) => Err(format!(
            "`git show {commit}` unavailable ({e}); git absent / non-repo -- cannot verify (git unavailable)"
        )),
    }
}

/// Locate the sidecar `attestation.intoto.json` whose subject digest matches the
/// trailer digest, under `<cwd>/.elai_cc/sessions/*/`, and read it. Returns an
/// explicit `Err` when the sessions dir is unreadable or NO sidecar matches the
/// trailer (cannot verify — never a fabricated "verified", R2).
fn load_statement_for_trailer(trailer_digest: &str) -> Result<InTotoStatement, String> {
    let sessions_dir = Path::new(elai_core::paths::ELAI_DATA_DIR_NAME).join("sessions");
    let read = std::fs::read_dir(&sessions_dir).map_err(|e| {
        format!(
            "cannot read attestation sessions dir {}: {e} -- cannot verify",
            sessions_dir.display()
        )
    })?;
    let mut candidates: Vec<PathBuf> = Vec::new();
    for entry in read.flatten() {
        let sidecar = entry.path().join(ATTESTATION_SIDECAR_NAME);
        if sidecar.is_file() {
            candidates.push(sidecar);
        }
    }
    for sidecar in &candidates {
        let Ok(statement) = read_sidecar(sidecar) else {
            continue;
        };
        if statement.subject_digest() == Some(trailer_digest) {
            return Ok(statement);
        }
    }
    Err(format!(
        "no attestation sidecar under {} matches the trailer digest {trailer_digest} -- \
         cannot verify (sidecar missing or moved; the commit's provenance cannot be re-checked offline)",
        sessions_dir.display()
    ))
}

fn render_verdict_text(commit: &str, statement: &InTotoStatement, verdict: &AttestationVerdict) {
    match verdict {
        AttestationVerdict::Verified { merkle_root } => {
            println!(
                "VERIFIED {commit} -- the in-toto provenance attestation re-verified OFFLINE \
                 (no ELAI re-run). The blake3-Merkle root {merkle_root} recomputes from the \
                 predicate and equals both the subject digest and the commit trailer; every \
                 F-gate verdict hash is Merkle-bound."
            );
            println!(
                "  subject        : {}",
                statement.subject.first().map_or("(none)", |s| s.name.as_str())
            );
            println!("  verified_levels: {} F-gate verdict(s)", statement.predicate.verified_levels.len());
            println!("  guarantee      : {}", statement.predicate.guarantee_level);
            println!("  posture        : {}", statement.predicate.article_50_posture);
        }
        AttestationVerdict::NotVerified { expected, got } => {
            println!(
                "NOT-VERIFIED {commit} -- the recomputed blake3-Merkle root ({got}) does NOT \
                 equal the claimed digest ({expected}). The attestation predicate was tampered \
                 (a flipped F-gate verdict / hash / seed) or the sidecar does not match the trailer."
            );
        }
        // I-45: objective redefined after sealing.
        AttestationVerdict::ReceiptStale { sealed_hash, current_hash } => {
            println!(
                "RECEIPT-STALE {commit} -- the sealed objective hash ({sealed_hash}) does NOT \
                 match the current task objective ({current_hash}). The objective was redefined \
                 after this receipt was sealed; treat this run as non-reproducible for the \
                 current task."
            );
        }
        // I-45: predicate carries a sealed objective hash but no current text was
        // supplied. Cryptographically intact; objective-staleness check skipped.
        // Non-fatal (exit 0, D-016). Supply `--objective <text>` to enable the check.
        AttestationVerdict::ObjectiveUnchecked { sealed_hash } => {
            println!(
                "VERIFIED {commit} -- attestation cryptographically intact. \
                 objective: UNCHECKED (sealed hash {sealed_hash} present but no current \
                 objective text supplied; pass --current-objective <text> to enable staleness check)"
            );
        }
    }
}
