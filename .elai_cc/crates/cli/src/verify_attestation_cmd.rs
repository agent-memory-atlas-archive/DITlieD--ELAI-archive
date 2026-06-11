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
    read_sidecar, verify_attestation_against_trailer, AttestationVerdict, InTotoStatement,
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
/// READ-ONLY (no `Ssb`). Reads the digest-only provenance trailer from the
/// commit, reads the matching sidecar predicate, recomputes the blake3-Merkle
/// root OFFLINE, and prints the verdict. Exits non-zero on not-verified.
pub(crate) fn run(commit: String, output_format: CliOutputFormat) -> Result<(), String> {
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
    let verdict = verify_attestation_against_trailer(&statement, &trailer_digest)
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

    // Exit non-zero on a not-verified verdict so CI can gate on it (mirrors
    // rra_cmd::run_verify). The verdict has already been printed.
    match verdict {
        AttestationVerdict::Verified { .. } => Ok(()),
        AttestationVerdict::NotVerified { .. } => std::process::exit(1),
    }
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
    }
}
