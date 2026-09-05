H-23 DECISIONS — same-seed re-run hash-equality METRIC_GATE (T-7)
Session: s-8404 | Date: 2026-06-05 | FSM: EXECUTING

WHAT
Built the reproducibility centerpiece: a gate that blake3-hashes the DECLARED
metric outputs of two SAME-SEED re-runs and asserts byte-equality. A hidden
unseeded RNG makes run #2's digest differ -> Divergent Block (the first divergent
metric named). A clean seeded experiment hashes Equal twice. Plus two LOCKED
rules.db sub-gates (seed-coverage AST + env-pin SchemaRequired) and the
+RerunHashEquality SSB lockstep.

KEY DECISIONS

D-1 (hazard C, RunArtifact field): DERIVE digests at verify-time; do NOT add a
RunArtifact field. The two metric-digest lists are the gate's INPUT (supplied by
the re-run entrypoint / the Tier-1 fixture), not a persisted RunArtifact field.
Adding a field would force: extend canonical_bundle_bytes tuple (rra.rs:281) or
root_blake3 silently won't cover it (tamper hole), bump RRA_FORMAT_VERSION + old
.rra read-compat, touch G-8 attestation merkle (attestation.rs:265) AND ts-rs
RunArtifact bindings (types.ts) = cross-stack drift. Deriving at verify-time is
genuinely possible (the entrypoint supplies the two digest lists), so the
low-blast path is correct. RESULT: rra.rs UNCHANGED; zero attestation/ts-rs/
format-version blast; e45 cross-lang drift only from the +1 action_type (intended
tripwire), not from a RunArtifact shape change.

D-2 (digest over DECLARED outputs, not stdout — R4): MetricDigest::from_bytes
hashes the declared metric output bytes only. The gate never reads stdout. The
combined per-run fingerprint is blake3 over the joined "<name>=<digest>" list.

D-3 (GD-1 REUSE, do NOT re-implement): the per-metric digest sequences are
wrapped as determinism::ActionTypeSequence(Vec<String>) and compared via
DriftComparator::compare, whose first-divergence loop names the first diverging
metric for free. The comparator's own DeterminismDrift emit is routed through a
dropped channel (the canonical artifact is the RerunHashEquality row, not a
duplicate DeterminismDrift row). blake3 16-hex convention reuses
rra::RRA_HEX_PREFIX_LEN. Neither RRA nor DriftComparator is re-implemented.

D-4 (hazard A, SEED fail-open): DeterminismConfig::from_env() silently defaults
the seed to 0 when ELAI_SEED is unset (determinism.rs:167). The gate does NOT use
that path: RerunInput.seed is Option<u64>, and a None Blocks fail-closed (reason
"missing_seed") rather than proceeding with seed=0. Test:
missing_seed_is_fail_closed_block_hazard_a. The env-pin SchemaRequired sub-gate
(REPRO-ENV-001) STRUCTURALLY enforces this: a run that never declared its seed
cannot satisfy the manifest schema.

D-5 (hazard B, NON-EMPTY payload): blake3(b"") is a fixed constant, so two empty
runs would hash Equal and fail-OPEN. The gate asserts the declared-output set is
non-empty AND non-trivial (>= MIN_DECLARED_METRICS=1 AND not all digests empty)
BEFORE an Equal can be a PASS (reason "empty_declared_outputs"). Tests:
empty_outputs_are_fail_closed_block_hazard_b + trivial_empty_digest_*.

D-6 (entrypoint): EXTENDED `elai replay --verify` with an ADDITIVE
`--metric-digest <mode>` flag (args.rs parse_replay_subcommand_args). The existing
`Verify { path }` arm is UNCHANGED (no regression). New ReplayAction::VerifyRerun
{ demo } dispatches rra_cmd::run_verify_rerun, which opens a real Ssb, drives
verify_rerun_digest over the Tier-1 fixture digests, and exits non-zero on
Divergent (mirrors run_verify's non-Verified exit contract). This is the W-1/W-3
production caller. The WIRING_CONTRACT's observable_effect (`replay --verify
--metric-digest`) is satisfied.

D-7 (seed sub-gates SHIPPED): the plan specifies them (Tasks 3+4), so both
shipped: ProjectRuleSeeder::seed_repro_rules() inserts REPRO-SEED-001 (AstPattern
seed-coverage) + REPRO-ENV-001 (SchemaRequired env-pin), both mutable=0, called
from runtime_setup.rs after the H-22 block. The tomls under
crates/rules/seed/repro/ are documentation artifacts (the Rust seeder does the DB
insert, same pattern as leakage/trading_nulls). The equality DECISION itself is
PURE (no rules.db threshold); these two rows are the static CAUSE-catchers + the
vibesec blake3 seed-integrity baseline.

D-8 (TD-5 SCOPE): same-env/same-seed equality ONLY. Cross-hardware GPU/FP bitwise
equality is physics (FMA/SIMD reassociation differs across SKUs); the gate is
never run across hardware, and a cross-hardware delta is recorded, never asserted.
Documented in the gate module header + the SSB doc comments.

TIER SPLIT
Tier-1 (this session, in-process, no sidecar): the gate consumes two
fixture-supplied digest lists; the contract test + the `--metric-digest` demo
prove the comparator + emit + the 3 fail-closed guards. Tier-2 (RUNPOD-PENDING,
deferred cloud-only): the live double-execution of a real Python experiment under
the recorded env + ELAI_SEED. The cheap seed-coverage AST gate is the per-edit
early-catch; the re-run is the sound late proof.

EXPECTED TEST OUTCOME
- h23_rerun_equality_contract.rs: 6 cases (identical=Equal+row, divergent=Block+
  first-metric, missing-seed=Block, empty=Block, trivial-empty=Block, missing-
  second-run=Block, schema>=228 + bijection) PASS.
- rerun_equality.rs unit tests (8): digest equality + first-divergence wrap +
  hazard A/B guards + metric-set mismatch + from_bytes blake3 prefix PASS.
- action_type_lock golden (228/505) + e45 (505) PASS.
- `elai replay --verify --metric-digest nondeterministic` lands a
  RerunHashEquality(Divergent) row in ssb.db (proves the SQLite CHECK site +
  W-1/W-3).

UNKNOWNS / RESIDUAL
- "Declared metric outputs" trust: a sandbagging experiment could declare a
  trivial output. H-25 (result manifest) binds the reported number to its
  provenance to close it; H-23 + H-25 ship together for the full claim. H-23
  rejects EMPTY (hazard B) but cannot prove a declared metric is non-sandbagged.
- The seed-coverage AstPattern tree-sitter query is a static leg (R5,
  necessary-not-sufficient); the runtime re-run is the load-bearing backstop.
- "ML reproducibility crisis" + "Thinking Machines batch-invariant kernels" are
  motivation-only (UNGROUNDED in the plan); TD-5 correctly scopes H-23 to NOT
  assert cross-hardware equality, so neither blocks. Determinism env-var set
  (CUBLAS_WORKSPACE_CONFIG=:16:8, PYTHONHASHSEED, torch.use_deterministic_
  algorithms) verified via PyTorch docs (docs.pytorch.org/docs/stable/notes/
  randomness.html, 2026-06-05) before authoring seed_coverage.toml.

SSB LOCKSTEP (14 sites, 227->228 / 504->505 / 424->425)
ssb.rs: SCHEMA_VERSION 228; CHECK 'rerun_hash_equality'; enum RerunHashEquality;
as_sql_str; from_str; RerunHashEqualityKind{Equal,Divergent}; RerunHashEquality
Payload; SsbPayload arm; action_type() arm; all_test_payloads fixture; count
assert 425. action_type_lock.rs: TOTAL_VARIANTS 505; ALL_VARIANTS; exhaustive_
match_guard. e45:294: 505.
