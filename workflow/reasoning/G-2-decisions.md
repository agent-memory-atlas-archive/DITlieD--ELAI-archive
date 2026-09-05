G-2 decisions — Reproducible-Run Artifact (RRA) [THE KEYSTONE]
session s-56068, 2026-06-03

WHAT
A single portable msgpack file bundling the SSB action stream + ELAI_SEED + HardwareClass + system_fingerprint pin + model pin + an evidence manifest of every available F-gate verdict hash + a blake3 root. New `elai run --emit-rra`, `elai rra show <file>`, `elai replay --verify <file>`. G-3/G-5/G-8/G-9/G-10 consume this artifact's evidence manifest (GD-1).

GROUND TRUTH (context-compiler + 2 explorers, s-56068, fresh elai.map): all produces (rra.rs RunArtifact/GuaranteeLevel/build_rra/write_rra/verify_rra/ReplayVerdict + run_artifact_emitted) are NEW. Baseline SCHEMA_VERSION 209 (ssb.rs:2921), TOTAL_VARIANTS 486 (action_type_lock.rs:560) -> 210/487.

KEY DECISIONS

DD1 COMPRESSION / NO NEW DEP (CLAUDE.md HARD RULE: no new deps without confirmation).
rmp-serde 1.3 IS a direct dep (orchestrator Cargo.toml:70) -> msgpack is fine. zstd is NOT a direct dep (only transitive via tantivy, Cargo.lock 0.13.3). DECISION: do NOT add zstd as a new direct dep this session. The .rra is serialized with rmp-serde (msgpack). If an EXISTING direct compression dep (flate2 / snap / lz4 / brotli) is already in any .elai_cc Cargo.toml, use it for the bundle frame; ELSE ship UNCOMPRESSED msgpack and leave a documented one-line "add zstd = \"0.13\"" enhancement (the version is already vendored transitively, verified, no slopsquat risk — but promotion needs explicit user confirmation). Compression is non-essential to the reproducibility guarantee. Carry this honestly in the final report so the user can opt into zstd.

DD2 EMIT POINT (W-3 real entrypoint + the recurring SSB-ctor wire-dark check).
The --emit-rra hook attaches at run_async EXIT, mirroring the existing SsbExportHandle / --export-ssb (run_subcommand.rs:312-316 start, 428-430 flush-at-exit) — NOT build_production_hook_dispatcher (runtime_setup.rs:766, which builds the PostToolUse/VibeGate tool-use dispatcher, not the artifact emit). The emit + the run_artifact_emitted SSB row MUST use the EXISTING run Arc<Ssb> handle (mirror archive_verify.rs:173-199 + OrchestratorRugPullSsbEmitter::new(ssb)); NEVER reconstruct Ssb::new (would bind a different session_id and split the stream). STANDING CHECK (bit G-4 + G-1): trace the row to a production path that has the live Ssb.

DD3 GUARANTEE = VERIFY-DIVERGENCE, not full re-execute (GD-5 HONEST SCOPE).
SSB stores tool I/O HASHED, not raw (args_hash / input_hash / cached_result_hash / classifier_verdict_hash; McpToolCalledPayload is metadata-only). Therefore `elai replay --verify` re-derives each hash from the bundled stream + re-checks every bundled F-gate verdict hash and reports PASS (stream + all hashes reproduced) or DIVERGENCE with the FIRST mismatching event. It does NOT full-re-execute body-reading steps or token streams (cloud-lane system_fingerprint opacity). The GuaranteeLevel field records this per-lane: local = action-stream-reproducible; cloud = action-stream + hash-chain verify-divergence, NOT token-reproducible. Do NOT bundle raw tool bodies (secret-exposure liability; G-9 write-time redaction is its precondition). Do NOT over-claim "re-executes the run."

DD4 F-GATE MANIFEST (GD-1 — CONSUME, re-implement NONE; re-stating F-1/F-4/F-9/F-10 = P0).
Read the verdict hashes from their existing payloads: F-1 source_blake3 (VibeGateBlockedPayload ssb.rs:24642 — NOTE default-OFF behind ELAI_VIBE_GATE_ENABLE, so a MISSING F-1 entry is a valid gate-off state, NOT a faithfulness failure), F-4 stdout_hash (TestOutcomePayload ssb.rs:10004), F-9 dep_name_hash:u64 (WriteTimeSlopsquatBlockedPayload ssb.rs:25702 — u64, unlike the hex-String others; the manifest schema MUST encode per-gate hash type or normalize before the manifest hash), F-10 delta_blake3 (security_monotonicity.rs:188, F-10 is landed). The model pin is sourced from run/backend identity SEPARATELY (NOT in DeterminismConfig determinism.rs:138, which carries seed:u64 + HardwareClass + system_fingerprint_pin only).

DD5 SSB STREAM READER (the gap): there is NO full-ordered untruncated reader. ssb_viewer::recent_actions (ssb_viewer.rs:1064) is DESC + truncated-payload summary. Add an ASC full-payload stream reader for the session so the bundle is faithful. The blake3 root is a Merkle/flat hash over the canonical bundle bytes (blake3 is the workspace hash).

DD6 RESUME IS OUT OF SCOPE (carried landmine): replay re-runs/verifies from the action-stream START. recover_from_checkpoint (bsp/engine.rs:887/900) is a PARTIAL STUB (validation/logging only, no channel-value restoration) = the RESUME gap, the OPTIONAL second half, a T7-gated follow-on — NOT part of G-2's done-gate. BspEngine "no production caller" is REFUTED (task_runner.rs:5025 BspEngine::new in async run_parallel_phases_via_bsp:4946 is a real non-test caller) — do NOT ground anything on that false premise.

DD7 CLI (hand-rolled args, NO clap): add --emit-rra as a flag on the run subcommand exactly like --export-ssb (args.rs:2442). Add `rra show` + `replay --verify` sub-verbs mirroring Freeze{action:FreezeAction} (args.rs:413/1250, dispatch main.rs:467); the CliAction enum is in args.rs (main.rs:439 holds match arms). commands_g2.rs is the natural slot.

DD8 STALE CONTRACT RECONCILE: e45_cross_language_drift_contract.rs:294 asserts TOTAL_VARIANTS=482 while live is 486 (pre-existing drift, failing independent of G-2). As part of the lockstep bump to 487, update that assertion to the correct current value so it does not mask as a G-2 regression. (Opportunistic; it is touching the same variant-count surface.)

FRONTEND PAIR: an RRA viewer panel mirroring F-13 SessionArchiveDrawer (closest: reads SSB-derived rows from disk) or the G-4 McpRugPullDrawer. Tauri command (commands_g2.rs) + DrawerKey + types.ts DTO. cross_lang_diff must pass.

EXPECTED TEST OUTCOME
Deterministic fixture: a fixture run produces an SSB stream + (gate-on) F-gate rows; `build_rra` bundles them; `write_rra` emits a .rra file + a run_artifact_emitted SSB row carrying the blake3 root; `verify_rra` on the same .rra returns PASS (stream + all present hashes reproduced); a TAMPERED .rra (flip one event/hash) returns DIVERGENCE naming the first mismatching event. Gate-off F-1 -> manifest entry absent, still PASS (not a failure). The run_artifact_emitted root re-computes byte-identically from the journal under ELAI_SEED (GD-2).

GD-4 / GD-5
GD-2: the blake3 root + manifest hash recompute from the SSB stream under ELAI_SEED (the metric is auditable). GD-5: GuaranteeLevel carries the honest per-lane state (cloud = verify-divergence not token-reproducible). Live end-to-end `elai run --emit-rra` smoke = W-5/Tier-2 separately tracked; the deterministic build/verify/divergence fixture is the Tier-1 fail-closed gate.

UNKNOWNS / RISKS
- Whether an existing direct compression dep exists (executor checks; else uncompressed + zstd-deferred).
- The ASC full-payload reader's cost on a large session (stream may be big) — fine for the fixture; note for large real runs.
- model_pin source (run/backend identity) — executor confirms the canonical field.
