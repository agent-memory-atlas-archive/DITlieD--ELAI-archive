G-8 decisions — Signed in-toto provenance attestation + Article-50 verifier CLI
session s-56068, 2026-06-04

WHAT
A custom in-toto v1 Statement + predicate (subject digest blake3; predicate carries model id, ELAI_SEED, an SLSA-VSA verifiedLevels rollup of every F-gate, evidence hashes, the SSB event seq) emitted at `elai run` exit to: (a) a provenance_attestation_emitted SSB row, (b) a sidecar attestation.intoto.json, (c) a digest-only git commit trailer. Signed via a LOCAL hand-rolled blake3-Merkle proof (OFFLINE DEFAULT). New `elai verify-attestation <commit>` re-verifies OFFLINE WITHOUT re-running ELAI ("trust without re-running", SLSA-VSA).

GROUND TRUTH (context-compiler + 2 explorers, s-56068, fresh elai.map): G-2 SHIPPED this run and is richer than the plan assumed. The "contracts crate broken by missing use std::path::Path" claim recurs in stale context but is FALSE — verified `cargo check -p elai_cc_contracts --tests` GREEN (only unused-import Path warnings). Baseline SCHEMA_VERSION 212 (ssb.rs:2947), TOTAL_VARIANTS 489 (action_type_lock.rs:583) -> 213/490.

KEY DECISIONS

DD1 SIGNING = HAND-ROLLED blake3-MERKLE, DEFAULT, NO NEW DEP. Mirror the in-tree blake3-signing precedent archive_crypto::placeholder_sign/placeholder_verify (archive_crypto.rs:84-112, recompute-and-compare offline posture) + G-2's rra::compute_root (rra.rs:296, blake3 over rmp_serde-canonical bytes, 16-hex prefix). cosign-keyless / sigstore / rekor / ed25519-dalek are ALL ABSENT from Cargo.lock — the opt-in keyless path is a NEW DEP and is DEFERRED this session (CLAUDE.md no-new-deps): ship it feature-gated (e.g. `feature = "cosign-attest"`), and the crate MUST cargo-check-green WITHOUT the feature. No live keyless signing this session; document it as a confirmable enhancement.

DD2 MODULE = crates/orchestrator/src/attestation.rs (NOT provenance.rs — that is the G-1 taint system; do not collide). REUSE G-2's rra.rs types (GD-1, REUSE-DON'T-FORK): GateVerdictHash (its field is ssb_event_seq:i64, G-2-locked — NOT the plan's ssb_event_id), extract_gate_manifest (the SHIPPED 4-gate F-1/F-4/F-9/F-10 VSA rollup), compute_root, truncate_hex. The attestation's verifiedLevels IS G-2's manifest re-expressed as an SLSA-VSA PASSED rollup — READ RunArtifact.manifest, re-implement NO F-gate.

DD3 DATA MODEL = in-toto v1 Statement (serde struct): { _type: "https://in-toto.io/Statement/v1", subject: [{ name, digest: { blake3: <root> } }], predicateType: <custom ELAI URI>, predicate: { model_id, elai_seed, verifiedLevels: [{gate, verdict, hash}], evidence_hashes, ssb_event_seq } }. blake3 digest is the workspace default; sha256 only under the deferred cosign-interop feature.

DD4 EMIT (3 targets) at the run exit, sharing the live run Arc<Ssb> (the recurring SSB-ctor + project_id-bucket checks): emit alongside G-2's emit_run_artifact at run_subcommand.rs:457/509, reusing the live rra_ssb Arc<Ssb> (project_id INHERITED from the run handle, NEVER a literal — the G-5 lesson).
  (a) provenance_attestation_emitted SSB row — NEW Pattern-2 single-variant (mirror the just-shipped ReviewSurfaceScored cascade, 8 sites). Payload = the in-toto predicate digest (the blake3-Merkle root) + the subject name. Bump SCHEMA 212->213 (ssb.rs:2947) + TOTAL_VARIANTS 489->490 (action_type_lock.rs:583) in lockstep; regen golden; update the all_test_payloads count assert (ssb.rs:34287, currently 409 -> 410).
  (b) sidecar attestation.intoto.json — the FULL Statement JSON written next to the run artifacts (mirror rra::write_rra:443 create_dir_all + fs::write, but JSON not msgpack).
  (c) git commit trailer — DIGEST ONLY (e.g. `ELAI-Provenance-Digest: <blake3-root>`) at git_commit.rs::format_commit_message:184. This is a NET-NEW positive trailer that MUST survive the existing strip-filter (:217-228, which strips Co-Authored-By etc.) — it is an ALLOWED trailer (not an AI-attribution trailer; CLAUDE.md bans Co-Authored-By, not this). SEAM: if computing the digest before the commit is awkward, the trailer reads the run's just-written attestation digest (from the sidecar or the SSB row); the sidecar + SSB row are the PRIMARY emit (always), the trailer is the git-binding (when the run creates a commit). The sidecar + SSB row alone satisfy the core observable_effect.

DD5 VERIFY = `elai verify-attestation <commit>` CLI. Mirror rra_cmd::run_verify:81 EXACTLY: READ-ONLY, opens NO Ssb, prints verified / not-verified, process::exit(1) on failure. Add a git-trailer READER shell-out (`git show`/`git log --format=%(trailers:key=...)`) — there is NO in-tree trailer reader — mirroring review_cmd::resolve_base_commit:104-134 error discipline: explicit Err on git-absent / non-repo / non-zero exit, NEVER a fabricated "verified". Re-verify OFFLINE: read the trailer digest + the sidecar Statement, recompute the blake3-Merkle root over the predicate, confirm it equals the trailer digest, and confirm the predicate's recorded F-gate verdict hashes are Merkle-bound (the root commits to them). Do NOT re-run any gate (trust-without-re-run, SLSA-VSA). add a Review-style CliAction VerifyAttestation verb (mirror G-2 Rra/Replay + G-5 Review).

DD6 ARTICLE-50 HONESTY (E-38 HARD RULE + GD-5): position as "designed to support Article-50 marking", NOT "certified compliant" (the final Code of Practice is June 2026; source code is not an enumerated Art-50 example). Pair ONLY with the transparency tier EUR 15M/3% (Art-50 / Art-99), NEVER the Art-5 prohibited tier EUR 35M/7%. CITE docs/strategy/ELAI-2026-POSTURE.md for the posture; do NOT assert the EUR figures or the Art mapping from training memory (E-38). If the posture doc lacks the exact figures, WebFetch the primary Art-99/Art-50 text. Any code comment / CLI text / doc that states a penalty number must carry the honest tier.

DD7 GD-1 COMPOSE: consume F-4 (anti-fabrication, extended off-machine) + G-2 (the RRA evidence manifest + SSB chaining). Re-implement NEITHER. The attestation EXTENDS F-4 from inside-the-run to off-machine verifiability.

DD8 FRONTEND_PAIR: check the WIRING_CONTRACT. If a frontend_pair is declared (an attestation viewer), mirror G-2's RraViewerPanel + a Tauri command (commands_g8.rs) + the 3 label-map sync (the G-5 lesson). If the contract marks it backend-only, add the explicit "backend-only-justification:" marker. Do NOT invent a frontend the contract doesn't ask for.

EXPECTED TEST OUTCOME
Deterministic fixture: a fixture run with F-gate verdicts -> build_attestation reads G-2's manifest, builds the in-toto Statement, signs via blake3-Merkle -> emits the SSB row + sidecar JSON (+ trailer if committing); `elai verify-attestation` on the sidecar+digest re-verifies OFFLINE = VERIFIED; a TAMPERED predicate (flip one verdict/hash) -> the recomputed root != the digest -> NOT-VERIFIED; a git-absent verify -> explicit "cannot verify (git unavailable)" Err, NEVER a fabricated "verified". The attestation digest re-computes byte-identically from the predicate under ELAI_SEED (GD-2).

GD-4 / GD-5 / W-5
GD-2: the Merkle root re-computes byte-identically from the predicate. GD-5: the Article-50 honest positioning + the deferred-cosign honest state (default is local-Merkle, NOT a sigstore-transparency-log guarantee). W-5: a live `elai run` emitting + `elai verify-attestation` on a real commit = Tier-2 separately tracked; the deterministic build/verify/tamper fixture is the Tier-1 gate.

UNKNOWNS / RISKS
- The trailer-vs-run-exit seam (DD4c): the executor picks the cleanest (sidecar+row always; trailer when a commit is made). Document the choice.
- in-toto predicateType URI: pick a stable ELAI custom URI (in-toto v1 allows custom predicates).
- ed25519/cosign deferral: the feature-gated opt-in must not break the default build; do NOT add the dep.
- The EUR figures: cite the posture doc; if absent, WebFetch — do not guess (E-38).
